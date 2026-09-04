#!/usr/bin/env python3
"""Does the first path a change request names identify the file it changes?

The planner has to pick one operand out of prose that may name several files.
The ladder's node prompts name three -- the source to edit, an effects record,
a proof note -- and an ordering rule that read the first *undelimited* path
picked the proof note, because the file to edit was the one the prompt had
bothered to mark up. Preferring markup only moves the failure: nothing stops a
request from marking up an incidental file and leaving its target bare.

This measures the alternative -- earliest path wins, delimiters ignored --
against labelled data this repository already holds and did not write for the
purpose: its own commit messages, each paired with the files that commit
actually touched. A commit message is a change request written after the fact,
by the same people whose requests the planner has to read.

Reported for every commit whose subject-and-body names two or more distinct
workspace paths, since a single-path message cannot discriminate any ordering.

The proxy is imperfect and worth stating plainly: a commit touches many files,
so "is this path among them" is a weaker question than "is this path the
operand", and a message may name a path for context rather than as a target.
It is used here for what it can settle -- the *relative* standing of candidate
ordering rules over hundreds of real messages -- and not as an accuracy figure
for the planner.
"""

import re
import subprocess
from collections import Counter


def is_workspace_path(token):
    """Transcribed from `is_workspace_path` in src/agentic_coding/structured_edit.rs."""
    if "." not in token:
        return False
    stem, _, extension = token.rpartition(".")
    return (
        stem != ""
        and not token.startswith("/")
        and ".." not in token.split("/")
        and 1 <= len(extension) <= 8
        and all(c.isascii() and c.isalnum() for c in extension)
        and not all(c.isascii() and c.isdigit() for c in extension)
        and all(c.isascii() and (c.isalnum() or c in "_-./") for c in token)
    )


TRIM = re.compile(r"^[^0-9A-Za-z_\-./]+|[^0-9A-Za-z_\-./]+$")


def paths_in(text):
    """Every workspace path in `text`, in order, deduplicated by first mention."""
    found = []
    for chunk in re.split(r"[\s,]+", text):
        token = TRIM.sub("", chunk).rstrip(".")
        if is_workspace_path(token) and token not in found:
            found.append(token)
    return found


def git(*args):
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True
    ).stdout


def labelled_commits():
    """Commits whose message names >= 2 paths, at least one of them changed."""
    log = git("log", "--no-merges", "--format=%H%x00%B%x01")
    for entry in log.split("\x01"):
        entry = entry.strip("\n")
        if not entry:
            continue
        sha, _, message = entry.partition("\x00")
        named = paths_in(message)
        if len(named) < 2:
            continue
        touched = set(git("show", "--name-only", "--format=", sha).split())
        # A message that names only paths the commit never touched has no
        # operand to identify, and cannot discriminate between rules.
        if touched and any(path in touched for path in named):
            yield sha, named, touched


def main():
    rows = list(labelled_commits())
    print(f"labelled commits (>= 2 named paths, >= 1 of them changed): {len(rows)}\n")

    rules = {
        "earliest named path": lambda named: named[0],
        "earliest that is workspace-relative (contains '/')":
            lambda named: next((path for path in named if "/" in path), None),
        "latest named path": lambda named: named[-1],
        "longest named path": lambda named: max(named, key=len),
    }
    print(f"{'rule':<52} {'correct':>14}")
    for label, pick in rules.items():
        considered = hits = 0
        for _, named, touched in rows:
            chosen = pick(named)
            if chosen is None:
                continue
            considered += 1
            hits += chosen in touched
        share = 100.0 * hits / considered if considered else 0.0
        print(f"  {label:<50} {hits:>4}/{considered:<4} {share:6.2f}%")

    opening_bare = sum(1 for _, named, _ in rows if "/" not in named[0])
    print(
        f"\nmessages that open with a bare basename rather than a path: "
        f"{opening_bare} ({100.0 * opening_bare / len(rows):.2f}%)"
    )
    print("  e.g. `release.yml` named before `.github/workflows/release.yml`")

    slashed = [
        (sha, [path for path in named if "/" in path], touched)
        for sha, named, touched in rows
    ]
    slashed = [
        row for row in slashed if len(row[1]) >= 2 and any(p in row[2] for p in row[1])
    ]
    positions = Counter(
        next(i for i, path in enumerate(named) if path in touched)
        for _, named, touched in slashed
    )
    total = sum(positions.values())
    print(
        f"\nrestricted to messages naming >= 2 workspace-relative paths: {len(slashed)}"
    )
    print(
        f"  the first such path is one the commit changed: "
        f"{positions[0]}/{total} ({100.0 * positions[0] / total:.2f}%)"
    )
    print(f"  earliest changed path by position: {dict(sorted(positions.items()))}")


if __name__ == "__main__":
    main()
