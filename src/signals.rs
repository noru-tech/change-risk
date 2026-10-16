//! Signal adapters: small, documented mappings from another tool's output to labels on the
//! change's paths. Each one reads a file the caller passes with `--signal KIND=PATH`, and only
//! ever labels paths the change touches. What each label means is the policy's business.
//!
//! | Kind | Reads | Labels |
//! | --- | --- | --- |
//! | `sarif` | A SARIF 2.1.0 log | `sarif:error`, `sarif:warning`, `sarif:note` on the changed files a result is located in |
//! | `privacy-datamap` | A Noru `privacy-datamap` manifest (`.noru/privacy-datamap.yml`) | `personal-data` on changed files a collection or declaration cites; `personal-data:special` when its categories include special-category data |
//! | `acc` | An agent-change-control manifest | `agent-authored` when the change's author is an agent; `acc:finding` when the change has an open finding; on every changed path |

use crate::model::{Signal, SignalLabel};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Special-category data in the Fideslang taxonomy (GDPR Article 9, and Article 10 criminal
/// data): a category equal to one of these, or below it, is special.
pub const SPECIAL_CATEGORIES: &[&str] = &[
    "user.authorization.biometric",
    "user.biometric",
    "user.criminal_history",
    "user.demographic.political_opinion",
    "user.demographic.race_ethnicity",
    "user.demographic.religious_belief",
    "user.demographic.sexual_orientation",
    "user.health_and_medical",
];

pub const KINDS: &[&str] = &["sarif", "privacy-datamap", "acc"];

fn is_special(category: &str) -> bool {
    SPECIAL_CATEGORIES.iter().any(|s| {
        category == *s
            || category
                .strip_prefix(s)
                .is_some_and(|rest| rest.starts_with('.'))
    })
}

/// A path as a tool wrote it, made repository-relative: `file://` and leading `./` or `/`
/// dropped, backslashes turned into slashes.
fn relative(path: &str) -> String {
    let p = path
        .strip_prefix("file://")
        .unwrap_or(path)
        .replace('\\', "/");
    let mut p = p.as_str();
    while let Some(rest) = p.strip_prefix("./") {
        p = rest;
    }
    p.trim_start_matches('/').to_string()
}

struct Labels {
    changed: BTreeSet<String>,
    out: BTreeMap<String, BTreeSet<String>>,
}

impl Labels {
    fn new(changed: &[String]) -> Self {
        Self {
            changed: changed.iter().cloned().collect(),
            out: BTreeMap::new(),
        }
    }

    /// Label `path` if the change touches it.
    fn add(&mut self, label: &str, path: &str) {
        let path = relative(path);
        if self.changed.contains(&path) {
            self.out.entry(label.into()).or_default().insert(path);
        }
    }

    fn add_all(&mut self, label: &str) {
        let all: Vec<String> = self.changed.iter().cloned().collect();
        self.out.entry(label.into()).or_default().extend(all);
    }

    fn finish(self) -> Vec<SignalLabel> {
        self.out
            .into_iter()
            .filter(|(_, paths)| !paths.is_empty())
            .map(|(label, paths)| SignalLabel {
                label,
                paths: paths.into_iter().collect(),
            })
            .collect()
    }
}

/// Read one signal file. `head` is the change's head commit, which the acc adapter matches.
pub fn read(
    kind: &str,
    file: &str,
    bytes: &[u8],
    changed: &[String],
    head: &str,
) -> Result<Signal> {
    let text = std::str::from_utf8(bytes).context("signal file is not UTF-8")?;
    let labels = match kind {
        "sarif" => sarif(text, changed)?,
        "privacy-datamap" => datamap(text, changed)?,
        "acc" => acc(text, changed, head)?,
        other => bail!(
            "unknown signal kind {other:?}; expected one of {}",
            KINDS.join(", ")
        ),
    };
    Ok(Signal {
        kind: kind.into(),
        file: file.into(),
        digest: crate::canonical::sha256(bytes),
        labels,
    })
}

fn parse(text: &str) -> Result<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Ok(v);
    }
    serde_saphyr::from_str(text).context("not valid JSON or YAML")
}

/// SARIF 2.1.0: every result's level (default `warning`) on every changed file it is located in.
fn sarif(text: &str, changed: &[String]) -> Result<Vec<SignalLabel>> {
    let log: Value = serde_json::from_str(text).context("SARIF log is not valid JSON")?;
    ensure!(
        log["version"].as_str() == Some("2.1.0"),
        "SARIF log version must be 2.1.0"
    );
    let mut labels = Labels::new(changed);
    for run in log["runs"].as_array().into_iter().flatten() {
        for result in run["results"].as_array().into_iter().flatten() {
            let level = result["level"].as_str().unwrap_or("warning");
            let level = match level {
                "error" | "warning" | "note" => level,
                // `none` marks a result that is not a problem.
                "none" => continue,
                other => {
                    bail!("SARIF result level {other:?} is not one of error, warning, note, none")
                }
            };
            for loc in result["locations"].as_array().into_iter().flatten() {
                if let Some(uri) = loc["physicalLocation"]["artifactLocation"]["uri"].as_str() {
                    labels.add(&format!("sarif:{level}"), uri);
                }
            }
        }
    }
    Ok(labels.finish())
}

fn file_of(reference: &str) -> &str {
    reference
        .rsplit_once(':')
        .filter(|(_, line)| line.chars().all(|c| c.is_ascii_digit()) && !line.is_empty())
        .map_or(reference, |(file, _)| file)
}

fn categories(node: &Value, out: &mut Vec<String>) {
    for c in node["data_categories"].as_array().into_iter().flatten() {
        if let Some(c) = c.as_str() {
            out.push(c.to_string());
        }
    }
    for field in node["fields"].as_array().into_iter().flatten() {
        categories(field, out);
    }
}

fn refs(node: &Value) -> Vec<String> {
    node["refs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r.as_str().map(|r| file_of(r).to_string()))
        .collect()
}

/// A `privacy-datamap` manifest: the files a collection or a privacy declaration cites, labelled
/// `personal-data` when its categories hold personal data, and also `personal-data:special` when
/// any is special-category.
fn datamap(text: &str, changed: &[String]) -> Result<Vec<SignalLabel>> {
    let doc = parse(text)?;
    ensure!(
        doc["piece"].as_str() == Some("privacy-datamap"),
        "not a privacy-datamap manifest (piece is not privacy-datamap)"
    );
    let mut labels = Labels::new(changed);
    let mut mark = |node: &Value, cats: Vec<String>| {
        if cats.is_empty() {
            return;
        }
        let special = cats.iter().any(|c| is_special(c));
        let mut files = refs(node);
        for field in node["fields"].as_array().into_iter().flatten() {
            files.extend(refs(field));
        }
        for f in files {
            labels.add("personal-data", &f);
            if special {
                labels.add("personal-data:special", &f);
            }
        }
    };
    for dataset in doc["dataset"].as_array().into_iter().flatten() {
        for collection in dataset["collections"].as_array().into_iter().flatten() {
            let mut cats = Vec::new();
            categories(collection, &mut cats);
            mark(collection, cats);
        }
    }
    for system in doc["system"].as_array().into_iter().flatten() {
        for decl in system["privacy_declarations"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let mut cats = Vec::new();
            categories(decl, &mut cats);
            mark(decl, cats);
        }
    }
    Ok(labels.finish())
}

/// An agent-change-control manifest: the change whose head commit is `head`.
fn acc(text: &str, changed: &[String], head: &str) -> Result<Vec<SignalLabel>> {
    let doc = parse(text)?;
    ensure!(
        doc["events"]["changes"].is_array(),
        "not an agent-change-control manifest (no events.changes)"
    );
    let mut labels = Labels::new(changed);
    let change = doc["events"]["changes"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| {
            c["head_sha"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case(head))
        });
    if let Some(c) = change {
        let id = c["id"].as_str().unwrap_or_default();
        if c["author"]["actor_id"]
            .as_str()
            .is_some_and(|a| a.starts_with("agent:"))
        {
            labels.add_all("agent-authored");
        }
        let open = doc["findings"].as_array().into_iter().flatten().any(|f| {
            f["change_id"].as_str() == Some(id)
                && f["disposition"]["status"].as_str().unwrap_or("open") == "open"
        });
        if open {
            labels.add_all("acc:finding");
        }
    }
    Ok(labels.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changed() -> Vec<String> {
        vec!["db/schema.sql".into(), "src/auth.rs".into()]
    }

    #[test]
    fn sarif_results_label_the_changed_files_they_point_at() {
        let log = r#"{"version":"2.1.0","runs":[{"results":[
            {"level":"error","locations":[{"physicalLocation":{"artifactLocation":{"uri":"./src/auth.rs"}}}]},
            {"locations":[{"physicalLocation":{"artifactLocation":{"uri":"src/other.rs"}}}]},
            {"level":"none","locations":[{"physicalLocation":{"artifactLocation":{"uri":"db/schema.sql"}}}]}
        ]}]}"#;
        let s = read("sarif", "x.sarif", log.as_bytes(), &changed(), "h").unwrap();
        assert_eq!(s.labels.len(), 1);
        assert_eq!(s.labels[0].label, "sarif:error");
        assert_eq!(s.labels[0].paths, vec!["src/auth.rs"]);
        assert!(s.digest.starts_with("sha256:"));
    }

    #[test]
    fn a_data_map_labels_cited_files_and_special_categories() {
        let map = "piece: privacy-datamap\ndataset:\n  - collections:\n      - refs: [\"db/schema.sql:2\"]\n        fields:\n          - name: diagnosis\n            data_categories: [user.health_and_medical]\n";
        let s = read("privacy-datamap", "m.yml", map.as_bytes(), &changed(), "h").unwrap();
        let names: Vec<&str> = s.labels.iter().map(|l| l.label.as_str()).collect();
        assert_eq!(names, ["personal-data", "personal-data:special"]);
        assert!(is_special("user.health_and_medical.genetic"));
        assert!(!is_special("user.health"));
    }

    #[test]
    fn an_acc_manifest_labels_agent_authorship_and_open_findings() {
        let m = r#"{"events":{"changes":[{"id":"c1","head_sha":"ABC","author":{"actor_id":"agent:claude-code"}}]},
            "findings":[{"change_id":"c1","disposition":{"status":"open"}}]}"#;
        let s = read("acc", "m.json", m.as_bytes(), &changed(), "abc").unwrap();
        let names: Vec<&str> = s.labels.iter().map(|l| l.label.as_str()).collect();
        assert_eq!(names, ["acc:finding", "agent-authored"]);
        assert_eq!(s.labels[0].paths.len(), 2);
        let none = read("acc", "m.json", m.as_bytes(), &changed(), "other").unwrap();
        assert!(none.labels.is_empty());
    }

    #[test]
    fn unknown_kinds_and_wrong_files_are_errors() {
        assert!(read("lint", "x", b"{}", &changed(), "h").is_err());
        assert!(read("sarif", "x", b"{\"version\":\"2.0.0\"}", &changed(), "h").is_err());
        assert!(read("privacy-datamap", "x", b"piece: other", &changed(), "h").is_err());
    }
}
