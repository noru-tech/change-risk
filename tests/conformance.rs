//! The conformance corpus, run against `crisk evaluate` through the same contract any other
//! implementation uses (`conformance/README.md`): facts in, an assessment on stdout.
//!
//! `UPDATE_CORPUS=1 cargo test --test conformance` rewrites each accept vector's `expected` from
//! this implementation. Review the diff before committing it: the corpus is the specification's
//! test, not this implementation's snapshot.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn vectors(kind: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join("conformance").join(kind))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    v.sort();
    v
}

/// The fields the corpus pins: everything derived, without messages or `generated`.
pub fn project(a: &Value) -> Value {
    let reasons: Vec<Value> = a["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let mut r = r.clone();
            r.as_object_mut().unwrap().remove("message");
            r
        })
        .collect();
    json!({
        "level": a["level"],
        "change_type": a["change_type"],
        "tier": a["tier"],
        "complete": a["complete"],
        "labels": a["labels"],
        "reasons": reasons,
        "emergency": a["emergency"],
        "requirements": a["requirements"],
        "facts_digest": a["facts_digest"],
    })
}

fn evaluate(facts: &Value, dir: &Path) -> std::process::Output {
    let input = dir.join("facts.json");
    std::fs::write(&input, serde_json::to_vec(facts).unwrap()).unwrap();
    Command::cargo_bin("crisk")
        .unwrap()
        .args(["-q", "evaluate", "-f", "json"])
        .arg(&input)
        .output()
        .unwrap()
}

#[test]
fn accept_vectors_reproduce_their_expected_assessment() {
    let update = std::env::var_os("UPDATE_CORPUS").is_some();
    let tmp = tempfile::TempDir::new().unwrap();
    let all = vectors("accept");
    assert!(all.len() >= 20, "the accept corpus shrank");
    for path in all {
        let mut doc: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let out = evaluate(&doc["facts"], tmp.path());
        let code = out.status.code().unwrap();
        assert!(
            code == 0 || code == 4,
            "{}: exit {code}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        let assessment: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            code == 4,
            assessment["complete"] == false,
            "{}: exit 4 exactly when incomplete",
            path.display()
        );
        let got = project(&assessment);
        if update {
            doc["expected"] = got;
            std::fs::write(&path, serde_json::to_string_pretty(&doc).unwrap() + "\n").unwrap();
        } else {
            assert_eq!(got, doc["expected"], "{}", path.display());
        }
    }
}

#[test]
fn reject_vectors_are_invalid_input() {
    let tmp = tempfile::TempDir::new().unwrap();
    for path in vectors("reject") {
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let out = evaluate(&doc["facts"], tmp.path());
        assert_eq!(out.status.code(), Some(3), "{}", path.display());
        assert!(out.stdout.is_empty(), "{}: wrote output", path.display());
        let stderr = String::from_utf8_lossy(&out.stderr);
        let hint = doc["error"].as_str().unwrap();
        assert!(stderr.contains(hint), "{}: {stderr}", path.display());
    }
}
