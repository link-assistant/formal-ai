#!/usr/bin/env python3
"""Plan independent node legs and the current-run leaf dependency wave."""
import itertools
import json
import re
import sys


def nodes(depth):
    return ["R"] if depth == 0 else [".".join(bits) for bits in itertools.product("12", repeat=depth)]


def focused(node, prefix):
    return not prefix or prefix == "R" or node == prefix or node.startswith(prefix + ".")


def plan(mode, prefix=""):
    if mode not in [str(depth) for depth in range(6)] + ["all"]:
        raise ValueError("depth must be 0 through 5, or all")
    if prefix and not re.fullmatch(r"R|[12](\.[12]){0,4}", prefix):
        raise ValueError("invalid binary-tree node filter")
    levels = list(range(5, -1, -1)) if mode == "all" else [int(mode)]
    expected = [node for depth in levels for node in nodes(depth) if focused(node, prefix)]
    if not expected:
        raise ValueError("the filter selects no node at the requested depth")
    legs, composites = [], []
    for depth in levels:
        selected = [node for node in nodes(depth) if focused(node, prefix)]
        groups = {}
        for node in selected:
            key = node.rsplit(".", 1)[0] if depth == 5 and "." in node and prefix != node else node
            groups.setdefault(key, []).append(node)
        for key, members in groups.items():
            leg = {"depth": depth, "filter": key, "id": f"depth-{depth}-{key}", "nodes": members}
            (composites if mode == "all" and depth == 4 else legs).append(leg)
    return {"requested_depth": mode, "node_filter": prefix or "none", "legs": legs,
            "composites": composites, "expected_nodes": expected}


if __name__ == "__main__":
    print(json.dumps(plan(*sys.argv[1:]), separators=(",", ":")))
