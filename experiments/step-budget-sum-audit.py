#!/usr/bin/env python3
"""Sum every job's declared step budgets and compare with its `timeout-minutes`.

Issue #1081. The repository enforces "each budget is at most 70% of the job
cap" but not "the budgets in a job sum to at most 70% of the cap", which the
js template's `tests/ci-timeouts.test.js` does enforce. This prints where the
missing half of the rule bites.

Budgets under mutually exclusive `if:` conditions cannot run in the same job,
so they are summed per condition and the largest group wins, plus every
unconditional budget.
"""
import glob, os, re, sys
from collections import defaultdict

for path in sorted(glob.glob(".github/workflows/*.yml")):
    text = open(path).read()
    lines = text.split("\n")
    job = None
    caps = {}
    steps = defaultdict(list)  # job -> [(step_name, condition, budget)]
    step_name = step_if = None
    for line in lines:
        m = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if m:
            job = m.group(1); step_name = step_if = None; continue
        if job is None:
            continue
        m = re.match(r"^    timeout-minutes:\s*(\S+)", line)
        if m:
            caps[job] = m.group(1); continue
        m = re.match(r"^      - (?:name|uses):\s*(.*)$", line)
        if m:
            step_name = m.group(1).strip().strip("'\""); step_if = None; continue
        m = re.match(r"^        if:\s*(.*)$", line)
        if m and step_name:
            step_if = m.group(1).strip(); continue
        m = re.match(r"^\s+TEST_BUDGET_SECONDS:\s*(\d+)", line)
        if m and step_name:
            steps[job].append((step_name, step_if, int(m.group(1))))
    for j, budgets in steps.items():
        cap = caps.get(j)
        try:
            cap_s = int(cap) * 60
        except (TypeError, ValueError):
            print(f"{path}::{j}  cap={cap!r} (not a literal) budgets={budgets}")
            continue
        groups = defaultdict(int)
        unconditional = 0
        for name, cond, b in budgets:
            if cond is None:
                unconditional += b
            else:
                groups[cond] += b
        effective = unconditional + (max(groups.values()) if groups else 0)
        naive = sum(b for _, _, b in budgets)
        flag = ""
        if effective * 100 > cap_s * 70:
            flag = "  <<< OVER 70% (grouped)"
        elif naive * 100 > cap_s * 70:
            flag = "  <<< over 70% only if groups are ignored"
        print(f"{path}::{j}  cap={cap}m({cap_s}s) naive_sum={naive} grouped_sum={effective} "
              f"grouped={effective*100/cap_s:.1f}%{flag}")
        for name, cond, b in budgets:
            print(f"      {b:>5}s  {b*100/cap_s:5.1f}%  {name}   if={cond}")
