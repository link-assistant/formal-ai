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
