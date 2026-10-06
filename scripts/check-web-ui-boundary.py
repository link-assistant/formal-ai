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
# The React UI layer is every JSX module under js/app/ (main.jsx was split into
# feature modules), so the ceiling measures their total: splitting a file must
# not count as moving logic out of the UI.
ui_lines = sum(len(path.read_text().splitlines()) for path in (ROOT / 'js/app').glob('*.jsx'))
limit = 2499 if args.strict else baseline['ui_jsx_line_ceiling']
if ui_lines > limit:
    errors.append(f'js/app/*.jsx have {ui_lines} lines in total; allowed {limit}')
if errors:
    raise SystemExit('\n'.join(errors))
print(f'UI boundary ratchet satisfied; final 2500-line migration target still requires --strict ({ui_lines} current js/app/*.jsx lines)')
