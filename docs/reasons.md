# Reason codes

Every assessment lists the reasons its tier follows from, in the order the [specification](../spec/change-risk.md#4-evaluation) derives them. Codes are stable; messages are informative and may change.

| Code | Meaning |
| --- | --- |
| [CR001](#cr001) | A label sets the level |
| [CR002](#cr002) | The change edits `.change-risk/` |
| [CR003](#cr003) | A declared risk raises the level |
| [CR004](#cr004) | A declaration is ignored |
| [CR005](#cr005) | An emergency was declared |
| [CR006](#cr006) | The emergency label was applied by someone who may not declare |
| [CR007](#cr007) | A standard change |
| [CR008](#cr008) | The default level |
| [CR009](#cr009) | No map or policy at the base revision |
| [CR010](#cr010) | An emergency cannot be established |
| [CR011](#cr011) | A would-be standard change is declared normal |
| [CR012](#cr012) | The change touches no path |


<a id="cr001"></a>
## CR001: A label sets the level

A changed path carries a label, from the [map](map-and-policy.md#the-sensitivity-map) or a [signal](signals.md), that the policy's `levels` gives a level. The level becomes at least that level. The reason names the label and the paths.

**What to do:** Nothing, if the label is right. If a path is labelled wrongly, fix the map on the target branch in its own pull request; that change is itself `high` (CR002).


<a id="cr002"></a>
## CR002: The change edits `.change-risk/`

The change touches the map, the policy, or anything else under `.change-risk/`. It is always `high`, whatever the policy says, because it changes the requirements every later change is held to, and it is assessed under the files already on the target branch.

**What to do:** Expected. Review it as a change to your change-management policy, and keep `.change-risk/` behind CODEOWNERS.


<a id="cr003"></a>
## CR003: A declared risk raises the level

The author declared a `risk` above what the paths imply. The level becomes the declared one.

**What to do:** Nothing. Declaring a higher risk is always allowed.


<a id="cr004"></a>
## CR004: A declaration is ignored

The author declared something that would lower the requirements or that cannot be declared: a `risk` below the computed level, `change_type: standard` (only the policy's catalogue makes a change standard), or `change_type: emergency` (an emergency is declared with the policy's label, by an allowed declarer).

**What to do:** Remove the declaration, or for an emergency ask an allowed declarer to apply the label.


<a id="cr005"></a>
## CR005: An emergency was declared

The policy's emergency label is on the change, and was last applied by an allowed declarer: a listed login, or a member of a listed team or group as observed at collection. The tier is `emergency`.

**What to do:** Merge under the emergency tier's requirements, and have someone independent of the author review the change within the policy's window. See [emergency changes](emergency.md).


<a id="cr006"></a>
## CR006: The emergency label was applied by someone who may not declare

The label is on the change, but whoever applied it last is not an allowed declarer. It is ignored, and the change keeps its computed tier.

**What to do:** Ask an allowed declarer to remove and reapply the label, or treat the change as normal.


<a id="cr007"></a>
## CR007: A standard change

Every changed path is in the policy's standard-change catalogue and nothing raised the level above `low`. The reason names the catalogue entries used.

**What to do:** Nothing. A standard change is pre-approved by the policy.


<a id="cr008"></a>
## CR008: The default level

Every assessment starts at the policy's `default_level` (`standard` in the built-in policy).

**What to do:** Nothing; this is the floor every other reason builds on.


<a id="cr009"></a>
## CR009: No map or policy at the base revision

`.change-risk/map.yml` or `.change-risk/policy.yml` does not exist on the target branch. Without a map no path carries a map label; without a policy the built-in one applies (default level `standard`, no emergency path, no tier requirements). A map or policy added by the change itself does not apply to it.

**What to do:** Add the files on the target branch, through their own pull request.


<a id="cr010"></a>
## CR010: An emergency cannot be established

The emergency label may have been applied by an allowed declarer, but crisk could not decide: the label event listing was incomplete, or a declarer team or group could not be read in full. The assessment is incomplete (exit 4) and the tier is computed as if there were no emergency.

**What to do:** Give the token what it needs to read label events and team or group membership (see [forges](forges.md)), raise `--max-pages`, and assess again.


<a id="cr011"></a>
## CR011: A would-be standard change is declared normal

Every path is in the catalogue at level `low`, but the author declared `change_type: normal`. It stays normal.

**What to do:** Nothing. Declaring a change normal is always allowed.


<a id="cr012"></a>
## CR012: The change touches no path

The diff between the merge base and the head is empty. The level is the default and the change is not a standard change.

**What to do:** Check the base and head the assessment used.
