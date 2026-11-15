# Follow-up to PR #1523: source acquisition and native artifact binding

PR #1523 merged as `101992d84e7b12c843722992050585192dfc3996` after all
102 exact-head checks were terminal: 33 succeeded and 69 were deliberately
skipped. The new main source is not a qualified release. Its native producer
compiled successfully and executed all eleven declared creation fixtures;
subsequent artifact binding failed before captures and publication.

## Observed failures and fixes

- [VS Code resolver job](https://github.com/link-assistant/formal-ai/actions/runs/41716378553/job/126197341261)
  exhausted five minutes fetching the complete repository before checkout or
  resolution. Its eleven imported modules use builtins and reside in
  `scripts/`; resolution needs no other repository assets. Retain those
  scripts and `package.json`, the exact `github.workflow_sha`, existing
  credentials policy, release criteria and all publication jobs. Allocate two
  minutes to acquisition, retain the original three-minute resolver bound and
  allow one minute for setup within a six-minute job.
- [Native producer](https://github.com/link-assistant/formal-ai/actions/runs/41716378863/job/126198094791)
  passed compilation and the actual eleven-fixture gate, then its 90-second
  binding deadline expired during the complete physical tracked-input witness.
  Git metadata reports 74,024 blobs and 1,947,940,314 logical bytes at this
  source. These are metadata measurements, not local physical file reads or a
  transfer-size estimate. The existing phase marker did not isolate inventory,
  test-tree inspection and physical hashing, so the exact expensive file or
  syscall is unproven.
- The full JavaScript group 2 ran 825 tests: 824 passed and its sole failure
  expected the old twenty-minute producer cap instead of the approved
  twenty-five-minute allocation. Its native source-contract twin retained the
  same obsolete expectation. Update both precise expectations, preserving all
  ordering, receipt and smoke-test requirements.
- [Corpus aggregate](https://github.com/link-assistant/formal-ai/actions/runs/41716378795/job/126212088432)
  spent its five-minute budget on initial fetch, before any corpus validator
  ran. The aggregator needs only `scripts/check-shard-results.sh`; retain all
  dependency/quorum and unknown-answer checks while narrowing its acquisition.

- [Public-download resolver](https://github.com/link-assistant/formal-ai/actions/runs/41721844212/job/126216362728)
  exhausted its three-minute acquisition budget before selection. Retain the
  exact trusted workflow source, its twelve builtin-only script imports and
  package boundary. Reserve separate two-minute checkout and three-minute
  selection phases within six minutes; every downstream public-byte, browser
  and Mac verification job remains unchanged.
- The full Coding Ladder shard lost 1,121 seconds to checkout before running
  only 746 seconds of its declared 1,200-second suite under a thirty-minute
  job. Its incomplete aggregate correctly failed. Printed baseline misses are
  not a newly established ratchet violation. Actual tasks clone the complete
  repository and execute declared verification commands, so sparse exclusions
  are not justified. Preserve the entire real benchmark, data and ratchets
  behind explicit main-only `full-ladder=true` or
  `FORMAL_AI_FULL_CODING_LADDER=run`. Ordinary dispatches do not opt in. Keep
  default registered consistency and mandatory native full/specification and
  Agent coding/reporting gates. Deferral earns no fresh benchmark credit;
  improving whole-tree transport and the benchmark phase allocation remains
  recorded technical debt. Vision, contributor guidance and standing CI
  doctrine record this release-first policy.

## Physical witness and resource limits

Keep every committed workspace blob. The synchronous reader reuses a buffer
bounded to one MiB, incrementally computes each Git SHA-1 and the existing
sorted `name\0length\0bytes` SHA-256 stream, checks descriptor identity and
size stability, and closes descriptors on failure. It rejects modified,
missing, symlinked and untracked test input. Git object contents, index values,
precomputed hashes and caches never replace actual physical bytes.

Add bounded aggregate diagnostics for inventory, test-tree inspection and
physical hashing; no file contents or names enter progress records. Retain the
receipt format and synchronous callers. Give only the artifact-binding command
a finite 300-second deadline; the separate compiled-fixture deadline remains
90 seconds and the producer remains bounded to twenty-five minutes. Actual
hosted timings must qualify the allocation. Streaming reduces allocation size;
it does not establish that the full witness will finish in ninety seconds.
Other capture and consumer budgets remain independently bounded and must be
observed, rather than inferred from the binding allowance.

Refresh the canonical release caller and its generated source/bindings, and
only the retained producer payload hash affected by the binding allowance.
Keep every existing staged operation and the deployed staged workflow exact.
The cheap native artifact and physical identity regressions join fast PR
coverage without removing any previous selection.

## Verification and work arrangement

Use three workers on this single repair PR: one monitors actual main/check and
release logs, one drafts the smallest guarded implementation through Formal AI
JavaScript, and one independently derives regressions and reviews source
boundaries. The root agent integrates explicit paths, batches all known fixes,
and pushes after the batch is committed. Failed English edit attempts remain
recorded; supplied bootstrap code executed through Formal AI earns zero
independent synthesis credit.

Local verification uses one focused JavaScript file at a time, with a
384-MiB heap and an external twenty-five-second deadline. No Rust compiler,
Rust test, native product, product WASM, Docker or app build runs locally.
The identity regression uses tiny independent Git/physical fixtures; it does
not hash the real repository. Its first two controls remain unchanged.

A passing PR or a skipped/no-op child does not establish production delivery.
After merge, require actual complete producer uploads, full native and client
gates, then a new source-qualified release and its Docker, desktop, download
and extension outcomes. Issue #1483 remains open; the separate compound/report,
lifecycle and client capability drafts have not been delivered by this CI fix.

## Separate released Start bridge probe

The released Start JavaScript bridge completed one synthetic file Read. A
separate exact replacement request returned a structured incomplete Read/Edit
capability gap with exit zero and no file changes; no editing success is
claimed. [Start issue #261](https://github.com/link-foundation/start/issues/261)
records the bounded typed-edit requirement using only synthetic/public data.
Private raw sessions and personal paths were not uploaded. This does not
qualify the pending Claude/Codex or general self-coding acceptance cases.

## Actual ab3 producer qualification and remaining JavaScript acquisition

At source `ab3c72a6fbbd4d92587581849dab863888fd5f5f`, the
[native producer](https://github.com/link-assistant/formal-ai/actions/runs/41728072415/job/126239294738)
actually verified all 74,026 tracked files and 1,947,978,151 physical bytes
with the unchanged canonical witness in about eighteen seconds. All eleven
creation fixtures, three latency/HTTP smoke cases, captures and every artifact
upload succeeded. Standard Agent and held-out client tests also passed,
including original hello-world, follow-up creation and actual program execution.
This establishes those scopes, not complete release publication.

[JavaScript group 7](https://github.com/link-assistant/formal-ai/actions/runs/41728072124/job/126238577448)
failed before setup or tests: the filtered commit fetch completed in about two
seconds, sparse mode was activated, and worktree checkout exceeded its existing
three-minute phase deadline. No JavaScript assertion failed in that attempt.
Reserve five minutes for materialization and twelve for the enclosing job: the
existing six-minute suite allowance plus one minute of setup remains intact.
All source patterns, the issue-702 fixture, event ref, depth/filter, low-speed
limits, matrix, commands and PR/main selection remain byte-identical. This is
finite headroom for observed acquisition variance, not a measured speedup.
Do not mix a partial Layered rerun with source-producing attempt custody. A
fresh source must qualify all mandatory checks and actual release products.

## Complete ab3 failure inventory and follow-up

The main run completed with 370 terminal checks: 112 successful, 254 skipped,
three failed and one cancelled. All actual failures are batched in this follow-up:

- Layered JavaScript acquisition exceeded three minutes before tests. Its job
  now reserves five minutes for checkout and twelve overall, retaining the
  six-minute suite and all inputs.
- The primary release workflow server-parity job spent almost fifteen minutes
  acquiring its full worktree and exhausted its enclosing deadline before any
  parity case ran. It now has an eighteen-minute checkout within a thirty-minute
  job, with finite low-speed HTTP limits. Full source, exact ref/depth, binary
  download and every parity case remain. This is the primary workflow job,
  distinct from Layered server parity. Staged release projections preserve all
  52 raw operations and 104 bindings; the deployed staged workflow is unchanged.
- Native shard two found a stale global parser-command occurrence assertion
  after adding mandatory unit coverage alongside the retained live-link job.
  The replacement checks each trigger-owned path, mandatory independent job and
  both parser steps structurally, keeping the live parser before link checking.
  The original native test name is retained with its body in a readable child
  module. Native execution is reserved for CI.

Both focused JavaScript files must pass locally before submission. All other
production gates and products still require a fresh main run after merge. No
release or Docker publication is credited from ab3. Source changes in this
follow-up were applied through the JavaScript Formal AI tool host using guarded
externally supplied helpers; independent synthesis credit is zero.

## Native compilation follow-up after PR 1525

PR [1525](https://github.com/link-assistant/formal-ai/pull/1525) merged as
`83686e2c3ffa003782fcfcf4b6ef832014812c3d` after all 99 checks passed
(33 success, 66 skipped), including actual JavaScript, TypeScript, Secrets and
parser checks. The JavaScript suite passed 1,897 tests with 17 skips.

The fresh main [Question necessity ratchet job](https://github.com/link-assistant/formal-ai/actions/runs/41736751822/job/126267806420)
then found a compile-time unused parent-module `unwrapped` import under
`-Dwarnings`. The extracted child owns its own used import; remove only the
obsolete parent import. This is a source-only correction, with no warning
exemption, assertion change or local Rust compilation. Other native targets
may report the same shared compilation cause. The source still requires a
fresh native CI and release qualification after the follow-up merge.

## Qualified native main and release transport follow-up

PR [1526](https://github.com/link-assistant/formal-ai/pull/1526) merged as
`94f3278cd9d90c65d0970400d4a11029683936b7` after all 89 PR checks passed.
On that exact main source the complete producer, all native full/specification
shards, four lint lanes, both client jobs, primary server parity and package
build actually succeeded. Primary parity compared 153 requests with no content
or protocol divergence. The previously failing native parser policy test
passed in full shard two. These are observed outcomes, separate from release
publication.

[Auto Release preparation](https://github.com/link-assistant/formal-ai/actions/runs/41738975507/job/126288149767)
failed before version computation or commit: its last authenticated GET, the
current concurrency group, returned no bytes after the five-second timeout.
The main run ended with 100 successful, 248 skipped and two failed checks; the
second failure was the pipeline summary. No new source child, tag, release or
Docker product was qualified. The entire failing step lasted about nineteen
seconds, so the existing thirty-second provider budget retains retry headroom.

The correction reserves the same two logical observation slots once and allows
at most two physical jobs/group pairs within the original shared deadline. A
transport failure discards the whole partial pair; the next attempt reacquires
jobs before group. Actual current-job identity is validated before any group
read or retry. Refusals, malformed or inactive jobs, membership errors, stale
observations and expired budgets remain failures. The existing five-second
per-read and pair-freshness bounds, executable checks, source inventories and
all staged operations/bindings remain. GitHub documents this endpoint and its
[Actions read permission](https://docs.github.com/en/rest/actions/concurrency-groups).

Nine offline JavaScript regressions retain the captured transport shape and
independent active-job/member checks; they join the default fast PR inventory.
The two maintained script postimages were drafted by seven actual English
`In file, replace «old» with «new».` requests through the current JavaScript
Formal AI, each with Read/Edit/Read and exact physical postimage checks.
Replacement content was externally supplied, so independent synthesis credit
is zero. Earlier unsupported content-header forms that produced malformed
private copies are retained separately as capability regressions. No Rust
compilation, native execution or Docker build ran locally. A fresh main and
actual publication/product chain still must qualify after merge.
