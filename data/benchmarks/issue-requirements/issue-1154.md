## What happened

Hive Mind Rust run, `--tool codex`, backend 0.352.1, log https://gist.github.com/konard/513141ac5f144b15e9593f05f86362e8.

Formal AI planned the work-item read through Codex's `exec_command` tool. The call Codex logged (`tool_name="exec_command"`) is

```
exec_command {"cmd":"gh issue view 'https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1' --json title --jq .title && echo && gh issue view '…/issues/1' --json body --jq .body"}
```

and it returned the title and body (exit 0). (Codex *displays* the command as `/bin/bash -lc "…"`; that wrapper is not in the arguments Formal AI receives back.) Formal AI then planned **the same read again, 78 times in a row** (each preceded by "Let me run the requested command to get that for you."), 6.2M input tokens, until Codex compacted the thread; after compaction it listed the directory, read `.gitkeep`, and answered "Read 1 file(s)". Nothing was implemented.

## Root cause

Codex's shell tool is `exec_command`, and its argument key is **`cmd`**. `rust/src/agentic_coding/progress.rs:466-472`:

```rust
fn command_argument(arguments: &str) -> Option<String> {
    let arguments: serde_json::Value = serde_json::from_str(arguments).ok()?;
    arguments.get("command").and_then(serde_json::Value::as_str).map(str::to_owned)
}
```

reads only `command`. So for Codex `issue_view_url` returns `None`, `attempted_work_item_reads` never records the URL, `fetched_pages` never receives the body, and `plan_work_item_read` (`general_execution.rs:252-271`) plans the identical call on every turn (78 times).

The same helper exists six more times, and the others already accept both keys: `file_read.rs:718` (`["command", "cmd"]`), `tool_result.rs:850`, `narration.rs:82`, `command_reroute.rs:401`, `evidence_record.rs:486`; `protocol_responses.rs:167` maps `command | cmd`. `workspace_change.rs:608` reads only `command`, like `progress.rs`. The #1133 fix (`progress.rs:170-176`) guarded the fetch tool against the same loop, not the run tool.

## Requirements (what to do)

1. One shared `command_argument` for the whole `agentic_coding` module, reading the argument key from the client's declared tool schema (the key whose schema describes the shell command), falling back to `command`, `cmd`, `script`, and an array form joined for `shell`-style tools. Delete the seven local copies (`progress.rs`, `file_read.rs`, `tool_result.rs`, `narration.rs`, `command_reroute.rs`, `evidence_record.rs`, `workspace_change.rs`).
2. Invariant independent of parsing: never plan a tool call whose name and arguments are byte-identical to one already attempted in the current turn window; if the planner would, report the stuck step and its last result instead of repeating it.
3. After a successful read, the fetched text drives `plan_work_item_execution` exactly as on the Claude and Agent paths.
4. Tests: replay the Codex transcript's first two Responses items (`function_call` `exec_command {"cmd": …}` + `function_call_output`) through `protocol/responses_input.rs` and assert the next plan is not another `gh issue view`; a table test over `{command}`, `{cmd}`, `{script}`, `{command:[…]}` yields the same URL from `issue_view_url`.

## How to test

`solve <rust issue> --model formal-ai --tool codex` (Hive Mind ≥ 2.32.0): the session performs one `gh issue view`, then writes `main.rs` + workflow, verifies, commits, pushes.


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
- hive-mind breaker for every adapter: https://github.com/link-assistant/hive-mind/issues/2316

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1154-codex-exec-command-cmd.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1154_command_argument_keys.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1154/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1154-codex-exec-command-cmd.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1154_command_argument_keys` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **formal-ai × codex (Rust)** passes on the released version, linked here.

## Depends on / blocks

- Depends on: nothing.
- Blocks: the codex row of the Hive Mind matrix; #1165 (E130), because discovery runs over the same progress scan.



