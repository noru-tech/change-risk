//! The public data model. Every type here mirrors a JSON Schema under `schemas/`; the schema is
//! the contract, these types are how `crisk` reads and writes it.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The specification version these types implement.
pub const VERSION: &str = "0.1";

/// The in-toto predicate type of an assessment.
pub const PREDICATE_TYPE: &str = "https://noru.tech/spec/ai-change-provenance/change-risk/v0.1";

/// Where the map and the policy live, in the base revision.
pub const MAP_PATH: &str = ".change-risk/map.yml";
pub const POLICY_PATH: &str = ".change-risk/policy.yml";
/// A change under this directory changes its own requirements, so it is always `high`.
pub const CONFIG_DIR: &str = ".change-risk/";

/// How risky a change is, from its paths, signals and declarations. Ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Low,
    Standard,
    High,
}

impl Level {
    pub const fn as_str(self) -> &'static str {
        match self {
            Level::Low => "low",
            Level::Standard => "standard",
            Level::High => "high",
        }
    }
}

/// The ITIL change type. Only a policy catalogue makes a change `standard`, and only a forge label
/// applied by an allowed declarer makes it `emergency`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Standard,
    Normal,
    Emergency,
}

impl ChangeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            ChangeType::Standard => "standard",
            ChangeType::Normal => "normal",
            ChangeType::Emergency => "emergency",
        }
    }
}

/// The tier requirements are chosen by: the level, or `emergency`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Low,
    Standard,
    High,
    Emergency,
}

impl Tier {
    pub const ALL: [Tier; 4] = [Tier::Low, Tier::Standard, Tier::High, Tier::Emergency];

    pub const fn as_str(self) -> &'static str {
        match self {
            Tier::Low => "low",
            Tier::Standard => "standard",
            Tier::High => "high",
            Tier::Emergency => "emergency",
        }
    }

    pub const fn from_level(level: Level) -> Self {
        match level {
            Level::Low => Tier::Low,
            Level::Standard => Tier::Standard,
            Level::High => Tier::High,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Map and policy: the two files read from the base revision.

/// `.change-risk/map.yml`: which paths carry which sensitivity labels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Map {
    pub version: String,
    #[serde(default)]
    pub paths: Vec<MapEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapEntry {
    /// A CODEOWNERS-style pattern (see `docs/map.md`).
    pub pattern: String,
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// `.change-risk/policy.yml`: what labels mean, which changes are standard, who may declare an
/// emergency, and what each tier requires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: String,
    /// The level of a change no label raises. Default `standard`.
    #[serde(default = "default_level")]
    pub default_level: Level,
    /// The level each label sets. A label not listed sets nothing.
    #[serde(default)]
    pub levels: BTreeMap<String, Level>,
    /// Pre-approved changes: a change whose every path matches one entry, at level `low`, is a
    /// standard change.
    #[serde(default)]
    pub standard_changes: Vec<StandardChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emergency: Option<EmergencyPolicy>,
    /// What each tier requires. Data for acc and for enforcement, not enforced here.
    #[serde(default)]
    pub tiers: BTreeMap<Tier, TierRequirements>,
    /// The tier acc applies to a change with no assessment.
    #[serde(default = "default_missing_tier")]
    pub missing_tier: Tier,
}

fn default_level() -> Level {
    Level::Standard
}

fn default_missing_tier() -> Tier {
    Tier::Standard
}

impl Policy {
    /// The built-in policy for a repository with no `.change-risk/policy.yml` at its base.
    pub fn builtin() -> Self {
        Self {
            version: VERSION.into(),
            default_level: Level::Standard,
            levels: BTreeMap::new(),
            standard_changes: Vec::new(),
            emergency: None,
            tiers: BTreeMap::new(),
            missing_tier: Tier::Standard,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandardChange {
    pub name: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmergencyPolicy {
    /// The forge label that declares an emergency.
    pub label: String,
    /// Who may apply it: forge logins (`alice`), GitHub teams (`team:ORG/SLUG`) or GitLab groups
    /// (`group:PATH`).
    pub declarers: Vec<String>,
}

/// What a tier requires. Every field is optional; acc reads the ones it enforces.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TierRequirements {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_human_approvals: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_review: Option<AgentReview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_reviewer_teams: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retroactive_review_within_days: Option<u32>,
}

/// Whether an independent agent review may stand in for a human approval in this tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentReview {
    Allowed,
    Forbidden,
}

// ---------------------------------------------------------------------------------------------
// Facts: what `collect` read, and all `evaluate` sees.

/// Everything known about one change, and nothing derived from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: String,
    pub generated: Generated,
    pub change: Change,
    /// The paths the change adds, modifies or deletes, sorted and unique. A rename is a delete and
    /// an add.
    pub paths: Vec<String>,
    /// The sensitivity map as it stood at the base revision, or null when there was none.
    pub map: Option<Map>,
    /// The tier policy as it stood at the base revision, or null when there was none.
    pub policy: Option<Policy>,
    pub sources: Sources,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declarations: Option<Declarations>,
    /// Label events on the change, oldest first. Null when no forge was read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<LabelEvents>,
    /// Declarer teams and groups, as their membership was observed at collection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub teams: Vec<Team>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<Signal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generated {
    pub tool: String,
    pub version: String,
}

impl Generated {
    pub fn this() -> Self {
        Self {
            tool: "crisk".into(),
            version: crate::version().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    /// `github:OWNER/REPO:pr:N` or `gitlab:GROUP/PROJECT:mr:N`, when a forge was read.
    pub id: Option<String>,
    pub url: Option<String>,
    /// The commit the map and policy were read from: the target branch.
    pub base: String,
    /// Where the change branched from the base; the diff starts here.
    pub merge_base: String,
    /// The change's head commit; the assessment's subject.
    pub head: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    pub map: Source,
    pub policy: Source,
}

/// Where a configuration file came from, and its digest over the bytes read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: String,
    /// The revision it was read from, or `file` for a file passed on the command line.
    pub revision: String,
    /// `sha256:` over the bytes read; null when the file was absent.
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declarations {
    /// `description` (the change's description on the forge) or `file`.
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_type: Option<ChangeType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<Level>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelEvents {
    /// False when the forge's event listing could not be read in full.
    pub complete: bool,
    pub events: Vec<LabelEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelEvent {
    pub label: String,
    pub action: LabelAction,
    /// `github:LOGIN` or `gitlab:USERNAME`.
    pub actor: String,
    /// RFC 3339, UTC.
    pub at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelAction {
    Added,
    Removed,
}

/// A declarer team or group: its members as observed when the facts were collected, which is not
/// necessarily who was a member when the label was applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Team {
    /// As the policy names it: `team:ORG/SLUG` or `group:PATH`.
    pub id: String,
    /// `github:LOGIN` or `gitlab:USERNAME`, sorted.
    pub members: Vec<String>,
    /// False when the membership could not be read in full.
    pub complete: bool,
    pub observed_at: String,
}

/// Labels a signal file contributed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signal {
    /// `sarif`, `privacy-datamap`, `acc` or `threat-model`.
    pub kind: String,
    pub file: String,
    pub digest: String,
    /// The date (`YYYY-MM-DD`, UTC) an adapter judged dated facts against: for `threat-model`, the
    /// head commit's committer date. Absent for adapters that read no dates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    pub labels: Vec<SignalLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalLabel {
    pub label: String,
    /// The changed paths it applies to, sorted.
    pub paths: Vec<String>,
}

// ---------------------------------------------------------------------------------------------
// Assessment: the predicate.

/// The tier of one change and why, with the facts it was derived from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assessment {
    pub version: String,
    pub generated: Generated,
    pub change: Change,
    pub level: Level,
    pub change_type: ChangeType,
    pub tier: Tier,
    /// False when an emergency could not be established because label events or a declarer
    /// team's membership could not be read in full.
    pub complete: bool,
    pub labels: Vec<AppliedLabel>,
    pub reasons: Vec<Reason>,
    pub emergency: Option<Emergency>,
    /// What the policy requires of this tier; null when it names nothing for it.
    pub requirements: Option<TierRequirements>,
    /// `sha256:` over the RFC 8785 bytes of `facts`.
    pub facts_digest: String,
    pub facts: Facts,
}

/// A label that applies to the change, from the map or a signal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedLabel {
    pub label: String,
    /// The changed paths it applies to, sorted.
    pub paths: Vec<String>,
    /// `map`, or `signal:<kind>`, sorted.
    pub sources: Vec<String>,
}

/// One step of the derivation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reason {
    pub code: String,
    pub message: String,
    /// The level this reason sets, when it sets one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Emergency {
    pub label: String,
    pub declared_by: String,
    pub declared_at: String,
    /// How many days after the merge an independent retroactive review is due; null when the
    /// policy names no window.
    pub review_within_days: Option<u32>,
}
