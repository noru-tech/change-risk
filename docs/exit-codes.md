# Exit codes

`crisk` exits with one of eight stable codes. A code's meaning never changes; a new condition gets a
new code. They are defined once, in `Exit` in [`src/lib.rs`](../src/lib.rs).

| Code | Meaning | Returned by |
| --- | --- | --- |
| [0](#0-success) | Success | every command |
| [1](#1-tier-at-or-above---fail-on) | The tier is at or above `--fail-on` | `assess`, `evaluate`, `render` |
| [2](#2-invalid-command-line-arguments) | Invalid command-line arguments, or a revision git cannot resolve | every command |
| [3](#3-invalid-input) | Invalid input | every command that reads a file |
| [4](#4-collection-incomplete) | Collection incomplete (takes precedence over 1) | `assess`, `evaluate`, `render`, `population` |
| [5](#5-authentication-rejected) | Authentication rejected | `collect`, `assess`, `population` |
| [6](#6-api-permission-rate-limit-or-transport-failure) | API, permission, rate-limit or transport failure | `collect`, `assess`, `population` |
| [7](#7-unsupported-forge-data) | Unsupported forge data | `collect`, `assess`, `population` |

Errors go to stderr as `error: …`, followed for the common ones by `help: …` and `see: <url>`.
None of these ever reaches stdout or a machine-readable output.

## 0: Success

The assessment, facts, document or rendering was written; with `--fail-on`, the tier is below it.

## 1: Tier at or above `--fail-on`

The assessment was written and its tier is at or above the `--fail-on` tier. Tiers order
`low < standard < high < emergency`, so `--fail-on high` also fails an emergency.

## 2: Invalid command-line arguments

An unknown command or flag, a bad value, a `--signal` that is not `KIND=PATH`, no base revision, or
a revision git cannot resolve in the clone (a shallow clone without the base or the head is the
usual cause; check out with full history).

## 3: Invalid input

A facts document, assessment, Statement, map, policy, declarations block or signal file that does not
match its schema or the rules of the specification, or an assessment whose fields do not follow from
its facts. A map or policy that is invalid **at the base revision** is also exit 3: nothing can be
assessed against it until it is fixed on the target branch.

## 4: Collection incomplete

An emergency could not be established (reason [CR010](reasons.md#cr010)): the label events or a
declarer team's membership could not be read in full. The assessment is still written, marked
`complete: false`. For `population`, the listing or a change's label events were incomplete. Exit 4
takes precedence over exit 1.

## 5: Authentication rejected

The forge rejected the token (401). Check it is set, current and valid for that forge.

## 6: API, permission, rate-limit or transport failure

The forge could not be reached, refused the request (403, 429), or the change does not exist. A
forbidden *optional* read, such as a team's members, is not an error: it is recorded as incomplete.

## 7: Unsupported forge data

The forge returned data crisk does not understand, such as a change without commit SHAs.
