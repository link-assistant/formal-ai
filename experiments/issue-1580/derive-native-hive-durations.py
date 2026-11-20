#!/usr/bin/env python3
"""Derive completed sequential case durations from the retained hosted Hive log."""
import argparse
from datetime import datetime
import gzip
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
LOG = ROOT / 'experiments/issue-1580/evidence/ci/native-families-7fce5c19/job-136011692528.stdout.gz'
OUTPUT = ROOT / 'scripts/ci-native-hive-durations.lino'


def measured_text():
    previous = None
    durations = {}
    for line in gzip.decompress(LOG.read_bytes()).decode().splitlines():
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
    rows = ['test_durations', '  source_run 44652802985',
            '  source_sha 7fce5c19a7297a121d92eb09e7fefa99ca649eaf',
            '  recorded 2026-11-20', '  unit seconds',
            '  default-seconds 60',
            '  purpose "Only completed sequential hosted cases are measured; unseen cases weigh 60 seconds."',
            '  regenerate "python3 experiments/issue-1580/derive-native-hive-durations.py --write"']
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
