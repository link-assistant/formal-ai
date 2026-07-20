#!/usr/bin/env python3
"""Replay `scripts/self-hosting-metric.rs` attribution for a commit range.

Issue #812: `Auto Release` aborts with

    self-hosting ratchet would fall from 32.68% to 18.24% for v0.301.0

This reproduces the measurement outside CI (no rust-script required) so the
per-commit attribution can be inspected: which commits carry usable
`Formal-AI-Session` / `Formal-AI-Evidence` trailers, which do not, and how many
changed lines each contributes.

    python3 experiments/self_hosting_ratchet_replay/replay.py v0.300.0 origin/main
"""

import subprocess
import sys

SESSION_TRAILER = "formal-ai-session:"
EVIDENCE_TRAILER = "formal-ai-evidence:"


def git(*args):
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True
    ).stdout


def trailer_values(commit, key):
    body = git("show", "-s", "--format=%B", commit)
    out = []
    for line in body.splitlines():
        line = line.strip()
        if line[: len(key)].lower() == key:
            value = line[len(key) :].strip()
            if value:
                out.append(value)
    return out


def changed_lines(commit):
    total = 0
    for line in git("show", "--format=", "--numstat", "--no-renames", commit).splitlines():
        fields = line.split("\t")
        if len(fields) < 2 or fields[0] == "-" or fields[1] == "-":
            continue
        total += int(fields[0]) + int(fields[1])
    return total


def attribution(commit):
    """Returns (attributed, reason). Mirrors commit_has_formal_ai_evidence."""
    sessions = trailer_values(commit, SESSION_TRAILER)
    evidence_paths = trailer_values(commit, EVIDENCE_TRAILER)
    if not sessions and not evidence_paths:
        return False, "no trailers"
    if not sessions or not evidence_paths:
        return False, "ERROR: must record both trailers"
    evidence = []
    for path in evidence_paths:
        try:
            content = git("show", f"{commit}:{path}")
        except subprocess.CalledProcessError:
            return False, f"ERROR: evidence {path} missing in commit"
        if "formal-ai" not in content.lower():
            return False, f"ERROR: evidence {path} does not identify formal-ai"
        evidence.append(content)
    for session in sessions:
        if not any(session in content for content in evidence):
            return False, f"ERROR: no evidence records session {session}"
    return True, "attributed"


def main():
    since = sys.argv[1] if len(sys.argv) > 1 else "v0.300.0"
    until = sys.argv[2] if len(sys.argv) > 2 else "HEAD"
    commits = [c for c in git("rev-list", "--reverse", "--no-merges", f"{since}..{until}").split() if c]

    total = 0
    self_total = 0
    self_commits = 0
    rows = []
    for commit in commits:
        lines = changed_lines(commit)
        ok, reason = attribution(commit)
        total += lines
        if ok:
            self_total += lines
            self_commits += 1
        subject = git("show", "-s", "--format=%s", commit).strip()
        rows.append((commit[:8], lines, ok, reason, subject))

    rows.sort(key=lambda r: -r[1])
    print(f"{'commit':10} {'lines':>8}  {'attr':5} {'reason':50} subject")
    for sha, lines, ok, reason, subject in rows:
        print(f"{sha:10} {lines:>8}  {str(ok):5} {reason[:50]:50} {subject[:60]}")

    pct = (self_total * 10000 + total // 2) // total if total else 0
    print()
    print(f"range          : {since}..{until}")
    print(f"commits        : {self_commits}/{len(commits)} attributed")
    print(f"changed lines  : {self_total}/{total}")
    print(f"percentage     : {pct // 100}.{pct % 100:02d}%")


if __name__ == "__main__":
    main()
