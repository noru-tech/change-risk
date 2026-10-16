## What

<!-- One or two sentences on the change. -->

## Why

<!-- The problem, the reason code, or the specification section it relates to. -->

## Checks

- [ ] `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
- [ ] Evaluation or schema changes: specification, schemas, `docs/reasons.md` and the conformance corpus updated together
- [ ] New or changed derivations have a conformance vector, and its `expected` was reviewed, not just regenerated
- [ ] No real repository data or tokens in the diff
