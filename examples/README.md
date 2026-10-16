# Examples

| File | What it shows |
| --- | --- |
| `map.yml` | A sensitivity map: CODEOWNERS-style patterns and their labels |
| `policy.yml` | A tier policy with levels, a standard-change catalogue, an emergency path and tier requirements |
| `declarations.yml` | Declarations from a file |
| `github-workflow.yml` | The GitHub Action on every pull request, with SARIF upload |

The GitLab component is [`templates/change-risk.yml`](../templates/change-risk.yml); see
[docs/gitlab.md](../docs/gitlab.md).

Try them without a forge:

```bash
crisk validate examples/policy.yml
crisk compile --to prose --policy examples/policy.yml --map examples/map.yml
crisk compile --to acc --policy examples/policy.yml
```
