# Complete terminal CI record for 9e945eae

All 32 workflows for published commit
`9e945eae3f1f3a03a2b9e3302bdb81000f7820b1` are terminal: 29 succeed,
one skips and two fail. The retained run observations, job states and complete
gzip log streams live under
`experiments/issue-1580/evidence/ci/seed-readers-9e945eae/`.
Thirty-one nonempty workflow logs were scanned for actual warning/error lines;
28 occurrences form 20 distinct messages. Full byte digests and line locations
are recorded. Echoed shell programs are separate from emitted annotations.

## Actionable failures

Layered CI run 44938456935 fails on six source/transport checks already repaired
in local commit 2c81c95, plus the unchanged stale WASM worker-input receipt.
The Pipeline run 44938457098 fails at lines 57558–57589: the native structured
output HTTP suite omits the `formalAiSeedRecordReaders` prerequisite (the exact
ReferenceError is at 57565). The same local repair adds the actual prerequisite
and a focused VM regression. The final Pipeline failure at 72747–72750 correctly
propagates that job failure; it is not an aggregation false positive.

All nine native contract groups and release registry scripts pass on this
published head. Later local changes require fresh hosted validation. The
cycle-rejection panic at 57251 belongs to a passing bounded negative control
(`nested_reusable_jobs_are_visible_and_cycles_are_bounded`, success at 57254),
within the 195 passing workflow contracts. It must not be silenced as a defect.

## Other diagnostics

The BuildKit canonical-registry failure at artifact-contract run 44938456883,
line 662, is the deliberate fallback control; the mirror branch succeeds.
The Windows artifact download at line 2281 is a genuine transient visibility
failure recovered by the second bounded download, followed by successful byte
round-trip verification. Its original failure remains visible. The Git pathspec
error at Layered line 26093 is the successful untracked-owner rejection test.

Pipeline size warnings at 2615–2635 name the traceability table, conversation UI,
eight immutable captures and the live canonical staged workflow. The first two
are now partitioned without changing behavior; the eight captures are losslessly
archived with complete-byte conservation. The canonical workflow remains open.

Layered groups 2 and 3 warn after 270/360 seconds (3540, 18658). Native coding-1
and Hive-3 warn after 675/900 seconds (69417, 72387). Complete per-file and native
timestamps are retained for a measured scheduling/compile-reuse repair. No clock
or warning threshold was increased. The crates.io token remains explicitly
unverified at Pipeline 2250: public reads cannot certify token authority.

The stale WASM receipt requires actual hosted regeneration. Its inactive
read-only proposal awaits the explicit build-policy exception; no local Rust
compilation, false identity substitution or hand-written receipt is claimed.

Supplied analysis and procedures receive zero independent Formal AI authorship
credit. This evidence records unfinished work and does not complete issue #1580.
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
