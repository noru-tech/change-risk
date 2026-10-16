# Conformance corpus

Facts in, assessment out. An evaluator conforms to Change Risk 0.1 when it reproduces every accept
vector and refuses every reject vector ([specification §11](../spec/change-risk.md#11-conformance)).

| Directory | Each file holds |
| --- | --- |
| `accept/` | `description`, `facts` and `expected`: the derived fields (`level`, `change_type`, `tier`, `complete`, `labels`, `reasons` without messages, `emergency`, `requirements`, `facts_digest`) |
| `reject/` | `description` and `facts` that must be refused |

## Contract

The verifier is a command line that receives one more argument, the path of a facts document. For
an accept vector it prints the assessment as JSON on stdout and exits 0, or 4 when the assessment is
incomplete. For a reject vector it exits 3.

```bash
python3 conformance/run.py --verifier "crisk -q evaluate -f json" --report report.json
```

The runner needs only Python 3's standard library.

## Coverage

The accept vectors exercise every reason code (CR001 to CR012), every tier, the standard-change
catalogue with and without a raising label or a declared `normal`, declarations that raise and that
are ignored, emergencies declared by a login, by a team member, by someone not allowed, removed, and
undecidable (incomplete team or events), case-insensitive labels and logins, and a declarer from
the other forge. The reject vectors cover ordering, pattern syntax, the emergency-only review window,
signals outside the change and unknown keys.
