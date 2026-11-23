# Current CI observation repairs, 2026-11-23

The current source is `cec9207c3e139e2212f6d4ea478ff1b88e17be7c`.
[Layered CI 45093374323](https://github.com/link-assistant/formal-ai/actions/runs/45093374323)
failed on the specification fixture binding and retained link-checker payload.
The native jobs in [Pipeline 45093374576](https://github.com/link-assistant/formal-ai/actions/runs/45093374576)
also expose a rejected-article URL in a successful guide and a stale WordNet
outcome assertion. The pipeline is still running; this is not a terminal cohort.

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
