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
