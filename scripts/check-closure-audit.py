#!/usr/bin/env python3
"""The honest total-closure gap equals the number reviewed in its ledger.

Issue #1138 B9, plan 09 leaf 7. ``scripts/audit-total-closure.py`` reports two
numbers: the historical one, which counts the glosses
``scripts/close-total.py`` generated as definitions, and the honest one, which
does not. Only the honest one measures grounding, so only the honest one is
ratcheted -- strictly, in both directions, the rule
``scripts/check-minimal-core-boundary.rs`` already applies to handler debt:

  * above the reviewed value, the gap grew and the commit must ground tokens;
  * below it, the gap shrank and the commit must lower the reviewed value, so
    the improvement is recorded instead of leaving room to regress into.

The same comparison is made by
``tests/unit/total_closure.rs::seed_closure_gap_only_shrinks``; this script is
the cheap form of it, so a CI gate can answer the question without building the
crate.

Usage::

    python3 scripts/check-closure-audit.py [ROOT]
"""

from __future__ import annotations

import os
import subprocess
import sys

LEDGER = "data/meta/closure-audit.lino"
MEASURE = "unresolved_distinct_honest"


def reviewed(root: str) -> int:
    """The value ``LEDGER`` records for ``MEASURE``."""
    path = os.path.join(root, LEDGER)
    measure: str | None = None
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            stripped = line.strip()
            if stripped.startswith("measure "):
                measure = stripped[len("measure ") :].strip().strip('"')
                continue
            if stripped.startswith("value ") and measure == MEASURE:
                return int(stripped[len("value ") :].strip().strip('"'))
    raise SystemExit(f"{LEDGER} names no `{MEASURE}` ceiling")


def measured(root: str) -> int:
    """The honest gap the audit reports for this checkout."""
    import json

    output = subprocess.run(
        [sys.executable, "scripts/audit-total-closure.py", "--json", root],
        capture_output=True,
        text=True,
        check=False,
        cwd=root or ".",
    )
    if not output.stdout:
        raise SystemExit(f"the audit emitted no JSON; stderr: {output.stderr}")
    return int(json.loads(output.stdout)[MEASURE])


def main(argv: list[str]) -> int:
    root = argv[1] if len(argv) > 1 else "."
    ceiling = reviewed(root)
    value = measured(root)
    print(f"closure audit ({LEDGER}):")
    print(f"  {MEASURE}: measured {value} / reviewed {ceiling}")
    if value > ceiling:
        print(
            f"::error file={LEDGER}::the honest closure gap grew from {ceiling} to "
            f"{value}. Ground the tokens in an authored meanings file the runtime "
            f"loads; the generator may propose work, never satisfy the gate."
        )
        return 1
    if value < ceiling:
        print(
            f"::error file={LEDGER}::the honest closure gap improved from {ceiling} "
            f"to {value}; lower the reviewed value in {LEDGER} in this commit, so "
            f"the ratchet records the improvement instead of leaving room to "
            f"regress into."
        )
        return 1
    print("closure audit holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
