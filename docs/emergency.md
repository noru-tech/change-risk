# Emergency changes

An emergency lowers what a change must meet **before** it merges, and opens an obligation: an
independent review **after** it merges, within a window the policy sets.

## Declaring one

Apply the policy's emergency label (for example `emergency`) to the pull or merge request. Only an
allowed declarer can: a login listed in `emergency.declarers`, or a member of a listed GitHub team
(`team:ORG/SLUG`) or GitLab group (`group:PATH`). `crisk` reads the label events and takes the last
event for that label:

- added by an allowed declarer: an emergency ([CR005](reasons.md#cr005)), tier `emergency`;
- added by anyone else: ignored ([CR006](reasons.md#cr006));
- removed: no emergency;
- cannot be decided (events or membership unreadable): incomplete ([CR010](reasons.md#cr010)).

The description cannot declare an emergency: its author edits it, and a label event records who
applied the label. `change_type: emergency` in a declarations block is ignored
([CR004](reasons.md#cr004)).

## The obligation

The assessment carries `emergency.review_within_days` from the policy's
`tiers.emergency.retroactive_review_within_days`. agent-change-control holds the change to it: the
change fails its independent-approval rule like any unreviewed change until someone independent of
its author records the retroactive review, within the window from the merge. Past the window, acc
reports the review as overdue.

## The population

An auditor needs every emergency, not a sample of the ones someone remembers:

```bash
crisk population github acme/api --since 2026-07-01 --until 2026-09-30 --base main -o q3-emergencies.json
```

It lists every merged change in the window that carries the label, who last applied it before the
merge, whether they were allowed to (`null` when that cannot be decided), and when the retroactive
review is due. It exits 4 when a listing or a change's events were incomplete.

The forge lists changes by the labels they carry **now**: a label removed after merging removes the
change from the population. Protect the label (on GitLab, a scoped label; on GitHub, restrict who can
triage) and keep assessments of every change if the population must be provably complete.
