# Forges

`crisk` reads a forge only when asked (`--github OWNER/REPO --pr N`, or `--gitlab GROUP/PROJECT --mr
N`), only with GET requests, and only what a tier depends on.

| Read | Why | GitHub | GitLab |
| --- | --- | --- | --- |
| The change | Description (declarations), base and head commits | `GET /repos/{repo}/pulls/{n}` | `GET /projects/{id}/merge_requests/{iid}` |
| Label events | Who applied the emergency label, and when | `GET /repos/{repo}/issues/{n}/events` | `GET /projects/{id}/merge_requests/{iid}/resource_label_events` |
| Declarer members | Whether that person may declare | `GET /orgs/{org}/teams/{slug}/members` | `GET /groups/{id}/members/all` |
| Labelled changes | The [emergency population](emergency.md#the-population) | `GET /repos/{repo}/issues?labels=…&state=closed` | `GET /projects/{id}/merge_requests?labels=…&state=merged` |

Without a forge, `crisk` assesses the commits you give it (`--base`, `--head`), reads declarations
only from `--declarations FILE`, and recognizes no emergency.

## Tokens

| Forge | Variable | Needs |
| --- | --- | --- |
| GitHub | `GITHUB_TOKEN` or `GH_TOKEN` | Pull requests and issues read. Team members need organization members read: the workflow `GITHUB_TOKEN` cannot read them, so use a GitHub App installation token or a fine-grained token with *Members: read* when the policy names `team:` declarers |
| GitLab | `GITLAB_TOKEN` (sent as `PRIVATE-TOKEN`), else `CI_JOB_TOKEN` (`JOB-TOKEN`) | A project or group access token with `read_api`. `CI_JOB_TOKEN` cannot read merge request label events |

A token is read from the environment at the point of use, sent only to an `https` origin, and never
written anywhere. GitHub is always `https://api.github.com`. GitLab is `https://gitlab.com/api/v4`
unless `--gitlab-url` (or `CI_API_V4_URL`, set in every GitLab job) names a self-managed instance.

## When a read is refused

A refused or partial **optional** read is recorded, not fatal: label events or a team's members that
could not be read in full make the facts incomplete, and an emergency that depends on them cannot be
established ([CR010](reasons.md#cr010), exit 4). The change itself must be readable: a missing change
is exit 6.

## What membership means

Team and group membership is recorded **as observed at collection**, with the time. Neither forge
exposes who was a member when a label was applied. If someone leaves a declarer team after applying
the label and before the assessment, the emergency is not recognized; if someone joins in between,
it is. Collect promptly, and keep the declarer list short.
