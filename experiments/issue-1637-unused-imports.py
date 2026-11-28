"""Issue #1637: flag imported names a Rust file no longer uses (a cheap stand-in for
`-D unused-imports` when cargo cannot run locally). Usage: python3 this.py FILE..."""
import re, sys
for path in sys.argv[1:]:
    src = open(path, encoding='utf-8').read()
    body = re.sub(r'(?ms)^\s*(pub(\([^)]*\))?\s+)?use\s[^;]*;', '', src)
    for stmt in re.findall(r'(?ms)^\s*use\s([^;]*);', src):
        names = re.findall(r'\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*(?=[,}]|$)', stmt.replace('\n', ' '))
        for name, alias in names:
            used = alias or name
            if used in ('self', 'super', 'crate', '*') or used[0].isdigit():
                continue
            if not re.search(r'\b' + re.escape(used) + r'\b', body):
                print(f'{path}: unused import {used}')
