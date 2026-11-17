#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
import shutil
path=Path('experiments/issue_1582/continuation-latest-main-commit.sh')
source=path.read_text().replace('git diff --cached --check\n','git diff --cached --check origin/main\n')
path.write_text(source)
shutil.copyfile('/tmp/issue-1582-latest-main-commit-drive.log','/tmp/issue-1582-merge-review/commit-check-attempt.log')
path=Path('docs/case-studies/issue-1582/README.md')
path.write_text(path.read_text()+'''

The first merge commit check compared incoming main logs with the old branch
head and reported their existing trailing whitespace. The corrected check
compares the staged PR diff with current main, preserving the canonical raw
log bytes. The failed operational attempt is retained separately.
''')
PY
python3 experiments/issue_1582/continuation-archive.py --source /tmp/issue-1582-merge-review --subdir latest-main
git add experiments/issue_1582 docs/case-studies/issue-1582 docs/case-studies/pull-request-1583
node scripts/check-file-size.mjs > /tmp/issue-1582-latest-main-final-size.log 2>&1
/tmp/issue-1582-js-tools/shellcheck-v0.11.0/shellcheck experiments/issue_1582/continuation-latest-main-*.sh
git diff --cached --check origin/main
git commit -m 'Merge main request-context support and preserve client fixes' > /tmp/issue-1582-latest-main-commit.log 2>&1
