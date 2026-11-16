## Issue #1158: Pull Request Feedback, Description and Readiness

Source: https://github.com/link-assistant/formal-ai/issues/1158 (a sub-issue of
[#1162](https://github.com/link-assistant/formal-ai/issues/1162)). Delivered
pull requests kept the orchestrator's placeholder title and description, stayed
in draft, and their feedback was never read. The case study is
`docs/case-studies/issue-1158/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1158-1 | Read pull request feedback before planning. | Partial: `pr_comments_command` reads comments and reviews through a bounded `gh pr view --json comments,reviews` filter before the description is edited or the pull request is marked ready. Feedback that asks for changes stops the run with `pr_feedback_review_report` (`rust/src/agentic_coding/command_reroute.rs`). Folding the newest reviewer instruction into the goal before the recipe is planned is not implemented. |
| R1158-2 | Update the description with the files, the verification and the closing reference. | Implemented: `pr_completion_commands` fills `pr_body_template` with the verified artifact, the verification commands and `Resolves <issue>`. When the target is an issue it also sets the pull request title to the issue title (`pull-request-edit-titled-command`), and it edits only when the text differs. Pinned by `rust/tests/web/issue-1162-delivery-commands.test.mjs` and `rust/tests/unit/agentic-coding/issue_1162_delivery_commands.rs`. |
| R1158-3 | Run `gh pr ready` when the work is verified and the tree is clean. | Implemented: `pr_ready_command` checks `git status --porcelain` and `isDraft` before `gh pr ready`. The CI fallback (`plan_recipe_ci_fallback`) confirms the pull request is no longer a draft on the verified head. |
| R1158-4 | The answer reports each step. | Implemented: `ci-verified-report` and `ci-verified-local-report` (`data/meta/recipe-resume.lino`) state that the feedback was read, the description was rewritten with the artifact, the commands and the closing reference, and that the pull request is ready. |
| R1158-5 | Manual Hive Mind end-to-end check on a released version. | Blocked: needs a released version and live Hive Mind credentials. |
