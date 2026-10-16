//! The committed shell completions and man pages match the CLI. `UPDATE_ASSETS=1 cargo test --test
//! assets` regenerates them.

mod common;

use common::crisk;
use std::path::{Path, PathBuf};

fn names(d: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d)
        .map(|r| {
            r.map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn completions_and_man_pages_match_the_cli() {
    let dir = tempfile::tempdir().unwrap();
    let fresh = |sub: &str| dir.path().join(sub);
    std::fs::create_dir_all(fresh("completions")).unwrap();
    for (shell, name) in [
        ("bash", "crisk.bash"),
        ("zsh", "_crisk"),
        ("fish", "crisk.fish"),
    ] {
        let out = crisk().args(["completions", shell]).output().unwrap();
        assert!(out.status.success());
        std::fs::write(fresh("completions").join(name), out.stdout).unwrap();
    }
    crisk()
        .args(["manpage", "--out-dir"])
        .arg(fresh("man"))
        .assert()
        .success();
    let update = std::env::var_os("UPDATE_ASSETS").is_some();
    for sub in ["completions", "man"] {
        let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(sub);
        if update {
            let _ = std::fs::remove_dir_all(&committed);
            std::fs::create_dir_all(&committed).unwrap();
            for name in names(&fresh(sub)) {
                std::fs::copy(fresh(sub).join(&name), committed.join(&name)).unwrap();
            }
        }
        assert_eq!(
            names(&committed),
            names(&fresh(sub)),
            "{sub}/ is stale: UPDATE_ASSETS=1 cargo test --test assets"
        );
        for name in names(&committed) {
            assert!(
                std::fs::read(committed.join(&name)).unwrap()
                    == std::fs::read(fresh(sub).join(&name)).unwrap(),
                "{sub}/{name} is stale: UPDATE_ASSETS=1 cargo test --test assets"
            );
        }
    }
}
