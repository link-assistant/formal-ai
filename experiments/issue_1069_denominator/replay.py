#!/usr/bin/env python3
"""Measure how much of each release cycle's denominator is recorded history.

`scripts/self-hosting-metric.rs` says what it intends to count:

    Changed lines are additions plus deletions reported by `git show --numstat`;
    merge commits, binary files and captured artifacts do not contribute.

`METRIC_VERSION = 2` implements "captured artifacts" as a list of *extensions*
(`log`, `jsonl`, `diff`, `patch`, `stderr`, `stdout`) plus lockfile names. That
catches a CI log; it does not catch the `.md`, `.json`, `.ts` and `.js` files
that sit beside it inside the same evidence bundle, nor the vendored upstream
templates checked into `dev/log/<...>/references/`.

This replay reports, per release cycle in the ledger, the changed-line total
under the committed v2 definition and under a location-aware one that also
treats the repository's own recorded-history trees as captured. CONTRIBUTING
names those trees together:

    Recorded history under `docs/case-studies/`, `dev/log/`, and
    `experiments/` is exempt -- a past run stays as it happened.

Nothing here changes the metric. It only measures what a change would do, so the
question "does this lower the floor or fix the measurement?" is answered with
numbers.

    python3 experiments/issue_1069_denominator/replay.py
"""

import subprocess
import sys

CAPTURED_EXTENSIONS = {"log", "jsonl", "diff", "patch", "stderr", "stdout"}
LOCKFILES = {
    "Cargo.lock", "bun.lock", "package-lock.json",
    "yarn.lock", "pnpm-lock.yaml", "poetry.lock", "uv.lock",
}
# The trees CONTRIBUTING calls recorded history.
RECORDED_HISTORY = ("dev/log/", "docs/case-studies/", "experiments/")

SESSION = "formal-ai-session:"
EVIDENCE = "formal-ai-evidence:"


def git(*args):
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True
    ).stdout


def is_captured_v2(path):
    name = path.rsplit("/", 1)[-1]
    if name in LOCKFILES:
        return True
    return "." in name and name.rsplit(".", 1)[1].lower() in CAPTURED_EXTENSIONS


def is_captured_v3(path):
    return is_captured_v2(path) or path.startswith(RECORDED_HISTORY)


def changed_lines(commit):
    """(v2 total, v3 total) for one commit."""
    v2 = v3 = 0
    numstat = git("show", "--format=", "--numstat", "--no-renames", commit)
    for line in numstat.splitlines():
        fields = line.split("\t")
        if len(fields) < 3 or fields[0] == "-" or fields[1] == "-":
            continue
        count = int(fields[0]) + int(fields[1])
        path = fields[2].strip().strip('"')
        if not is_captured_v2(path):
            v2 += count
        if not is_captured_v3(path):
            v3 += count
    return v2, v3


def attributed(commit):
    """Mirrors commit_has_formal_ai_evidence, leniently."""
    body = git("show", "-s", "--format=%B", commit)
    sessions, evidence_paths = [], []
    for line in body.splitlines():
        stripped = line.strip()
        low = stripped.lower()
        if low.startswith(SESSION):
            sessions.append(stripped[len(SESSION):].strip())
        elif low.startswith(EVIDENCE):
            evidence_paths.append(stripped[len(EVIDENCE):].strip())
    if not sessions or not evidence_paths:
        return False
    blobs = []
    for path in evidence_paths:
        obj = f"{commit}:{path}"
        try:
            kind = git("cat-file", "-t", obj).strip()
        except subprocess.CalledProcessError:
            return False
        if kind == "blob":
            blobs.append(git("show", obj))
        elif kind == "tree":
            listing = git("ls-tree", "-r", "--name-only", commit, "--", path)
            for entry in listing.splitlines():
                if entry:
                    blobs.append(git("show", f"{commit}:{entry}"))
    identifying = [b for b in blobs if "formal-ai" in b.lower()]
    if not identifying:
        return False
    return all(any(s in b for b in identifying) for s in sessions)


def measure(since, until):
    commits = [c for c in git("rev-list", "--reverse", "--no-merges",
                              f"{since}..{until}").splitlines() if c]
    totals = dict(v2=0, v3=0, self_v2=0, self_v3=0, commits=len(commits), self_commits=0)
    for commit in commits:
        v2, v3 = changed_lines(commit)
        totals["v2"] += v2
        totals["v3"] += v3
        if attributed(commit):
            totals["self_v2"] += v2
            totals["self_v3"] += v3
            totals["self_commits"] += 1
    return totals


def basis_points(part, whole):
    return 0 if whole == 0 else part * 10000 // whole


def percent(bp):
    return f"{bp // 100}.{bp % 100:02d}%"


def ledger_rows(path="data/meta/self-hosting-ledger.lino"):
    rows, current = [], None
    for line in open(path):
        stripped = line.strip()
        if stripped == "release":
            current = {}
            rows.append(current)
            continue
        if current is None or " " not in stripped:
            continue
        key, _, value = stripped.partition(" ")
        current[key] = value.strip().strip('"')
    return rows


def main():
    rows = [r for r in ledger_rows() if r.get("metric_version") == "2"]
    print(f"{'cycle':<22}{'v2 changed':>12}{'v3 changed':>12}{'recorded':>10}"
          f"{'v2 share':>10}{'v3 share':>10}")
    replayed = []
    for row in rows:
        totals = measure(row["since"], row["until"])
        share_v2 = basis_points(totals["self_v2"], totals["v2"])
        share_v3 = basis_points(totals["self_v3"], totals["v3"])
        recorded = totals["v2"] - totals["v3"]
        pct = 0 if totals["v2"] == 0 else 100 * recorded / totals["v2"]
        print(f"{row['tag']:<22}{totals['v2']:>12}{totals['v3']:>12}"
              f"{pct:>9.1f}%{percent(share_v2):>10}{percent(share_v3):>10}")
        replayed.append((row["tag"], totals))

    open_cycle = measure(rows[-1]["tag"], "HEAD")
    recorded = open_cycle["v2"] - open_cycle["v3"]
    pct = 0 if open_cycle["v2"] == 0 else 100 * recorded / open_cycle["v2"]
    print(f"{'HEAD (open cycle)':<22}{open_cycle['v2']:>12}{open_cycle['v3']:>12}"
          f"{pct:>9.1f}%"
          f"{percent(basis_points(open_cycle['self_v2'], open_cycle['v2'])):>10}"
          f"{percent(basis_points(open_cycle['self_v3'], open_cycle['v3'])):>10}")

    # The trailing target is the weighted share of the last `window` rows. A
    # definition change is only honest if it carries this forward rather than
    # resetting it, so report both.
    window = int(rows[-1].get("trailing_window", "3"))
    tail = replayed[-window:]
    for label, key_self, key_all in (("v2", "self_v2", "v2"), ("v3", "self_v3", "v3")):
        self_lines = sum(t[key_self] for _, t in tail)
        all_lines = sum(t[key_all] for _, t in tail)
        print(f"\ntrailing target under {label} "
              f"(last {window} recorded cycles, replayed): "
              f"{percent(basis_points(self_lines, all_lines))} "
              f"({self_lines}/{all_lines})")

    # What the open cycle would have to add to clear each target.
    for label, key_self, key_all in (("v2", "self_v2", "v2"), ("v3", "self_v3", "v3")):
        self_lines = sum(t[key_self] for _, t in tail[1:]) + open_cycle[key_self]
        all_lines = sum(t[key_all] for _, t in tail[1:]) + open_cycle[key_all]
        target = basis_points(sum(t[key_self] for _, t in tail),
                              sum(t[key_all] for _, t in tail))
        # solve (self + x) / (all + x) >= target/10000 for x
        numerator = target * all_lines - 10000 * self_lines
        denominator = 10000 - target
        needed = max(0, -(-numerator // denominator)) if denominator else 0
        print(f"authored lines the open cycle still needs under {label}: {needed}")


if __name__ == "__main__":
    sys.exit(main())
