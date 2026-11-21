# Macro observation and runtime budget checkpoint

The dependency-maintenance workflow previously read `git ls-files` through
process substitution. A Git failure after partial output exited successfully
and could resolve an incomplete tree. Capture the complete inventory before
invoking any resolver. Its actual shell regression fails against the original
workflow and passes after repair, with all four maintenance cases passing.

The link recheck bounded each request but slept without accounting for its
remaining shared budget, then allowed a request its entire individual timeout.
Stop before a retry delay consumes the remaining budget and cap every request
at the remaining allowance. Both new controls fail against the original source
and pass after repair. Fake timers avoid wall-clock stress. All 16 existing/new
recheck cases and three real local HTTP cases pass, preserving permanent 404
failures and requiring an actual accepted response after a transient failure.

All 19 security adapter groups pass. The optional macro observer is restricted
to an explicit read-only Rust dispatch, after the existing producer analysis.
It retains original database/pack metadata, version, query, command streams,
BQRS and decoded contexts without replacing the security suite. Local mock
success only verifies adapter transport; actual QL compilation, hosted rows and
semantic interpretation remain unverified.

The original 3bb3083b Security run
[44765053281](https://github.com/link-assistant/formal-ai/actions/runs/44765053281)
preserves zero findings and zero warning/error notifications in Rust and Actions
SARIF. All 631 informational unresolved macro records remain visible. Source
include-path normalization maps 342 records to directly included files,
including all 282 production records. This source relationship is a hypothesis
to verify against actual expanded AST rows, not semantic coverage proof.

Five touched JavaScript gates and whitespace checks pass. Original-failure and
passing logs are under
`experiments/issue-1580/evidence/checks/template-runtime-and-macro-observation-verified/`.
The first evidence harness run preserved both intended failures but expected a
different default Node reporter; only the harness reporter was corrected.
Primary CodeQL sources, historical reports, full research responses, failed
drafts, corrections and original runtime source bytes are retained under
`experiments/issue-1580/evidence/checks/codeql-macro-source-investigation/`.

Pinned Rust template review now covers 309/373 files and 37,626 literal lines;
64 files and unverified counterpart practices remain open. The full issue,
other template comparisons, source warnings, dependency migrations, strict
original-native acceptance, production publication/delivery and full PR review
remain unfinished. No local Rust/QL compilation runs. Supplied work receives
zero independent Formal AI authorship/reasoning credit. The PR stays draft.
