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
# not count as moving logic out of the UI. The ratchet counts non-whitespace
# characters, so wrapping a long line for readability (R1188-U23) neither
# raises nor hides the size; --strict keeps the final line target.
jsx_texts = [path.read_text() for path in (ROOT / 'js/app').glob('*.jsx')]
ui_lines = sum(len(text.splitlines()) for text in jsx_texts)
ui_characters = sum(len(re.sub(r'\s', '', text)) for text in jsx_texts)
if args.strict and ui_lines > 2499:
    errors.append(f'js/app/*.jsx have {ui_lines} lines in total; allowed 2499')
if ui_characters > baseline['ui_jsx_character_ceiling']:
    errors.append(f"js/app/*.jsx have {ui_characters} non-whitespace characters in total; allowed {baseline['ui_jsx_character_ceiling']}")
if errors:
    raise SystemExit('\n'.join(errors))
print(f'UI boundary ratchet satisfied ({ui_characters} non-whitespace characters); final 2500-line migration target still requires --strict ({ui_lines} current js/app/*.jsx lines)')
