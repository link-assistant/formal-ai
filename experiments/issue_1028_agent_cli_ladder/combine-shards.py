#!/usr/bin/env python3
"""Join exact planned node evidence, preserving the complete ladder corpus."""
import json
import os
import shutil
import sys
from pathlib import Path
from evidence import FIELDS, IDENTITY, counts, fail, selected, validate


def main():
    if len(sys.argv) not in (4, 5):
        fail("usage: combine-shards.py <shards-dir> <out-dir> <expected-count> [plan.json]")
    shards_dir, out_dir, expected = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3])
    shards = sorted(path.parent for path in shards_dir.glob("*/ladder-result.lino"))
    if expected < 1 or len(shards) != expected:
        fail(f"the plan has {expected} shard(s), {len(shards)} wrote a result")
    measurements = [validate(shard) for shard in shards]
    identity = measurements[0][0]
    if any(any(values[key] != identity[key] for key in IDENTITY) for values, *_ in measurements):
        fail("shards disagree on source tree, corpus or authored-rules identity")
    for key in ("source_commit", "source_tree"):
        expected_identity = os.environ.get("LADDER_" + key.upper())
        if not expected_identity or identity[key] != expected_identity:
            fail(f"evidence {key} does not match the actual joining checkout")
    tree = measurements[0][1]
    tuples = [(values["requested_depth"], values["node_filter"]) for values, *_ in measurements]
    if len(tuples) != len(set(tuples)):
        fail("two shards measured the same depth/subtree")
    if len(sys.argv) == 5:
        plan = json.loads(Path(sys.argv[4]).read_text())
        legs = plan["legs"] + plan["composites"]
        declared = {(str(leg["depth"]), leg["filter"]): set(leg["nodes"]) for leg in legs}
        if len(declared) != len(legs) or set(tuples) != set(declared):
            fail("evidence legs differ from the exact planned depth/subtree set")
        for values, _, chosen, _ in measurements:
            if chosen != declared[(values["requested_depth"], values["node_filter"])]:
                fail("a leg selected a different node set than planned")
        mode, prefix = plan["requested_depth"], plan["node_filter"]
        expected_nodes = selected(tree, mode, prefix)
        if set(plan["expected_nodes"]) != expected_nodes or len(plan["expected_nodes"]) != len(expected_nodes):
            fail("the plan itself loses or repeats corpus nodes")
    else:
        depths = {values["requested_depth"] for values, *_ in measurements}
        if len(depths) != 1:
            fail("mixed-depth evidence requires an exact plan")
        mode = depths.pop()
        prefix = tuples[0][1] if len(tuples) == 1 else "none"
        expected_nodes = selected(tree, mode, prefix)
    seen, verdicts, selected_rows, run_rows, duration_rows = set(), {}, [], [], []
    for shard, (_, _, chosen, observed) in zip(shards, measurements):
        if seen & chosen:
            fail(f"overlapping measured nodes: {sorted(seen & chosen)}")
        seen |= chosen
        verdicts.update(observed)
        selected_rows.extend((shard / "selected.tsv").read_text().splitlines())
        run_rows.extend((shard / "run.log").read_text().splitlines())
        duration_rows.extend((shard / "durations.tsv").read_text().splitlines())
    if seen != expected_nodes:
        fail(f"missing nodes {sorted(expected_nodes - seen)}; extra nodes {sorted(seen - expected_nodes)}")
    # Validate everything before copying any evidence into the joined output.
    out_dir.mkdir(parents=True, exist_ok=True)
    for shard, (_, _, chosen, _) in zip(shards, measurements):
        for node in sorted(chosen):
            if (shard / node).is_dir():
                shutil.copytree(shard / node, out_dir / node, dirs_exist_ok=False)
    for shared in ("tree.tsv", "leaves.tsv"):
        shutil.copyfile(shards[0] / shared, out_dir / shared)
    selected_rows.sort(key=lambda row: (-tree[row.split("\t", 1)[0]][0], row.split("\t", 1)[0]))
    (out_dir / "selected.tsv").write_text("".join(f"{row}\n" for row in selected_rows))
    (out_dir / "run.log").write_text("".join(f"{row}\n" for row in run_rows))
    (out_dir / "durations.tsv").write_text("".join(f"{row}\n" for row in duration_rows))
    combined = {key: identity[key] for key in IDENTITY}
    combined.update(requested_depth=mode, node_filter=prefix)
    combined.update(counts(tree, seen, verdicts, identity["authored_rules_enabled"]))
    lines = ["ladder_result"] + [f'  {key} "{combined[key]}"' for key in FIELDS if key in combined]
    (out_dir / "ladder-result.lino").write_text("\n".join(lines) + "\n")
    table = "\n".join("| " + " | ".join(row.split("\t", 2)) + " |" for row in run_rows if row.count("\t") == 2)
    (out_dir / "README.md").write_text(
        "# Agent CLI binary-tree ladder run\n\n"
        f"Joined from {len(shards)} exact planned leg(s).\n\n"
        + "".join(f"- {key.replace('_', ' ')}: {combined[key]}\n" for key in FIELDS if key in combined)
        + "\n| node | verdict | detail |\n| --- | --- | --- |\n" + table + "\n")
    print((out_dir / "ladder-result.lino").read_text(), end="")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError) as error:
        print(f"::error title=Agent CLI ladder evidence does not form one run::{error}", file=sys.stderr)
        raise SystemExit(1)
