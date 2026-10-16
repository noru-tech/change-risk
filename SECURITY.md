# Security Policy

## Supported versions

This project is pre-1.0; security fixes are applied to the latest release on the `main` branch.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |

## Reporting a vulnerability

Please report security issues **privately**: do not open a public issue for an unfixed
vulnerability.

- Email: **security@noru.tech** with a subject line beginning `[SECURITY] change-risk`.
- Or use GitHub **private vulnerability reporting**:
  <https://github.com/noru-tech/change-risk/security/advisories/new> (Security →
  *Report a vulnerability*).

Please include a description of the issue, the affected version or commit, reproduction steps or a
proof of concept, and the impact you foresee. Do not put tokens or private repository data in a
report unless they are essential to demonstrate the issue.

We aim to acknowledge reports within **5 business days** and to provide a remediation timeline after
triage. We will credit reporters who wish to be named once a fix is released.

## Scope and threat model

`crisk` is a local command-line tool. It reads a local clone with `git`, optionally a forge's REST
API, and files the caller names; it evaluates the result offline.

- **A change cannot choose its own requirements.** The sensitivity map and the tier policy are read
  from the base revision with `git`, never from the working tree or the change. A change to
  `.change-risk/` is always `high`. Declarations can only raise a change's level, and an emergency
  is only recognized from a forge label event whose actor is an allowed declarer.
- Forge responses, repository contents and signal files are untrusted data. Nothing in them is
  executed. Revisions that could be read as `git` options or ranges are refused.
- Forge requests are GET only, with redirects disabled, bounded pages and response sizes, and fixed
  error messages rather than response bodies or headers. Tokens (`GITHUB_TOKEN`, `GH_TOKEN`,
  `GITLAB_TOKEN`, `CI_JOB_TOKEN`) are read from the environment at the point of use, sent only to an
  `https` origin, and never serialized or logged. Test-only loopback origins refuse tokens.
- Input files are limited to 32 MiB and parsed as data with `serde_json` or `serde-saphyr`. JSON
  Schemas are embedded; remote schema resolution is disabled.
- `evaluate`, `render`, `validate` and `compile --policy` perform no network calls and read no
  `git`. Only `collect`, `assess` and `population` contact a forge, and only when asked to.
- Release binaries are built by GitHub Actions from tagged commits, published with SHA-256 checksums
  and GitHub artifact attestations.

An assessment is evidence of what the facts were and what follows from them. It is not a signature:
sign it with the DSSE signer you already trust, and verify the signature before relying on it.
Team membership is observed at collection time, not at the time a label was applied.
