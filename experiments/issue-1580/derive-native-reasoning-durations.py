#!/usr/bin/env python3
"""Project sequential hosted reasoning observations into reproducible shard weights."""
import argparse
from datetime import datetime
import gzip
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
LOG = ROOT / 'experiments/issue-1580/evidence/checks/native-reasoning-budget/job-136423862321.stdout.gz'
OUTPUT = ROOT / 'scripts/ci-native-reasoning-durations.lino'


def measured_text():
    previous = None
    durations = {}
    summary = None
    for line in gzip.decompress(LOG.read_bytes()).decode().splitlines():
        timestamp, _, message = line.partition(' ')
        if not timestamp.endswith('Z'):
            continue
        if re.fullmatch(r'running 148 tests', message):
            previous = datetime.fromisoformat(timestamp)
        match = re.fullmatch(r'test ([A-Za-z0-9_:]+) \.{3} ok', message)
        if match:
            assert previous is not None
            now = datetime.fromisoformat(timestamp)
            name = match[1]
            assert name not in durations and '::' in name
            assert not name.startswith(('benchmarks::', 'issue_1133_hive_mind_three_runs::'))
            durations[name] = max((now - previous).total_seconds(), 0.001)
            previous = now
        if message.startswith('test result:'):
            summary = message
    assert len(durations) == 148 and summary is not None
    assert summary.startswith('test result: ok. 148 passed; 0 failed; 0 ignored;')
    rows = ['test_durations', '  source_run 44775337494',
            '  source_job 136423862321',
            '  source_sha 39196d1fb218d0ef492f90915270f9ae3ac28ba1',
            '  recorded 2026-11-20', '  unit seconds', '  default-seconds 60',
            '  purpose "All 148 sequential cases passed before the job clock expired; unseen cases weigh 60 seconds."',
            '  regenerate "python3 experiments/issue-1580/derive-native-reasoning-durations.py --write"']
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
        assert OUTPUT.read_text() == text, 'hosted reasoning duration projection is stale'
    else:
        print(text, end='')
