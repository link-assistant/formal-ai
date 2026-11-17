#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
import shutil
shutil.copyfile('/tmp/issue-1582-merge-review/api-permissions.log','/tmp/issue-1582-merge-review/api-permissions-first.log')
shutil.copyfile('/tmp/issue-1582-api-languages-drive.log','/tmp/issue-1582-merge-review/api-drive.log')
shutil.copyfile('/tmp/issue-1582-latest-main-commit-retry-drive.log','/tmp/issue-1582-merge-review/commit-check-second-attempt.log')
path=Path('rust/tests/web/issue-1582-permissions.test.mjs')
source=path.read_text()
needle='  for(const language of Object.keys(examples)){'
assert needle in source
source=source.replace(needle,"  const registry=readFileSync(new URL('../../../data/seed/languages.lino',import.meta.url),'utf8');\n  assert.deepEqual(Object.keys(examples).sort(),[...registry.matchAll(/^  language (\\S+)$/gmu)].map(match=>match[1]).sort());\n"+needle)
path.write_text(source)
path=Path('docs/case-studies/issue-1582/README.md')
path.write_text(path.read_text()+'''

A final integration test checks permission guidance through Chat Completions
and Responses with coding-agent system instructions: all five registered
languages pass all ten requests. It compares the fixture languages with the
actual language registry. The permission test file now passes 36 checks,
bringing the merged focused set to 52. The staged whitespace check excludes
verbatim `dev/log/` captures from both branches; all remaining PR files are
checked against main. Both failed broad whitespace checks are retained, and
no captured log was rewritten.
''')
path=Path('experiments/issue_1582/continuation-plan.md')
path.write_text(path.read_text()+'''
The final API composition test adds ten real chat/Responses requests across
all five registered languages, with a registry equality check. The permission
file passes 36 checks; the merged focused set is 52. This is supplied test
execution with zero independent authorship credit.
''')
PY
node --max-old-space-size=384 --test --test-concurrency=1 rust/tests/web/issue-1582-permissions.test.mjs > /tmp/issue-1582-merge-review/api-permissions.log 2>&1
node experiments/formal_ai_subagent/local-gates.mjs --only check-tests-as-docs > /tmp/issue-1582-merge-review/final-small-gates.log 2>&1
node experiments/formal_ai_subagent/local-gates.mjs --only check-abbreviations >> /tmp/issue-1582-merge-review/final-small-gates.log 2>&1
python3 experiments/issue_1582/continuation-archive.py --source /tmp/issue-1582-merge-review --subdir latest-main
git add experiments/issue_1582 docs/case-studies/issue-1582 docs/case-studies/pull-request-1583 rust/tests/web/issue-1582-permissions.test.mjs
node scripts/check-file-size.mjs > /tmp/issue-1582-latest-main-final-size.log 2>&1
/tmp/issue-1582-js-tools/shellcheck-v0.11.0/shellcheck experiments/issue_1582/continuation-latest-main-*.sh experiments/issue_1582/continuation-api-language-check.sh
git diff --cached --check origin/main -- . ':(exclude)dev/log/**'
git commit -m 'Merge main request-context support and verify client API language parity' > /tmp/issue-1582-latest-main-commit.log 2>&1
