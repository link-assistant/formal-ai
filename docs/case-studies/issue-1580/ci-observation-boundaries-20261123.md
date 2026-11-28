# Current CI observation repairs, 2026-11-23

The current source is `cec9207c3e139e2212f6d4ea478ff1b88e17be7c`.
[Layered CI 45093374323](https://github.com/link-assistant/formal-ai/actions/runs/45093374323)
failed on the specification fixture binding and retained link-checker payload.
The native jobs in [Pipeline 45093374576](https://github.com/link-assistant/formal-ai/actions/runs/45093374576)
also expose a rejected-article URL in a successful guide and a stale WordNet
outcome assertion. The complete cohort became terminal at 02:18:26Z: 34
workflows, 31 successful, two failed and one explicit skip. The two failures are
the linked Layered and Pipeline runs. The native producer and nine of ten native
contract groups succeed; the native compile-input admission job fails its actual
JavaScript guide assertion, rather than Rust compilation. Complete Pipeline log
lines 66208–66209 and 77318 identify the guide and WordNet failures. The apparent
cycle panic at line 65496 belongs to a passing negative-control test.

The guide accepted the matching article's steps and digest but retained the
previous unrelated article's `no_relevant_result` detail. Both extractors now
return the accepted capture URL when they contribute steps. The original native
assertion remains. The new worker regression fails before this repair and passes
after it, including an assertion that the rejected URL is absent from the reply.
The full procedural suite passes all 18 cases.

The WordNet seed declares an English endpoint, so a missing offline English
capture is `offline_cache_miss`, rather than an unbound template. The native test
still requires every source and status, and additionally requires the actual
English URL. A worker control checks English and all four unsupported locales;
the full six-case neighboring suite passes. Native execution of these repairs
remains pending on the next hosted head.

The governed specification is regenerated through the maintained complete reader.
Its 1,193 declarations, 223 supported cases, 970 unsupported cases, assertions,
source manifest and complete nested objects are unchanged after normalizing just
four occurrences of the reviewed `rust/tests/unit/mod.rs` fixture hash. Both
fixture-integrity cases and all 34 Unicode/complete-object cases pass.
All 46 retained heavy-job payloads are compared: only the reviewed link-checker
payload from commit `6b957403972952ab07e58c8b88d7ba25bdf9b801` changes its witness.
Its former source bytes match the previous witness. All 29 production-profile
cases pass; the checker, production dispatch conditions and other bodies remain.

The successful [read-only Security producer 45093409488](https://github.com/link-assistant/formal-ai/actions/runs/45093409488)
contains registry connection-reset errors in its independent Cargo audits
(full log lines 529–624), despite a zero exit. The audit wrapper now retains
the full original stream, fails incomplete observations even at exit zero,
and retries only positively identified transient yank-check errors up to three
times. Only a later complete audit can recover. Ordinary findings retain their
status. Existing primary and independent advisory policies are preserved.
Four new observation assertions fail against the preceding implementation;
all 16 inventory cases and 11 neighboring audit-proof cases pass after repair.
The workflow-diagnostics and ten how-to synthesis cases also pass.

The combined initial local command exceeded Formal AI's 60-second shell budget.
It is retained as an incomplete observation. The affected full suites were then
run sequentially in a tracked detached worker, with complete terminal results.
No Rust was compiled locally. Formatting checks pass for both changed Rust files.

Evidence is the round-trip-verified 34-member archive and per-member hashes under
`experiments/issue-1580/evidence/checks/ci-observation-boundaries-20261123/`.
It retains the complete original native job logs, Security log, before/after
tests, fixture objects, source comparison, supplied draft scripts and command
outcomes. Supplied commands and conclusions receive zero independent authorship
credit. These focused checks do not replace full repository gates, terminal
current-head CI, both actual native-artifact reruns or the unfinished acceptance
criteria of issue 1580. The pull request remains draft and unmerged.


The first complete pre-push replay on clean `648569ae4` has 87 passes, two
failures and 54 hosted exclusions. Both failures identify the same one-line
growth above the existing 735-line how-to worker ceiling. Selecting the accepted
URL directly in the returned detail preserves behavior within that ceiling;
the ceiling and its assertions remain unchanged. The focused recovered-guide
regression passes after this refinement. The complete corrected inventory is
still pending, and its outcome must be retained before pushing.

The terminal cohort's complete CLI logs, job metadata, successive observations,
failed gate streams and byte/decoded hashes are retained under
`experiments/issue-1580/evidence/ci/cec9207c3-terminal/`. Every archive member is
round-trip verified. The terminal failures are all covered by this batch; this
does not establish the next head's outcome.

The corrected clean `423be15fc` replay passes the 18 procedure cases and again
has 87 gate passes, two failures and 54 hosted exclusions. This time both failures
identify the terminal evidence manifest's 3,462 lines, rather than the worker
budget. Compacting individual inventory records preserves the complete parsed
manifest object, all hashes and the original archive, while satisfying the same
file-size limit.

A separate actual regression control catches a stale-capture age warning being
overwritten by the accepted URL. The shared native and worker walkers now keep
that warning when a stale capture contributes items; failed extraction can still
report its own failure. The new worker regression fails before the change and
passes afterward, together with the recovered-article control. Formatting and
source/projection currency checks pass. The full 19-case suite and complete gate
inventory still require their next clean replay. The complete preceding gate
streams, failed reproduction, passing controls and supplied scripts are retained
with round-trip hashes in
`experiments/issue-1580/evidence/checks/stale-capture-warning-20261123/`.

The final clean replay on `4b909224c85a100fb5ef4e9e124decbc29b9195d` completes at
`2026-11-23T02:58:22.402388+00:00`: all 19 procedure cases pass, and the complete
143-entry local inventory has 89 passes, zero failures and 54 explicit hosted
exclusions. The tree is clean before and after the sequential run. Every gate
stream, full suite output, exact command and source identity is retained with
round-trip hashes in
`experiments/issue-1580/evidence/checks/stale-preserving-clean-replay/`.
The final evidence commit changes documentation and retained observations only;
native execution and the next hosted cohort remain separate qualifications.
