#!/usr/bin/env python3
"""Run the change-risk conformance corpus against an evaluator.

The contract (conformance/README.md): the verifier is a command line that takes one extra argument,
the path of a facts document, and

  * for a valid document, prints the assessment as JSON on stdout and exits 0, or 4 when the
    assessment is incomplete;
  * for an invalid document, exits 3.

Standard library only. Usage:

    python3 conformance/run.py --verifier "crisk -q evaluate -f json" [--report report.json]

Exit status: 0 when every vector passes, 1 otherwise, 2 on usage errors.
"""
import argparse
import json
import pathlib
import shlex
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent


def project(a):
    """The fields the corpus pins: everything derived, without messages or `generated`."""
    reasons = []
    for r in a["reasons"]:
        r = dict(r)
        r.pop("message", None)
        reasons.append(r)
    return {
        "level": a["level"],
        "change_type": a["change_type"],
        "tier": a["tier"],
        "complete": a["complete"],
        "labels": a["labels"],
        "reasons": reasons,
        "emergency": a["emergency"],
        "requirements": a["requirements"],
        "facts_digest": a["facts_digest"],
    }


def run(verifier, facts, tmp):
    path = pathlib.Path(tmp) / "facts.json"
    path.write_text(json.dumps(facts))
    return subprocess.run(verifier + [str(path)], capture_output=True, timeout=60)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--verifier", required=True, help="the evaluator command line")
    parser.add_argument("--report", help="write a JSON report here")
    args = parser.parse_args()
    verifier = shlex.split(args.verifier)
    results = []
    with tempfile.TemporaryDirectory() as tmp:
        for kind in ("accept", "reject"):
            for vector in sorted((ROOT / kind).glob("*.json")):
                doc = json.loads(vector.read_text())
                name = f"{kind}/{vector.stem}"
                try:
                    out = run(verifier, doc["facts"], tmp)
                except Exception as exc:  # noqa: BLE001 - a crashing verifier is an error result
                    results.append({"vector": name, "result": "error", "detail": str(exc)})
                    continue
                if kind == "reject":
                    ok = out.returncode == 3
                    detail = "" if ok else f"exit {out.returncode}, expected 3"
                else:
                    try:
                        got = project(json.loads(out.stdout))
                    except Exception as exc:  # noqa: BLE001
                        results.append({"vector": name, "result": "error", "detail": f"exit {out.returncode}: unreadable output ({exc})"})
                        continue
                    expected_code = 0 if doc["expected"]["complete"] else 4
                    ok = out.returncode == expected_code and got == doc["expected"]
                    detail = "" if ok else (
                        f"exit {out.returncode}, expected {expected_code}"
                        if out.returncode != expected_code
                        else "assessment differs from expected"
                    )
                results.append({"vector": name, "result": "pass" if ok else "fail", "detail": detail})
    totals = {k: sum(1 for r in results if r["result"] == k) for k in ("pass", "fail", "error")}
    for r in results:
        if r["result"] != "pass":
            print(f"{r['result'].upper():5} {r['vector']}: {r['detail']}")
    print(f"{totals['pass']} passed, {totals['fail']} failed, {totals['error']} errors")
    if args.report:
        pathlib.Path(args.report).write_text(json.dumps({"totals": totals, "results": results}, indent=2) + "\n")
    return 0 if totals["fail"] == 0 and totals["error"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
