## Issue #1160: The Regular Hive Mind Prompt

Source: https://github.com/link-assistant/formal-ai/issues/1160 (a sub-issue of
[#1162](https://github.com/link-assistant/formal-ai/issues/1162)). The long
Hive Mind prompt (prepared branch, pull request and uncommitted changes) was
not handled the way the short prompt was. The case study is
`docs/case-studies/issue-1160/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1160-1 | Fixtures of the regular Hive Mind prompt. | Implemented: `rust/tests/fixtures/hive-mind/` holds Hive Mind's formal-ai, regular and restart prompts verbatim, a fake `gh` and the offline delivery harness. They drive `rust/tests/unit/issue_1160_regular_prompt.rs` and `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs`. The older fixtures `rust/tests/fixtures/issue-1364-claude-restart.txt`, `rust/tests/fixtures/hive-workflow-original-task.json` and `rust/tests/fixtures/issue-1364-hello-world-issues.json` remain. The redacted logs are in `docs/case-studies/issue-1162/raw-data/hive-mind-logs/`. |
| R1160-2 | The router extracts the branch, the pull request and the feedback. | Partial: the branch comes from `branch-binding-patterns` ("Your prepared branch: …"), and `plan_restart` (`rust/src/agentic_coding/restart_feedback.rs`) reads the pull request and the changed paths. The feedback text is not extracted from the prompt; it is read later through `pr_comments_command`. |
| R1160-3 | With an uncommitted `Main.jar`, the plan is ignore or delete, commit, push, ready. | Implemented: `prepared_work_hygiene` ignores or deletes the output, commits and pushes (`rust/tests/web/issue-1364-restart-hygiene.test.mjs`). With the pull request bound from the prompt, the recipe completion then marks it ready. The restart case of `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs` leaves an uncommitted `Main.jar` after a real Kotlin delivery: `.gitignore` gains `Main.jar`, which is committed and pushed, `Main.jar` is never tracked, and the pull request stays ready. `rust/tests/unit/issue_1160_regular_prompt.rs` pins the same path from Rust. |
| R1160-4 | The short prompt still works. | Implemented: the short `Implement <issue>` prompt drives the recipe in `rust/tests/web/issue-1162-delivery-commands.test.mjs`. |
| R1160-5 | Manual Hive Mind end-to-end check on a released version. | Blocked: needs a released version and live Hive Mind credentials. |
