#!/usr/bin/env python3
"""Compare the compile rate of two GitHub Actions job logs.

Both logs must come from the same cargo invocation on the same lockfile, so
that the set of crates is identical and the only variable is how fast the
runner got through it.  Written for issue #1021, finding 34: the
`Test (macos-15-intel / specification)` shard uses ~70% of its 1200s budget on
a healthy runner, and the question was whether a red shard had compiled
different work or the same work more slowly.

cargo schedules ready units across jobserver slots, so the *order* differs run
to run even when the *set* does not.  Comparing by index would therefore
compare unrelated crates; this compares each crate against itself and reports
the ratio at deciles of the baseline's own progress.

    python3 experiments/issue_1021_compile_rate_compare.py \
        ci-logs/job-96699699539.log ci-logs/job-96716556814.log
"""

import datetime
import re
import sys

CRATE = re.compile(r"--crate-name (\S+)")
STAMP = re.compile(r"(\S+Z) ")


def crate_offsets(path):
    """Return {crate_name: seconds after the first rustc invocation}."""
    seen = {}
    start = None
    with open(path, errors="replace") as handle:
        for line in handle:
            crate = CRATE.search(line)
            stamp = STAMP.match(line)
            if not (crate and stamp and "Running" in line):
                continue
            when = datetime.datetime.fromisoformat(
                stamp.group(1).replace("Z", "+00:00")
            )
            if start is None:
                start = when
            seen.setdefault(crate.group(1), (when - start).total_seconds())
    return seen


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    baseline, compared = (crate_offsets(p) for p in argv[1:])
    shared = sorted(
        (offset, name) for name, offset in baseline.items() if name in compared
    )
    print(f"{len(shared)} crates compiled by both "
          f"(baseline saw {len(baseline)}, compared saw {len(compared)})")
    print(f"{'decile of baseline progress':<30}{'baseline':>11}"
          f"{'compared':>11}{'ratio':>8}")
    for decile in range(1, 11):
        index = min(len(shared) * decile // 10, len(shared) - 1)
        offset, name = shared[index]
        other = compared[name]
        ratio = other / offset if offset else float("nan")
        print(f"{decile * 10:>3}% {name[:24]:<25}{offset:>10.1f}s"
              f"{other:>10.1f}s{ratio:>7.2f}x")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
