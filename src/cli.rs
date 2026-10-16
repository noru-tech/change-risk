//! The `crisk` command line.

use crate::collect::forge::{Forge, GITHUB_API, GITLAB_API, Github, Gitlab};
use crate::collect::{self, ForgeTarget};
use crate::compile::{self, Inputs, Target};
use crate::model::*;
use crate::output::{self, Format};
use crate::{Exit, exit_for, failure, failure_with_hint};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const LONG_ABOUT: &str = "\
crisk — how risky is this change, and which controls must it meet?

Computes a tier (low, standard, high or emergency) for one change from the paths it touches, a
sensitivity map and a tier policy read from the base revision, declarations that can only raise it,
an emergency label applied by an allowed declarer, and labels from other tools' findings. No
content analysis and no LLM: the same facts always give the same tier and the same reasons.

Only `collect`, `assess` and `population` read `git` or a forge, and only `collect`, `assess` and
`population` read the clock (to record when team membership was observed). `evaluate`, `render`,
`validate` and `compile --policy` touch neither.

Exit codes: 0 ok · 1 tier at or above --fail-on · 2 usage · 3 invalid input · 4 collection
incomplete (beats 1) · 5 authentication rejected · 6 API, permission, rate-limit or transport
failure · 7 unsupported forge data.";

#[derive(Debug, Parser)]
#[command(name = "crisk", version, about = "crisk — change risk tiers from declared facts", long_about = LONG_ABOUT, propagate_version = true)]
pub struct Cli {
    /// Suppress status lines on stderr.
    #[arg(short, long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Collect the facts of one change: paths from git, map and policy from the base revision,
    /// declarations and label events from the forge, labels from signal files.
    Collect(CollectArgs),
    /// Evaluate a facts document into an assessment, offline.
    Evaluate(EvaluateArgs),
    /// Collect and evaluate one change.
    Assess(AssessArgs),
    /// Render an assessment (or a Statement) in another format, after validating it.
    Render(RenderArgs),
    /// Validate a facts document, assessment, Statement, map, policy, requirements document or
    /// emergency population.
    Validate(ValidateArgs),
    /// Compile the tier policy into acc's tier section, a requirements document or prose.
    Compile(CompileArgs),
    /// List the emergency population: merged changes carrying the emergency label in a window.
    Population(PopulationArgs),
    /// Generate shell completions.
    Completions(CompletionsArgs),
    /// Generate man pages.
    Manpage(ManpageArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ForgeArgs {
    /// Read the change from this GitHub repository (OWNER/REPO); requires --pr.
    #[arg(
        long,
        value_name = "OWNER/REPO",
        requires = "pr",
        conflicts_with = "gitlab"
    )]
    pub github: Option<String>,
    /// The pull request number.
    #[arg(long, value_name = "N", requires = "github")]
    pub pr: Option<u64>,
    /// Read the change from this GitLab project (GROUP/PROJECT); requires --mr.
    #[arg(long, value_name = "GROUP/PROJECT", requires = "mr")]
    pub gitlab: Option<String>,
    /// The merge request IID.
    #[arg(long, value_name = "N", requires = "gitlab")]
    pub mr: Option<u64>,
    /// The GitLab API base URL (https only).
    #[arg(long, value_name = "URL", env = "CI_API_V4_URL", default_value = GITLAB_API)]
    pub gitlab_url: String,
    /// Maximum pages per list endpoint (1..=1000).
    #[arg(long, value_name = "N", default_value_t = 100)]
    pub max_pages: usize,
}

#[derive(Debug, Clone, Args)]
pub struct CollectArgs {
    #[command(flatten)]
    pub forge: ForgeArgs,
    /// The target branch commit the map and policy are read from (default: the forge change's
    /// base).
    #[arg(long, value_name = "REV")]
    pub base: Option<String>,
    /// The change's head (default: the forge change's head, else HEAD).
    #[arg(long, value_name = "REV")]
    pub head: Option<String>,
    /// The local clone.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub checkout: PathBuf,
    /// Declarations from a YAML file instead of the change's description.
    #[arg(long, value_name = "FILE")]
    pub declarations: Option<PathBuf>,
    /// A signal file: sarif=PATH, privacy-datamap=PATH or acc=PATH (repeatable).
    #[arg(long, value_name = "KIND=PATH")]
    pub signal: Vec<String>,
    /// Write the facts to FILE instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct OutputArgs {
    /// Output format; inferred from the --output name when omitted, else `table`.
    #[arg(short, long, value_enum)]
    pub format: Option<Format>,
    /// Write to FILE instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// Exit 1 when the tier is at or above TIER (emergency is above high).
    #[arg(long, value_enum, value_name = "TIER")]
    pub fail_on: Option<TierArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TierArg {
    Low,
    Standard,
    High,
    Emergency,
}

impl From<TierArg> for Tier {
    fn from(t: TierArg) -> Self {
        match t {
            TierArg::Low => Tier::Low,
            TierArg::Standard => Tier::Standard,
            TierArg::High => Tier::High,
            TierArg::Emergency => Tier::Emergency,
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct EvaluateArgs {
    /// A facts document written by `collect`.
    pub facts: PathBuf,
    #[command(flatten)]
    pub output: OutputArgs,
}

#[derive(Debug, Clone, Args)]
pub struct AssessArgs {
    #[command(flatten)]
    pub collect: CollectArgs,
    /// Output format; inferred from the --output name when omitted, else `table`.
    #[arg(short, long, value_enum)]
    pub format: Option<Format>,
    /// Exit 1 when the tier is at or above TIER (emergency is above high).
    #[arg(long, value_enum, value_name = "TIER")]
    pub fail_on: Option<TierArg>,
}

#[derive(Debug, Clone, Args)]
pub struct RenderArgs {
    /// An assessment, or an in-toto Statement whose predicate is one.
    pub input: PathBuf,
    #[command(flatten)]
    pub output: OutputArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ValidateArgs {
    pub input: PathBuf,
    /// Which document it is; detected when omitted.
    #[arg(long, value_enum)]
    pub kind: Option<Kind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Kind {
    Facts,
    Assessment,
    Statement,
    Map,
    Policy,
    Requirements,
    Population,
}

#[derive(Debug, Clone, Args)]
pub struct CompileArgs {
    /// What to produce.
    #[arg(long, value_enum)]
    pub to: Target,
    /// The policy file; without it, the policy is read from --base.
    #[arg(long, value_name = "FILE", conflicts_with = "base")]
    pub policy: Option<PathBuf>,
    /// The map file, with --policy.
    #[arg(long, value_name = "FILE", requires = "policy")]
    pub map: Option<PathBuf>,
    /// Read .change-risk/ at this revision of --checkout (default HEAD).
    #[arg(long, value_name = "REV")]
    pub base: Option<String>,
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub checkout: PathBuf,
    /// Write to FILE instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct PopulationArgs {
    /// `github` or `gitlab`.
    #[arg(value_parser = ["github", "gitlab"])]
    pub forge: String,
    /// OWNER/REPO on GitHub, GROUP/PROJECT on GitLab.
    pub repository: String,
    /// First day of the window, inclusive (YYYY-MM-DD, UTC).
    #[arg(long, value_name = "DATE")]
    pub since: String,
    /// Last day of the window, inclusive (YYYY-MM-DD, UTC).
    #[arg(long, value_name = "DATE")]
    pub until: String,
    /// The policy file; without it, the policy is read from --base of --checkout.
    #[arg(long, value_name = "FILE", conflicts_with = "base")]
    pub policy: Option<PathBuf>,
    #[arg(long, value_name = "REV")]
    pub base: Option<String>,
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub checkout: PathBuf,
    #[arg(long, value_name = "URL", env = "CI_API_V4_URL", default_value = GITLAB_API)]
    pub gitlab_url: String,
    #[arg(long, value_name = "N", default_value_t = 100)]
    pub max_pages: usize,
    /// Write to FILE instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct CompletionsArgs {
    pub shell: clap_complete::Shell,
}

#[derive(Debug, Clone, Args)]
pub struct ManpageArgs {
    /// Write one page per command into DIR.
    #[arg(long, value_name = "DIR")]
    pub out_dir: PathBuf,
}

struct Ctx {
    quiet: bool,
}

impl Ctx {
    fn note(&self, msg: impl AsRef<str>) {
        if !self.quiet {
            eprintln!("{}", msg.as_ref());
        }
    }
}

const MAX_INPUT: u64 = 32 * 1024 * 1024;

fn read(path: &Path) -> Result<String> {
    let meta =
        std::fs::metadata(path).with_context(|| format!("unable to read {}", path.display()))?;
    if meta.len() > MAX_INPUT {
        return Err(failure(Exit::InvalidInput, "input exceeds 32 MiB"));
    }
    std::fs::read_to_string(path).with_context(|| format!("unable to read {}", path.display()))
}

fn parse(text: &str) -> Result<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Ok(v);
    }
    serde_saphyr::from_str(text).map_err(|_| anyhow::anyhow!("input is not valid JSON or YAML"))
}

fn write(data: &str, path: Option<&Path>) -> Result<()> {
    match path {
        Some(p) => {
            if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(p, data).with_context(|| format!("unable to write {}", p.display()))
        }
        None => {
            use std::io::Write;
            let mut out = std::io::stdout().lock();
            out.write_all(data.as_bytes())?;
            if !data.ends_with('\n') {
                out.write_all(b"\n")?;
            }
            Ok(())
        }
    }
}

/// When membership was observed: `CRISK_NOW` (tests), else the clock.
fn now() -> Result<String> {
    if let Ok(v) = std::env::var("CRISK_NOW") {
        let t = DateTime::parse_from_rfc3339(&v)
            .map_err(|_| failure(Exit::Usage, "CRISK_NOW is not an RFC 3339 time"))?;
        return Ok(t
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::Secs, true));
    }
    Ok(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn github(repo: &str, max_pages: usize) -> Result<Github> {
    let base = env("CRISK_GITHUB_API_URL").unwrap_or_else(|| GITHUB_API.into());
    let token = env("GITHUB_TOKEN").or_else(|| env("GH_TOKEN"));
    Github::new(repo, token, &base, max_pages)
}

fn gitlab(project: &str, url: &str, max_pages: usize) -> Result<Gitlab> {
    let token = env("GITLAB_TOKEN")
        .map(|t| ("PRIVATE-TOKEN", t))
        .or_else(|| env("CI_JOB_TOKEN").map(|t| ("JOB-TOKEN", t)));
    Gitlab::new(project, token, url, max_pages)
}

fn forge_target(a: &ForgeArgs) -> Result<Option<ForgeTarget>> {
    if let (Some(repo), Some(pr)) = (&a.github, a.pr) {
        return Ok(Some(ForgeTarget {
            forge: Box::new(github(repo, a.max_pages)?),
            number: pr,
        }));
    }
    if let (Some(project), Some(mr)) = (&a.gitlab, a.mr) {
        return Ok(Some(ForgeTarget {
            forge: Box::new(gitlab(project, &a.gitlab_url, a.max_pages)?),
            number: mr,
        }));
    }
    Ok(None)
}

fn signals(specs: &[String]) -> Result<Vec<(String, PathBuf)>> {
    specs
        .iter()
        .map(|s| {
            let (kind, path) = s
                .split_once('=')
                .filter(|(k, p)| !k.is_empty() && !p.is_empty())
                .ok_or_else(|| {
                    failure_with_hint(
                        Exit::Usage,
                        format!("invalid --signal {s:?}"),
                        "use KIND=PATH, for example --signal sarif=results.sarif",
                    )
                })?;
            if !crate::signals::KINDS.contains(&kind) {
                return Err(failure_with_hint(
                    Exit::Usage,
                    format!("unknown signal kind {kind:?}"),
                    format!("use one of {}", crate::signals::KINDS.join(", ")),
                ));
            }
            Ok((kind.to_string(), PathBuf::from(path)))
        })
        .collect()
}

fn collect_facts(a: &CollectArgs) -> Result<Facts> {
    collect::collect(collect::Options {
        checkout: a.checkout.clone(),
        base: a.base.clone(),
        head: a.head.clone(),
        forge: forge_target(&a.forge)?,
        declarations: a.declarations.clone(),
        signals: signals(&a.signal)?,
        now: now()?,
    })
}

fn format_for(format: Option<Format>, output: Option<&Path>) -> Format {
    format
        .or_else(|| output.and_then(Format::infer))
        .unwrap_or(Format::Table)
}

/// Write `a` and decide the exit: 4 when incomplete, else 1 when at or above `fail_on`.
fn finish(
    ctx: &Ctx,
    a: &Assessment,
    format: Format,
    output: Option<&Path>,
    fail_on: Option<TierArg>,
) -> Result<Exit> {
    write(&output::render(a, format)?, output)?;
    ctx.note(format!(
        "Tier {} (level {}, {} change){}",
        a.tier.as_str(),
        a.level.as_str(),
        a.change_type.as_str(),
        if a.complete { "" } else { "; incomplete" }
    ));
    if !a.complete {
        ctx.note("warning: an emergency could not be established; label events or a declarer's membership could not be read in full");
        return Ok(Exit::Incomplete);
    }
    if fail_on.is_some_and(|t| a.tier >= Tier::from(t)) {
        return Ok(Exit::TierExceeded);
    }
    Ok(Exit::Ok)
}

fn load_assessment(path: &Path) -> Result<Assessment> {
    let v = parse(&read(path)?)?;
    if v.get("_type").is_some() {
        return output::validate_statement(&v);
    }
    crate::schema::check(&v, "assessment")?;
    let a: Assessment = serde_json::from_value(v)?;
    crate::evaluate::validate_assessment(&a)?;
    Ok(a)
}

fn detect(v: &Value) -> Option<Kind> {
    if v.get("_type").is_some() {
        return Some(Kind::Statement);
    }
    match v["kind"].as_str() {
        Some("change-risk-requirements") => return Some(Kind::Requirements),
        Some("change-risk-emergency-population") => return Some(Kind::Population),
        _ => {}
    }
    if v.get("facts").is_some() {
        Some(Kind::Assessment)
    } else if v.get("change").is_some() && v.get("paths").is_some() {
        Some(Kind::Facts)
    } else if v.get("paths").is_some() {
        Some(Kind::Map)
    } else if v.get("version").is_some() {
        Some(Kind::Policy)
    } else {
        None
    }
}

fn validate(a: &ValidateArgs) -> Result<&'static str> {
    let v = parse(&read(&a.input)?)?;
    let kind = a.kind.or_else(|| detect(&v)).ok_or_else(|| {
        failure_with_hint(Exit::InvalidInput, "unrecognized document", "pass --kind")
    })?;
    Ok(match kind {
        Kind::Facts => {
            crate::schema::check(&v, "facts")?;
            crate::evaluate::validate_facts(&serde_json::from_value(v)?)?;
            "Valid facts"
        }
        Kind::Assessment => {
            crate::schema::check(&v, "assessment")?;
            crate::evaluate::validate_assessment(&serde_json::from_value(v)?)?;
            "Valid assessment"
        }
        Kind::Statement => {
            output::validate_statement(&v)?;
            "Valid statement"
        }
        Kind::Map => {
            crate::schema::check(&v, "map")?;
            crate::evaluate::validate_map(&serde_json::from_value(v)?)?;
            "Valid map"
        }
        Kind::Policy => {
            crate::schema::check(&v, "policy")?;
            crate::evaluate::validate_policy(&serde_json::from_value(v)?)?;
            "Valid policy"
        }
        Kind::Requirements => {
            crate::schema::check(&v, "requirements")?;
            "Valid requirements document"
        }
        Kind::Population => {
            crate::schema::check(&v, "population")?;
            "Valid emergency population"
        }
    })
}

fn yaml_file<T: serde::de::DeserializeOwned>(path: &Path, schema: &str) -> Result<(T, String)> {
    let text = read(path)?;
    let v: Value = serde_saphyr::from_str(&text).map_err(|_| {
        failure(
            Exit::InvalidInput,
            format!("{} is not valid YAML", path.display()),
        )
    })?;
    crate::schema::check(&v, schema).with_context(|| path.display().to_string())?;
    Ok((
        serde_json::from_value(v)?,
        crate::canonical::sha256(text.as_bytes()),
    ))
}

/// The policy and map, from files or from a revision of a clone.
fn inputs(
    policy: Option<&Path>,
    map: Option<&Path>,
    base: Option<&str>,
    checkout: &Path,
) -> Result<Inputs> {
    if let Some(p) = policy {
        let (policy, policy_digest) = yaml_file::<Policy>(p, "policy")?;
        crate::evaluate::validate_policy(&policy)?;
        let (map, map_digest) = match map {
            Some(m) => {
                let (map, d) = yaml_file::<Map>(m, "map")?;
                crate::evaluate::validate_map(&map)?;
                (Some(map), Some(d))
            }
            None => (None, None),
        };
        return Ok(Inputs {
            policy: Some(policy),
            policy_digest: Some(policy_digest),
            map,
            map_digest,
            origin: "files".into(),
        });
    }
    let git = collect::git::Git::new(checkout);
    let rev = git.resolve(base.unwrap_or("HEAD"))?;
    let read_at = |path: &str, schema: &str| -> Result<(Option<Value>, Option<String>)> {
        match git.show(&rev, path)? {
            Some(bytes) => {
                let text = String::from_utf8(bytes)
                    .map_err(|_| failure(Exit::InvalidInput, format!("{path} is not UTF-8")))?;
                let v: Value = serde_saphyr::from_str(&text).map_err(|_| {
                    failure(Exit::InvalidInput, format!("{path} is not valid YAML"))
                })?;
                crate::schema::check(&v, schema).with_context(|| path.to_string())?;
                Ok((Some(v), Some(crate::canonical::sha256(text.as_bytes()))))
            }
            None => Ok((None, None)),
        }
    };
    let (policy, policy_digest) = read_at(POLICY_PATH, "policy")?;
    let (map, map_digest) = read_at(MAP_PATH, "map")?;
    let policy: Option<Policy> = policy.map(serde_json::from_value).transpose()?;
    let map: Option<Map> = map.map(serde_json::from_value).transpose()?;
    if let Some(p) = &policy {
        crate::evaluate::validate_policy(p)?;
    }
    if let Some(m) = &map {
        crate::evaluate::validate_map(m)?;
    }
    Ok(Inputs {
        policy,
        policy_digest,
        map,
        map_digest,
        origin: format!("revision {}", &rev[..12]),
    })
}

fn day(s: &str, end: bool) -> Result<DateTime<Utc>> {
    let d = s.parse::<NaiveDate>().map_err(|_| {
        failure_with_hint(
            Exit::Usage,
            format!("invalid date: {s}"),
            "use a calendar date such as 2026-09-30",
        )
    })?;
    let t = if end {
        d.and_hms_opt(23, 59, 59)
    } else {
        d.and_hms_opt(0, 0, 0)
    }
    .expect("a valid time of day");
    Ok(t.and_utc())
}

fn run(cli: Cli) -> Result<Exit> {
    let ctx = Ctx { quiet: cli.quiet };
    match cli.command {
        Command::Collect(a) => {
            let facts = collect_facts(&a)?;
            write(&crate::canonical::jcs_bytes(&facts)?, a.output.as_deref())?;
            ctx.note(format!(
                "Collected {} path(s) between {} and {}",
                facts.paths.len(),
                &facts.change.merge_base[..12],
                &facts.change.head[..12]
            ));
            Ok(Exit::Ok)
        }
        Command::Evaluate(a) => {
            let v = parse(&read(&a.facts)?)?;
            crate::schema::check(&v, "facts")?;
            let facts: Facts = serde_json::from_value(v)?;
            let assessment = crate::evaluate::evaluate(facts)?;
            let format = format_for(a.output.format, a.output.output.as_deref());
            finish(
                &ctx,
                &assessment,
                format,
                a.output.output.as_deref(),
                a.output.fail_on,
            )
        }
        Command::Assess(a) => {
            let facts = collect_facts(&a.collect)?;
            let assessment = crate::evaluate::evaluate(facts)?;
            let format = format_for(a.format, a.collect.output.as_deref());
            finish(
                &ctx,
                &assessment,
                format,
                a.collect.output.as_deref(),
                a.fail_on,
            )
        }
        Command::Render(a) => {
            let assessment = load_assessment(&a.input)?;
            let format = format_for(a.output.format, a.output.output.as_deref());
            finish(
                &ctx,
                &assessment,
                format,
                a.output.output.as_deref(),
                a.output.fail_on,
            )
        }
        Command::Validate(a) => {
            let message = validate(&a)?;
            ctx.note(message);
            Ok(Exit::Ok)
        }
        Command::Compile(a) => {
            let inputs = inputs(
                a.policy.as_deref(),
                a.map.as_deref(),
                a.base.as_deref(),
                &a.checkout,
            )?;
            write(&compile::render(&inputs, a.to)?, a.output.as_deref())?;
            Ok(Exit::Ok)
        }
        Command::Population(a) => {
            let since = day(&a.since, false)?;
            let until = day(&a.until, true)?;
            if since > until {
                return Err(failure(Exit::Usage, "--since is after --until"));
            }
            let inputs = inputs(a.policy.as_deref(), None, a.base.as_deref(), &a.checkout)?;
            let policy = inputs.policy.unwrap_or_else(Policy::builtin);
            let forge: Box<dyn Forge> = match a.forge.as_str() {
                "github" => Box::new(github(&a.repository, a.max_pages)?),
                _ => Box::new(gitlab(&a.repository, &a.gitlab_url, a.max_pages)?),
            };
            let repository = format!("{}:{}", forge.name(), a.repository);
            let doc = crate::population::population(
                forge.as_ref(),
                &repository,
                &policy,
                since,
                until,
                &now()?,
            )?;
            let complete = doc["complete"].as_bool().unwrap_or(false);
            write(&crate::canonical::jcs_bytes(&doc)?, a.output.as_deref())?;
            ctx.note(format!(
                "{} emergency change(s) merged between {} and {}",
                doc["changes"].as_array().map_or(0, Vec::len),
                a.since,
                a.until
            ));
            Ok(if complete { Exit::Ok } else { Exit::Incomplete })
        }
        Command::Completions(a) => {
            let mut cmd = Cli::command();
            clap_complete::generate(a.shell, &mut cmd, "crisk", &mut std::io::stdout());
            Ok(Exit::Ok)
        }
        Command::Manpage(a) => {
            std::fs::create_dir_all(&a.out_dir)?;
            clap_mangen::generate_to(Cli::command(), &a.out_dir)?;
            Ok(Exit::Ok)
        }
    }
}

/// Parse the command line, run it, report any error, and return the exit status.
pub fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::from(Exit::Usage.code() as u8)
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match run(cli) {
        Ok(exit) => exit.into(),
        Err(err) => {
            eprintln!("error: {err:#}");
            if let Some(f) = err.downcast_ref::<crate::Failure>()
                && let Some(hint) = &f.hint
            {
                eprintln!("help: {hint}");
                eprintln!("see: {}", f.exit.doc_url());
            }
            exit_for(&err).into()
        }
    }
}
