#!/usr/bin/env python3
"""Retain the largest completed sequential coding observation from each source."""
import argparse
from datetime import datetime
import gzip
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / 'experiments/issue-1580/evidence/checks/native-coding-budget'
OUTPUT = ROOT / 'scripts/ci-native-coding-durations.lino'


def measured_text():
    identity = json.loads((EVIDENCE / 'identity.json').read_text())
    durations = {}
    for observation in identity['observations']:
        raw = gzip.decompress((EVIDENCE / observation['file']).read_bytes())
        assert hashlib.sha256(raw).hexdigest() == observation['rawSha256']
        previous = None
        completed = 0
        summary = None
        for line in raw.decode().splitlines():
            timestamp, _, message = line.lstrip('\ufeff').partition(' ')
            if not timestamp.endswith('Z'):
                continue
            now = datetime.fromisoformat(timestamp)
            if re.fullmatch(r'running 62 tests', message):
                previous = now
            match = re.fullmatch(r'test ([A-Za-z0-9_:]+) \.{3} ok', message)
            if match:
                assert previous is not None
                seconds = max((now - previous).total_seconds(), 0.001)
                durations[match[1]] = max(durations.get(match[1], 0), seconds)
                completed += 1
                previous = now
            if message.startswith('test result:'):
                summary = message
        if observation['conclusion'] == 'success':
            assert completed == 62 and summary.startswith('test result: ok. 62 passed; 0 failed; 0 ignored;')
        else:
            assert observation['conclusion'] == 'cancelled' and completed == 58 and summary is None
    assert len(durations) == 62
    rows = ['test_durations', '  recorded 2026-11-21', '  unit seconds', '  default-seconds 60',
            '  purpose "Largest completed coding observations; no duration assigned to unfinished cases."',
            '  regenerate "python3 experiments/issue-1580/derive-native-coding-durations.py --write"']
    for observation in identity['observations']:
        rows.append('  source_job ' + str(observation['job']))
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
        assert OUTPUT.read_text() == text, 'hosted coding duration projection is stale'
    else:
        print(text, end='')
