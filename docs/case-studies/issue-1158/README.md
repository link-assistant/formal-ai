# Issue #1158: Pull request completion and feedback

After a verified recipe commit, PR targets retrieve comments and reviews, update their description from artifact and verification commands, and mark draft PRs ready only with a clean working tree. Already-ready PRs are accepted. Nonempty comments, requested changes, or unreadable feedback block readiness with the observed feedback.

## Requirement status

Partial. Feedback retrieval, description editing and guarded readiness are drafted. Deriving and applying arbitrary review-requested source edits remains open; feedback is never represented as resolved merely because it was fetched.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.
