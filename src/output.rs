//! Rendering an assessment: JSON (the predicate), a table, Markdown, an in-toto Statement, SARIF
//! 2.1.0 and GitLab Code Quality. JSON outputs are RFC 8785 bytes with no trailing newline, so
//! their digests are reproducible from the value alone.

use crate::model::*;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

pub const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// The assessment, as RFC 8785 JSON.
    Json,
    /// A summary for people.
    Table,
    /// A summary in Markdown, for a job summary or a merge request note.
    Markdown,
    /// An unsigned in-toto Statement v1 whose subject is the head commit.
    InToto,
    /// SARIF 2.1.0, one result per reason that names paths.
    Sarif,
    /// GitLab Code Quality, one issue per reason and path.
    Codequality,
}

impl Format {
    /// The format an output file name implies.
    pub fn infer(path: &std::path::Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?.to_ascii_lowercase();
        Some(if name.ends_with(".intoto.json") {
            Format::InToto
        } else if name.ends_with(".sarif") || name.ends_with(".sarif.json") {
            Format::Sarif
        } else if name.ends_with("code-quality-report.json") || name.ends_with(".codequality.json")
        {
            Format::Codequality
        } else if name.ends_with(".json") {
            Format::Json
        } else if name.ends_with(".md") {
            Format::Markdown
        } else {
            return None;
        })
    }
}

pub fn render(a: &Assessment, format: Format) -> Result<String> {
    Ok(match format {
        Format::Json => crate::canonical::jcs_bytes(a)?,
        Format::Table => table(a),
        Format::Markdown => markdown(a),
        Format::InToto => crate::canonical::jcs_bytes(&statement(a)?)?,
        Format::Sarif => crate::canonical::jcs_bytes(&sarif(a))?,
        Format::Codequality => crate::canonical::jcs_bytes(&codequality(a))?,
    })
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

fn requirements_line(r: &TierRequirements) -> String {
    let mut parts = Vec::new();
    if let Some(n) = r.min_human_approvals {
        parts.push(format!("{n} human approval(s)"));
    }
    if let Some(a) = r.agent_review {
        parts.push(format!(
            "agent review {}",
            match a {
                AgentReview::Allowed => "allowed",
                AgentReview::Forbidden => "forbidden",
            }
        ));
    }
    if !r.required_reviewer_teams.is_empty() {
        parts.push(format!(
            "review by {}",
            r.required_reviewer_teams.join(", ")
        ));
    }
    if !r.required_checks.is_empty() {
        parts.push(format!("checks {}", r.required_checks.join(", ")));
    }
    if let Some(d) = r.retroactive_review_within_days {
        parts.push(format!("retroactive review within {d} day(s)"));
    }
    if parts.is_empty() {
        "nothing named".into()
    } else {
        parts.join("; ")
    }
}

fn table(a: &Assessment) -> String {
    let mut out = format!(
        "tier      {} (level {}, {} change){}\n",
        a.tier.as_str(),
        a.level.as_str(),
        a.change_type.as_str(),
        if a.complete { "" } else { ", INCOMPLETE" }
    );
    out.push_str(&format!(
        "change    {}  base {}  head {}\n",
        a.change.id.as_deref().unwrap_or("-"),
        short(&a.change.base),
        short(&a.change.head)
    ));
    if !a.labels.is_empty() {
        let labels: Vec<String> = a
            .labels
            .iter()
            .map(|l| format!("{} ({})", l.label, l.paths.len()))
            .collect();
        out.push_str(&format!("labels    {}\n", labels.join(", ")));
    }
    out.push_str("reasons\n");
    for r in &a.reasons {
        out.push_str(&format!("  {}  {}\n", r.code, r.message));
    }
    out.push_str(&format!(
        "requires  {}\n",
        a.requirements
            .as_ref()
            .map_or_else(|| "nothing named for this tier".into(), requirements_line)
    ));
    if let Some(e) = &a.emergency {
        out.push_str(&format!(
            "emergency declared by {} at {}{}\n",
            e.declared_by,
            e.declared_at,
            e.review_within_days
                .map(|d| format!("; retroactive review due {d} day(s) after merge"))
                .unwrap_or_default()
        ));
    }
    out
}

fn markdown(a: &Assessment) -> String {
    let mut out = format!(
        "### Change risk: {}\n\nLevel **{}**, **{}** change{}.\n\n",
        a.tier.as_str(),
        a.level.as_str(),
        a.change_type.as_str(),
        if a.complete {
            ""
        } else {
            " (**incomplete**: an emergency could not be established)"
        }
    );
    out.push_str("| Reason | |\n| --- | --- |\n");
    for r in &a.reasons {
        out.push_str(&format!(
            "| [{}]({}) | {} |\n",
            r.code,
            crate::reason_doc_url(&r.code),
            r.message.replace('|', "\\|")
        ));
    }
    out.push_str(&format!(
        "\nThis tier requires: {}.\n",
        a.requirements
            .as_ref()
            .map_or_else(|| "nothing named".into(), requirements_line)
    ));
    out
}

/// The in-toto Statement: one subject, the head commit, named by the change.
pub fn statement(a: &Assessment) -> Result<Value> {
    Ok(json!({
        "_type": STATEMENT_TYPE,
        "subject": [{
            "name": a.change.id.clone().unwrap_or_else(|| format!("git:{}", a.change.head)),
            "digest": {"gitCommit": a.change.head},
        }],
        "predicateType": PREDICATE_TYPE,
        "predicate": serde_json::to_value(a)?,
    }))
}

/// Validate a Statement: its schema, its predicate, and a subject that is the predicate's head.
pub fn validate_statement(v: &Value) -> Result<Assessment> {
    crate::schema::check(v, "statement")?;
    let a: Assessment = serde_json::from_value(v["predicate"].clone()).context("the predicate")?;
    crate::evaluate::validate_assessment(&a)?;
    ensure!(
        v["subject"][0]["digest"]["gitCommit"].as_str() == Some(a.change.head.as_str()),
        "the statement's subject is not the assessed change's head commit"
    );
    Ok(a)
}

fn sarif(a: &Assessment) -> Value {
    let mut codes: Vec<&str> = a
        .reasons
        .iter()
        .filter(|r| !r.paths.is_empty())
        .map(|r| r.code.as_str())
        .collect();
    codes.sort();
    codes.dedup();
    let rules: Vec<Value> = codes
        .iter()
        .map(|c| {
            json!({
                "id": c,
                "helpUri": crate::reason_doc_url(c),
                "shortDescription": {"text": format!("change-risk reason {c}")},
            })
        })
        .collect();
    let results: Vec<Value> = a
        .reasons
        .iter()
        .filter(|r| !r.paths.is_empty())
        .map(|r| {
            let level = if r.level.is_some_and(|l| l == Level::High) {
                "warning"
            } else {
                "note"
            };
            json!({
                "ruleId": r.code,
                "level": level,
                "message": {"text": format!("{} Tier: {}.", r.message, a.tier.as_str())},
                "locations": r.paths.iter().take(10).map(|p| json!({
                    "physicalLocation": {"artifactLocation": {"uri": p}}
                })).collect::<Vec<_>>(),
                "partialFingerprints": {
                    "changeRisk/v1": crate::canonical::sha256(format!("{}\n{}", r.code, r.label.as_deref().unwrap_or("")).as_bytes()),
                },
            })
        })
        .collect();
    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {
                "name": "change-risk",
                "version": crate::version(),
                "informationUri": "https://github.com/noru-tech/change-risk",
                "rules": rules,
            }},
            "results": results,
            "properties": {
                "tier": a.tier.as_str(),
                "level": a.level.as_str(),
                "changeType": a.change_type.as_str(),
                "complete": a.complete,
            },
        }],
    })
}

fn codequality(a: &Assessment) -> Value {
    let mut issues = Vec::new();
    for r in a.reasons.iter().filter(|r| !r.paths.is_empty()) {
        let severity = match r.level {
            Some(Level::High) => "major",
            Some(Level::Standard) => "minor",
            _ => "info",
        };
        for p in &r.paths {
            issues.push(json!({
                "description": format!("{} ({}; tier {})", r.message, r.code, a.tier.as_str()),
                "check_name": format!("change-risk/{}", r.code),
                "fingerprint": crate::canonical::sha256(format!("{}\n{}\n{}", r.code, r.label.as_deref().unwrap_or(""), p).as_bytes())[7..39].to_string(),
                "severity": severity,
                "location": {"path": p, "lines": {"begin": 1}},
            }));
        }
    }
    Value::Array(issues)
}
