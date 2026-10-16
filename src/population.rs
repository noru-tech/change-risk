//! The emergency population: every merged change in a window that carries the policy's emergency
//! label, who applied it, whether they were allowed to, and when its retroactive review is due.
//! It is what an auditor samples from to see how often the emergency path is used.
//!
//! The forge lists changes by the labels they carry *now*. A change whose emergency label was
//! removed after merging is not listed; `docs/emergency.md` says so.

use crate::collect::forge::Forge;
use crate::model::*;
use crate::{Exit, failure_with_hint};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde_json::{Value, json};

fn parse(at: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(at)
        .with_context(|| format!("{at:?} is not an RFC 3339 time"))?
        .with_timezone(&Utc))
}

fn format(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// The population of `forge`'s `repository` merged in `[since, until]`, under `policy`.
pub fn population(
    forge: &dyn Forge,
    repository: &str,
    policy: &Policy,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
    now: &str,
) -> Result<Value> {
    let rule = policy.emergency.as_ref().ok_or_else(|| {
        failure_with_hint(
            Exit::Usage,
            "the policy defines no emergency label",
            "add an `emergency` section to .change-risk/policy.yml",
        )
    })?;
    let days = policy
        .tiers
        .get(&Tier::Emergency)
        .and_then(|r| r.retroactive_review_within_days);
    let mut teams = Vec::new();
    for d in &rule.declarers {
        if let Some(t) = forge.team(d, now)? {
            teams.push(t);
        }
    }
    let (listed, mut complete) = forge.labelled(&rule.label)?;
    let mut changes = Vec::new();
    for (number, merged_at) in listed {
        let merged = parse(&merged_at)?;
        if merged < since || merged > until {
            continue;
        }
        let change = forge.change(number)?;
        let events = forge.label_events(number)?;
        complete &= events.complete;
        let declared = events
            .events
            .iter()
            .filter(|e| e.label.eq_ignore_ascii_case(&rule.label) && e.action == LabelAction::Added)
            .rfind(|e| parse(&e.at).is_ok_and(|t| t <= merged));
        let allowed = declared.map(|e| {
            let verdicts: Vec<Option<bool>> = rule
                .declarers
                .iter()
                .map(|d| crate::evaluate::declares(&e.actor, d, &teams))
                .collect();
            if verdicts.contains(&Some(true)) {
                Some(true)
            } else if verdicts.contains(&None) {
                None
            } else {
                Some(false)
            }
        });
        changes.push(json!({
            "change": change.id,
            "url": change.url,
            "merged_at": format(merged),
            "declared_by": declared.map(|e| e.actor.clone()),
            "declared_at": declared.map(|e| e.at.clone()),
            "allowed": allowed.flatten(),
            "review_within_days": days,
            "review_due": days.map(|d| format(merged + Duration::days(i64::from(d)))),
        }));
    }
    changes.sort_by(|a, b| {
        (a["merged_at"].as_str(), a["change"].as_str())
            .cmp(&(b["merged_at"].as_str(), b["change"].as_str()))
    });
    Ok(json!({
        "version": VERSION,
        "kind": "change-risk-emergency-population",
        "generated": Generated::this(),
        "repository": repository,
        "label": rule.label,
        "window": {"since": format(since), "until": format(until)},
        "complete": complete && teams.iter().all(|t| t.complete),
        "changes": changes,
    }))
}
