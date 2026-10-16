# Signals

A signal is another tool's output, handed to `crisk` with `--signal KIND=PATH`. A small, fixed
adapter turns it into labels on the paths the change touches, and nothing else: a signal never
labels a path outside the change. What a label *means* is the policy's business (`levels`).

| Kind | Reads | Labels |
| --- | --- | --- |
| `sarif` | A SARIF 2.1.0 log (any scanner) | `sarif:error`, `sarif:warning`, `sarif:note` on each changed file a result is located in. A result without a level is `warning`; `none` is skipped |
| `privacy-datamap` | A Noru `privacy-datamap` manifest (`.noru/privacy-datamap.yml`) | `personal-data` on each changed file that a collection or privacy declaration with data categories cites (`refs`); also `personal-data:special` when a category is special-category data |
| `acc` | An agent-change-control manifest (JSON or YAML) | On every changed path: `agent-authored` when the change with this head commit has an `agent:` author, `acc:finding` when it has an open finding |

Special categories are the Fideslang categories at or below `user.authorization.biometric`,
`user.biometric`, `user.criminal_history`, `user.demographic.political_opinion`,
`user.demographic.race_ethnicity`, `user.demographic.religious_belief`,
`user.demographic.sexual_orientation` and `user.health_and_medical`: GDPR Article 9 data, and
Article 10 criminal-offence data.

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
