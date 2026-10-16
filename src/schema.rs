//! Validation against the JSON Schemas under `schemas/`, embedded in the binary.

use anyhow::{Result, bail};
use serde_json::Value;

/// The embedded schema called `name`.
pub fn raw(name: &str) -> Result<&'static str> {
    Ok(match name {
        "map" => include_str!("../schemas/map.schema.json"),
        "policy" => include_str!("../schemas/policy.schema.json"),
        "facts" => include_str!("../schemas/facts.schema.json"),
        "assessment" => include_str!("../schemas/assessment.schema.json"),
        "statement" => include_str!("../schemas/statement.schema.json"),
        "requirements" => include_str!("../schemas/requirements.schema.json"),
        "population" => include_str!("../schemas/population.schema.json"),
        _ => bail!("unknown schema {name}"),
    })
}

/// Validate `value` against schema `name`. The error names the first failing location, never
/// the value found there, which may come from an untrusted file.
pub fn check(value: &Value, name: &str) -> Result<()> {
    let definition: Value = serde_json::from_str(raw(name)?)?;
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&definition)?;
    let mut locations: Vec<String> = validator
        .iter_errors(value)
        .map(|e| e.instance_path().to_string())
        .collect();
    locations.sort();
    if let Some(first) = locations.first() {
        let at = if first.is_empty() { "/" } else { first };
        bail!("does not match the {name} schema at {at}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_schema_compiles() {
        for name in [
            "map",
            "policy",
            "facts",
            "assessment",
            "statement",
            "requirements",
            "population",
        ] {
            let definition: Value = serde_json::from_str(raw(name).unwrap()).unwrap();
            assert!(jsonschema::validator_for(&definition).is_ok(), "{name}");
        }
    }

    #[test]
    fn a_map_is_checked() {
        let ok = serde_json::json!({"version": "0.1", "paths": [{"pattern": "/src/auth/", "labels": ["auth"]}]});
        assert!(check(&ok, "map").is_ok());
        let bad =
            serde_json::json!({"version": "0.1", "paths": [{"pattern": "!x", "labels": ["Auth"]}]});
        let err = check(&bad, "map").unwrap_err().to_string();
        assert!(err.contains("/paths/0/"), "{err}");
    }
}
