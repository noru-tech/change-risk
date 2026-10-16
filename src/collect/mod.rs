//! Collection: the facts of one change, from a local clone, optionally a forge, and signal files.
//!
//! The map and the policy are read from the **base** revision with `git`, never from the working
//! tree or the change: a change must not be able to choose its own requirements. A change to
//! `.change-risk/` itself is always `high` (see `crate::evaluate`).

pub mod forge;
pub mod git;
pub mod http;

use crate::model::*;
use crate::{Exit, failure, failure_with_hint};
use anyhow::{Context, Result};
use forge::Forge;
use git::Git;
use std::path::PathBuf;

/// Which change to read from a forge.
pub struct ForgeTarget {
    pub forge: Box<dyn Forge>,
    pub number: u64,
}

pub struct Options {
    pub checkout: PathBuf,
    /// The target branch commit; defaults to the forge change's base.
    pub base: Option<String>,
    /// The change's head; defaults to the forge change's head, else `HEAD`.
    pub head: Option<String>,
    pub forge: Option<ForgeTarget>,
    /// Declarations from a file instead of the change's description.
    pub declarations: Option<PathBuf>,
    /// `(kind, path)` signal files.
    pub signals: Vec<(String, PathBuf)>,
    /// RFC 3339 time recorded as when declarer membership was observed.
    pub now: String,
}

fn parse_yaml<T: serde::de::DeserializeOwned>(bytes: &[u8], what: &str, schema: &str) -> Result<T> {
    let text = std::str::from_utf8(bytes).with_context(|| format!("{what} is not UTF-8"))?;
    let value: serde_json::Value =
        serde_saphyr::from_str(text).with_context(|| format!("{what} is not valid YAML"))?;
    crate::schema::check(&value, schema).with_context(|| what.to_string())?;
    serde_json::from_value(value).with_context(|| what.to_string())
}

/// Read a configuration file at the base revision.
fn config<T: serde::de::DeserializeOwned>(
    git: &Git,
    base: &str,
    path: &str,
    schema: &str,
) -> Result<(Option<T>, Source)> {
    let bytes = git.show(base, path)?;
    let source = Source {
        path: path.into(),
        revision: base.into(),
        digest: bytes.as_deref().map(crate::canonical::sha256),
    };
    let value = bytes
        .map(|b| parse_yaml(&b, &format!("{path} at {base}"), schema))
        .transpose()?;
    Ok((value, source))
}

pub fn collect(opts: Options) -> Result<Facts> {
    let git = Git::new(&opts.checkout);
    let remote = opts
        .forge
        .as_ref()
        .map(|t| t.forge.change(t.number))
        .transpose()?;

    let base_rev = opts
        .base
        .clone()
        .or_else(|| remote.as_ref().map(|c| c.base.clone()))
        .ok_or_else(|| {
            failure_with_hint(
                Exit::Usage,
                "no base revision",
                "pass --base (the target branch), or read the change from a forge with --github/--gitlab",
            )
        })?;
    let head_rev = opts
        .head
        .clone()
        .or_else(|| remote.as_ref().map(|c| c.head.clone()))
        .unwrap_or_else(|| "HEAD".into());
    let base = git.resolve(&base_rev)?;
    let head = git.resolve(&head_rev)?;
    let merge_base = git.merge_base(&base, &head)?;
    let paths = git.changed(&merge_base, &head)?;

    let (map, map_source) = config::<Map>(&git, &base, MAP_PATH, "map")?;
    let (policy, policy_source) = config::<Policy>(&git, &base, POLICY_PATH, "policy")?;

    let declarations = match (&opts.declarations, &remote) {
        (Some(path), _) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("unable to read {}", path.display()))?;
            Some(crate::declarations::from_file(&text)?)
        }
        (None, Some(change)) => crate::declarations::from_description(&change.description)
            .context("the change's description")?,
        (None, None) => None,
    };

    let mut labels = None;
    let mut teams = Vec::new();
    if let Some(target) = &opts.forge {
        labels = Some(target.forge.label_events(target.number)?);
        let declarers = policy
            .as_ref()
            .and_then(|p| p.emergency.as_ref())
            .map(|e| e.declarers.clone())
            .unwrap_or_default();
        for d in declarers {
            if teams.iter().any(|t: &Team| t.id == d) {
                continue;
            }
            if let Some(team) = target.forge.team(&d, &opts.now)? {
                teams.push(team);
            }
        }
        teams.sort_by(|a, b| a.id.cmp(&b.id));
    }

    let mut signals = Vec::new();
    for (kind, path) in &opts.signals {
        let bytes =
            std::fs::read(path).with_context(|| format!("unable to read {}", path.display()))?;
        let signal = crate::signals::read(kind, &path.display().to_string(), &bytes, &paths, &head)
            .with_context(|| format!("signal {kind}={}", path.display()))?;
        signals.push(signal);
    }

    let facts = Facts {
        version: VERSION.into(),
        generated: Generated::this(),
        change: Change {
            id: remote.as_ref().map(|c| c.id.clone()),
            url: remote.as_ref().and_then(|c| c.url.clone()),
            base,
            merge_base,
            head,
        },
        paths,
        map,
        policy,
        sources: Sources {
            map: map_source,
            policy: policy_source,
        },
        declarations,
        labels,
        teams,
        signals,
    };
    crate::evaluate::validate_facts(&facts)
        .map_err(|e| failure(Exit::InvalidInput, format!("{e:#}")))?;
    Ok(facts)
}
