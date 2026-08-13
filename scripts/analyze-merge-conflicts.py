#!/usr/bin/env python3
"""Rank the repository paths that actually had to be conflict-resolved.

Issue #991 review feedback: "find a way to reduce possibility of conflicts in
these files in the future ... analyze previous conflicts in these files in
previous pull requests."

`git log --merges --cc` prints, for a merge commit, only the hunks that differ
from *every* parent. For an ordinary two-parent merge that is exactly the set of
lines the person (or the automation) had to resolve by hand: a clean
auto-merge leaves the result identical to one parent and prints nothing. So the
per-path frequency in that log is a direct, evidence-backed ranking of which
files force manual conflict resolution, without needing to re-run any merge.

Usage:
    python3 scripts/analyze-merge-conflicts.py            # ranked report
    python3 scripts/analyze-merge-conflicts.py --top 60   # longer ranking
    python3 scripts/analyze-merge-conflicts.py --json     # machine-readable

The classification below is the structural *cause*, which is what determines
the fix: a derived artifact is regenerated, an append-only list is split, a
sequentially numbered file is renamed. `data/meta/merge-conflict-policy.lino`
records which mechanism each cause is assigned to, and
`scripts/check-merge-conflict-policy.rs` fails CI when a ranked path is not
covered by one.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import subprocess
import sys

# Structural causes, most specific first. Each predicate answers: *why* does
# this path collide when two branches touch it independently?
CAUSES: list[tuple[str, "re.Pattern[str]"]] = [
    (
        "derived-artifact",
        re.compile(
            r"^(data/meta/self-ast/|data/meta/self-ast\.lino$"
            r"|data/seed/closure-generated-|data/meta/seed-metadata-gaps-)"
        ),
    ),
    (
        "append-only-list",
        re.compile(
            r"((^|/)mod\.rs$"
            r"|^src/lib\.rs$"
            r"|^src/web/formal_ai_worker\.js$"
            r"|^src/seed/embedded\.rs$"
            r"|^src/web/seed_loader\.js$"
            r"|^scripts/hardcoded-language-allowlist\.txt$)"
        ),
    ),
    (
        "append-only-document",
        re.compile(
            r"^(REQUIREMENTS|README|ARCHITECTURE|CHANGELOG|ROADMAP|GOALS|VISION"
            r"|NON-GOALS|CONTRIBUTING)\.md$"
        ),
    ),
    ("sequential-file-name", re.compile(r"^src/web/worker/formal_ai_worker_\d+\.js$")),
    ("automation-placeholder", re.compile(r"^\.gitkeep$")),
    ("lockfile-or-manifest", re.compile(r"^(Cargo\.(lock|toml)|package\.json|bun\.lock)$")),
    ("ci-workflow", re.compile(r"^\.github/workflows/")),
    ("shared-source", re.compile(r"^(src/|tests/|scripts/|examples/|data/|docs/)")),
]


def cause_of(path: str) -> str:
    for name, pattern in CAUSES:
        if pattern.search(path):
            return name
    return "other"


def conflicted_paths() -> collections.Counter:
    """Count how often each path appears in a merge commit's combined diff."""
    log = subprocess.run(
        ["git", "log", "--merges", "--cc", "--name-only", "--format=%x00"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    counts: collections.Counter = collections.Counter()
    for line in log.splitlines():
        path = line.strip("\x00").strip()
        if path:
            counts[path] += 1
    return counts


def merge_count() -> int:
    out = subprocess.run(
        ["git", "rev-list", "--merges", "--count", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return int(out.strip() or 0)


def report(counts: collections.Counter, top: int) -> dict:
    by_cause: collections.Counter = collections.Counter()
    for path, count in counts.items():
        by_cause[cause_of(path)] += count
    total = sum(counts.values())
    return {
        "merges_scanned": merge_count(),
        "conflict_events": total,
        "distinct_paths": len(counts),
        "by_cause": [
            {"cause": cause, "events": count, "share": round(count / total, 4) if total else 0.0}
            for cause, count in by_cause.most_common()
        ],
        "top_paths": [
            {"path": path, "events": count, "cause": cause_of(path)}
            for path, count in counts.most_common(top)
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--top", type=int, default=40, help="how many paths to rank")
    parser.add_argument("--json", action="store_true", help="emit JSON instead of a table")
    args = parser.parse_args()

    counts = conflicted_paths()
    if not counts:
        print("No merge commits with conflict resolutions found.")
        return 0

    data = report(counts, args.top)
    if args.json:
        json.dump(data, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    print(
        f"{data['merges_scanned']} merge commits scanned; "
        f"{data['conflict_events']} conflict-resolution events across "
        f"{data['distinct_paths']} paths.\n"
    )
    print("By structural cause:")
    for row in data["by_cause"]:
        print(f"  {row['events']:5d}  {row['share'] * 100:5.1f}%  {row['cause']}")
    print(f"\nTop {args.top} paths:")
    for row in data["top_paths"]:
        print(f"  {row['events']:5d}  {row['cause']:22s}  {row['path']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
