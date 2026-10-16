# Signals

A signal is another tool's output, handed to `crisk` with `--signal KIND=PATH`. A small, fixed
adapter turns it into labels on the paths the change touches, and nothing else: a signal never
labels a path outside the change. What a label *means* is the policy's business (`levels`).

| Kind | Reads | Labels |
| --- | --- | --- |
| `sarif` | A SARIF 2.1.0 log (any scanner) | `sarif:error`, `sarif:warning`, `sarif:note` on each changed file a result is located in. A result without a level is `warning`; `none` is skipped |
| `privacy-datamap` | A Noru `privacy-datamap` manifest (`.noru/privacy-datamap.yml`) | `personal-data` on each changed file that a collection or privacy declaration with data categories cites (`refs`); also `personal-data:special` when a category is special-category data |
| `acc` | An agent-change-control manifest (JSON or YAML) | On every changed path: `agent-authored` when the change with this head commit has an `agent:` author, `acc:finding` when it has an open finding |
| `threat-model` | A Noru `threat-model` manifest (`.noru/threat-model.yml`) | On each changed path whose map labels name a component (`component:<id>`): `threat-model:expired` when any judgement of a threat on that component expired before the head commit's date, `threat-model:unmodelled` when the model judges no threat on it |

Special categories are the Fideslang categories at or below `user.authorization.biometric`,
`user.biometric`, `user.criminal_history`, `user.demographic.political_opinion`,
`user.demographic.race_ethnicity`, `user.demographic.religious_belief`,
`user.demographic.sexual_orientation` and `user.health_and_medical`: GDPR Article 9 data, and
Article 10 criminal-offence data.

## Threat models

A threat model names components, not paths, so the sensitivity map says which paths belong to
which component, with a label `component:<id>` where `<id>` is the component's id in the model:

```yaml
# .change-risk/map.yml (read from the target branch)
paths:
  - pattern: /infra/assets/
    labels: [infra, "component:assets-bucket"]
  - pattern: /src/session/
    labels: ["component:session"]
```

The `threat-model` adapter reads the manifest the Noru `threat-model` piece writes, where every
judged threat carries an owner and an `expires_at`. A component is **expired** when any judgement of
a threat on it expired before the head commit's committer date (UTC): a judgement stands on its
expiry day. It is **unmodelled** when the model judges no threat on it; a risk whose interpretation
has no expiry date yet, an unsigned skeleton, is not a judgement. The date is recorded on the signal
as `as_of`, so the same change gets the same labels however late it is assessed. Renewing the
judgements in the same change, by committing an updated manifest, is how a change clears them.

```yaml
# .change-risk/policy.yml
levels:
  threat-model:expired: high
  threat-model:unmodelled: standard
```

Each signal is recorded in the facts with its file name and the SHA-256 of its bytes, so an
assessment says exactly which findings it took into account.

```yaml
# .change-risk/policy.yml
levels:
  sarif:error: high
  personal-data: standard
  personal-data:special: high
  agent-authored: standard
```

```bash
crisk assess --base main \
  --signal sarif=semgrep.sarif \
  --signal privacy-datamap=.noru/privacy-datamap.yml
```

A SARIF path is made repository-relative before matching: a `file://` prefix, leading `./` and `/`
are dropped. A result located outside the repository never matches.
