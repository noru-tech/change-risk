# Change Risk 0.1

**Status:** draft. **Predicate type:** `https://noru.tech/spec/ai-change-provenance/change-risk/v0.1`.

Change Risk answers one question about a source change, deterministically: **how risky is it, and
therefore which controls must it meet?** The answer is a *tier* (`low`, `standard`, `high` or
`emergency`) and the *reasons* that produced it, derived only from declared facts: the paths the
change touches, a sensitivity map and a tier policy kept in the repository, what the author
declared, a forge label and who applied it, and labels other tools' findings contribute.

It is part of the [AI Change Provenance](https://noru.tech/spec/ai-change-provenance/) family. It
shares that specification's predicate namespace and serialization (RFC 8785), and an
agent-change-control evaluator reads a Change Risk assessment to decide which requirements a change
must meet (§10). It has its own version.

The key words MUST, MUST NOT, SHOULD and MAY are to be interpreted as described in RFC 2119.

## 1. Non-goals

- **No content analysis.** A tier is never derived from what a diff says, only from where it is.
- **No model.** No LLM or statistical classifier is involved anywhere in the derivation.
- **No enforcement.** An assessment states requirements; branch rules, CODEOWNERS, approval rules
  and acc enforce them. An implementation MUST NOT write any of those.

## 2. Inputs

### 2.1 The change

A change is identified by three commits in a git repository:

- **base**: the commit of the target branch the change is assessed against;
- **merge base**: the best common ancestor of base and head (`git merge-base`);
- **head**: the change's last commit, the assessment's subject.

The **changed paths** are the paths that differ between the merge base and the head, as `git diff
--name-only --no-renames` reports them: a rename contributes both the old and the new path.
Paths are repository-relative, use `/`, and are sorted by their UTF-8 bytes without duplicates.

### 2.2 The base revision rule

The sensitivity map (`.change-risk/map.yml`) and the tier policy (`.change-risk/policy.yml`) MUST
be read from the **base** commit, never from the head, the working tree or any other revision the
change controls. A change MUST NOT be able to lower the requirements it is held to.

A change that touches any path under `.change-risk/` is always `high` (CR002): it changes the
rules later changes are held to, and is itself assessed under the rules it would replace.

When either file is absent at the base, the facts record it as absent (`null`), and evaluation uses
an empty map or the built-in policy (§2.4) respectively (CR009).

### 2.3 The sensitivity map

```yaml
version: "0.1"
paths:
  - pattern: /src/auth/
    labels: [auth]
  - pattern: "*.tf"
    labels: [infra]
```

A **label** matches `^[a-z0-9][a-z0-9:._-]*$`. A path carries the union of the labels of every
entry whose pattern matches it.

Patterns use the subset of CODEOWNERS syntax GitHub documents, so that one pattern means the same
thing in both files:

1. Paths are relative to the repository root and use `/`.
2. A pattern with a leading `/`, or with a `/` anywhere but at its end, is anchored to the root.
   Any other pattern matches at any depth (`*.sql` matches `db/a.sql`).
3. A pattern ending in `/` matches everything below that directory. A pattern without the trailing
   `/` also matches everything below a directory of that name.
4. `*` matches any characters except `/`; `**` matches across `/`; `?` matches one character
   except `/`.
5. A pattern beginning with `!`, or containing `[`, `]`, `\` or whitespace, is invalid.

### 2.4 The tier policy

```yaml
version: "0.1"
default_level: low                 # default standard
levels:                            # the level each label sets
  auth: high
  migrations: standard
  sarif:error: high
standard_changes:                  # the standard-change catalogue
  - name: documentation
    paths: ["/docs/", "*.md"]
emergency:
  label: emergency
  declarers: [alice, "team:acme/sre", "group:acme/sre"]
tiers:                             # what each tier requires (data, §8)
  low:       { min_human_approvals: 1, agent_review: allowed }
  standard:  { min_human_approvals: 1 }
  high:      { min_human_approvals: 2, required_reviewer_teams: [acme/security], required_checks: [test] }
  emergency: { min_human_approvals: 0, retroactive_review_within_days: 3 }
missing_tier: standard             # the tier for a change with no assessment (§10)
```

The **built-in policy** has `default_level: standard`, no levels, no catalogue, no emergency, no tier
requirements, and `missing_tier: standard`.

`retroactive_review_within_days` is valid only under `emergency`. Declarers are forge logins,
`team:ORG/SLUG` (a GitHub team) or `group:PATH` (a GitLab group).

### 2.5 Declarations

The author MAY declare facts in one HTML comment in the change's description, or in a file:

```text
<!-- change-risk
change_type: normal
risk: high
rollback: Revert the merge; the migration is additive.
-->
```

The block is YAML with the optional keys `change_type` (`standard`, `normal`, `emergency`), `risk`
(`low`, `standard`, `high`) and `rollback` (text, at most 4000 bytes). A description with more than
one block, an unclosed block or an unknown key is invalid. Declarations can only raise requirements
(§4).

### 2.6 Label events and declarers

When the change is read from a forge, the facts record every event that added or removed a label on
it, oldest first, with the actor (`github:LOGIN` or `gitlab:USERNAME`, lowercased) and the time, and
whether the listing was complete.

For each `team:` or `group:` declarer the policy names, the facts record its members as observed
**at collection** and whether the listing was complete. Membership at the time the label was
applied is not available from either forge; the observation time is recorded so that a reader can
tell.

### 2.7 Signals

A signal is another tool's output, read by a documented adapter that turns it into labels on
**changed** paths only:

| Kind | Input | Labels |
| --- | --- | --- |
| `sarif` | SARIF 2.1.0 | `sarif:error`, `sarif:warning`, `sarif:note`: a result's level (default `warning`; `none` is skipped) on each changed file it is located in |
| `privacy-datamap` | A Noru `privacy-datamap` manifest | `personal-data` on each changed file a collection or privacy declaration with data categories cites; `personal-data:special` as well when a category is at or below a special category (§2.7.1) |
| `acc` | An agent-change-control manifest | On every changed path: `agent-authored` when the change whose `head_sha` is the head has an `agent:` author; `acc:finding` when that change has a finding whose disposition is open |

#### 2.7.1 Special categories

`user.authorization.biometric`, `user.biometric`, `user.criminal_history`,
`user.demographic.political_opinion`, `user.demographic.race_ethnicity`,
`user.demographic.religious_belief`, `user.demographic.sexual_orientation`,
`user.health_and_medical`: GDPR Article 9 data and Article 10 criminal-offence data in the
Fideslang taxonomy.

## 3. Facts

The facts document ([schema](../schemas/facts.schema.json)) holds every input above and nothing
derived from it: the change (§2.1), the changed paths, the map and policy as read (or `null`), their
sources (path, revision and the SHA-256 of the bytes read, or `null` when absent), the declarations,
the label events, the observed teams and the signals.

Besides its schema, a facts document is valid only if: its paths are sorted and unique and
repository-relative; every map and catalogue pattern is valid (§2.3); every label is valid; the
policy satisfies §2.4; label events are in time order; teams are sorted by id with sorted, unique
members; and every signal label applies only to changed paths. An evaluator MUST refuse an invalid
facts document.

## 4. Evaluation

Evaluation is a pure function of the facts. It MUST NOT read the network, the clock, the
environment or the repository. The reasons are written in this order.

1. **Configuration.** For an absent map, and for an absent policy, a CR009 reason. For no changed
   paths, CR012.
2. **Default level.** The level starts at the policy's `default_level` (CR008, with that level).
3. **Labels.** The applied labels are the union of map labels (source `map`) and signal labels
   (source `signal:KIND`), each with the sorted changed paths it applies to, sorted by label. For
   each applied label the policy gives a level, in label order, a CR001 reason with that level, the
   label and its paths; the level becomes the maximum. Levels order `low < standard < high`.
4. **Self-modification.** If any changed path starts with `.change-risk/`, a CR002 reason with
   level `high` and those paths; the level becomes `high`.
5. **Declared risk.** A declared `risk` above the level raises it (CR003, with that level); one
   below it is ignored (CR004); an equal one changes nothing and has no reason.
6. **Emergency.** Only when the policy has an `emergency` section and label events were read. Take
   the last event whose label equals the emergency label without regard to case.
   - If there is none, or it removed the label: no emergency. If the event listing is incomplete,
     a CR010 reason and the assessment is incomplete.
   - Otherwise its actor is an allowed declarer when, for any declarer: a login equals the actor's
     login without regard to case; or a `team:` declarer is a recorded team that lists the actor
     and the actor is a GitHub actor; or a `group:` declarer is a recorded group that lists the
     actor and the actor is a GitLab actor. A `team:` declarer never matches a GitLab actor, nor a
     `group:` declarer a GitHub one.
   - Allowed: CR005 (with the label), and the change is an emergency.
   - Not allowed, but some declarer could not be decided (a team or group not recorded or not
     listed completely, for an actor of its forge) or the event listing is incomplete: CR010 and
     the assessment is incomplete.
   - Otherwise: CR006 (with the label); the label is ignored.
7. **Declared change type.** A declared `standard` is ignored (CR004). A declared `emergency` is
   ignored (CR004) unless step 6 found an emergency.
8. **Change type.** `emergency` when step 6 found one. Otherwise `standard` when there is at least
   one changed path, every changed path matches a pattern of some catalogue entry, and the level is
   `low`: CR007, naming the entries used (sorted, comma-separated), unless `normal` was declared,
   in which case CR011 and `normal`. Otherwise `normal`.
9. **Tier.** `emergency` for an emergency change, otherwise the level. The requirements are the
   policy's entry for that tier, or `null`.

### 4.1 Reason codes

| Code | Meaning |
| --- | --- |
| CR001 | A label sets the level |
| CR002 | The change edits `.change-risk/` |
| CR003 | A declared risk raises the level |
| CR004 | A declaration is ignored: it would lower requirements, or cannot be declared |
| CR005 | An emergency was declared by an allowed declarer |
| CR006 | The emergency label was applied by someone who may not declare; ignored |
| CR007 | Every path is in the standard-change catalogue |
| CR008 | The policy's default level |
| CR009 | The map or the policy is absent at the base revision |
| CR010 | An emergency cannot be established: label events or a declarer's membership incomplete |
| CR011 | A would-be standard change is declared normal |
| CR012 | The change touches no path |

Reason messages are informative; only codes, levels, labels and paths are normative.

## 5. Assessment

The assessment ([schema](../schemas/assessment.schema.json)) records the level, change type, tier,
completeness, applied labels, reasons, the accepted emergency (label, declarer, time, and the
policy's `retroactive_review_within_days` or `null`), the requirements, `facts_digest` (SHA-256 over
the RFC 8785 bytes of the facts) and the facts themselves.

An assessment is valid only if evaluating its embedded facts reproduces every field except
`generated`. A consumer MUST re-evaluate before trusting a derived field.

An assessment is **complete** unless step 6 recorded CR010. An incomplete assessment MUST NOT be
read as "no emergency": the emergency could not be established either way.

## 6. Serialization

Every JSON document is written as its RFC 8785 bytes, as AI Change Provenance §8.2 specifies, with
no trailing newline. Digests are `sha256:` and the lowercase hex of SHA-256 over those bytes.

## 7. in-toto predicate

An assessment is the predicate of an in-toto Statement v1
([schema](../schemas/statement.schema.json)):

- `predicateType`: `https://noru.tech/spec/ai-change-provenance/change-risk/v0.1`;
- `subject`: exactly one entry, named by the change id (or `git:` and the head commit when there is
  none), whose digest is `{"gitCommit": <head>}`.

A verifier MUST check that the subject's commit is the predicate's `change.head`, and SHOULD
validate the predicate (§5). Statements are unsigned; sign them with the DSSE signer you trust.

## 8. Requirements document

The requirements document ([schema](../schemas/requirements.schema.json)) is the policy's tier
section restated for enforcement tools: per tier, the requirements and the map entries whose labels
route a change to that tier (the highest level any of its labels sets), the emergency label,
declarers and review window, and `missing_tier`. It is data. Branch rules, CODEOWNERS and approval
rules derived from it are the consuming tool's.

## 9. Emergency population

For an auditor, the merged changes in a window that carry the emergency label
([schema](../schemas/population.schema.json)): who last applied the label at or before the merge,
whether they were an allowed declarer (`null` when that cannot be decided), and when the
retroactive review is due (merge time plus the policy's window). Forges list changes by the labels
they carry now: a label removed after the merge removes the change from the population, and the
document says how it was produced so that this can be accounted for.

## 10. Relationship to AI Change Provenance

An agent-change-control evaluator reads an assessment bound to a change's head commit (an attestation
with this predicate type) and applies the requirements of its tier, as its own policy states them.
`crisk compile --to acc` renders the `tiers` and `missing_tier` section of that policy from this
one, so the two cannot drift. A change with no assessment is held to `missing_tier`.

## 11. Conformance

An evaluator conforms if, for every vector in [`conformance/`](../conformance/README.md): each
accept vector's facts evaluate to an assessment whose derived fields equal the vector's `expected`
(reason messages and `generated` excluded), with exit status 0, or 4 when incomplete; and each
reject vector's facts are refused.

## 12. Security considerations

- The base revision rule (§2.2) is the whole defence against a change lowering its own bar. A
  repository SHOULD protect `.change-risk/` with CODEOWNERS and branch rules.
- Declarations are the author's: they only ever raise requirements.
- An emergency rests on a forge's label event, which records who applied the label; the
  description, which the author edits, cannot declare one.
- Team and group membership is observed at collection, not at the time of the label.
- Paths in the facts come from the repository and may contain any characters a filename can;
  renderers MUST escape them.

## Changelog

- **0.1** (draft): first version.
