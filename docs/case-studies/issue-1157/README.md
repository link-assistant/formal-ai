# Issue #1157: Idempotent recipe commits

Recipe commits stage only declared artifacts and inspect the cached diff restricted to those paths. No-change runs emit __formal_ai_commit_unchanged, then still push and print the current hash. Continuations consume the planner evidence window, retaining already completed writes and commands.

## Requirement status

Partial. Idempotent commit and continuation replay are drafted. Output ignore patterns and verification of artifacts after externally modified continuation state remain open. No local builds or tests were run.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.
