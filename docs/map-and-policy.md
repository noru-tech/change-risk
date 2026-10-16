# The map and the policy

Two files in `.change-risk/`, both read from the **target branch** when a change is assessed, never
from the change. A pull request that edits either is assessed under the versions it would replace,
and is always `high` ([CR002](reasons.md#cr002)). Put `.change-risk/` behind CODEOWNERS.

Validate them with `crisk validate .change-risk/map.yml` and `crisk validate .change-risk/policy.yml`.
Complete examples: [`examples/map.yml`](../examples/map.yml), [`examples/policy.yml`](../examples/policy.yml).

## The sensitivity map

`.change-risk/map.yml` ([schema](../schemas/map.schema.json)) says which paths carry which labels:

```yaml
version: "0.1"
paths:
  - pattern: /src/auth/
    labels: [auth]
  - pattern: /src/billing/
    labels: [payments, personal-data]
  - pattern: /db/migrations/
    labels: [migrations]
  - pattern: "*.tf"
    labels: [infra]
    note: Terraform anywhere in the repository
```

A path carries every label of every entry that matches it. Patterns are the CODEOWNERS subset:

| Pattern | Matches |
| --- | --- |
| `/src/auth/` | everything under `src/auth/` at the root |
| `src/auth` | `src/auth` itself and everything under it, at the root (a `/` inside anchors it) |
| `*.sql` | any `.sql` file at any depth |
| `/src/*.rs` | `.rs` files directly in `src/` |
| `/src/**/*.rs` | `.rs` files anywhere under `src/` |

`!` negation, `[ ]` ranges, `\` escapes and whitespace are refused: CODEOWNERS does not support them
either, and a pattern should not mean two things in two files.

Labels are lowercase: `^[a-z0-9][a-z0-9:._-]*$`. Signals contribute labels with a `:` in them
(`sarif:error`, `personal-data:special`); see [signals](signals.md).

## The tier policy

`.change-risk/policy.yml` ([schema](../schemas/policy.schema.json)):

| Field | Meaning |
| --- | --- |
| `default_level` | The level of a change nothing raises: `low`, `standard` (default) or `high` |
| `levels` | The level each label sets. A label not listed sets nothing |
| `standard_changes` | The standard-change catalogue: named lists of patterns. A change whose every path is in it, at level `low`, is a pre-approved standard change |
| `emergency.label` | The forge label that declares an emergency |
| `emergency.declarers` | Who may apply it: logins, `team:ORG/SLUG` (GitHub), `group:PATH` (GitLab) |
| `tiers` | Per tier (`low`, `standard`, `high`, `emergency`): `min_human_approvals`, `agent_review` (`allowed`, `forbidden`), `required_reviewer_teams`, `required_checks`, and for `emergency` only `retroactive_review_within_days` |
| `missing_tier` | The tier agent-change-control applies to a change with no assessment (default `standard`) |

How a tier follows from these is in the [specification](../spec/change-risk.md#4-evaluation) and,
code by code, in [reasons](reasons.md). In short: the level is the highest of the default, every
label's level, `high` for a change to `.change-risk/`, and a declared risk; the tier is that level,
or `emergency`.

`tiers` is data: `crisk` enforces none of it. agent-change-control reads it (through `crisk compile
--to acc`), and the [requirements document](outputs.md#requirements-document) restates it for
branch rules and CODEOWNERS.

Without a policy on the target branch, the built-in one applies: every change is `standard`, there
is no emergency path and no tier requires anything ([CR009](reasons.md#cr009)).
