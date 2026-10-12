"""Validate one ladder evidence set against its tree, corpus and observations."""
import hashlib
import re
from pathlib import Path

FIELDS = ("requested_depth", "node_filter", "authored_rules_enabled", "source_commit", "source_tree",
          "tree_sha256", "leaves_sha256", "selected_nodes", "failures", "deepest_passing_level",
          "leaf_nodes_selected", "leaf_nodes_passing", "leaf_nodes_passing_without_authored_rules")
IDENTITY = ("authored_rules_enabled", "source_commit", "source_tree", "tree_sha256", "leaves_sha256")


def fail(message):
    raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def result(path):
    values = {}
    for line in Path(path).read_text().splitlines():
        parts = line.strip().split(" ", 1)
        if len(parts) == 2 and parts[0] in FIELDS:
            if parts[0] in values:
                fail(f"duplicate result field: {parts[0]}")
            values[parts[0]] = parts[1].strip().strip('"')
    return values


def corpus(path):
    rows = {}
    for row in (Path(path) / "tree.tsv").read_text().splitlines():
        node, depth, *_ = row.split("\t")
        if not re.fullmatch(r"R|[12](\.[12]){0,4}", node) or int(depth) != (0 if node == "R" else node.count(".") + 1):
            fail(f"invalid canonical tree node: {node}")
        if node in rows:
            fail(f"duplicate tree node: {node}")
        rows[node] = (int(depth), row)
    if len(rows) != 63 or sum(depth == 5 for depth, _ in rows.values()) != 32:
        fail("the canonical corpus must contain all 63 nodes and 32 leaves")
    return rows


def selected(tree, mode, prefix):
    if mode not in [str(depth) for depth in range(6)] + ["all"]:
        fail(f"invalid requested depth: {mode}")
    if prefix not in ("none", "R") and not re.fullmatch(r"[12](\.[12]){0,4}", prefix):
        fail(f"invalid node filter: {prefix}")
    return {node for node, (depth, _) in tree.items()
            if (mode == "all" or depth == int(mode))
            and (prefix in ("none", "R") or node == prefix or node.startswith(prefix + "."))}


def observations(path):
    verdicts = {}
    for row in (Path(path) / "run.log").read_text().splitlines():
        fields = row.split("\t")
        if len(fields) != 3 or fields[1] not in ("PASS", "FAIL"):
            continue
        if fields[0] in verdicts:
            fail(f"duplicate measured node: {fields[0]}")
        verdicts[fields[0]] = fields[1]
    return verdicts


def counts(tree, chosen, verdicts, rules):
    leaf = {node for node in chosen if tree[node][0] == 5}
    deepest = "none"
    for depth in range(5, -1, -1):
        level = {node for node in chosen if tree[node][0] == depth}
        if not level:
            continue
        if any(verdicts[node] != "PASS" for node in level):
            break
        deepest = str(depth)
    values = {"selected_nodes": str(len(chosen)), "failures": str(sum(verdicts[node] == "FAIL" for node in chosen)),
              "deepest_passing_level": deepest, "leaf_nodes_selected": str(len(leaf)),
              "leaf_nodes_passing": str(sum(verdicts[node] == "PASS" for node in leaf))}
    if rules == "false":
        values["leaf_nodes_passing_without_authored_rules"] = values["leaf_nodes_passing"]
    return values


def validate(path):
    path = Path(path)
    values = result(path / "ladder-result.lino")
    for key in FIELDS[:-1]:
        if key not in values:
            fail(f"{path.name} omits result field {key}")
    if values["authored_rules_enabled"] not in ("true", "false"):
        fail("invalid authored-rules identity")
    for key in ("source_commit", "source_tree"):
        if not re.fullmatch(r"[0-9a-f]{40}", values[key]):
            fail(f"invalid {key}")
    for key, filename in (("tree_sha256", "tree.tsv"), ("leaves_sha256", "leaves.tsv")):
        if values[key] != digest(path / filename):
            fail(f"{path.name} has conflicting {filename} bytes")
    tree = corpus(path)
    chosen = selected(tree, values["requested_depth"], values["node_filter"])
    rows = (path / "selected.tsv").read_text().splitlines()
    ids = [row.split("\t", 1)[0] for row in rows]
    if len(ids) != len(set(ids)) or set(ids) != chosen:
        fail(f"{path.name} selected nodes do not match its depth/filter")
    if any(row != tree[node][1] for node, row in zip(ids, rows)):
        fail(f"{path.name} selected rows disagree with its corpus")
    timings = {}
    for row in (path / "durations.tsv").read_text().splitlines():
        node, depth, elapsed = row.split("\t")
        if node in timings or node not in chosen or int(depth) != tree[node][0] or int(elapsed) < 0:
            fail(f"{path.name} has duplicate, foreign or invalid elapsed measurements")
        timings[node] = int(elapsed)
    if set(timings) != chosen:
        fail(f"{path.name} omits actual node elapsed measurements")
    verdicts = observations(path)
    if set(verdicts) != chosen:
        fail(f"{path.name} must report one disposition for every selected node")
    expected = counts(tree, chosen, verdicts, values["authored_rules_enabled"])
    if any(values.get(key) != value for key, value in expected.items()):
        fail(f"{path.name} result counts disagree with measured node dispositions")
    for node in chosen:
        if verdicts[node] == "PASS" and not (path / node / "effect.lino").is_file():
            fail(f"{path.name} claims passing node {node} without its effect")
    return values, tree, chosen, verdicts
