#!/usr/bin/env python3
"""Import only verified current-run leaf artifacts for composite node legs."""
import shutil
import sys
from pathlib import Path
from evidence import IDENTITY, digest, fail, validate


def main():
    source, target = map(Path, sys.argv[1:3])
    commit, tree, rules = sys.argv[3:6]
    values, rows, chosen, verdicts = validate(source)
    expected = dict(zip(IDENTITY, (rules, commit, tree, digest(target / "tree.tsv"), digest(target / "leaves.tsv"))))
    if any(values[key] != expected[key] for key in IDENTITY) or values["requested_depth"] != "5":
        fail("child evidence belongs to another source tree, corpus, rule mode or depth")
    parents = [row.split("\t")[0] for row in (target / "selected.tsv").read_text().splitlines()]
    needed = set()
    for parent in parents:
        if parent not in rows or rows[parent][0] != 4:
            fail("child imports are only valid for selected depth-four composites")
        needed.update(rows[parent][1].split("\t")[4:6])
    if not needed or not needed <= chosen:
        fail("child evidence omits a selected composite's expected leaves")
    for node in sorted(needed):
        if rows[node][0] != 5 or verdicts[node] != "PASS":
            continue
        effect = source / node / "effect.lino"
        diff = source / node / "change.diff"
        fields = effect.read_text().splitlines()
        if f"node_path={node}" not in fields or "node_kind=leaf" not in fields or not diff.is_file() or not diff.stat().st_size:
            fail(f"verified leaf {node} has no matching effect and diff")
        shutil.copytree(source / node, target / node, dirs_exist_ok=False)
        print(f"{node}\t{target / node / 'effect.lino'}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError) as error:
        print(f"::error title=Invalid current-run child evidence::{error}", file=sys.stderr)
        raise SystemExit(1)
