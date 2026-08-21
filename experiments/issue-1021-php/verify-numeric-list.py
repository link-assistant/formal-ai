#!/usr/bin/env python3
"""Issue #1021: run every PHP program the numeric-list composer generates and
compare its real output with the result the answer claims.

Usage:
    cargo run --example issue_1021_php_numeric_list > /tmp/php_numeric.log
    python3 experiments/issue-1021-php/verify-numeric-list.py /tmp/php_numeric.log
"""
import re
import subprocess
import sys
import tempfile
from pathlib import Path

log = Path(sys.argv[1]).read_text()
blocks = re.findall(r"```php\n(.*?)```\n\nResult: ([^\n]*)", log, re.S)
if not blocks:
    sys.exit("no php blocks found")
failures = 0
for index, (code, claimed) in enumerate(blocks):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "main.php"
        path.write_text(code)
        lint = subprocess.run(["php", "-l", str(path)], capture_output=True, text=True)
        run = subprocess.run(["php", str(path)], capture_output=True, text=True)
    actual = run.stdout.strip()
    ok = lint.returncode == 0 and run.returncode == 0 and actual == claimed.strip()
    failures += not ok
    print(f"{'ok  ' if ok else 'FAIL'} #{index}: claimed {claimed.strip()!r} actual {actual!r}")
    if not ok:
        print(lint.stdout, run.stderr, sep="\n")
print(f"\n{len(blocks) - failures}/{len(blocks)} verified")
sys.exit(1 if failures else 0)
