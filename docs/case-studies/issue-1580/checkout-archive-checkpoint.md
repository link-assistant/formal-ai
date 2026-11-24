# Fast source checkout archive checkpoint

This is an unfinished checkpoint for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581),
not full acceptance of issue #1580.

## Published source and failure

All 23 workflows on `6b1d75a62a610e58914ef594c107f3fa402ded9d` completed.
Twenty-two pass after one unchanged-head failed-job retry. Layered CI
[44207032757](https://github.com/link-assistant/formal-ai/actions/runs/44207032757)
passes its retry. The pipeline's native type/lint job passes, but pipeline
[44207033000](https://github.com/link-assistant/formal-ai/actions/runs/44207033000)
still fails before fast source checks execute. Its second-attempt complete log
records curl 28, early EOF and invalid index-pack output at lines 123–128,
followed by the required aggregate failure at lines 12226–12227.

The checkout reports missing promisor object
`9df886c42a1e2082f1471383d986fea4d531f8ac`, the six-byte `.bun-version`.
A fresh seven-file remote partial checkout succeeds locally under the same
1024-byte/20-second low-speed policy, using 3.662 MB of pack data and 38,912 KiB
peak child memory. This proves availability from this different host/Git
version; it does not reproduce the full hosted checkout or establish the
transport failure's root cause. Both head and hosted merge trees are identical.
All attempts, complete logs, hashes and source identities are retained under
`experiments/issue-1580/evidence/ci/published-6b1d75a6/`.

## Smaller source input set

The fast source job previously selected 33,599 tracked files, approximately
733 MB. Its existing exclusions already omit unmeasured historical archives.
Extend that policy to `.gz`, `.log`, `.zip`, `.tgz` and `.diff` files only under
`docs/case-studies/` and `experiments/*/evidence/`. The proposed selection retains
24,020 files, approximately 413 MB, a reduction of 319,759,182 bytes.
These are working-tree byte counts, not measured remote pack sizes.

The regression file passes 15/16 before the workflow change, failing precisely
on retained historical archive payloads, and 16/16 afterward. It checks every
tracked measured source and all readable source inputs, requires each omitted
archive to remain unmeasured, and retains authored diagnostic code, JSON and
Links Notation. Source checks, event ref selection, credential settings, shallow
blobless acquisition, job/checkout clocks and low-speed limits remain intact.

The actual 20 underlying fast source checks pass against a temporary copy
materialized with the proposed selection. The first outer invocation hits
Formal AI's 60-second shell limit after emitting all 20 individual records;
19 checks pass and the hosted-only debt CLI refuses local execution. Preserve
that incomplete invocation and its refusal. A separate invocation of the same
exported debt measurement/check functions passes without impersonating GitHub
Actions. The hosted-only governance wrapper's timing is not tested locally.
The temporary copy is removed; no local native build or execution occurred.
Receipts and complete individual logs are under
`experiments/issue-1580/evidence/checks/fast-source-archive-checkout/`.

This is verified input reduction. Its effect on the hosted checkout remains
unverified until the next published commit completes CI. Do not represent a
smaller checkout as a proved transport root cause or an already green pipeline.

The required deployed staged-release check initially rejects the stale canonical
source after the checkout change. Refresh that non-release job through the
maintained generator. All 52 staged operations, 104 bindings and the deployed
staged workflow remain byte-for-byte unchanged; only the canonical source hash
changes in the packet. The deployed projection check then passes. Seven touched
local gates and the pinned-runner check pass. All three stage transition
regressions pass. The first invocation preserves a 2/3 result because inherited
`NODE_OPTIONS` violates the proof subprocess guard. Clearing that inherited
variable, while retaining the test process CLI heap cap, passes 3/3; the
production environment guard remains intact.

The final publication checks find one new source-budget failure: the expanded
checkout block makes the canonical workflow 1,513 lines, above the unchanged
1,500-line ceiling. Preserve both failing gate invocations and their complete
diagnostics. Use the workflow's existing compact flow style for exactly the same
parsed configuration; the canonical source returns to 1,494 lines. The maintained
generator verifies identical staged workflow bytes and all 104 bindings again.
Recheck the affected size gates, source profile, stage transition and projection
before publication; their final receipts are retained separately.

## Java historical comparison

Literal review now covers pages 1–38 and 48–55 of 254: 52 complete files, two
partial files and 2,141 complete source lines. Pages 39–47 remain literally
unread. A full parsed comparison separately covers the compile/test OSV
records: all three differences are read and equal vulnerability subtrees reuse
the fully read compile response. This does not turn skipped literal fragments
into source-read credit. The historical plugin scan finds no package sources;
resolved plugin audit coverage still needs its distinct inventory.

The historical actionlint screenshot is authenticated, image-validated and
viewed. Its receipt records a `concurrency.queue` rejection; it is not current
application UI or a new actionlint execution. Normalized historical logs are
identified as normalized, rather than original command-byte evidence.

Complete template applicability, native regressions, benchmark capability,
live identity/delivery and all previously unchecked acceptance criteria remain
unfinished. Independent Formal AI reasoning acceptance remains 0/20; supplied
scripts, commands and patches receive zero independent authorship credit.


## Published 7e89122c result

The 2026-11-18 01:41 UTC snapshot records all 23 workflows passing on
`7e89122c9d030b3af5eec2d346141bba3ff3fdab`. Pipeline 44213727766 passes its
source checkout and checks. Layered run 44213727267 has a separate checkout-only
failure followed by one successful unchanged-head failed-job retry. Preserve both
attempts; the Layered checkout is unchanged by this patch. These observations
verify the published source checks, without proving a transport root cause or
native runtime acceptance. See the [native evidence checkpoint](native-evidence-checkpoint.md).


## JavaScript test checkout on 9095abcb

All 23 workflows finish on published head
`9095abcbcccdd5518ce2c9318d3ebc6eb8914fc8`: 22 pass. Layered CI run
[44262999002](https://github.com/link-assistant/formal-ai/actions/runs/44262999002)
fails in both JavaScript groups during checkout in attempt 1 and in a single
unchanged-head failed-job retry, attempt 2. The complete 171-line retry job logs
for jobs 134665248683 and 134665249345 are literally read. Lines 124–126 record
curl 28 (1024 bytes/second for 20 seconds), a bad pack header and a missing
promisor fetch; line 155 records exit 128. Tests never start. The two groups run
in AWS us-west-2 and Azure eastus. These observations do not establish the
server-side cause or total missing-blob pack size.

The existing JavaScript selection includes approximately 631.7 MB of tracked
blobs. Extend the fast source job's historical archive policy to the JavaScript
test checkout, with explicit inclusions for the client transcripts, issue-982
native-session log and issue-932 language logs that tests actually consume.
Retain all measured source, every Cargo.lock, JSON session operands and the real
nested-context learning report. The existing event ref, shallow blobless fetch,
credential handling, 5-minute checkout cap, 12-minute job cap, 1024-byte/20-second
low-speed policy and 360-second test budget remain intact.

The new regression fails 16/17 before the workflow selection changes, precisely
because historical archives remain selected, and passes 17/17 afterward.
A temporary selection retains 23,313 files and 301,945,027 current working-tree
bytes. All 88 tests across eight individually run JavaScript files pass in that
selection, including actual compressed client and native-session operands and
all seven language-verification logs. Every selected original file's SHA-256 is
unchanged afterward. Peak child RSS is 390,252 KiB; cgroup OOM counters stay zero.
The temporary hard links and isolated Git index are removed. The first foreground
harness is interrupted by the local Formal AI 60-second command cap after three
files pass; its incomplete records are preserved separately. The complete
background harness finishes in 80.58 seconds. The 178-line combined log view is
literally read in full. No native build or execution runs locally.

All receipts and complete logs are under
`experiments/issue-1580/evidence/checks/javascript-archive-checkout/`; exact-head
workflow, job and failed attempt records are under
`experiments/issue-1580/evidence/ci/executor-status-9095abcb/`.
All eight affected local gates and the whitespace check pass; complete gate logs
are literally read. The smaller selection is locally verified. A new hosted
checkout is still required; working-tree size reduction alone does not prove
transport causality.

The current native workflow passes independently: job
[134660858273](https://github.com/link-assistant/formal-ai/actions/runs/44262999525/job/134660858273)
actually executes and passes four executor tests, one calendar/currency test and
one forced-function test, followed by two native HTTP tests. The executor test
replays all 29 shared cases and complete event sequences. Only log lines
1605–1722 are claimed literally reviewed; the complete immutable log is retained.
This verifies those named regressions, not the broad unfinished issue acceptance.
Independent Formal AI reasoning acceptance remains 0/20.

The final staged gate run catches the newly tracked abbreviated gate-log filename
(91 names against the unchanged 90-name ceiling). Rename only that archive to
`gate-07-check-tests-as-documentation.log.gz`, preserving compressed and raw bytes
and the failed log. All three corrected staged gates pass. Preserve the literal
review diff compressed, including its patch-context spaces, before the final
whitespace check.


## Published bfe5ac0f result

All 23 workflows pass on `bfe5ac0f3babe512f1bf0a03935bd1d220f9e40f`.
[Layered CI 44269762053](https://github.com/link-assistant/formal-ai/actions/runs/44269762053)
finishes without a retry. The two original source checkouts finish in 19 and
17 seconds. The complete JavaScript shard totals report 1,440 passes and four
skips in group one, and 1,253 passes and 13 skips in group two, with no failures
or cancellations. Focused changed-component checks also execute successfully.
The original limits and test budgets remain intact. This verifies this hosted
checkout and these tests; the earlier transport root cause remains unknown.

Complete direct job logs, paginated workflow/job responses, source identities
and bounded literal-review receipts are retained under
`experiments/issue-1580/evidence/ci/javascript-checkout-bfe5ac0f/`.
The logs contain 4,997 and 2,745 lines; only the specifically recorded metadata,
command headers and test totals are claimed literally read. The first collector
refuses terminal escape sequences before downloading one log; retain that failure
and retry with GitHub CLI's explicit escape-sequence flag. No test rerun follows.

All 83 lines of the three published source patches are literally reviewed.
The full PR diff remains unavailable through `gh pr diff` because GitHub rejects
more than 300 changed files; individual commit patches and blob identities remain
reviewable. Preserve the original immutable publication receipts and the later
review receipt separately. Expand two newly archived abbreviated filenames
without changing either archive's bytes or the original manifest.
The broader issue remains unfinished, and independent reasoning acceptance
remains 0/20.
