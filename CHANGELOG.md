# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-16

### Added
- The `threat-model` signal (`--signal threat-model=.noru/threat-model.yml`): a Noru
  `threat-model` manifest, judged as of the head commit's committer date. A changed path whose map
  labels name a component (`component:<id>`) carries `threat-model:expired` when a judgement of a
  threat on that component expired before that date, and `threat-model:unmodelled` when the model
  judges no threat on it. The map, read from the base revision, is what ties paths to components.
- A signal records the date it judged dated facts against, `as_of` (optional; the facts,
  assessment and Statement schemas accept it). Specification §2.7, §2.7.2.
- Conformance vector `accept/24-threat-model-signal`.

## [0.1.0] - 2026-10-16

### Added
- First version: the change-risk 0.1 specification, schemas and conformance corpus, and `crisk`.
  - `collect`, `evaluate` and `assess`: a tier (low, standard, high, emergency) for one change from
    the paths it touches (git, from the merge base), a sensitivity map and tier policy read from the
    base revision, declarations that can only raise it, an emergency label applied by an allowed
    declarer (GitHub and GitLab label events, team and group membership), and labels from SARIF,
    privacy data map and agent-change-control signals.
  - Outputs: the assessment (RFC 8785 JSON), a table, Markdown, an in-toto Statement, SARIF 2.1.0
    and GitLab Code Quality; `render` and `validate`.
  - `compile`: agent-change-control's tier section, a requirements document, and the policy as prose.
  - `population`: the emergency changes merged in a window, who declared them and when their
    retroactive review is due.
  - A GitHub Action and a GitLab CI/CD component.

[Unreleased]: https://github.com/noru-tech/change-risk/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/noru-tech/change-risk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/noru-tech/change-risk/releases/tag/v0.1.0
