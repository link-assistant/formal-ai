# Issue #1158: Pull request completion and feedback

After a verified recipe commit, PR targets retrieve comments and reviews, update their description from artifact and verification commands, and mark draft PRs ready only with a clean working tree. Already-ready PRs are accepted. Nonempty comments, requested changes, or unreadable feedback block readiness with the observed feedback.

## Requirement status

Partial. Feedback retrieval, description editing and guarded readiness are drafted. Deriving and applying arbitrary review-requested source edits remains open; feedback is never represented as resolved merely because it was fetched.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.

## Issue #1162 follow-up (2026-11-16)

**Root cause.** Hive Mind's formal-ai prompt says "Update the pull request at <url>." That wording did not bind the delivery to the pull request, so feedback was never read and the title, description and ready state were never written. The 2026-11-10 pull requests kept placeholder titles and descriptions, and one stayed a draft.

**Fix** (commits `cc1f10b7c`, `056b2cac3`). The wording binds the pull request. After the verified commit the delivery reads PR feedback with a bounded `jq` query, writes the title from the issue, writes a description that names the verified artifact and the verification commands and ends with `Resolves <issue>`, and then runs `gh pr ready` once the tree is clean.

**Tests.** `rust/tests/unit/issue_1158_pull_request_steps.rs`; `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs` (`pr view`, `pr checks`, `pr edit` before `pr ready`); `rust/tests/web/hive-own-session-feedback.test.mjs`.

**Still partial.** Feedback is read and blocks readiness, but arbitrary review-requested source edits are not derived from it.

The manual end-to-end check on a released version is still **blocked**: it needs a release that contains this change and Hive Mind credentials, and neither is available to the solver. The issue therefore stays open after pull request https://github.com/link-assistant/formal-ai/pull/1579.
