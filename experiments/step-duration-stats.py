#!/usr/bin/env python3
"""Per-step duration statistics from `scripts/collect-job-durations.sh` output.

Issue #1081. The collector emits six fields for a job row and eight for a step
row; this reads the step rows and reports, per step name, the worst and median
duration over the runs that finished green and over every run that ran at all.
The gap between the two columns is the survivorship bias the audit scripts were
losing their evidence to: a step killed at its budget is dropped by a
success-only filter, and it is the only observation that proves the budget is
too small.

Usage:  python3 experiments/step-duration-stats.py <durations.tsv> [name...]
"""

import sys
import statistics
from collections import defaultdict
from datetime import datetime, timezone

RAN = {"success", "failure", "cancelled", "timed_out"}


def seconds(value: str) -> float:
    return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=timezone.utc
    ).timestamp()


def main() -> int:
    path = sys.argv[1]
    wanted = [name.lower() for name in sys.argv[2:]]

    green: dict[str, list[float]] = defaultdict(list)
    every: dict[str, list[tuple[float, str, str]]] = defaultdict(list)
    for line in open(path, encoding="utf-8"):
        fields = line.rstrip("\n").split("\t")
        if len(fields) != 8 or fields[6] != "step":
            continue
        run_id, _workflow, _job, conclusion, started, completed, _, step = fields
        if conclusion not in RAN:
            continue
        duration = seconds(completed) - seconds(started)
        every[step].append((duration, conclusion, run_id))
        if conclusion == "success":
            green[step].append(duration)

    rows = []
    for step, observations in every.items():
        if wanted and not any(w in step.lower() for w in wanted):
            continue
        worst, conclusion, run_id = max(observations)
        greens = green.get(step, [])
        rows.append(
            (
                worst,
                step,
                max(greens) if greens else 0.0,
                statistics.median(greens) if greens else 0.0,
                len(greens),
                len(observations),
                conclusion,
                run_id,
            )
        )
    rows.sort(reverse=True)

    print(
        f"{'worst':>7} {'concl':>9} {'green max':>9} {'green med':>9} "
        f"{'green n':>7} {'all n':>5}  step"
    )
    for worst, step, gmax, gmed, gn, an, conclusion, run_id in rows:
        print(
            f"{worst:7.0f} {conclusion:>9} {gmax:9.0f} {gmed:9.0f} "
            f"{gn:7d} {an:5d}  {step}   (run {run_id})"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
