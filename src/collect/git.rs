//! Read-only `git` in a local clone: resolve revisions, the merge base, the changed paths, and a
//! file as it stood at a revision.

use crate::{Exit, failure_with_hint};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const HISTORY_HINT: &str = "check out with enough history for both commits (actions/checkout `fetch-depth: 0`, GitLab `GIT_DEPTH: 0`)";

pub struct Git {
    dir: PathBuf,
}

/// A revision as the caller wrote it: refused when it could be read as an option or a range.
fn plain(rev: &str) -> Result<&str> {
    if rev.is_empty()
        || rev.starts_with('-')
        || rev.contains("..")
        || rev.chars().any(char::is_whitespace)
    {
        return Err(failure_with_hint(
            Exit::Usage,
            format!("invalid revision: {rev:?}"),
            "pass a commit SHA, a branch or a tag",
        ));
    }
    Ok(rev)
}

impl Git {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<std::process::Output> {
        Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| {
                failure_with_hint(
                    Exit::Usage,
                    format!("unable to run git: {e}"),
                    "crisk reads the change from a local clone; install git",
                )
            })
    }

    fn ok(&self, args: &[&str]) -> Result<Vec<u8>> {
        let out = self.run(args)?;
        if !out.status.success() {
            return Err(failure_with_hint(
                Exit::Usage,
                format!(
                    "git {}: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&out.stderr).trim()
                ),
                HISTORY_HINT,
            ));
        }
        Ok(out.stdout)
    }

    /// The full commit SHA `rev` names.
    pub fn resolve(&self, rev: &str) -> Result<String> {
        let rev = plain(rev)?;
        let out = self.ok(&["rev-parse", "--verify", &format!("{rev}^{{commit}}")])?;
        Ok(String::from_utf8_lossy(&out).trim().to_ascii_lowercase())
    }

    pub fn merge_base(&self, base: &str, head: &str) -> Result<String> {
        let out = self.ok(&["merge-base", base, head])?;
        Ok(String::from_utf8_lossy(&out).trim().to_ascii_lowercase())
    }

    /// The paths changed between `from` and `to`, sorted and unique; a rename is a delete and an
    /// add, so both names appear.
    pub fn changed(&self, from: &str, to: &str) -> Result<Vec<String>> {
        let out = self.ok(&["diff", "--name-only", "--no-renames", "-z", from, to, "--"])?;
        let mut paths: Vec<String> = out
            .split(|b| *b == 0)
            .filter(|p| !p.is_empty())
            .map(|p| String::from_utf8_lossy(p).into_owned())
            .collect();
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    /// The commit's committer date, as a UTC calendar date (`YYYY-MM-DD`).
    pub fn committed_on(&self, rev: &str) -> Result<String> {
        let rev = plain(rev)?;
        let out = self.ok(&["log", "-1", "--format=%ct", rev, "--"])?;
        let seconds: i64 = String::from_utf8_lossy(&out)
            .trim()
            .parse()
            .context("git log did not print a commit time")?;
        let at = chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, 0)
            .context("the commit time is out of range")?;
        Ok(at.format("%Y-%m-%d").to_string())
    }

    /// The bytes of `path` at `rev`, or None when it does not exist there.
    pub fn show(&self, rev: &str, path: &str) -> Result<Option<Vec<u8>>> {
        let spec = format!("{rev}:{path}");
        if !self.run(&["cat-file", "-e", &spec])?.status.success() {
            return Ok(None);
        }
        Ok(Some(self.ok(&["cat-file", "blob", &spec])?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_that_look_like_options_or_ranges_are_refused() {
        for bad in ["", "-x", "a..b", "a b"] {
            assert!(plain(bad).is_err(), "{bad:?}");
        }
        assert!(plain("main").is_ok());
    }
}
