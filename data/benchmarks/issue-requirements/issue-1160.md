## Why

Hive Mind currently sends `--model formal-ai` a private six-line prompt with an empty system prompt (`src/formal-ai-prompt.lib.mjs`, hive-mind #2158) because the regular prompt "exposes incidental shell-language cues to Formal AI's deterministic intent router". The operator's requirement is that Formal AI is not special in Hive Mind ("in auto-restart/auto-resume it should be just like other models in all the same tools"), and the private prompt is exactly what made the 2026-09-27 restarts byte-identical (Formal AI never learned about the untracked `Main.jar`). Hive Mind will remove the special prompt (link-assistant/hive-mind#2313); Formal AI must accept the regular prompt.

## Requirements (what to do)

- Add Hive Mind's regular user prompt and system prompt (`src/claude.prompts.lib.mjs`, `src/agent.prompts.lib.mjs`, `src/codex.prompts.lib.mjs` rendered for a continue-mode PR with feedback lines, including the `Uncommitted files:` block and the `gh pr ready` instruction) as fixtures under `rust/tests/fixtures/hive-mind/`.
- The intent router must classify that prompt as a repository work item for the issue URL it names, extract the branch, the PR, and the feedback lines (uncommitted files, new comments, failed checks), and not be derailed by the shell snippets or examples inside the prompt.
- The restart-with-feedback case must change the plan: given `Uncommitted files: ?? Main.jar`, the plan is "ignore or delete `Main.jar`, commit, push, ready", not a re-implementation.
- Keep the short prompt working too (both are valid inputs).

## How to test

Unit tests on the fixtures; then a Hive Mind run with the special prompt removed produces the same PR as with an LLM model.


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
- hive-mind one prompt for every model: https://github.com/link-assistant/hive-mind/issues/2313 and https://github.com/link-assistant/hive-mind/issues/2319

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1160-regular-hive-mind-prompt.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1160_regular_prompt.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1160/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1160-regular-hive-mind-prompt.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1160_regular_prompt` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **all three formal-ai rows** passes on the released version, linked here.

## Depends on / blocks

- **Urgent:** link-assistant/hive-mind v2.33.1 (PR #2321, 2026-09-28) already removed the Formal AI prompt dialect and the empty system prompt, so every `solve --model formal-ai` run now sends Formal AI the regular user and system prompts. Formal AI 0.352.1 has not been tested with them. Capture the exact prompts Hive Mind v2.33.1 sends (run `solve … --model formal-ai --tool claude --dry-run --verbose` or read `src/claude.prompts.lib.mjs`, `src/agent.prompts.lib.mjs`, `src/codex.prompts.lib.mjs` at v2.33.1) as the fixtures.
- Depends on: nothing.
- Blocks: #1158 (E123), and every formal-ai row of the Hive Mind matrix.



