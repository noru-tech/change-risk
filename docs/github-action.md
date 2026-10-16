# GitHub Action

```yaml
on:
  pull_request:
    types: [opened, synchronize, reopened, labeled, unlabeled, edited]

permissions:
  contents: read
  pull-requests: read
  issues: read
  security-events: write   # upload-sarif

jobs:
  change-risk:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
        with:
          fetch-depth: 0          # the merge base and the base branch must be in the clone
      - id: risk
        uses: noru-tech/change-risk@v0.1.0
        with:
          fail-on: ""             # or `emergency`, to make an emergency visible as a failed check
          sarif: change-risk.sarif
      - uses: github/codeql-action/upload-sarif@v4
        with:
          sarif_file: change-risk.sarif
          category: change-risk
      - run: echo "tier ${{ steps.risk.outputs.tier }}"
```

Run it on `labeled` and `unlabeled` too: applying the emergency label changes the tier. Pin both
actions to commit SHAs in production.

| Input | Default | |
| --- | --- | --- |
| `version` | `0.1.0` | The crisk release to install |
| `verify-attestation` | `true` | Verify the release archive with `gh attestation verify` first |
| `pr` | the event's pull request | Empty reads no forge |
| `repository` | `github.repository` | |
| `base`, `head` | the pull request's | Revisions in the clone |
| `token` | `github.token` | Team declarers need *Members: read*, which the workflow token cannot have; see [forges](forges.md#tokens) |
| `signals` | | One `KIND=PATH` per line |
| `declarations` | | A YAML file instead of the description |
| `fail-on` | | Fail at or above this tier |
| `output` | `change-risk.json` | The assessment |
| `sarif` | | Also write SARIF here |
| `summary` | `true` | Append the assessment to the job summary |
| `max-pages` | `100` | |

Outputs: `tier`, `level`, `change-type`, `complete`, `exit-code`, `assessment`.

The action never fails on the tier unless `fail-on` says so; it fails on exit 4 (an emergency that
could not be established) and on errors. Feed the assessment to agent-change-control as an
attestation to have it apply the tier's requirements.
