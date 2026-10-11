## What happened

Hive Mind Scala run, `--tool agent`, backend 0.352.1, log https://gist.github.com/konard/8a196a1d179ecb105304232a46e09ede.

The planned `gh issue view` came back with

```
To get started with GitHub CLI, please run:  gh auth login
Alternatively, populate the GH_TOKEN environment variable …
```

(a Hive Mind environment defect: link-assistant/hive-mind#2314). Formal AI then wrote `.formal-ai/general-change-plan.lino` and answered:

> Planned, not executed … Nothing the request names was changed. The only file this run wrote is its own plan record

The Agent CLI had offered `webfetch`, `websearch`, `read`, `bash` and ten other tools (`tools` array of the first request). None was tried after `gh` failed.

## Root cause (verified in code and log)

1. The Agent CLI sent the result **without its exit status**. Its own record says `"metadata": {"exit": 4}`, but the tool message Formal AI received was only `{"role":"tool","tool_call_id":"call_d5f4f5aa42e4af89","content":"To get started with GitHub CLI, please run:  gh auth login\n…"}` (link-assistant/agent#317).
2. For `Run` results Formal AI never infers failure from text: `Progress::scan` calls `failure_message(&raw, message.is_error, capability != Capability::Run)` (`progress.rs:88-92`), and `failure_message` (`tool_result.rs:131-148`) only reads an explicit flag or a parsed exit code. So the result counted as a **successful** read.
3. `Progress::scan` then stored the auth message as the issue text: `fetched_pages.push((url, text))` (`progress.rs:143-151`).
4. `repository_work_item_objective` returned that text, so `plan_work_item_read` and its `webfetch` fallback were **never reached**; `plan_work_item_execution("To get started with GitHub CLI…")` found no artifact, the executor wrote the plan record, and `finish_general_change` answered `planned_not_executed` (`general_execution.rs:136-148, 313-320`).

## Requirements (what to do)

1. Make the read observe its own status whatever the client sends: plan `gh issue view … ; printf '\n__formal_ai_exit=%s\n' "$?"` (as data in `data/meta/work-item-steps.lino`), and treat a non-zero status as a failed read.
2. Validate the read's shape before accepting it as the issue: the first line is the title and the rest the body, as the command prints them; output that does not have that shape is a failed read, recorded with its text.
3. On a failed read, walk the fallbacks in order and record each attempt: the client's fetch tool (`webfetch`) on the issue URL, `curl -fsSL https://api.github.com/repos/<owner>/<repo>/issues/<n>` (public repositories), `gh api`, then the PR title and body and the branch name.
4. Only when all fail is `planned_not_executed` honest, and the answer must list every read tried with its result (today it does not mention the `gh` failure). Never spend the only write on the plan record when the goal could not be read.
5. Replay test from this transcript (tools list with `webfetch`, the `gh` result without exit status): the next plan is the `webfetch` of the issue URL; with the sentinel line `__formal_ai_exit=4` the read is a failure.

## How to test

With `GH_TOKEN` unset and `XDG_CONFIG_HOME` pointing at an empty dir, run the agent CLI against a local `formal-ai serve` with the Hive Mind prompt for the Scala issue: the session must end with `Main.scala` + workflow written, verified, committed, pushed.


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
- hive-mind agent gh auth: https://github.com/link-assistant/hive-mind/issues/2314

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1155-work-item-read-validation.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1155_work_item_read_failure.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1155/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1155-work-item-read-validation.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1155_work_item_read_failure` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **formal-ai × agent (Scala)** passes on the released version, linked here.

## Depends on / blocks

- Depends on: nothing (the exit sentinel makes the fix independent of link-assistant/agent#317, which fixes the same gap for every model).
- Blocks: the agent row of the Hive Mind matrix.



