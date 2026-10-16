//! CODEOWNERS-style path patterns, for the sensitivity map and the standard-change catalogue.
//!
//! The syntax is the subset GitHub's CODEOWNERS documents, so the same pattern means the same
//! thing in both files:
//!
//! * `/` separates directories; paths are relative to the repository root.
//! * A leading `/` anchors the pattern to the root. So does a `/` anywhere but at the end.
//!   Otherwise the pattern matches at any depth (`*.sql` matches `db/a.sql`).
//! * A trailing `/` matches the directory's contents. A pattern without one also matches
//!   everything under a directory of that name.
//! * `*` matches within one path segment, `**` across segments, `?` one character.
//! * `!` negation, `[ ]` ranges and `\` escapes are refused, because CODEOWNERS does not support
//!   them and a pattern should not mean two things.

use anyhow::{Result, bail};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// One compiled pattern.
#[derive(Debug, Clone)]
pub struct Pattern {
    source: String,
    set: GlobSet,
}

impl Pattern {
    pub fn new(pattern: &str) -> Result<Self> {
        let p = pattern.trim();
        if p.is_empty() || p == "/" {
            bail!("empty path pattern");
        }
        if p.starts_with('!') || p.contains(['[', ']', '\\']) {
            bail!(
                "path pattern {pattern:?} uses `!`, `[ ]` or `\\`, which CODEOWNERS syntax does not support"
            );
        }
        if p.chars().any(char::is_whitespace) {
            bail!("path pattern {pattern:?} contains whitespace");
        }
        let dir_only = p.ends_with('/');
        let trimmed = p.trim_end_matches('/');
        let anchored = trimmed.starts_with('/') || trimmed.contains('/');
        let body = trimmed.trim_start_matches('/');
        if body.is_empty() {
            bail!("empty path pattern");
        }
        let base = if anchored {
            body.to_string()
        } else {
            format!("**/{body}")
        };
        let mut globs = vec![format!("{base}/**")];
        if !dir_only {
            globs.push(base);
        }
        let mut builder = GlobSetBuilder::new();
        for g in globs {
            builder.add(
                GlobBuilder::new(&g)
                    .literal_separator(true)
                    .backslash_escape(false)
                    .build()?,
            );
        }
        Ok(Self {
            source: pattern.to_string(),
            set: builder.build()?,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Whether `path` (repository-relative, `/`-separated, no leading `/`) matches.
    pub fn matches(&self, path: &str) -> bool {
        self.set.is_match(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pattern: &str, path: &str) -> bool {
        Pattern::new(pattern).unwrap().matches(path)
    }

    #[test]
    fn unanchored_names_match_at_any_depth() {
        assert!(m("*.sql", "a.sql"));
        assert!(m("*.sql", "db/migrations/a.sql"));
        assert!(m("auth", "src/auth/login.rs"));
        assert!(!m("*.sql", "a.sqlx"));
    }

    #[test]
    fn a_slash_anchors_to_the_root() {
        assert!(m("/src/auth/", "src/auth/login.rs"));
        assert!(m("src/auth", "src/auth/deep/login.rs"));
        assert!(!m("src/auth", "lib/src/auth/login.rs"));
        assert!(m("/Dockerfile", "Dockerfile"));
        assert!(!m("/Dockerfile", "web/Dockerfile"));
    }

    #[test]
    fn a_trailing_slash_matches_contents_only() {
        assert!(m("docs/", "docs/a.md"));
        assert!(!m("docs/", "docs"));
        assert!(m("docs", "docs"));
    }

    #[test]
    fn a_star_stays_in_its_segment() {
        assert!(m("/src/*.rs", "src/main.rs"));
        assert!(!m("/src/*.rs", "src/cli/main.rs"));
        assert!(m("/src/**/*.rs", "src/cli/main.rs"));
    }

    #[test]
    fn unsupported_syntax_is_refused() {
        for bad in ["", "/", "!secret", "[ab].rs", "a\\b", "a b"] {
            assert!(Pattern::new(bad).is_err(), "{bad:?}");
        }
    }
}
