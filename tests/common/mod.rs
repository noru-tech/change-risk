#![allow(dead_code)]

use assert_cmd::Command;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// `crisk` with no forge credentials or API overrides from the environment.
pub fn crisk() -> Command {
    let mut cmd = Command::cargo_bin("crisk").unwrap();
    for var in [
        "GITHUB_TOKEN",
        "GH_TOKEN",
        "GITLAB_TOKEN",
        "CI_JOB_TOKEN",
        "CI_API_V4_URL",
        "CRISK_GITHUB_API_URL",
        "GIT_DIR",
    ] {
        cmd.env_remove(var);
    }
    cmd.env("CRISK_NOW", "2026-10-14T12:00:00Z");
    cmd
}

/// A throwaway repository with deterministic commits.
pub struct Repo {
    pub dir: TempDir,
}

impl Repo {
    pub fn new() -> Self {
        let r = Self {
            dir: TempDir::new().unwrap(),
        };
        r.git(&["init", "-q", "-b", "main"]);
        r
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(self.path())
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.com",
            ])
            .args(args)
            .env_remove("GIT_DIR")
            .env("GIT_AUTHOR_DATE", "2026-10-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-10-01T00:00:00Z")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    pub fn write(&self, path: &str, text: &str) {
        let p = self.path().join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// Stage everything and commit; returns the new commit.
    pub fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }
}

pub const MAP: &str = "version: \"0.1\"\npaths:\n  - pattern: /src/auth/\n    labels: [auth]\n  - pattern: /db/\n    labels: [migrations]\n";

pub const POLICY: &str = "version: \"0.1\"
default_level: low
levels:
  auth: high
  migrations: standard
  sarif:error: high
  agent-authored: standard
standard_changes:
  - name: documentation
    paths: [\"/docs/\", \"*.md\"]
emergency:
  label: emergency
  declarers: [alice, \"team:acme/sre\", \"group:acme/sre\"]
tiers:
  low: {min_human_approvals: 1, agent_review: allowed}
  standard: {min_human_approvals: 1}
  high: {min_human_approvals: 2, required_reviewer_teams: [acme/security]}
  emergency: {min_human_approvals: 0, retroactive_review_within_days: 3}
missing_tier: standard
";

/// A repository whose `main` holds the map and policy and one source file, and a `feature`
/// branch off it. Returns (repo, base commit).
pub fn configured() -> (Repo, String) {
    let r = Repo::new();
    r.write(".change-risk/map.yml", MAP);
    r.write(".change-risk/policy.yml", POLICY);
    r.write("src/auth/login.rs", "fn login() {}\n");
    r.write("src/app.rs", "fn main() {}\n");
    r.write("docs/guide.md", "# Guide\n");
    let base = r.commit("base");
    r.git(&["checkout", "-q", "-b", "feature"]);
    (r, base)
}

/// A loopback HTTP server answering GET requests from a fixed table, recording each request
/// path and its Authorization-type headers.
pub struct Server {
    pub url: String,
    pub requests: Arc<Mutex<Vec<String>>>,
}

pub fn serve(routes: BTreeMap<String, (u16, String)>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&requests);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
            let mut header = String::new();
            while reader.read_line(&mut header).is_ok() && header.trim() != "" {
                header.clear();
            }
            log.lock().unwrap().push(path.clone());
            // Pages after the first are empty unless the table names them.
            let key = path.clone();
            let (status, body) = routes.get(&key).cloned().unwrap_or_else(|| {
                let base = key
                    .split("per_page=")
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(['?', '&']);
                if key.contains("page=1") {
                    routes
                        .get(base)
                        .cloned()
                        .unwrap_or((404, "{\"message\":\"Not Found\"}".into()))
                } else {
                    (404, "{\"message\":\"Not Found\"}".into())
                }
            });
            let reason = if status == 200 { "OK" } else { "Error" };
            let _ = write!(
                stream,
                "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    Server { url, requests }
}
