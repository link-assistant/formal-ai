#!/usr/bin/env python3
"""Issue #1191, requirement 12: peak resident memory and wall time of one command.

    python3 experiments/issue-1191-peak-rss.py node experiments/formal_ai_subagent/local-gates.mjs

ru_maxrss of RUSAGE_CHILDREN is the peak of the largest single descendant that
was waited for (Linux reports kibibytes). It is not the sum of concurrent
processes, so it is a lower bound on the task's total footprint.
"""
import resource, subprocess, sys, time

start = time.monotonic()
code = subprocess.call(sys.argv[1:])
peak_kib = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
print(f"exit={code} wall_s={time.monotonic() - start:.0f} peak_rss_mib={peak_kib / 1024:.0f}", file=sys.stderr)
sys.exit(code)
