## What happened

Kotlin run: the main session committed and pushed correctly but left `Main.jar` (output of `kotlinc … -d Main.jar`) untracked. Hive Mind restarted the session to deal with the untracked file. The restart session (same prompt) re-wrote the same three files, re-ran `kotlinc` and the verify script, and ran the commit chain from `data/meta/work-item-steps.lino`:

```
git add -- {files} && git commit --only -q -m … -- {files} && git push -q origin {branch} && git rev-parse HEAD
```

which exited 1 with `nothing added to commit but untracked files present`. Hive Mind treated the failed final tool call as a failed session; the second restart did the same; the run stopped with `no_progress_between_sessions` and the PR was demoted to draft.

## Root causes (Formal AI side)

1. The build output is not ignored or cleaned. `tests/verify-output.sh` and the workflow need `Main.jar` at run time, but it must not be left in the working tree: either `.gitignore` it (`*.jar`, `*.class`, `target/`, …, per language) or build into a temp dir.
2. The commit chain is not idempotent. `git commit --only` on an already-committed tree is a failure exit, so re-running a finished work item looks like an error.
3. A continuation request whose artifact already exists, verified and committed, is re-executed from scratch. The executor should recognise "already done" (files identical, HEAD contains them, verify passes) and answer with the existing commit instead of redoing the work — the same deterministic policy Hive Mind's no-progress detector then sees as "no change".

## Requirements (what to do)

- Build outputs go to a temporary directory outside the checkout (`kotlinc … -d "$build_dir/Main.jar"`, `cargo --target-dir`, `scalac -d`), with the verify script and the workflow using the same path; and the recipe writes a `.gitignore` for the language's output patterns (taken from the toolchain documentation, #1168 (E133) discovery) so a manual build stays clean too.
- Commit step: check `git status --porcelain -- {files}` first; if clean, skip commit/push and report the existing HEAD. Treat "nothing to commit" as success with an explanatory sentence.
- On continuation, verify before executing: if the artifact and workflow match the recipe and the verification command passes, report completion with the existing commit.
- Tests for all three: a second run on an already-solved checkout ends with success and no failed tool call; the working tree is clean after `kotlinc`.

## How to test

Run the Kotlin work item twice against the same checkout; the second run must end successfully without a failed tool call, and `git status --porcelain` must be empty after each run.


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
- hive-mind identical restart prompt: https://github.com/link-assistant/hive-mind/issues/2313
- hive-mind auto-commit of build artifacts: https://github.com/link-assistant/hive-mind/issues/2315

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1157-idempotent-commit-clean-build.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1157_idempotent_commit.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1157/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1157-idempotent-commit-clean-build.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1157_idempotent_commit` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **formal-ai × claude (Kotlin)** passes on the released version, linked here.

## Depends on / blocks

- Depends on: nothing.
- Blocks: every Hive Mind restart (an untracked build output restarts the session); the claude row of the matrix.



