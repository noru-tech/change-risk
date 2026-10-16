//! Facts in, assessment out. Deterministic: no network, no clock, no `git`, no environment.
//!
//! The derivation, in the order its reasons are written (codes in `docs/reasons.md`):
//!
//! 1. A map or policy absent at the base revision falls back to the built-in default (CR009).
//!    An empty diff is noted (CR012).
//! 2. The level starts at the policy's `default_level` (CR008). Each applied label the policy gives
//!    a level raises it to at least that level (CR001). A change to `.change-risk/` is always
//!    `high` (CR002). A declared `risk` can raise the level (CR003) and never lower it (CR004).
//! 3. The change type is `emergency` when the policy's emergency label is on the change and was
//!    last applied by an allowed declarer (CR005); a label applied by anyone else is ignored
//!    (CR006); when the label events or a declarer team's membership are incomplete, the emergency
//!    cannot be established and the assessment is incomplete (CR010). Otherwise `standard` when
//!    every path is in the policy's standard-change catalogue and the level is `low` (CR007), and
//!    `normal` otherwise. A declared `normal` keeps a would-be standard change normal (CR011); a
//!    declared `standard` or `emergency` is ignored (CR004).
//! 4. The tier is `emergency` for an emergency change, and the level otherwise.

use crate::model::*;
use crate::pattern::Pattern;
use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeMap, BTreeSet};

/// The label pattern: lowercase, digits, and `:._-` after the first character.
pub fn valid_label(label: &str) -> bool {
    let mut chars = label.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || ":._-".contains(c))
}

/// Check what a schema cannot: versions, ordering, patterns that compile, labels, and the
/// emergency-only requirement.
pub fn validate_facts(f: &Facts) -> Result<()> {
    ensure!(f.version == VERSION, "facts version must be {VERSION}");
    ensure!(
        f.paths.windows(2).all(|w| w[0] < w[1]),
        "paths must be sorted and unique"
    );
    for p in &f.paths {
        ensure!(
            !p.is_empty() && !p.starts_with('/') && !p.contains('\\'),
            "path {p:?} is not repository-relative"
        );
    }
    if let Some(map) = &f.map {
        validate_map(map)?;
    }
    if let Some(policy) = &f.policy {
        validate_policy(policy)?;
    }
    if let Some(labels) = &f.labels {
        ensure!(
            labels.events.windows(2).all(|w| w[0].at <= w[1].at),
            "label events must be in time order"
        );
    }
    ensure!(
        f.teams.windows(2).all(|w| w[0].id < w[1].id),
        "teams must be sorted by id and unique"
    );
    for t in &f.teams {
        ensure!(
            t.members.windows(2).all(|w| w[0] < w[1]),
            "team {} members must be sorted and unique",
            t.id
        );
    }
    for s in &f.signals {
        for l in &s.labels {
            ensure!(
                valid_label(&l.label),
                "signal label {:?} is invalid",
                l.label
            );
            for p in &l.paths {
                ensure!(
                    f.paths.binary_search(p).is_ok(),
                    "signal {} labels {p}, which the change does not touch",
                    s.file
                );
            }
        }
    }
    Ok(())
}

pub fn validate_map(map: &Map) -> Result<()> {
    ensure!(map.version == VERSION, "map version must be {VERSION}");
    for (i, e) in map.paths.iter().enumerate() {
        Pattern::new(&e.pattern).with_context(|| format!("map entry {}", i + 1))?;
        ensure!(!e.labels.is_empty(), "map entry {} names no label", i + 1);
        for l in &e.labels {
            ensure!(
                valid_label(l),
                "map entry {}: label {l:?} is invalid",
                i + 1
            );
        }
    }
    Ok(())
}

pub fn validate_policy(policy: &Policy) -> Result<()> {
    ensure!(
        policy.version == VERSION,
        "policy version must be {VERSION}"
    );
    for l in policy.levels.keys() {
        ensure!(valid_label(l), "policy level for invalid label {l:?}");
    }
    for c in &policy.standard_changes {
        ensure!(
            !c.paths.is_empty(),
            "standard change {:?} lists no path",
            c.name
        );
        for p in &c.paths {
            Pattern::new(p).with_context(|| format!("standard change {:?}", c.name))?;
        }
    }
    if let Some(e) = &policy.emergency {
        ensure!(!e.label.trim().is_empty(), "emergency label is empty");
        ensure!(!e.declarers.is_empty(), "emergency names no declarer");
        for d in &e.declarers {
            ensure!(
                !d.trim().is_empty() && !d.contains(char::is_whitespace),
                "emergency declarer {d:?} is invalid"
            );
            if let Some(rest) = d.strip_prefix("team:") {
                ensure!(
                    rest.split('/').count() == 2 && !rest.contains("//"),
                    "team declarer {d:?} must be team:ORG/SLUG"
                );
            }
        }
    }
    for (tier, r) in &policy.tiers {
        if r.retroactive_review_within_days.is_some() {
            ensure!(
                *tier == Tier::Emergency,
                "retroactive_review_within_days applies to the emergency tier only"
            );
        }
        if let Some(days) = r.retroactive_review_within_days {
            ensure!(
                days >= 1,
                "retroactive_review_within_days must be at least 1"
            );
        }
    }
    Ok(())
}

fn reason(code: &str, message: String) -> Reason {
    Reason {
        code: code.into(),
        message,
        level: None,
        label: None,
        paths: Vec::new(),
    }
}

/// The labels that apply to the change, from the map and from signals, sorted by label.
pub fn applied_labels(f: &Facts) -> Result<Vec<AppliedLabel>> {
    let mut by_label: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    if let Some(map) = &f.map {
        for entry in &map.paths {
            let pattern = Pattern::new(&entry.pattern)?;
            let hit: Vec<&String> = f.paths.iter().filter(|p| pattern.matches(p)).collect();
            if hit.is_empty() {
                continue;
            }
            for label in &entry.labels {
                let (paths, sources) = by_label.entry(label.clone()).or_default();
                paths.extend(hit.iter().map(|p| (*p).clone()));
                sources.insert("map".into());
            }
        }
    }
    for s in &f.signals {
        for l in &s.labels {
            if l.paths.is_empty() {
                continue;
            }
            let (paths, sources) = by_label.entry(l.label.clone()).or_default();
            paths.extend(l.paths.iter().cloned());
            sources.insert(format!("signal:{}", s.kind));
        }
    }
    Ok(by_label
        .into_iter()
        .map(|(label, (paths, sources))| AppliedLabel {
            label,
            paths: paths.into_iter().collect(),
            sources: sources.into_iter().collect(),
        })
        .collect())
}

/// Whether `actor` (`forge:login`) is `declarer`, or a member of it, given the observed teams.
/// Returns None when that depends on a team whose membership is unknown or incomplete.
pub fn declares(actor: &str, declarer: &str, teams: &[Team]) -> Option<bool> {
    // A GitHub team holds GitHub accounts and a GitLab group GitLab accounts, and nothing else.
    let forge = actor.split_once(':').map_or("", |(forge, _)| forge);
    if (declarer.starts_with("team:") && forge != "github")
        || (declarer.starts_with("group:") && forge != "gitlab")
    {
        return Some(false);
    }
    if declarer.starts_with("team:") || declarer.starts_with("group:") {
        let team = teams.iter().find(|t| t.id == declarer)?;
        if team.members.iter().any(|m| m == actor) {
            return Some(true);
        }
        return team.complete.then_some(false);
    }
    let login = actor.split_once(':').map_or(actor, |(_, login)| login);
    Some(login.eq_ignore_ascii_case(declarer))
}

/// The emergency the facts establish, and whether that could be decided: an accepted emergency, or
/// none, with `false` when the label events or a declarer's membership were incomplete.
fn emergency(f: &Facts, policy: &Policy, reasons: &mut Vec<Reason>) -> (Option<Emergency>, bool) {
    let (Some(rule), Some(labels)) = (&policy.emergency, &f.labels) else {
        return (None, true);
    };
    let last = labels
        .events
        .iter()
        .rfind(|e| e.label.eq_ignore_ascii_case(&rule.label));
    let Some(event) = last.filter(|e| e.action == LabelAction::Added) else {
        if !labels.complete {
            reasons.push(reason(
                "CR010",
                format!(
                    "The label events could not be read in full, so whether `{}` was applied cannot be established.",
                    rule.label
                ),
            ));
            return (None, false);
        }
        return (None, true);
    };
    let verdicts: Vec<Option<bool>> = rule
        .declarers
        .iter()
        .map(|d| declares(&event.actor, d, &f.teams))
        .collect();
    if verdicts.contains(&Some(true)) {
        let days = policy
            .tiers
            .get(&Tier::Emergency)
            .and_then(|r| r.retroactive_review_within_days);
        let mut r = reason(
            "CR005",
            format!(
                "`{}` was applied by {}, who may declare an emergency.",
                rule.label, event.actor
            ),
        );
        r.label = Some(rule.label.clone());
        reasons.push(r);
        return (
            Some(Emergency {
                label: rule.label.clone(),
                declared_by: event.actor.clone(),
                declared_at: event.at.clone(),
                review_within_days: days,
            }),
            true,
        );
    }
    if verdicts.contains(&None) || !labels.complete {
        let mut r = reason(
            "CR010",
            format!(
                "`{}` was applied by {}, and whether they may declare an emergency cannot be established: a declarer team's membership or the label events could not be read in full.",
                rule.label, event.actor
            ),
        );
        r.label = Some(rule.label.clone());
        reasons.push(r);
        return (None, false);
    }
    let mut r = reason(
        "CR006",
        format!(
            "`{}` was applied by {}, who may not declare an emergency; it is ignored.",
            rule.label, event.actor
        ),
    );
    r.label = Some(rule.label.clone());
    reasons.push(r);
    (None, true)
}

/// Evaluate `f` into an assessment.
pub fn evaluate(f: Facts) -> Result<Assessment> {
    validate_facts(&f)?;
    let mut reasons = Vec::new();
    if f.map.is_none() {
        reasons.push(reason(
            "CR009",
            format!("No {MAP_PATH} at the base revision; no path carries a label from the map."),
        ));
    }
    if f.policy.is_none() {
        reasons.push(reason(
            "CR009",
            format!("No {POLICY_PATH} at the base revision; the built-in policy applies."),
        ));
    }
    let policy = f.policy.clone().unwrap_or_else(Policy::builtin);
    if f.paths.is_empty() {
        reasons.push(reason("CR012", "The change touches no path.".into()));
    }

    let mut level = policy.default_level;
    let mut r = reason(
        "CR008",
        format!("The policy's default level is {}.", level.as_str()),
    );
    r.level = Some(level);
    reasons.push(r);

    let labels = applied_labels(&f)?;
    for l in &labels {
        if let Some(&set) = policy.levels.get(&l.label) {
            let mut r = reason(
                "CR001",
                format!("Label `{}` sets the level to {}.", l.label, set.as_str()),
            );
            r.level = Some(set);
            r.label = Some(l.label.clone());
            r.paths = l.paths.clone();
            reasons.push(r);
            level = level.max(set);
        }
    }

    let own: Vec<String> = f
        .paths
        .iter()
        .filter(|p| p.starts_with(CONFIG_DIR))
        .cloned()
        .collect();
    if !own.is_empty() {
        let mut r = reason(
            "CR002",
            format!(
                "The change edits {CONFIG_DIR}, which decides its own requirements; it is high."
            ),
        );
        r.level = Some(Level::High);
        r.paths = own;
        reasons.push(r);
        level = Level::High;
    }

    let declared = f.declarations.as_ref();
    if let Some(risk) = declared.and_then(|d| d.risk) {
        if risk > level {
            let mut r = reason(
                "CR003",
                format!("The declared risk {} raises the level.", risk.as_str()),
            );
            r.level = Some(risk);
            reasons.push(r);
            level = risk;
        } else if risk < level {
            reasons.push(reason(
                "CR004",
                format!(
                    "The declared risk {} is ignored: a declaration can raise the level, never lower it.",
                    risk.as_str()
                ),
            ));
        }
    }

    let (emergency, complete) = emergency(&f, &policy, &mut reasons);
    let declared_type = declared.and_then(|d| d.change_type);
    match declared_type {
        Some(ChangeType::Standard) => reasons.push(reason(
            "CR004",
            "The declared change type standard is ignored: only the policy's standard-change catalogue makes a change standard.".into(),
        )),
        Some(ChangeType::Emergency) if emergency.is_none() => reasons.push(reason(
            "CR004",
            "The declared change type emergency is ignored: an emergency is declared with the policy's label, by someone allowed to.".into(),
        )),
        _ => {}
    }

    let change_type = if emergency.is_some() {
        ChangeType::Emergency
    } else {
        let catalogue: Vec<(String, Vec<Pattern>)> = policy
            .standard_changes
            .iter()
            .map(|c| {
                Ok((
                    c.name.clone(),
                    c.paths
                        .iter()
                        .map(|p| Pattern::new(p))
                        .collect::<Result<Vec<_>>>()?,
                ))
            })
            .collect::<Result<_>>()?;
        let mut used = BTreeSet::new();
        let all_standard = !f.paths.is_empty()
            && f.paths.iter().all(|p| {
                catalogue
                    .iter()
                    .find(|(_, pats)| pats.iter().any(|pat| pat.matches(p)))
                    .map(|(name, _)| used.insert(name.clone()))
                    .is_some()
            });
        if all_standard && level == Level::Low {
            if declared_type == Some(ChangeType::Normal) {
                reasons.push(reason(
                    "CR011",
                    "Every path is in the standard-change catalogue, but the change is declared normal.".into(),
                ));
                ChangeType::Normal
            } else {
                reasons.push(reason(
                    "CR007",
                    format!(
                        "Every path is in the standard-change catalogue ({}).",
                        used.into_iter().collect::<Vec<_>>().join(", ")
                    ),
                ));
                ChangeType::Standard
            }
        } else {
            ChangeType::Normal
        }
    };

    let tier = if change_type == ChangeType::Emergency {
        Tier::Emergency
    } else {
        Tier::from_level(level)
    };
    let requirements = policy.tiers.get(&tier).cloned();
    let facts_digest = crate::canonical::digest(&f)?;
    Ok(Assessment {
        version: VERSION.into(),
        generated: Generated::this(),
        change: f.change.clone(),
        level,
        change_type,
        tier,
        complete,
        labels,
        reasons,
        emergency,
        requirements,
        facts_digest,
        facts: f,
    })
}

/// Validate an assessment: its facts validate, and evaluating them again reproduces every derived
/// field. `generated` may differ (another implementation, another version).
pub fn validate_assessment(a: &Assessment) -> Result<()> {
    ensure!(a.version == VERSION, "assessment version must be {VERSION}");
    let mut again = evaluate(a.facts.clone()).context("the embedded facts")?;
    again.generated = a.generated.clone();
    if again != *a {
        for (name, ok) in [
            ("level", again.level == a.level),
            ("change_type", again.change_type == a.change_type),
            ("tier", again.tier == a.tier),
            ("complete", again.complete == a.complete),
            ("labels", again.labels == a.labels),
            ("reasons", again.reasons == a.reasons),
            ("emergency", again.emergency == a.emergency),
            ("requirements", again.requirements == a.requirements),
            ("facts_digest", again.facts_digest == a.facts_digest),
            ("change", again.change == a.change),
        ] {
            if !ok {
                bail!("`{name}` does not follow from the embedded facts");
            }
        }
        bail!("the assessment does not follow from the embedded facts");
    }
    Ok(())
}
