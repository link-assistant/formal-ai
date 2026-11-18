from pathlib import Path
import shutil
review=Path('/tmp/issue-1582-review')
for name in ['before-drive','after-drive','verify-drive','fixed-full','shell-gate','size-gate','abbrev-gate','failed-run']:
 suffix='.json' if name=='failed-run' else '.log'
 source=Path('/tmp/issue-1582-secrets-'+name+suffix)
 assert source.exists(),source
 shutil.copyfile(source,review/source.name)
for source,target in [('/tmp/issue-1582-final-pipeline-failed.log','ci-logs/pipeline-44152920794.log'),('/tmp/issue-1582-current-runs.json','secrets-round-runs.json'),('/home/box/.npm/_logs/2026-11-17T19_02_13_667Z-debug-0.log','npm-e2big-debug.log')]:
 target=review/target;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
assert 'No secrets found.' in (review/'issue-1582-secrets-fixed-full.log').read_text()
for name in ['shell','size','abbrev']:
 assert 'failed: none' in (review/('issue-1582-secrets-'+name+'-gate.log')).read_text()
path=Path('experiments/issue_1582/continuation-secrets-probe.mjs')
source=path.read_text().replace("const mode=process.argv[2];", "const mode=process.argv[2];\nconst cli=process.env.SECRETLINT_CLI;\nif(!cli || !fs.statSync(cli).isFile())throw Error('Set SECRETLINT_CLI to the pinned secretlint 13.0.5 CLI');")
source=source.replace("'/home/box/.npm/_npx/9f9ee0a01c627d54/node_modules/secretlint/bin/secretlint.js'",'cli')
path.write_text(source)
text='''

### Final-head Secrets Scan failure and repair

The evidence head `a0fb531c57a3067f48640519a884ac97adfd7656` completed
21 workflows successfully, including Layered CI. Its
[CI/CD run](https://github.com/link-assistant/formal-ai/actions/runs/44152920794)
failed only Secrets Scan and its aggregate status. The complete retained log
`ci-logs/pipeline-44152920794.log.gz`, decoded lines 7978-7981, shows 1,729
files followed by exit 123 without a credential finding. The run started at
18:54:50Z after that head was pushed and finished at 18:57:29Z.

The exact command reproduced exit 123 locally. npm's retained debug log
reports `spawn E2BIG`: xargs' default 128 KiB batch becomes one larger quoted
shell argument inside npm, exceeding Linux's per-argument limit. Invoking the
same pinned scanner directly on all files passed in both glob and literal
modes. This was an invocation-size failure, not a credential finding.

`scripts/check-secrets.sh` now caps xargs input batches at 16 KiB, leaving room
for quoting and npm's command prefix. The existing hosted subprocess-boundary
test file adds a finite 1,800-path fixture with spaces, apostrophes and Unicode.
It fails before the fix with exit 123, passes afterward, checks every filename
arrives exactly once in order, and checks that scanner findings still fail the
gate. All 14 checks in that file pass. The real pinned scanner then passed all
1,729 changed files, and shell lint, file-size parity and abbreviation gates
passed. These complete driver and verification logs are retained; supplied
recipes receive zero independent authorship credit. The repair preserves the
pinned scanner and every selected file. The next head's hosted checks are
tracked separately without recursively committing their own logs.

This CI repair does not resolve the previously recorded Gemini input-context,
native runtime verification or dependency audit blockers. The PR remains draft.
'''
path=Path('docs/case-studies/issue-1582/README.md');path.write_text(path.read_text()+text)
path=Path('docs/case-studies/pull-request-1583/README.md')
path.write_text(path.read_text()+'''\n\nThe evidence head `a0fb531c` exposed an independent CI invocation bug:
[Secrets Scan](https://github.com/link-assistant/formal-ai/actions/runs/44152920794)
exited 123 after npm hit `spawn E2BIG` while shell-quoting the changed-file list.
The repair bounds batches at 16 KiB. Its 1,800-path regression fails before and
passes after; all 14 subprocess checks and the real 1,729-file scan pass.
The issue case study retains the full failure and verification logs. All other
21 workflows on that evidence head passed. Current-head checks are verified
through the PR checks view; unchanged acceptance blockers keep this PR draft.
''')
path=Path('experiments/issue_1582/continuation-plan.md')
path.write_text(path.read_text()+'''\nAdditional units 2/9/10/16/19/20 verification: the evidence head completed
21 workflows successfully; Secrets Scan failed on npm `spawn E2BIG`. A
reproducing 1,800-path regression failed before and passed after bounding xargs
batches at 16 KiB. All 14 subprocess checks, real 1,729-file scan and three
relevant gates pass. The complete CI failure was collected before preparing
one additional repair round. The frozen denominator remains 20 work units.
''')
body=review/'pull-request-body.md'
source=body.read_text().replace('The final evidence commit changes no application source and is checked separately.', 'The evidence head exposed a Secrets Scan invocation bug: npm hit `spawn E2BIG` on the changed-file list. A 16 KiB batch limit repairs it; a 1,800-path regression fails before and passes after, all 14 subprocess checks pass, and the real 1,729-file scan passes. [The failed run](https://github.com/link-assistant/formal-ai/actions/runs/44152920794) and local before/after evidence are retained. The repair head is checked separately.')
body.write_text(source)
