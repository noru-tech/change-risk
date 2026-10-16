//! Reading changes, label events, declarer membership and the emergency population from GitHub
//! and GitLab, against a loopback server that answers from a fixed table.

mod common;

use common::{Repo, configured, crisk, serve};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn routes(pairs: &[(String, Value)]) -> BTreeMap<String, (u16, String)> {
    pairs
        .iter()
        .map(|(k, v)| (k.clone(), (200, v.to_string())))
        .collect()
}

/// A feature branch touching src/app.rs; returns (repo, base, head).
fn change() -> (Repo, String, String) {
    let (repo, base) = configured();
    repo.write("src/app.rs", "fn main() { patched() }\n");
    let head = repo.commit("hotfix");
    (repo, base, head)
}

fn github_pr(base: &str, head: &str, body: &str) -> Value {
    json!({
        "html_url": "https://github.com/acme/api/pull/7",
        "body": body,
        "base": {"sha": base},
        "head": {"sha": head},
        "merged_at": null,
    })
}

fn label(event: &str, login: &str, at: &str) -> Value {
    json!({"event": event, "label": {"name": "emergency"}, "actor": {"login": login}, "created_at": at})
}

fn github_assess(repo: &Repo, url: &str) -> (i32, Value) {
    let out = crisk()
        .env("CRISK_GITHUB_API_URL", url)
        .args([
            "-q",
            "assess",
            "--github",
            "acme/api",
            "--pr",
            "7",
            "-f",
            "json",
            "--checkout",
        ])
        .arg(repo.path())
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let v = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|_| panic!("exit {code}: {}", String::from_utf8_lossy(&out.stderr)));
    (code, v)
}

#[test]
fn github_emergency_declared_by_an_allowed_login() {
    let (repo, base, head) = change();
    let server = serve(routes(&[
        (
            "/repos/acme/api/pulls/7".into(),
            github_pr(
                &base,
                &head,
                "Hotfix.\n\n<!-- change-risk\nrollback: Revert.\n-->\n",
            ),
        ),
        (
            "/repos/acme/api/issues/7/events".into(),
            json!([
                {"event": "labeled", "label": {"name": "bug"}, "actor": {"login": "bob"}, "created_at": "2026-10-01T09:00:00Z"},
                label("labeled", "Alice", "2026-10-01T10:00:00Z"),
                {"event": "commented", "actor": {"login": "bob"}, "created_at": "2026-10-01T10:30:00Z"},
            ]),
        ),
        (
            "/orgs/acme/teams/sre/members".into(),
            json!([{"login": "sam"}]),
        ),
    ]));
    let (code, a) = github_assess(&repo, &server.url);
    assert_eq!(code, 0);
    assert_eq!(a["tier"], "emergency");
    assert_eq!(a["change"]["id"], "github:acme/api:pr:7");
    assert_eq!(a["emergency"]["declared_by"], "github:alice");
    assert_eq!(a["emergency"]["review_within_days"], 3);
    assert_eq!(a["facts"]["declarations"]["rollback"], "Revert.");
    assert_eq!(a["facts"]["teams"][0]["members"], json!(["github:sam"]));
    assert_eq!(
        a["facts"]["teams"][0]["observed_at"],
        "2026-10-14T12:00:00Z"
    );
    // GET only, on the expected paths.
    let seen = server.requests.lock().unwrap().clone();
    assert!(
        seen.iter()
            .all(|p| p.starts_with("/repos/acme/api/") || p.starts_with("/orgs/acme/teams/sre/")),
        "{seen:?}"
    );
}

#[test]
fn github_emergency_by_a_team_member_and_by_anyone_else() {
    let (repo, base, head) = change();
    let server = serve(routes(&[
        (
            "/repos/acme/api/pulls/7".into(),
            github_pr(&base, &head, ""),
        ),
        (
            "/repos/acme/api/issues/7/events".into(),
            json!([label("labeled", "sam", "2026-10-01T10:00:00Z")]),
        ),
        (
            "/orgs/acme/teams/sre/members".into(),
            json!([{"login": "sam"}]),
        ),
    ]));
    let (_, a) = github_assess(&repo, &server.url);
    assert_eq!(a["tier"], "emergency");

    let (repo, base, head) = change();
    let server = serve(routes(&[
        (
            "/repos/acme/api/pulls/7".into(),
            github_pr(&base, &head, ""),
        ),
        (
            "/repos/acme/api/issues/7/events".into(),
            json!([label("labeled", "mallory", "2026-10-01T10:00:00Z")]),
        ),
        (
            "/orgs/acme/teams/sre/members".into(),
            json!([{"login": "sam"}]),
        ),
    ]));
    let (code, a) = github_assess(&repo, &server.url);
    assert_eq!(code, 0);
    assert_eq!(a["tier"], "low");
    assert!(
        a["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["code"] == "CR006")
    );
}

#[test]
fn github_unreadable_team_membership_is_incomplete() {
    let (repo, base, head) = change();
    let mut table = routes(&[
        (
            "/repos/acme/api/pulls/7".into(),
            github_pr(&base, &head, ""),
        ),
        (
            "/repos/acme/api/issues/7/events".into(),
            json!([label("labeled", "sam", "2026-10-01T10:00:00Z")]),
        ),
    ]);
    table.insert(
        "/orgs/acme/teams/sre/members".into(),
        (403, "{\"message\":\"Must have admin rights\"}".into()),
    );
    let server = serve(table);
    let (code, a) = github_assess(&repo, &server.url);
    assert_eq!(code, 4);
    assert_eq!(a["complete"], false);
    assert_eq!(a["tier"], "low");
    assert_eq!(a["facts"]["teams"][0]["complete"], false);
}

#[test]
fn github_missing_pull_request_is_an_api_failure() {
    let (repo, _, _) = change();
    let server = serve(BTreeMap::new());
    crisk()
        .env("CRISK_GITHUB_API_URL", &server.url)
        .args(["assess", "--github", "acme/api", "--pr", "7", "--checkout"])
        .arg(repo.path())
        .assert()
        .code(6);
}

#[test]
fn a_token_is_never_sent_to_a_plain_http_origin() {
    let (repo, _, _) = change();
    let server = serve(BTreeMap::new());
    crisk()
        .env("CRISK_GITHUB_API_URL", &server.url)
        .env("GITHUB_TOKEN", "not-a-real-token")
        .args(["assess", "--github", "acme/api", "--pr", "7", "--checkout"])
        .arg(repo.path())
        .assert()
        .code(2);
    assert!(server.requests.lock().unwrap().is_empty());
}

#[test]
fn gitlab_emergency_declared_by_a_group_member() {
    let (repo, base, head) = change();
    let server = serve(routes(&[
        (
            "/projects/acme%2Fapi/merge_requests/9".into(),
            json!({"web_url": "https://gitlab.com/acme/api/-/merge_requests/9", "description": "<!-- change-risk\nrisk: standard\n-->",
                   "diff_refs": {"start_sha": base, "base_sha": base, "head_sha": head}, "merged_at": null}),
        ),
        (
            "/projects/acme%2Fapi/merge_requests/9/resource_label_events".into(),
            json!([{"action": "add", "label": {"name": "emergency"}, "user": {"username": "Sam"}, "created_at": "2026-10-01T10:00:00Z"}]),
        ),
        (
            "/groups/acme%2Fsre/members/all".into(),
            json!([{"username": "sam", "state": "active"}]),
        ),
    ]));
    let out = crisk()
        .args([
            "-q",
            "assess",
            "--gitlab",
            "acme/api",
            "--mr",
            "9",
            "--gitlab-url",
            &server.url,
            "-f",
            "json",
            "--checkout",
        ])
        .arg(repo.path())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let a: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(a["change"]["id"], "gitlab:acme/api:mr:9");
    assert_eq!(a["tier"], "emergency");
    assert_eq!(a["level"], "standard");
    assert_eq!(a["emergency"]["declared_by"], "gitlab:sam");
}

#[test]
fn github_population_lists_emergencies_in_the_window() {
    let (repo, _, _) = change();
    let server = serve(routes(&[
        (
            "/repos/acme/api/issues?labels=emergency&state=closed".into(),
            json!([
                {"number": 7, "pull_request": {"merged_at": "2026-10-02T08:00:00Z"}},
                {"number": 8, "pull_request": {"merged_at": "2026-11-02T08:00:00Z"}},
                {"number": 9, "pull_request": {"merged_at": null}},
                {"number": 10},
            ]),
        ),
        (
            "/repos/acme/api/pulls/7".into(),
            github_pr(&"1".repeat(40), &"2".repeat(40), ""),
        ),
        (
            "/repos/acme/api/issues/7/events".into(),
            json!([label("labeled", "mallory", "2026-10-01T10:00:00Z")]),
        ),
        (
            "/orgs/acme/teams/sre/members".into(),
            json!([{"login": "sam"}]),
        ),
    ]));
    let out = crisk()
        .env("CRISK_GITHUB_API_URL", &server.url)
        .args([
            "-q",
            "population",
            "github",
            "acme/api",
            "--since",
            "2026-10-01",
            "--until",
            "2026-10-31",
            "--base",
            "main",
            "--checkout",
        ])
        .arg(repo.path())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let p: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(p["kind"], "change-risk-emergency-population");
    let changes = p["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["change"], "github:acme/api:pr:7");
    assert_eq!(changes[0]["declared_by"], "github:mallory");
    assert_eq!(changes[0]["allowed"], false);
    assert_eq!(changes[0]["review_due"], "2026-10-05T08:00:00Z");

    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("population.json");
    std::fs::write(&file, &out.stdout).unwrap();
    crisk().arg("validate").arg(&file).assert().success();
}
