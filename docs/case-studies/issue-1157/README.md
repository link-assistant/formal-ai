# Issue #1157: Idempotent recipe commits

Recipe commits stage only declared artifacts and inspect the cached diff restricted to those paths. No-change runs emit __formal_ai_commit_unchanged, then still push and print the current hash. Continuations consume the planner evidence window, retaining already completed writes and commands.

## Requirement status

Partial. Idempotent commit and continuation replay are drafted. Output ignore patterns and verification of artifacts after externally modified continuation state remain open. No local builds or tests were run.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.

## Issue #1162 follow-up (2026-11-16)

**Root cause.** The restart in the 2026-09-27 Kotlin run redid the same work, and its commit chain exited 1 on "nothing to commit". `Main.jar` was left untracked, and in the 2026-11-10 runs a Rust binary and session logs were committed.

**Fix** (commits `cc1f10b7c`, `718c340e9`). Staging names its files with literal pathspecs. An unchanged tree is reported as success ("Nothing to commit: the verified files already match the last commit …"), not as a marker or a failure. Build outputs are ignored through `.gitignore`. `gh pr edit` and `gh pr ready` run only when the pull request differs.

**Tests.** `rust/tests/unit/issue_1157_idempotent_commit.rs` runs a delivery twice against a real temporary repository and requires the second run to be a clean no-op. `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs` does the same through Hive Mind's verbatim prompt, including the Kotlin `Main.jar` restart.

The manual end-to-end check on a released version is still **blocked**: it needs a release that contains this change and Hive Mind credentials, and neither is available to the solver. The issue therefore stays open after pull request https://github.com/link-assistant/formal-ai/pull/1579.
