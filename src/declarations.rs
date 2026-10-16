//! Declarations: what the author says about a change, in its description or in a file.
//!
//! In a description they are one HTML comment, invisible when the description is rendered:
//!
//! ```text
//! <!-- change-risk
//! change_type: normal
//! risk: high
//! rollback: Revert the merge; the migration is additive.
//! -->
//! ```
//!
//! The block is YAML with three optional keys. A declaration can only raise what the change must
//! meet: a declared `risk` below the computed level, and a declared `standard` or `emergency`
//! change type, are recorded and ignored (see `crate::evaluate`).

use crate::model::{ChangeType, Declarations, Level};
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;

const OPEN: &str = "<!-- change-risk";
const CLOSE: &str = "-->";
const MAX_ROLLBACK: usize = 4000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    change_type: Option<ChangeType>,
    risk: Option<Level>,
    rollback: Option<String>,
}

fn parse(yaml: &str, source: &str) -> Result<Declarations> {
    let block: Block = if yaml.trim().is_empty() {
        Block {
            change_type: None,
            risk: None,
            rollback: None,
        }
    } else {
        serde_saphyr::from_str(yaml).context("declarations are not valid")?
    };
    let rollback = block
        .rollback
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty());
    if let Some(r) = &rollback {
        ensure!(
            r.len() <= MAX_ROLLBACK,
            "the rollback declaration exceeds {MAX_ROLLBACK} bytes"
        );
    }
    Ok(Declarations {
        source: source.into(),
        change_type: block.change_type,
        risk: block.risk,
        rollback,
    })
}

/// The declarations in a change description: None when it has no block. More than one block is
/// an error, because the reader could not tell which one the author meant.
pub fn from_description(text: &str) -> Result<Option<Declarations>> {
    let text = text.replace("\r\n", "\n");
    let mut found = None;
    let mut rest = text.as_str();
    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];
        if !after.starts_with(['\n', ' ', '\t']) {
            rest = after;
            continue;
        }
        let Some(end) = after.find(CLOSE) else {
            bail!("the change-risk block in the description is not closed with -->");
        };
        if found.is_some() {
            bail!("the description has more than one change-risk block");
        }
        found = Some(parse(&after[..end], "description")?);
        rest = &after[end + CLOSE.len()..];
    }
    Ok(found)
}

/// The declarations in a YAML file passed with `--declarations`.
pub fn from_file(text: &str) -> Result<Declarations> {
    parse(text, "file")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_in_a_description() {
        let d = from_description(
            "Fixes login.\r\n\r\n<!-- change-risk\r\nchange_type: normal\r\nrisk: high\r\nrollback: |\r\n  Revert.\r\n-->\r\nThanks",
        )
        .unwrap()
        .unwrap();
        assert_eq!(d.source, "description");
        assert_eq!(d.change_type, Some(ChangeType::Normal));
        assert_eq!(d.risk, Some(Level::High));
        assert_eq!(d.rollback.as_deref(), Some("Revert."));
    }

    #[test]
    fn no_block_is_none_and_lookalikes_are_ignored() {
        assert!(from_description("nothing here").unwrap().is_none());
        assert!(
            from_description("<!-- change-risk-free -->")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn malformed_blocks_are_errors() {
        assert!(from_description("<!-- change-risk\nrisk: high\n").is_err());
        assert!(from_description("<!-- change-risk\nrisk: extreme\n-->").is_err());
        assert!(from_description("<!-- change-risk\nowner: me\n-->").is_err());
        assert!(
            from_description("<!-- change-risk\n-->\n<!-- change-risk\nrisk: low\n-->").is_err()
        );
    }
}
