//! `change-risk` — the library half of the `crisk` command-line tool.
//!
//! Design in one paragraph: [`collect`] reads the facts of one change (its changed paths from
//! `git`, the sensitivity map and tier policy from the *base* revision, declarations and label
//! events from the forge, and labels from signal files) into the public [`model::Facts`] shape.
//! [`evaluate`] turns facts into an [`model::Assessment`] — a tier and the reasons that produced
//! it — without touching the network, the clock, `git` or the environment. [`output`] renders an
//! assessment as JSON, a table, an in-toto Statement, SARIF or GitLab Code Quality, and
//! [`compile`] turns a tier policy into acc's tier section, a requirements document or prose.

pub mod canonical;
pub mod cli;
pub mod collect;
pub mod compile;
pub mod declarations;
pub mod evaluate;
pub mod model;
pub mod output;
pub mod pattern;
pub mod population;
pub mod schema;
pub mod signals;

use std::borrow::Cow;

/// Crate version, as compiled in.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Base URL of the documentation tree.
pub const DOCS_BASE_URL: &str = "https://github.com/noru-tech/change-risk/blob/main/docs";

/// The documentation of a reason code (`CR001`).
pub fn reason_doc_url(code: &str) -> String {
    format!("{DOCS_BASE_URL}/reasons.md#{}", code.to_ascii_lowercase())
}

/// Process exit codes used by `crisk`. Their meaning never changes; a new condition gets a new
/// code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// Success; with `--fail-on`, the tier is below the threshold.
    Ok = 0,
    /// The tier is at or above `--fail-on`.
    TierExceeded = 1,
    /// Invalid command-line arguments.
    Usage = 2,
    /// Invalid input: a facts document, assessment, map, policy or signal file.
    InvalidInput = 3,
    /// Collection incomplete: label events or a declarer team could not be read, so an emergency
    /// cannot be established. Takes precedence over 1.
    Incomplete = 4,
    /// Authentication rejected by the forge.
    Auth = 5,
    /// API, permission, rate-limit or transport failure.
    Api = 6,
    /// The forge returned data `crisk` does not understand.
    Unsupported = 7,
}

impl Exit {
    pub const fn code(self) -> i32 {
        self as i32
    }

    /// The anchor of this code's section in `docs/exit-codes.md`.
    pub const fn anchor(self) -> &'static str {
        match self {
            Exit::Ok => "0-success",
            Exit::TierExceeded => "1-tier-at-or-above---fail-on",
            Exit::Usage => "2-invalid-command-line-arguments",
            Exit::InvalidInput => "3-invalid-input",
            Exit::Incomplete => "4-collection-incomplete",
            Exit::Auth => "5-authentication-rejected",
            Exit::Api => "6-api-permission-rate-limit-or-transport-failure",
            Exit::Unsupported => "7-unsupported-forge-data",
        }
    }

    /// The documentation of this code: its section in `docs/exit-codes.md`.
    pub fn doc_url(self) -> String {
        format!("{DOCS_BASE_URL}/exit-codes.md#{}", self.anchor())
    }
}

impl From<Exit> for std::process::ExitCode {
    fn from(e: Exit) -> Self {
        std::process::ExitCode::from(e.code() as u8)
    }
}

/// An error that carries the exit status it must produce, and optionally how to fix it. Forge
/// failures use fixed messages, so response bodies and headers never reach the terminal.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Failure {
    pub exit: Exit,
    pub message: Cow<'static, str>,
    pub hint: Option<Cow<'static, str>>,
}

/// An error with an exit status.
pub fn failure(exit: Exit, message: impl Into<Cow<'static, str>>) -> anyhow::Error {
    Failure {
        exit,
        message: message.into(),
        hint: None,
    }
    .into()
}

/// An error with an exit status and a one-sentence hint.
pub fn failure_with_hint(
    exit: Exit,
    message: impl Into<Cow<'static, str>>,
    hint: impl Into<Cow<'static, str>>,
) -> anyhow::Error {
    Failure {
        exit,
        message: message.into(),
        hint: Some(hint.into()),
    }
    .into()
}

/// The exit status an error produces: its own, or 3 (invalid input) for any other error.
pub fn exit_for(err: &anyhow::Error) -> Exit {
    err.downcast_ref::<Failure>()
        .map_or(Exit::InvalidInput, |f| f.exit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_stable() {
        let pinned = [
            (Exit::Ok, 0),
            (Exit::TierExceeded, 1),
            (Exit::Usage, 2),
            (Exit::InvalidInput, 3),
            (Exit::Incomplete, 4),
            (Exit::Auth, 5),
            (Exit::Api, 6),
            (Exit::Unsupported, 7),
        ];
        for (exit, code) in pinned {
            assert_eq!(exit.code(), code);
        }
    }
}
