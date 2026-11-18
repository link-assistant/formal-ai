# Algebra calculation-cue checkpoint

This unfinished checkpoint belongs to [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The complete issue remains open.

## Reproduction and root causes

The retained main `015a6831` specification log reports algebra substitution
failures for the original result 11, the renumbered result 17 and the shared
fixture result 17. Source-operand evaluation must not require separately solved
assignment fragments to be answered. The existing parent-validation correction
already distinguishes source operands from consumed child answers, but its real
native algebra path still needs hosted execution.

The retained native baseline also fails `If x = 3 and y = 4, what is x*x + 2*y?`.
Replaying that request through the current JavaScript WorkerHost reproduces a
second defect: specialized algebra extractors recognize evaluation commands and
"value of" while omitting existing calculation-question cues. The focused new
file records six passes and two failures before the change. Both the original
question and its renumbered x=4, y=6 variant return `calculation_error`.

## Change and local evidence

Reuse the existing arithmetic-expression parser as the algebra extractor's
fallback in JavaScript and the explicit calculation-candidate parser in Rust.
Preserve all existing marker paths. Variable binding, substitution and actual
evaluation remain necessary; unsupported prose and unbound variables cannot
claim algebra substitution. Regenerate the TypeScript mirror with the maintained
writer. The worker retains its existing 786-line ceiling.

All eight new WorkerHost cases pass: four positive evaluations and four negative
controls for missing assignments, unbound variables, unrelated prose and bare
assignments. All 12 adjacent calculation-shape cases and 25 executor cases pass.
Rust 2024 formatting, whitespace and six affected generated/translation/size/
budget/abbreviation gates pass. Preserve complete before/after payloads, hashes
and individual gate logs under
`experiments/issue-1580/evidence/checks/algebra-calculation-cues/`.

Register the new JavaScript file in the existing fast PR inventory and record
the main-only algebra regression in its ledger. Append a real UniversalSolver
test to the existing isolated native executor target. It checks all five
original and renumbered requests, exact numeric answers, substitution/evaluation
evidence, child observations and emitted Links Notation. No new native target
or local Rust build is introduced.

## Remaining verification

The complete 138-entry shared-data inventory passes all 86 executable gates,
with 52 explicit native/CI-context exclusions and zero failures, in 1,018.572
seconds. All 86 complete gate payloads are archived and digest-verified. All four
ledger tests and 17 fast-profile tests pass. Strengthen the final JavaScript test
to check the user-visible result as well as substitution/evaluation evidence;
all eight cases pass again. Memory events record zero OOMs and zero OOM kills.
Hosted native execution of the five algebra cases remains pending.
JavaScript success does not establish the Rust solver's result. This change
does not complete the other synthesis, parent-coverage, provenance, benchmark,
template, dependency or delivery requirements. Supplied patches, scripts,
commands and conclusions receive zero independent authorship credit;
independent acceptance remains 0/20 and the PR stays draft and unmerged.

The first publication attempt stopped before commit or push: staging the new `gate-check-tests-as-docs.log.gz` evidence archive raised the abbreviated filename count from 90 to 91. The archive was renamed to `gate-check-tests-as-documentation.log.gz`, its receipt references were updated, and the failed attempt was retained. No abbreviation ceiling or source code was changed by this correction.
