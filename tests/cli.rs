//! `crisk` against real git history: the base revision rule, declarations, signals, outputs,
//! compilation and exit codes.

mod common;

use common::{Repo, configured, crisk};
use predicates::prelude::*;
use serde_json::Value;

fn assess(repo: &Repo, base: &str, extra: &[&str]) -> (i32, Value) {
    let out = crisk()
        .arg("-q")
        .arg("assess")
        .arg("--checkout")
        .arg(repo.path())
        .args(["--base", base, "-f", "json"])
        .args(extra)
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let v = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|_| panic!("exit {code}: {}", String::from_utf8_lossy(&out.stderr)));
    (code, v)
}

fn codes(a: &Value) -> Vec<String> {
    a["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["code"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn paths_map_to_a_tier_and_the_requirements_follow() {
    let (repo, base) = configured();
    repo.write("src/auth/login.rs", "fn login() { check() }\n");
    repo.write("db/001.sql", "create table t ();\n");
    let head = repo.commit("touch auth and a migration");
    let (code, a) = assess(&repo, "main", &[]);
    assert_eq!(code, 0);
    assert_eq!(a["tier"], "high");
    assert_eq!(a["change"]["base"], base);
    assert_eq!(a["change"]["head"], head);
    assert_eq!(a["change"]["merge_base"], base);
    assert_eq!(
        a["facts"]["paths"],
        serde_json::json!(["db/001.sql", "src/auth/login.rs"])
    );
    assert_eq!(a["requirements"]["min_human_approvals"], 2);
    assert_eq!(a["facts"]["sources"]["policy"]["revision"], base);
    assert!(
        a["facts"]["sources"]["policy"]["digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
}

#[test]
fn the_policy_is_read_from_the_base_never_from_the_change() {
    let (repo, _) = configured();
    // The change lowers auth to low in its own copy of the policy, and touches auth.
    repo.write(
        ".change-risk/policy.yml",
        &common::POLICY.replace("auth: high", "auth: low"),
    );
    repo.write("src/auth/login.rs", "fn login() { skip() }\n");
    repo.commit("lower my own requirements");
    let (_, a) = assess(&repo, "main", &[]);
    assert_eq!(a["tier"], "high");
    assert_eq!(a["facts"]["policy"]["levels"]["auth"], "high");
    let c = codes(&a);
    assert!(c.contains(&"CR002".into()), "{c:?}");
    assert!(c.contains(&"CR001".into()), "{c:?}");
}

#[test]
fn the_diff_starts_at_the_merge_base() {
    let (repo, _) = configured();
    repo.write("docs/guide.md", "# Guide, edited\n");
    repo.commit("docs only");
    // main moves on after the branch was cut; its changes are not this change's.
    repo.git(&["checkout", "-q", "main"]);
    repo.write("src/auth/login.rs", "fn login() { other() }\n");
    repo.commit("someone else's auth change");
    repo.git(&["checkout", "-q", "feature"]);
    let (_, a) = assess(&repo, "main", &[]);
    assert_eq!(a["facts"]["paths"], serde_json::json!(["docs/guide.md"]));
    assert_eq!(a["change_type"], "standard");
    assert_eq!(a["tier"], "low");
}

#[test]
fn no_configuration_at_the_base_uses_the_builtin_policy() {
    let repo = Repo::new();
    repo.write("a.rs", "a\n");
    repo.commit("base");
    repo.git(&["checkout", "-q", "-b", "feature"]);
    // Adding configuration in the change does not apply to the change.
    repo.write(".change-risk/policy.yml", common::POLICY);
    repo.write("a.rs", "b\n");
    repo.commit("add a policy");
    let (_, a) = assess(&repo, "main", &[]);
    assert_eq!(a["facts"]["policy"], Value::Null);
    assert_eq!(a["tier"], "high", "a change to .change-risk/ is high");
    assert_eq!(codes(&a)[..2], ["CR009".to_string(), "CR009".to_string()]);
}

#[test]
fn declarations_from_a_file_can_only_raise() {
    let (repo, _) = configured();
    repo.write("src/app.rs", "fn main() { run() }\n");
    repo.commit("app");
    let file = repo.path().join("decl.yml");
    std::fs::write(&file, "risk: high\nrollback: Revert the merge.\n").unwrap();
    let (_, a) = assess(&repo, "main", &["--declarations", file.to_str().unwrap()]);
    assert_eq!(a["tier"], "high");
    assert_eq!(a["facts"]["declarations"]["rollback"], "Revert the merge.");
    std::fs::write(&file, "change_type: emergency\n").unwrap();
    let (_, a) = assess(&repo, "main", &["--declarations", file.to_str().unwrap()]);
    assert_eq!(a["tier"], "low");
    assert!(codes(&a).contains(&"CR004".into()));
}

#[test]
fn signals_label_changed_files() {
    let (repo, _) = configured();
    repo.write("src/app.rs", "fn main() { unsafe_thing() }\n");
    let head = repo.commit("app");
    let sarif = repo.path().join("results.sarif");
    std::fs::write(
        &sarif,
        r#"{"version":"2.1.0","runs":[{"results":[{"level":"error","locations":[{"physicalLocation":{"artifactLocation":{"uri":"src/app.rs"}}}]}]}]}"#,
    )
    .unwrap();
    let manifest = repo.path().join("acc.json");
    std::fs::write(
        &manifest,
        format!(r#"{{"events":{{"changes":[{{"id":"c","head_sha":"{head}","author":{{"actor_id":"agent:codex"}}}}]}},"findings":[]}}"#),
    )
    .unwrap();
    let (_, a) = assess(
        &repo,
        "main",
        &[
            "--signal",
            &format!("sarif={}", sarif.display()),
            "--signal",
            &format!("acc={}", manifest.display()),
        ],
    );
    assert_eq!(a["tier"], "high");
    let labels: Vec<&str> = a["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, ["agent-authored", "sarif:error"]);
    assert_eq!(a["facts"]["signals"].as_array().unwrap().len(), 2);
}

#[test]
fn threat_model_signal_raises_changes_to_expired_or_unmodelled_components() {
    // The map, read from main, ties paths to threat-model components; every fixture commit is
    // committed on 2026-10-01, the date the judgements are held against.
    let r = Repo::new();
    r.write(
        ".change-risk/map.yml",
        "version: \"0.1\"\npaths:\n  - pattern: /infra/assets/\n    labels: [\"component:assets-bucket\"]\n  - pattern: /src/session/\n    labels: [\"component:session\"]\n  - pattern: /src/search/\n    labels: [\"component:search\"]\n",
    );
    r.write(
        ".change-risk/policy.yml",
        "version: \"0.1\"\ndefault_level: low\nlevels:\n  threat-model:expired: high\n  threat-model:unmodelled: standard\ntiers:\n  low: {min_human_approvals: 1}\n  standard: {min_human_approvals: 1}\n  high: {min_human_approvals: 2}\nmissing_tier: standard\n",
    );
    r.write("infra/assets/bucket.tf", "bucket {}\n");
    r.write("src/session/store.rs", "fn store() {}\n");
    r.write("src/search/index.rs", "fn index() {}\n");
    r.commit("base");
    r.git(&["checkout", "-q", "-b", "feature"]);
    let manifest = r.path().join("threat-model.yml");
    std::fs::write(
        &manifest,
        "piece: threat-model\nrisks:\n  - target: {kind: component, id: assets-bucket}\n    interpretation: {expires_at: \"2026-09-30\"}\n  - target: {kind: component, id: session}\n    interpretation: {expires_at: \"2026-10-01\"}\n",
    )
    .unwrap();
    let signal = format!("threat-model={}", manifest.display());

    // Touching the session component only: its judgement stands on its expiry day.
    r.write("src/session/store.rs", "fn store() { save() }\n");
    r.commit("session");
    let (_, a) = assess(&r, "main", &["--signal", &signal]);
    assert_eq!(a["tier"], "low");
    assert_eq!(a["facts"]["signals"][0]["as_of"], "2026-10-01");
    assert!(
        a["facts"]["signals"][0]["labels"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // The search component is in the map but nobody modelled it.
    r.write("src/search/index.rs", "fn index() { rank() }\n");
    r.commit("search");
    let (_, a) = assess(&r, "main", &["--signal", &signal]);
    assert_eq!(a["tier"], "standard");

    // The assets bucket's judgement expired the day before the head commit.
    r.write("infra/assets/bucket.tf", "bucket { public = false }\n");
    r.commit("assets");
    let (_, a) = assess(&r, "main", &["--signal", &signal]);
    assert_eq!(a["tier"], "high");
    let labels: Vec<(&str, Vec<&str>)> = a["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["label"].as_str().unwrap(),
                l["paths"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| p.as_str().unwrap())
                    .collect(),
            )
        })
        .filter(|(l, _)| l.starts_with("threat-model:"))
        .collect();
    assert_eq!(
        labels,
        [
            ("threat-model:expired", vec!["infra/assets/bucket.tf"]),
            ("threat-model:unmodelled", vec!["src/search/index.rs"]),
        ]
    );
}

#[test]
fn outputs_render_and_validate() {
    let (repo, _) = configured();
    repo.write("src/auth/login.rs", "fn login() { check() }\n");
    repo.commit("auth");
    let dir = tempfile::TempDir::new().unwrap();
    let json = dir.path().join("assessment.json");
    crisk()
        .args(["-q", "assess", "--base", "main", "--checkout"])
        .arg(repo.path())
        .arg("-o")
        .arg(&json)
        .assert()
        .success();
    crisk()
        .arg("validate")
        .arg(&json)
        .assert()
        .success()
        .stderr(predicate::str::contains("Valid assessment"));
    for (name, check) in [
        ("s.intoto.json", "Valid statement"),
        ("r.sarif", ""),
        ("gl-code-quality-report.json", ""),
        ("summary.md", ""),
    ] {
        let out = dir.path().join(name);
        crisk()
            .args(["-q", "render"])
            .arg(&json)
            .arg("-o")
            .arg(&out)
            .assert()
            .success();
        if !check.is_empty() {
            crisk()
                .arg("validate")
                .arg(&out)
                .assert()
                .success()
                .stderr(predicate::str::contains(check));
        }
    }
    let sarif: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("r.sarif")).unwrap())
            .unwrap();
    assert_eq!(sarif["runs"][0]["results"][0]["ruleId"], "CR001");
    let cq: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("gl-code-quality-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(cq[0]["location"]["path"], "src/auth/login.rs");
    assert_eq!(cq[0]["severity"], "major");
    let md = std::fs::read_to_string(dir.path().join("summary.md")).unwrap();
    assert!(md.starts_with("### Change risk: high"));

    // A tampered assessment does not validate.
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    v["tier"] = "low".into();
    std::fs::write(&json, v.to_string()).unwrap();
    crisk()
        .arg("validate")
        .arg(&json)
        .assert()
        .code(3)
        .stderr(predicate::str::contains("`tier` does not follow"));
}

#[test]
fn fail_on_and_exit_codes() {
    let (repo, _) = configured();
    repo.write("src/auth/login.rs", "x\n");
    repo.commit("auth");
    let run = |tier: &str| {
        crisk()
            .args([
                "-q",
                "assess",
                "--base",
                "main",
                "--fail-on",
                tier,
                "--checkout",
            ])
            .arg(repo.path())
            .output()
            .unwrap()
            .status
            .code()
    };
    assert_eq!(run("high"), Some(1));
    assert_eq!(run("emergency"), Some(0));
    // Usage errors.
    crisk()
        .args(["assess", "--checkout"])
        .arg(repo.path())
        .assert()
        .code(2);
    crisk()
        .args(["assess", "--base", "-x", "--checkout"])
        .arg(repo.path())
        .assert()
        .code(2);
    crisk()
        .args([
            "assess",
            "--base",
            "main",
            "--signal",
            "lint=x",
            "--checkout",
        ])
        .arg(repo.path())
        .assert()
        .code(2);
    crisk()
        .args(["assess", "--base", "no-such-branch", "--checkout"])
        .arg(repo.path())
        .assert()
        .code(2);
}

#[test]
fn an_invalid_policy_at_the_base_is_invalid_input() {
    let repo = Repo::new();
    repo.write(
        ".change-risk/policy.yml",
        "version: \"0.1\"\ndefault_level: extreme\n",
    );
    repo.commit("base");
    crisk()
        .args(["assess", "--base", "main", "--checkout"])
        .arg(repo.path())
        .assert()
        .code(3)
        .stderr(predicate::str::contains("policy schema"));
}

#[test]
fn compile_renders_three_targets() {
    let (repo, _) = configured();
    let acc = crisk()
        .args(["compile", "--to", "acc", "--base", "main", "--checkout"])
        .arg(repo.path())
        .output()
        .unwrap();
    let acc = String::from_utf8(acc.stdout).unwrap();
    assert!(
        acc.contains("  high:\n    min_human_approvals: 2\n"),
        "{acc}"
    );
    assert!(acc.ends_with("missing_tier: standard\n"));

    let dir = tempfile::TempDir::new().unwrap();
    let req = dir.path().join("requirements.json");
    crisk()
        .args([
            "compile",
            "--to",
            "requirements",
            "--base",
            "main",
            "--checkout",
        ])
        .arg(repo.path())
        .arg("-o")
        .arg(&req)
        .assert()
        .success();
    crisk()
        .arg("validate")
        .arg(&req)
        .assert()
        .success()
        .stderr(predicate::str::contains("Valid requirements"));
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&req).unwrap()).unwrap();
    assert_eq!(
        doc["tiers"]["high"]["protected_paths"][0]["pattern"],
        "/src/auth/"
    );
    assert_eq!(
        doc["tiers"]["standard"]["protected_paths"][0]["pattern"],
        "/db/"
    );
    assert_eq!(doc["emergency"]["review_within_days"], 3);

    // From files, the same text twice.
    let policy = repo.path().join(".change-risk/policy.yml");
    let map = repo.path().join(".change-risk/map.yml");
    let prose = || {
        crisk()
            .args(["compile", "--to", "prose", "--policy"])
            .arg(&policy)
            .arg("--map")
            .arg(&map)
            .output()
            .unwrap()
            .stdout
    };
    let first = prose();
    assert_eq!(first, prose());
    let text = String::from_utf8(first).unwrap();
    assert!(text.starts_with("# Change risk policy\n"));
    assert!(text.contains("| `/src/auth/` | `auth` | high |"));
}

#[test]
fn validate_detects_documents() {
    let (repo, _) = configured();
    for (file, message) in [
        (".change-risk/map.yml", "Valid map"),
        (".change-risk/policy.yml", "Valid policy"),
    ] {
        crisk()
            .arg("validate")
            .arg(repo.path().join(file))
            .assert()
            .success()
            .stderr(predicate::str::contains(message));
    }
}
