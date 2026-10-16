//! What `crisk` reads from a forge: a change's description, base and head, its label events, a
//! declarer team's members, and the changes carrying a label. GitHub and GitLab, GET only.
//!
//! | Read | GitHub | GitLab |
//! | --- | --- | --- |
//! | The change | `GET /repos/{repo}/pulls/{n}` | `GET /projects/{id}/merge_requests/{iid}` |
//! | Label events | `GET /repos/{repo}/issues/{n}/events` (`labeled`, `unlabeled`) | `GET /projects/{id}/merge_requests/{iid}/resource_label_events` |
//! | Declarer members | `GET /orgs/{org}/teams/{slug}/members` for `team:ORG/SLUG` | `GET /groups/{id}/members/all` for `group:PATH` |
//! | Changes with a label | `GET /repos/{repo}/issues?labels=…&state=closed` | `GET /projects/{id}/merge_requests?labels=…&state=merged` |

use super::http::{Http, segment};
use crate::model::{LabelAction, LabelEvent, LabelEvents, Team};
use crate::{Exit, failure, failure_with_hint};
use anyhow::Result;
use serde_json::Value;

pub const GITHUB_API: &str = "https://api.github.com";
pub const GITLAB_API: &str = "https://gitlab.com/api/v4";

/// One change, as the forge reports it.
#[derive(Debug, Clone)]
pub struct ForgeChange {
    pub id: String,
    pub url: Option<String>,
    pub description: String,
    pub base: String,
    pub head: String,
    pub merged_at: Option<String>,
}

pub trait Forge {
    /// `github` or `gitlab`, the prefix of every actor and change id.
    fn name(&self) -> &'static str;
    fn change(&self, number: u64) -> Result<ForgeChange>;
    fn label_events(&self, number: u64) -> Result<LabelEvents>;
    /// The members of a declarer this forge knows how to read (`team:` on GitHub, `group:` on
    /// GitLab); None for any other declarer.
    fn team(&self, declarer: &str, observed_at: &str) -> Result<Option<Team>>;
    /// The numbers of merged changes that carry `label` now, with their merge time.
    fn labelled(&self, label: &str) -> Result<(Vec<(u64, String)>, bool)>;
}

fn text(v: &Value, key: &str) -> Option<String> {
    v[key].as_str().map(String::from)
}

fn required(v: &Value, key: &str, what: &str) -> Result<String> {
    text(v, key).ok_or_else(|| failure(Exit::Unsupported, format!("{what} has no {key}")))
}

fn sha(value: Option<String>, what: &str) -> Result<String> {
    let s = value.ok_or_else(|| failure(Exit::Unsupported, format!("{what} has no commit SHA")))?;
    if s.len() < 7 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(failure(
            Exit::Unsupported,
            format!("{what} commit is not a SHA"),
        ));
    }
    Ok(s.to_ascii_lowercase())
}

/// Sort events oldest first, keeping the forge's order for equal times.
fn ordered(mut events: Vec<LabelEvent>, complete: bool) -> LabelEvents {
    events.sort_by(|a, b| a.at.cmp(&b.at));
    LabelEvents { complete, events }
}

// ---------------------------------------------------------------------------------------------

pub struct Github {
    http: Http,
    /// `OWNER/REPO`.
    repo: String,
}

impl Github {
    pub fn new(repo: &str, token: Option<String>, base: &str, max_pages: usize) -> Result<Self> {
        let valid = repo.split('/').count() == 2
            && repo.split('/').all(|p| {
                !p.is_empty()
                    && p.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
            });
        if !valid {
            return Err(failure_with_hint(
                Exit::Usage,
                format!("invalid GitHub repository: {repo:?}"),
                "use OWNER/REPO",
            ));
        }
        let http = Http::new(
            "GitHub",
            base,
            token.map(|t| ("Authorization", format!("Bearer {t}"))),
            vec![
                ("Accept", "application/vnd.github+json"),
                ("X-GitHub-Api-Version", "2022-11-28"),
            ],
            max_pages,
        )?;
        Ok(Self {
            http,
            repo: repo.into(),
        })
    }
}

impl Forge for Github {
    fn name(&self) -> &'static str {
        "github"
    }

    fn change(&self, number: u64) -> Result<ForgeChange> {
        let pr = self
            .http
            .object(&format!("/repos/{}/pulls/{number}", self.repo))?;
        Ok(ForgeChange {
            id: format!("github:{}:pr:{number}", self.repo),
            url: text(&pr, "html_url"),
            description: text(&pr, "body").unwrap_or_default(),
            base: sha(text(&pr["base"], "sha"), "the pull request's base")?,
            head: sha(text(&pr["head"], "sha"), "the pull request's head")?,
            merged_at: text(&pr, "merged_at"),
        })
    }

    fn label_events(&self, number: u64) -> Result<LabelEvents> {
        let (items, complete) = self.http.pages(
            &format!("/repos/{}/issues/{number}/events", self.repo),
            true,
        )?;
        let mut events = Vec::new();
        for e in &items {
            let action = match e["event"].as_str() {
                Some("labeled") => LabelAction::Added,
                Some("unlabeled") => LabelAction::Removed,
                _ => continue,
            };
            events.push(LabelEvent {
                label: required(&e["label"], "name", "a label event")?,
                action,
                actor: format!(
                    "github:{}",
                    required(&e["actor"], "login", "a label event")?.to_ascii_lowercase()
                ),
                at: required(e, "created_at", "a label event")?,
            });
        }
        Ok(ordered(events, complete))
    }

    fn team(&self, declarer: &str, observed_at: &str) -> Result<Option<Team>> {
        let Some((org, slug)) = declarer
            .strip_prefix("team:")
            .and_then(|r| r.split_once('/'))
        else {
            return Ok(None);
        };
        let (items, complete) = self.http.pages(
            &format!("/orgs/{}/teams/{}/members", segment(org), segment(slug)),
            true,
        )?;
        let mut members: Vec<String> = items
            .iter()
            .filter_map(|m| m["login"].as_str())
            .map(|l| format!("github:{}", l.to_ascii_lowercase()))
            .collect();
        members.sort();
        members.dedup();
        Ok(Some(Team {
            id: declarer.into(),
            members,
            complete,
            observed_at: observed_at.into(),
        }))
    }

    fn labelled(&self, label: &str) -> Result<(Vec<(u64, String)>, bool)> {
        let (items, mut complete) = self.http.pages(
            &format!(
                "/repos/{}/issues?labels={}&state=closed",
                self.repo,
                segment(label)
            ),
            false,
        )?;
        let mut out = Vec::new();
        for issue in items.iter().filter(|i| i["pull_request"].is_object()) {
            let Some(number) = issue["number"].as_u64() else {
                complete = false;
                continue;
            };
            if let Some(merged) = issue["pull_request"]["merged_at"].as_str() {
                out.push((number, merged.to_string()));
            }
        }
        Ok((out, complete))
    }
}

// ---------------------------------------------------------------------------------------------

pub struct Gitlab {
    http: Http,
    /// `GROUP/PROJECT`, possibly with subgroups.
    project: String,
}

impl Gitlab {
    /// `token` is `(header, value)`: `PRIVATE-TOKEN` for a personal, project or group access
    /// token, `JOB-TOKEN` for `CI_JOB_TOKEN`.
    pub fn new(
        project: &str,
        token: Option<(&'static str, String)>,
        base: &str,
        max_pages: usize,
    ) -> Result<Self> {
        if project.is_empty() || project.starts_with('/') || project.contains(char::is_whitespace) {
            return Err(failure_with_hint(
                Exit::Usage,
                format!("invalid GitLab project: {project:?}"),
                "use GROUP/PROJECT",
            ));
        }
        let http = Http::new("GitLab", base, token, vec![], max_pages)?;
        Ok(Self {
            http,
            project: project.into(),
        })
    }

    fn id(&self) -> String {
        segment(&self.project)
    }
}

impl Forge for Gitlab {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    fn change(&self, number: u64) -> Result<ForgeChange> {
        let mr = self
            .http
            .object(&format!("/projects/{}/merge_requests/{number}", self.id()))?;
        let refs = &mr["diff_refs"];
        Ok(ForgeChange {
            id: format!("gitlab:{}:mr:{number}", self.project),
            url: text(&mr, "web_url"),
            description: text(&mr, "description").unwrap_or_default(),
            base: sha(
                text(refs, "start_sha").or_else(|| text(refs, "base_sha")),
                "the merge request's target",
            )?,
            head: sha(text(refs, "head_sha"), "the merge request's head")?,
            merged_at: text(&mr, "merged_at"),
        })
    }

    fn label_events(&self, number: u64) -> Result<LabelEvents> {
        let (items, mut complete) = self.http.pages(
            &format!(
                "/projects/{}/merge_requests/{number}/resource_label_events",
                self.id()
            ),
            true,
        )?;
        let mut events = Vec::new();
        for e in &items {
            let action = match e["action"].as_str() {
                Some("add") => LabelAction::Added,
                Some("remove") => LabelAction::Removed,
                _ => continue,
            };
            // A deleted label leaves events whose label is null: what they were is unknown.
            let Some(label) = e["label"]["name"].as_str() else {
                complete = false;
                continue;
            };
            events.push(LabelEvent {
                label: label.into(),
                action,
                actor: format!(
                    "gitlab:{}",
                    required(&e["user"], "username", "a label event")?.to_ascii_lowercase()
                ),
                at: required(e, "created_at", "a label event")?,
            });
        }
        Ok(ordered(events, complete))
    }

    fn team(&self, declarer: &str, observed_at: &str) -> Result<Option<Team>> {
        let Some(group) = declarer.strip_prefix("group:") else {
            return Ok(None);
        };
        let (items, complete) = self
            .http
            .pages(&format!("/groups/{}/members/all", segment(group)), true)?;
        let mut members: Vec<String> = items
            .iter()
            .filter(|m| m["state"].as_str().is_none_or(|s| s == "active"))
            .filter_map(|m| m["username"].as_str())
            .map(|u| format!("gitlab:{}", u.to_ascii_lowercase()))
            .collect();
        members.sort();
        members.dedup();
        Ok(Some(Team {
            id: declarer.into(),
            members,
            complete,
            observed_at: observed_at.into(),
        }))
    }

    fn labelled(&self, label: &str) -> Result<(Vec<(u64, String)>, bool)> {
        let (items, mut complete) = self.http.pages(
            &format!(
                "/projects/{}/merge_requests?labels={}&state=merged",
                self.id(),
                segment(label)
            ),
            false,
        )?;
        let mut out = Vec::new();
        for mr in &items {
            match (mr["iid"].as_u64(), mr["merged_at"].as_str()) {
                (Some(iid), Some(merged)) => out.push((iid, merged.to_string())),
                _ => complete = false,
            }
        }
        Ok((out, complete))
    }
}
