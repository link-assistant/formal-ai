## What happened

Hive Mind's prompt (both its Formal AI variant and the regular one) asks the model to *update the pull request*, *review and address all feedback recorded on that pull request*, and — in the regular prompt — to run `gh pr ready <n>` when done and keep the PR description current. Across all 2026-09-27 sessions Formal AI:

- never read a single PR comment (the Kotlin PR had 60+ comments including the `Auto-restart … Uncommitted files: ?? Main.jar` instruction);
- never edited the PR description (Kotlin body still says "1 file(s) modified" from July);
- never ran `gh pr ready`.

Claude Code / Codex with an LLM model do all three from the same prompt, which is why their Hello World PRs come out clean. `data/seed/contribution-artifacts.lino:67-68` knows `gh pr edit` / `gh pr ready` as actions, but no work-item step uses them (`data/meta/work-item-steps.lino` has only the read, commit and workflow templates).

## Requirements (what to do)

Extend the repository work item with, as data in `work-item-steps.lino` and code in `work_item_steps.rs` / `general_execution.rs`:

1. **Read feedback**: `gh pr view <pr> --comments --json comments,reviews` (and the review threads) before planning; treat the newest non-bot instruction as part of the goal (e.g. "commit or discard `Main.jar`").
2. **Update the PR description** after the commit: summary of files changed, how it was verified, `Fixes <issue url>` preserved.
3. **Mark ready**: `gh pr ready <pr>` when verification passed and the tree is clean.
4. Report each of these in the final answer so Hive Mind's summary shows them.

Tests: replay fixtures with a PR that has an instruction comment; assert the plan includes the comment read and the resulting change; assert `gh pr edit --body` and `gh pr ready` are planned after a successful commit (mock `gh`).

## How to test

Hive Mind Kotlin run: the PR body is updated by Formal AI, and the run ends ready-for-review without Hive Mind having to convert it.


## Shared evidence (2026-09-27 runs)

| Repo / tool | PR | Session start | Outcome |
|---|---|---|---|
| Kotlin, `--tool claude` | https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/pull/2 | 15:02:53Z | `Main.kt` committed, then 2 identical failed restarts, stop `no_progress_between_sessions`, PR left **draft** |
| Scala, `--tool agent` | https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2 | 15:09:06Z | `gh` unauthenticated inside the agent session → Formal AI `planned_not_executed`, PR left **draft** |
| Rust, `--tool codex` | https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2 | 15:15:15Z | 78 identical `gh issue view` calls (6.2M input tokens), no change, stop `draft_pull_request` after 3 fake "restores", PR left **draft** |

Runtime for all three: `solve v2.32.0`, task image `konard/hive-mind-dind:2.32.0`, Formal AI serving backend `0.352.1` (local wrapper `0.351.0`), `@link-assistant/agent` 0.26.5. Command (identical except `--tool`):

```
solve <issue-url> --model formal-ai --tool <claude|agent|codex> --attach-logs --verbose --no-tool-check --disable-report-issue --language en
```
(`--auto-restart-until-mergeable` defaults to `true` in `src/solve.config.lib.mjs:283-286`.)

Full logs (gists uploaded by solve):
- Kotlin main session: https://gist.github.com/konard/b0b660367fcd7e5a4c21a833cff4c5d4 · restart 1: https://gist.github.com/konard/f2683d22d41085bbe01bb45ab768b6de · restart 2: https://gist.github.com/konard/71bbc9af9e04dce7f98e11929bc30fa0 · final: https://gist.github.com/konard/bdca731664778563b104343fe3696335
- Scala (agent): https://gist.github.com/konard/8a196a1d179ecb105304232a46e09ede
- Rust (codex): https://gist.github.com/konard/513141ac5f144b15e9593f05f86362e8

Task issues are identical Hello World specs (print exactly `Hello, World!`, add a GitHub Actions workflow), e.g. https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/issues/1.

## Related (other repositories)
- hive-mind PR description regeneration: https://github.com/link-assistant/hive-mind/issues/2318

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1158-pr-feedback-body-ready.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1158_pull_request_steps.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1158/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1158-pr-feedback-body-ready.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1158_pull_request_steps` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **all three formal-ai rows** passes on the released version, linked here.

## Depends on / blocks

- Depends on: #1160 (E125) for reading the feedback lines Hive Mind now sends in the regular prompt.
- Blocks: the matrix assertions "PR ready for review" and "body lists every file" for all formal-ai rows.



