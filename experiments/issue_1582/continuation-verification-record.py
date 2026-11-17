"""Supplied continuation record; claims are bounded by retained execution evidence."""
from pathlib import Path
import json
case=Path('docs/case-studies/issue-1582/README.md')
text=case.read_text()
section='''

### Continuation verification and remaining limits

The permission examples are frozen in
`data/parity/issue-1582-permission-answers.json`. Both the serving-worker test
and the drafted Rust unit test assert these exact localized answers in addition
to routing evidence. The Rust recognizer normalizes seed surfaces and rejects
empty surfaces, matching the JavaScript/TypeScript contract.

The focused JavaScript results are 19 permission checks, 12 defaults/context
checks, 10 structured-log checks and six real Node CLI checks: **47 passed**.
The governed-witness suite passes **34/34**; the merged issue-1588 suite passes
**62/62**. Rustfmt passes on the changed native files. These are JavaScript,
source-consistency and formatting observations, not native test execution.

After registering the native unit module, four existing complete programs
observe the changed ancestor `rust/tests/unit/mod.rs`. The second reviewed
witness refresh compares every complete object, changes only those four
provenance hashes and preserves all 221 cases, their assertions, initial values,
unsupported records and source manifest. The exact comparison receipt is
`raw-data/local/continuation-witness-refresh.json`.

The complete local-gate run exercised all 138 entries: 129 passed or were
explicitly skipped, and nine failed. Eight failures were repaired or resolved
by installing the actual local prerequisites: the abbreviated recipe filename,
two lower abbreviation ceilings, the runtime-feature source hash, exact native
answer examples, and missing i18n/YAML/ShellCheck/pinned-parser tools. Their
focused reruns pass. The initial logs remain under
`raw-data/local/continuation/gates-initial/`, with final reruns alongside them.

The remaining dependency-audit failure is new evidence, not a passing gate.
[GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf)
was updated/reviewed at 2026-11-17T15:56:44Z, after the earlier green source
head. GitHub currently flags every published picomatch version through 4.0.7
and lists no patched release. The desktop and VS Code build dependency trees
are affected. Registry metadata, audit output and upstream proposals are
retained. The [maintainer disputes the advisory](https://github.com/micromatch/picomatch/pull/235#issuecomment-6315749381),
but GitHub has not withdrawn it. Proposed fixes #237/#238/#246 are unmerged;
#238 also documents an uncovered pattern class. Neither disabling the audit
nor substituting an unqualified vendored implementation establishes the
existing dependency contract. Dependency qualification remains unfinished.

The fresh Gemini source probe pins
`4fa64e71b9b4e1bb65a86cc2863574f561b82340` (2026-11-16T17:38:59Z).
Its actual budget consumers still select 1,048,576 for the unknown model while
the served metadata advertises 1,234,567. The open upstream issue, fixed native
test policy and unavailable reporter Mac logs remain explicit limits.

The Formal AI shell has a fixed 60-second command deadline and cleans up
detached children. Combined checks and the pinned-source acceptance check
therefore used explicit outer-command verification exceptions, sequentially,
with complete logs; no local Rust compilation occurred. Temporary tool
installations are excluded from repository source measurements. Supplied repair
recipes receive zero independent authorship credit. Of the frozen 20 work
units, one independent requirement-gap query was attempted (5%); none produced
an independently accepted implementation (0%). Those percentages do not count
command execution or supplied edits as reasoning or authorship.
'''
if '### Continuation verification and remaining limits' not in text:
 case.write_text(text+section)
requirements=Path('docs/requirements/issue-1582-client-read-access-and-context.md')
text=requirements.read_text().replace('native wrapper and file sink changed, awaiting native CI.','native wrapper and file sink changed; runtime regressions remain unexecuted under pre-merge policy. Production type/lint checks do not execute them.').replace('Native adapter regression awaits CI.','Native adapter runtime regression remains unexecuted under pre-merge policy.').replace('final current-head CI evidence remains to collect.','current continuation CI evidence is tracked in the PR case study; the new picomatch advisory also leaves dependency qualification unfinished.')
requirements.write_text(text)
plan=Path('experiments/issue_1582/continuation-plan.md')
text=plan.read_text()
text=text[:text.index('Status:')]+'''Status after implementation and local verification:

- Units 1-7, 9-12: reviewed source/configuration census and implemented the
  newly reproduced routing cause in all three source roots; 19 shared JS/TS
  checks pass. Installed vendor CLI execution is not inferred from the census.
- Units 8 and 14: Gemini input context remains unfinished; current pinned
  upstream consumers ignore advertised custom-model context. Evidence retained.
- Unit 13: six real Node CLI checks pass; native wrapper execution is unfinished.
- Unit 15: five drafted native regressions remain unexecuted under pre-merge
  build policy. Rustfmt/type/lint evidence is not runtime evidence.
- Unit 16: 47 focused JS checks, 34 witness checks and 62 merged regression
  checks pass. All 138 gates ran; eight of nine failures have passing focused
  reruns. The newly listed picomatch advisory remains an external audit failure.
- Units 17-18: case/requirement records updated; complete production diff read.
  GitHub rejects `gh pr diff` for this >300-file archive; paginated file metadata
  and local merge-base diffs are the explicit review fallback.
- Units 19-20: pending the single push gate, current-head hosted checks and
  remote review metadata. Readiness cannot be claimed while acceptance is open.

Independent Formal AI query attempts: 1/20 (5%). Independently accepted authored
changes: 0/20 (0%). Supplied commands and recipes get zero authorship credit.
'''
plan.write_text(text)
pr=Path('docs/case-studies/pull-request-1583')
(pr/'raw-data').mkdir(parents=True,exist_ok=True)
source=Path('/tmp/issue-1582-review')
for name in ['pr-current.json','pr-comments-current.json','inline-comments.json','reviews.json','recent-runs.json']:
 if (source/name).exists():(pr/'raw-data'/name).write_bytes((source/name).read_bytes())
(pr/'README.md').write_text('''# Pull request #1583 review record

PR: https://github.com/link-assistant/formal-ai/pull/1583
Issue: https://github.com/link-assistant/formal-ai/issues/1582

The issue's reproductions, product requirements, root causes and hashed evidence
belong in [the issue case study](../issue-1582/README.md). This record tracks the
review conversation and validation decisions for the pull request.

On 2026-11-17 at 14:55Z the prior session explicitly left the PR draft with
Gemini input-context support and native regression execution unfinished. A
15:06Z conversation comment said ready to merge; that statement does not
resolve either acceptance criterion and the actual PR remains draft.

At 17:29:18Z the reviewer requested a complete check of supported tools and
natural/programming languages. All conversation comments, inline review
comments and review decisions were read. The latter two inventories are empty.
The continuation enumerates 12 adapters, 25 declared modes, five natural
languages and the JavaScript, TypeScript and Rust source roots. Source/config
inspection is distinguished from an installed third-party client execution.

Main at `512ce428029a92687628b4180309ee5ab660fe08` was merged forward in
`d9d3f299b`; both regression inventories and complete governed cases survive.
The review found paraphrases missed permission guidance and quoted task input
could take over its routing. A shared fixture reproduced 12 failures out of 18
checks. Seed aliases, anchored ownership and quote exclusion repair the cause
in all three roots; 19 final JS/TS checks pass. Native regressions are drafted,
not executed, because the repository excludes pre-merge native test builds.

Local verification passes 47 focused JS checks, 34 complete-witness checks and
62 checks for the merged main changes. The full 138-entry gate inventory was
run. Eight of its nine initial failures have passing focused reruns; the last
is a newly reviewed picomatch advisory with no patched release. The advisory
and upstream response are preserved in the issue archive. No audit waiver or
unqualified dependency replacement was used to manufacture a green result.

Current-head hosted runs will be recorded after the complete repair batch is
pushed. The PR stays draft and unmerged while Gemini input context, native
runtime validation or dependency qualification is unfinished. The positive
closing reference required by the current task remains in the PR description;
it does not claim that pending acceptance is complete.
''')
print(json.dumps({'caseLines':len(case.read_text().splitlines()),'nativeRuntimeExecuted':False,'dependencyAuditPassing':False}))
