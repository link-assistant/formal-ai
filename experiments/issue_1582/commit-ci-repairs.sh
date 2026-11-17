#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
rg -q '^138/138 passed or skipped; failed: none$' /tmp/issue-1582-gates-round-8.log
python3 - <<'PY'
from pathlib import Path
p = Path('docs/case-studies/issue-1582/README.md')
s = p.read_text()
s += '''
The eighth complete local gate round, after these CI repairs, also passes or
explicitly skips all 138 entries with no failures. The role registry's independent
Python check passes, and all 22 logs for the first repair head are retained.
The next pushed head must still be checked on hosted CI; earlier passing checks
are not reused as evidence for it.
'''
p.write_text(s)
p = Path('docs/case-studies/issue-1582/work-plan.md')
s = p.read_text().replace('round seven verifies the repaired final tree: 138/138 passed or explicitly skipped, no failures.', 'round seven verifies the native-route repairs and round eight verifies the subsequent CI fixes: each has 138/138 passed or explicitly skipped, no failures.')
p.write_text(s)
p = Path('experiments/issue_1582/pull-request.md')
s = p.read_text().replace('The complete local gate inventory passes or explicitly skips all 138 entries with no failures;', 'The complete local gate inventory passes or explicitly skips all 138 entries with no failures, including round eight after the first hosted CI repairs;')
s = s.replace('**Unfinished acceptance criterion:**', 'The first hosted repair head completed 22 workflows: 19 succeeded. All three failures were investigated together: regenerate the permission-role registry, refresh the governed specification witness (complete inventory unchanged; 36 focused witness tests pass), and acknowledge the exact unmaintained chrono advisory through the existing dependency-graph-checked upstream-exception mechanism. Latest introducing dependencies still require chrono; migration reports are https://github.com/link-foundation/command-stream/issues/238 and https://github.com/link-assistant/calculator/issues/265. Every log and the advisory/source evidence are retained. The production native type/lint job passed; the four native regressions remain unexecuted. The next repair head still requires hosted CI verification.\n\n**Unfinished acceptance criterion:**')
p.write_text(s)
PY
python3 experiments/issue_1582/archive-evidence.py
git add .cargo/audit.toml data/meta/notation-ratchet.lino data/seed/roles.lino experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8
git diff --cached --check
git commit -m "fix: refresh governed registries and record blocked chrono advisory"
git add docs/case-studies/issue-1582 experiments/issue_1582
git diff --cached --check
git commit -m "docs: retain complete repair CI logs and witness review evidence"
test -z "$(git status --porcelain)"
