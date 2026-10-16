# GitLab

The component in [`templates/change-risk.yml`](../templates/change-risk.yml) assesses every merge
request and reports the reasons as a Code Quality report.

```yaml
include:
  - remote: https://raw.githubusercontent.com/noru-tech/change-risk/v0.2.0/templates/change-risk.yml
    inputs:
      fail_on: ""
      signals: "sarif=gl-sast-report.sarif"
```

It needs a masked CI/CD variable `GITLAB_TOKEN` holding a project access token with `read_api`:
`CI_JOB_TOKEN` cannot read merge request label events, so without it no emergency can be
recognized. `GIT_DEPTH: 0` is set by the component, because the merge base and the target branch
must be in the clone.

| Input | Default | |
| --- | --- | --- |
| `stage` | `test` | |
| `version` | `0.2.0` | The crisk release to install |
| `image` | `alpine:3.20` | Needs `apk`, or `curl`, `git` and `xz` |
| `signals` | | Space-separated `KIND=PATH` |
| `fail_on` | | Fail at or above this tier |
| `max_pages` | `100` | |

Artifacts: `change-risk.json` (the assessment) and `gl-code-quality-report.json`. Code Quality
widgets are available on every GitLab tier; the job log has the full table.

Declarer groups (`group:PATH`) are read with `GET /groups/{id}/members/all`, which includes
inherited members. Self-managed GitLab works through `CI_API_V4_URL`, which every job sets.
