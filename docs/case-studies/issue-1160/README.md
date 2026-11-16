# Issue #1160: Hive Mind prepared work continuation

The planner recognizes the marked UNCOMMITTED CHANGES DETECTED fenced porcelain listing before ordinary issue planning. Valid relative paths are committed explicitly after diff validation, pushed to the prepared branch, described on the prepared PR, and made ready when feedback and clean-tree checks permit. Failed steps stop with evidence.

## Requirement status

Partial. Existing-tree continuation is drafted. Quoted porcelain paths and rename records are rejected rather than guessed. Applying arbitrary reviewer source changes and deriving substantive verification commands from restart context remain open. No builds/tests or pushes were performed.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.

## Issue #1162 follow-up (2026-11-16)

**Root cause.** Hive Mind's prompts (the formal-ai prompt, the regular prompt and the restart prompt that lists uncommitted changes) are now kept verbatim in `rust/tests/fixtures/hive-mind/`. The formal-ai wording "Update the pull request at" did not bind the pull request (see #1158). On restart, a stray `Main.jar` listed as uncommitted was treated as work to commit.

**Fix** (commits `cc1f10b7c`, `056b2cac3`). Both prompt styles bind the issue, the branch and the pull request. The restart listing is matched by its exact porcelain. Build outputs such as `Main.jar` are ignored through `.gitignore`, not committed. Formal AI's own bookkeeping notices are not treated as requested work.

**Tests.** `rust/tests/unit/issue_1160_regular_prompt.rs` (3 tests: the recorded prompts name one issue, branch and pull request; the regular prompt delivers a ready pull request; the restart with a stray Kotlin jar commits, pushes and keeps the pull request ready). `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs` runs the same prompts through the JS planner.

The manual end-to-end check on a released version is still **blocked**: it needs a release that contains this change and Hive Mind credentials, and neither is available to the solver. The issue therefore stays open after pull request https://github.com/link-assistant/formal-ai/pull/1579.
