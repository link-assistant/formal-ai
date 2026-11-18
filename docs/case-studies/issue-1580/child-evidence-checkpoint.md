# Selected-child evidence checkpoint

This is an unfinished checkpoint for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The original issue and all wider unchecked acceptance criteria remain open.

## Reproduction and cause

Solve an unknown root containing two independent tasks whose answers contain
distinct evidence and one shared evidence entry. The JavaScript answer adapter
drops the child evidence when constructing a draft. Composition has no evidence
field, and adopting its text spreads the original root answer, retaining the
rejected root's evidence. The Rust draft and production adapter have the same
missing transfer. An alternative request must retain only its selected child.

Three new JavaScript regressions fail against published source bfe5ac0f; the
before run has ten passes and three failures. Preserve the complete assertions
under `experiments/issue-1580/evidence/checks/child-evidence-provenance/`.
The tests cover selected children, stable deduplication, unused alternatives,
cache reuse, direct answers and preservation of the host's original evidence.

## Draft change

Copy evidence into JavaScript drafts and native drafts. Composition concatenates
only the selected children's evidence, deduplicating while preserving first-use
order. Adoption replaces the rejected root's evidence with the composition's
evidence. Direct answers keep their evidence, and unresolved compositions do not
adopt evidence from rejected children. Regenerate worker and TypeScript mirrors
through the maintained writers. Add one native executor test for conjunctions
and alternatives; native execution remains pending CI.

The final focused run passes all 13 executor tests, including explicit evidence
assertions on both the cached candidate and its source. The 15 adjacent factual-QA
tests, both existing shared-fixture checks, specification fixture integrity and
Rust 2024 formatting pass. The original 29 fixture cases and event traces remain
unchanged. Regenerate the stale JavaScript-to-Rust projection with the maintained,
pinned translator; its verification passes. Preserve the original stale-projection
failure and its successful correction separately.

The required full shared-data inventory records 85 passes, 52 explicit native or
CI-context exclusions and one missing local ShellCheck failure. Restore the
official ShellCheck release, verify its archive against GitHub's SHA-256 digest,
then pass the remaining gate over all 70 maintained shell scripts. Thus all 86
executable entries pass. Preserve the original aggregate, every individual gate
log and the focused correction with hashes; literal review covers the aggregate
and failed payload, not all successful individual payloads. Expand abbreviated
archive filenames without changing their bytes and retain alias receipts. No
local native build runs; the fifth native executor test still awaits CI.

## Published compile failure and correction

Head `ffebedb1` completes 23 workflows: 22 pass and the CI/CD Pipeline fails.
The native library check reports E0063 at `solver_synthesis.rs:219` and `:261`:
two validation-only `Draft` constructors omit the new `evidence` field. Preserve
the entire 16,099-line failed workflow and complete native job log under
`experiments/issue-1580/evidence/ci/child-evidence-ffebedb1/`. Literal review of
the failed workflow covers lines 15,761–15,775; the native job error window is
also read. The fifth native executor test does not execute on this failed head.

Initialize both temporary validation drafts with empty evidence. Their verdict
checks consume intent and content; actual synthesized answer evidence continues
to be assembled by `finalize_composed_candidate` from the event log. Audit all
Rust `Draft` constructors and imports, including the separate coding type.
The correction passes 13 executor tests, six procedure-census tests, five relevant
JavaScript gates, Rust 2024 formatting and whitespace checks. Native compilation
and the fifth executor regression remain pending the next hosted run.

The passing JavaScript jobs on `ffebedb1` report 1,440 and 1,256 passes, four and
13 skips, zero failures and zero cancellations. Complete logs and hash checks
are retained; literal review covers the final totals and named error windows,
not every successful log line. These results do not establish the wider issue.

## Limits

This change carries evidence already supplied by selected children. It does not
validate a citation's truth, establish semantic parent coverage, bind passages to
questions, or solve multi-passage aggregation. Those acceptance criteria remain
unfinished. Supplied diagnostics, commands, patches and conclusions receive zero
independent authorship credit; independent acceptance remains 0/20.
