#!/usr/bin/env python3
"""Static UI boundary ratchet; --strict enforces the issue #951 final target."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--strict', action='store_true')
args = parser.parse_args()
baseline = json.loads((ROOT / 'docs/case-studies/issue-951/function-inventory.json').read_text())
errors = []
for relative, allowed in baseline['legacy_functions'].items():
    path = ROOT / relative
    if not path.exists():
        continue
    names = set(re.findall(r'^(?:export\s+)?(?:async\s+)?function\s+([a-z][A-Za-z0-9_]*)\s*\(', path.read_text(), re.M))
    introduced = names - set(allowed)
    if introduced:
        errors.append(relative + ': new lowercase functions: ' + ', '.join(sorted(introduced)))
for path in (ROOT / 'js/app').glob('*'):
    if path.suffix not in {'.js', '.jsx'} or path.relative_to(ROOT).as_posix() in baseline['legacy_functions']:
        continue
    if re.search(r'^(?:export\s+)?(?:async\s+)?function\s+[a-z]', path.read_text(), re.M):
        errors.append(str(path.relative_to(ROOT)) + ': unregistered lowercase function')
main_lines = len((ROOT / 'js/app/main.jsx').read_text().splitlines())
limit = 2499 if args.strict else baseline['main_line_ceiling']
if main_lines > limit:
    errors.append(f'main.jsx has {main_lines} lines; allowed {limit}')
if errors:
    raise SystemExit('\n'.join(errors))
print(f'UI boundary ratchet satisfied; final 2500-line migration target still requires --strict ({main_lines} current lines)')
