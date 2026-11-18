from pathlib import Path
import hashlib, json, shutil, subprocess
review=Path('/tmp/issue-1582-merge-review')
for source,name in [('/tmp/issue-1582-latest-main-fetch.log','fetch.log'),('/tmp/issue-1582-latest-main-commits.log','main-commits.log'),('/tmp/issue-1582-latest-main-stat.log','main-stat.log'),('/tmp/issue-1582-latest-main-merge-drive.log','merge-drive.log'),('/tmp/issue-1582-latest-main-source.diff','js-source.diff'),('experiments/formal_ai_subagent/sandboxes/gates/check-javascript-dependencies.log','dependency-audit.log')]:
 shutil.copyfile(source,review/name)
assert (review/'all-gates.exit').read_text().strip()=='1'
assert '137/138 passed or skipped; failed: check-javascript-dependencies' in (review/'all-gates.log').read_text()
for filename,count in [('request-context.log',4),('permissions.log',35),('defaults.log',12)]:
 assert 'pass '+str(count) in (review/filename).read_text(),filename
parent=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
previous=[r for r in json.loads((review/'previous-head-runs.json').read_text()) if r['headSha']==parent]
assert len(previous)==23 and all(r['status']=='completed' and r['conclusion']=='success' for r in previous)
witness='experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8'
assert subprocess.check_output(['git','hash-object',witness])==subprocess.check_output(['git','rev-parse','HEAD:'+witness])
(review/'receipt.json').write_text(json.dumps({'mergeParent':parent,'mergedMain':subprocess.check_output(['git','rev-parse','origin/main'],text=True).strip(),'focusedChecks':51,'localGates':138,'passedOrSkipped':137,'remainingFailure':'check-javascript-dependencies: GHSA-fc5q-39xc-vjpf','previousHeadSuccessfulWorkflows':23,'governedWitnessBytesUnchanged':True,'localRustBuild':False},indent=2)+'\n')
# Allow a new evidence round to retain its logs without overwriting earlier rounds.
path=Path('experiments/issue_1582/continuation-archive.py')
source=path.read_text().replace('import gzip, hashlib, json, shutil','import argparse, gzip, hashlib, json, shutil')
source=source.replace("root=Path('docs/case-studies/issue-1582')", "parser=argparse.ArgumentParser()\nparser.add_argument('--source',default='/tmp/issue-1582-review')\nparser.add_argument('--subdir',default='')\nargs=parser.parse_args()\nroot=Path('docs/case-studies/issue-1582')")
source=source.replace("temporary=Path('/tmp/issue-1582-review')", "temporary=Path(args.source)\ndestination=local/args.subdir")
source=source.replace('retain(source,local/source.relative_to(temporary))','retain(source,destination/source.relative_to(temporary))')
source=source.replace("if source.exists():retain(source,local/source.name)","if not args.subdir and source.exists():retain(source,local/source.name)")
source=source.replace("for source in Path('experiments/formal_ai_subagent/sandboxes/gates').glob('*.log'):retain(source,local/'gates'/source.name)","if not args.subdir:\n for source in Path('experiments/formal_ai_subagent/sandboxes/gates').glob('*.log'):retain(source,local/'gates'/source.name)")
assert 'destination=local/args.subdir' in source
path.write_text(source)
path=Path('docs/case-studies/issue-1582/README.md')
path.write_text(path.read_text()+'''

### Merge of the request-context changes from main

While the scanner repair was being verified, main advanced to
`826289fc78154dcd894199ed966f366176be0afb` (PR #1595). The merge preserves
both request-context handling and client permission routing. Only three
requirement/status files conflicted. Both issues' traceability rows survive;
the maintained generators regenerated status and progressive-plan projections
from 1,529 requirements, with the existing partial client criteria retained.

All 51 focused checks pass on the merged tree: four request-context checks,
35 shared permission checks and 12 retrieval/log/context defaults checks.
The complete 138-entry JavaScript gate inventory ran sequentially: 137 pass
or are explicit policy skips, and only the known dependency audit advisory
fails. The governed witness bytes are unchanged. No local Rust build ran.
The full gate run used the previously documented outer-command exception to
the Formal AI shell's fixed 60-second deadline; supplied recipes receive zero
independent authorship credit. Logs for this round live under
`raw-data/local/continuation/latest-main/` and preserve earlier rounds.

All 23 workflows passed for scanner-repair head `721d89d03`. Its retained
CI/CD log records the 1,752-file scan at lines 4665-4667, and its Layered CI
log records the 1,800-path regression at line 2966. These verify the repair
on the hosted runner. The merge head's checks are verified separately through
the PR checks view. Gemini input context, native runtime validation and
picomatch qualification remain unfinished, so the PR remains draft.
''')
path=Path('docs/case-studies/pull-request-1583/README.md')
path.write_text(path.read_text()+'''

All 23 workflows passed for scanner-repair head `721d89d03`, including the
real scan and the new hosted regression. Main then advanced to `826289fc7`
(PR #1595). The merge preserves both request-context and permission behavior;
51 focused checks pass, and the full gate rerun passes/skips 137 of 138,
with only the same active dependency advisory failing. Requirement projections
now include all 1,529 records. New raw logs are kept in a separate latest-main
round; final merge-head checks are tracked through the PR checks view.
''')
path=Path('experiments/issue_1582/continuation-plan.md')
path.write_text(path.read_text()+'''

Units 2/3/16/17/19/20, latest main: merge `826289fc7` (PR #1595),
preserve both requirement tables, regenerate 1,529-record projections.
All 51 focused checks pass. The complete 138-entry gate rerun passes/skips
137, with only the previously recorded picomatch advisory failing. All 23
workflows on scanner-repair head `721d89d03` pass. The governed witness is
byte-identical. No local Rust build; full-gate execution uses the documented
outer-command deadline exception. Final merge-head CI remains to be checked.
''')
