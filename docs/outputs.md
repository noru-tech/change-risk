# Outputs

| Command | Writes |
| --- | --- |
| `crisk collect` | The [facts](../schemas/facts.schema.json): everything read, nothing derived |
| `crisk evaluate FACTS` / `crisk assess` | The assessment, in any format below |
| `crisk render ASSESSMENT` | An assessment or Statement in another format, after validating it |
| `crisk compile --to acc\|requirements\|prose` | The policy for agent-change-control, for enforcement tools, or for people |
| `crisk population` | The [emergency population](emergency.md#the-population) |

JSON is always RFC 8785 bytes without a trailing newline, so its digest is reproducible.

## Assessment formats

`--format`, or inferred from the `--output` name:

| Format | Name | What |
| --- | --- | --- |
| `json` | `*.json` | The assessment ([schema](../schemas/assessment.schema.json)), with its facts embedded |
| `in-toto` | `*.intoto.json` | An unsigned in-toto Statement v1: subject the head commit, predicate the assessment |
| `sarif` | `*.sarif` | SARIF 2.1.0: one result per reason that names paths, `warning` for a reason that sets `high`, `note` otherwise; the tier in the run's properties |
| `codequality` | `gl-code-quality-report.json` | GitLab Code Quality: one issue per reason and path |
| `markdown` | `*.md` | A summary for a job summary or a merge request note |
| `table` | (default) | A summary for a terminal |

`crisk validate` checks any of them that is JSON by re-evaluating the embedded facts: an assessment
whose tier, labels or reasons do not follow from its facts is invalid.

## Signing

Sign the Statement with the signer you already use. For GitHub artifact attestations, attest the
JSON assessment file as both subject and predicate (`actions/attest` with `subject-path` and
`predicate-path`, predicate type `https://noru.tech/spec/ai-change-provenance/change-risk/v0.1`); with
cosign, `cosign attest-blob --statement change.intoto.json`.

## Requirements document

`crisk compile --to requirements` ([schema](../schemas/requirements.schema.json)) restates the
policy for tools that enforce it: per tier, the requirements and the map patterns that route a change
to that tier; the emergency label, declarers and window; and `missing_tier`. A ruleset or CODEOWNERS
generator reads `protected_paths` and `required_reviewer_teams` from it; `crisk` writes neither.

## acc's tier section

`crisk compile --to acc` writes the `tiers` and `missing_tier` section of an agent-change-control
policy from `.change-risk/policy.yml`, so the requirements acc enforces per tier are the ones this
policy states. `required_checks` is not part of it: acc requires checks through its own `checks`
block.

## Prose

`crisk compile --to prose` renders the policy and map as Markdown, from a fixed template: the same
files always give the same text. It is meant for the people who approve the policy and the auditors
who read it, so the written policy and the enforced one are the same file.
