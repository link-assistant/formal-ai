#!/usr/bin/env python3
"""Derive completed sequential case durations from the retained hosted Hive log."""
import argparse
from datetime import datetime
import gzip
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
SOURCES = [
    (44652802985, '7fce5c19a7297a121d92eb09e7fefa99ca649eaf', 136011692528),
    (44794246520, '1d7ba479b92fed6c1d6735b442579af9190a6b28', 136489959548),
    (44794246520, '1d7ba479b92fed6c1d6735b442579af9190a6b28', 136489959657),
    (44794246520, '1d7ba479b92fed6c1d6735b442579af9190a6b28', 136489959672),
]
OUTPUT = ROOT / 'scripts/ci-native-hive-durations.lino'


def completed_durations(text):
    previous = None
    durations = {}
    for line in text.splitlines():
        fields = line.split('\t', 2)
        if len(fields) != 3:
            continue
        timestamp, _, message = fields[2].partition(' ')
        if not timestamp.endswith('Z'):
            continue
        if re.fullmatch(r'running \d+ tests', message):
            previous = datetime.fromisoformat(timestamp)
        match = re.fullmatch(r'test (issue_1133_hive_mind_three_runs::\S+) \.{3} ok', message)
        if match:
            assert previous is not None
            now = datetime.fromisoformat(timestamp)
            name = match[1]
            assert name not in durations
            durations[name] = max((now - previous).total_seconds(), 0.001)
            previous = now
    assert durations, 'no completed cases; do not invent measured weights'
    return durations


def measured_text():
    durations = {}
    for run, head, job in SOURCES:
        path = ROOT / f'experiments/issue-1580/evidence/ci/native-families-{head[:8]}/job-{job}.stdout.gz'
        observed = completed_durations(gzip.decompress(path.read_bytes()).decode())
        for name, seconds in observed.items():
            # Retain the slowest completed observation; fresh faster runs must
            # not erase an earlier measured scheduling cost.
            durations[name] = max(durations.get(name, 0), seconds)
    rows = ['test_durations']
    for run, head, job in SOURCES:
        rows.extend([f'  source_run {run}', f'  source_sha {head}', f'  source_job {job}'])
    rows.extend([
            '  recorded 2026-11-20', '  unit seconds',
            '  default-seconds 60',
            '  purpose "Maximum completed sequential hosted duration per case; unseen cases weigh 60 seconds."',
            '  regenerate "python3 experiments/issue-1580/derive-native-hive-durations.py --write"'])
    for name, seconds in sorted(durations.items(), key=lambda row: (-row[1], row[0])):
        rows.extend([f'  test "{name}"', f'    seconds {seconds:.3f}'])
    return '\n'.join(rows) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--write', action='store_true')
    mode.add_argument('--check', action='store_true')
    options = parser.parse_args()
    text = measured_text()
    if options.write:
        OUTPUT.write_text(text)
    elif options.check:
        assert OUTPUT.read_text() == text, 'hosted duration projection is stale'
    else:
        print(text, end='')
