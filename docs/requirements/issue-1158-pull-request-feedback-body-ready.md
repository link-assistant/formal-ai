## Issue #1158: Pull request completion and feedback

Source: https://github.com/link-assistant/formal-ai/issues/1158

| Requirement | Drafted behavior | Status |
|---|---|---|
| Preserve the requested behavior in the agentic harness | After a verified recipe commit, PR targets retrieve comments and reviews, update their description from artifact and verification commands, and mark draft PRs ready only with a clean working tree. Already-ready PRs are accepted. Nonempty comments, requested changes, or unreadable feedback block readiness with the observed feedback. | Partial. Feedback retrieval, description editing and guarded readiness are drafted. Deriving and applying arbitrary review-requested source edits remains open; feedback is never represented as resolved merely because it was fetched. |
