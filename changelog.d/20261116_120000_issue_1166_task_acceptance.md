---
bump: minor
---

### Added
- Independent task acceptance (#1166), in Rust (`rust/src/agentic_coding/task_acceptance.rs`, `task_condition.rs`), JavaScript and generated TypeScript. It reads only the task text and the candidate's files and observed output:
  - every clause gets a verdict (discharged, contradicted, unresolved, inapplicable) with a justification and its byte span;
  - judgment clauses stay unresolved, and nothing is reported as a generic "style" pass;
  - the workflow path (including #1450's `ci/` convention), triggers, file name, runtime and observed status are checked against the issue;
  - conditional outputs in the supported subset become a truth table, so reversed comparisons, `and`/`or` and negation stay distinct, and unsupported grammar is a gap;
  - each task gets a canonical meaning and an exact identity.
- A finished recipe's answer lists the actionable verdicts and says the task is not complete while a required clause is contradicted or unresolved.
- One shared fixture (`rust/tests/fixtures/issue-1166/acceptance.json`) for the native and JS tests. It includes executed Python candidates; one shares its generated test's misunderstanding and is still rejected.

### Fixed
- An expected output mentioned twice is printed once; an explicit `twice` prints it twice. Conditional outputs are no longer bound as unconditional literals.
