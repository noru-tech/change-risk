# change-risk

> How risky is this change, and which controls must it therefore meet? Deterministically, from
> declared facts.

[![ci](https://github.com/noru-tech/change-risk/actions/workflows/ci.yml/badge.svg)](https://github.com/noru-tech/change-risk/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`crisk` gives a source change a **tier** (`low`, `standard`, `high` or `emergency`) and the
**reasons** for it, from:

- the **paths** it touches, read from git between the merge base and the head;
- a **sensitivity map** and a **tier policy** in `.change-risk/`, read from the **target branch**,
  never from the change, so a change cannot lower its own bar;
- what the author **declared** (ITIL `change_type`, `risk`, `rollback`), which can only raise it;
- an **emergency label** on the pull or merge request, recognized only when an allowed declarer
  applied it;
- **signals** from other tools: SARIF findings, a privacy data map, an agent-change-control manifest.

No content analysis and no model: the same facts always give the same tier and the same reasons.
The result is an in-toto predicate that
[agent-change-control](https://github.com/noru-tech/agent-change-control) reads to apply each
tier's requirements, plus SARIF, GitLab Code Quality, a requirements document for enforcement tools
and the policy as prose for auditors.

```console
$ crisk assess --base main
tier      high (level high, normal change)
change    -  base 1a7a1b9754e8  head 194da29db944
labels    auth (1), migrations (1)
reasons
  CR008  The policy's default level is low.
  CR001  Label `auth` sets the level to high.
  CR001  Label `migrations` sets the level to standard.
requires  2 human approval(s); review by acme/security
```

## Install

```bash
cargo install change-risk --locked
```

Prebuilt binaries for Linux (static musl) and macOS are attached to every
[release](https://github.com/noru-tech/change-risk/releases), with SHA-256 checksums and GitHub
artifact attestations. Verify before you run:

```bash
gh attestation verify change-risk-x86_64-unknown-linux-musl.tar.xz --repo noru-tech/change-risk
```

## Quick start

1. Write a [map and a policy](docs/map-and-policy.md) (start from [`examples/`](examples/)) and
   merge them to the target branch.
2. Assess a branch: `crisk assess --base main`.
3. In CI: the [GitHub Action](docs/github-action.md) or the [GitLab component](docs/gitlab.md).

## Commands

| Command | What it does |
| --- | --- |
| `crisk assess [--github OWNER/REPO --pr N \| --gitlab GROUP/PROJECT --mr N] [--base REV] [--head REV]` | Collect and evaluate one change |
| `crisk collect …` | Write the facts only |
| `crisk evaluate FACTS` | Evaluate facts offline, deterministically |
| `crisk render ASSESSMENT -f FORMAT` | Validate an assessment and render it as `json`, `table`, `markdown`, `in-toto`, `sarif` or `codequality` |
| `crisk validate FILE` | Validate facts, an assessment, a Statement, a map, a policy, a requirements document or a population |
| `crisk compile --to acc\|requirements\|prose` | The policy as acc's tier section, a requirements document, or prose |
| `crisk population github\|gitlab REPO --since D --until D` | Every emergency change merged in a window |
| `crisk completions SHELL` / `crisk manpage --out-dir DIR` | Shell completions and man pages |

Common flags: `--signal KIND=PATH` (repeatable), `--declarations FILE`, `--fail-on TIER`,
`-f/--format`, `-o/--output`, `--checkout DIR`, `-q`.

## Exit codes

`0` ok · `1` tier at or above `--fail-on` · `2` usage · `3` invalid input · `4` incomplete: an
emergency could not be established (beats 1) · `5` authentication rejected · `6` API failure · `7`
unsupported forge data. See [docs/exit-codes.md](docs/exit-codes.md).

## Documentation

- [Specification](spec/change-risk.md): inputs, the derivation, the predicate, conformance
- [Map and policy](docs/map-and-policy.md) · [Reason codes](docs/reasons.md) · [Signals](docs/signals.md)
- [Forges and tokens](docs/forges.md) · [Emergency changes](docs/emergency.md) · [Outputs](docs/outputs.md)
- [GitHub Action](docs/github-action.md) · [GitLab component](docs/gitlab.md)
- [Conformance corpus](conformance/README.md) · [Schemas](schemas/)

## Development

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
python3 conformance/run.py --verifier "target/debug/crisk -q evaluate -f json"
```

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE). Copyright (c) 2026 Noru.
