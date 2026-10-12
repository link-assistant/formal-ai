# Issue #1154 — Codex re-plans the same `gh issue view` 78 times

2026-09-27, Hive Mind Rust run, `--tool codex`, backend 0.352.1.
Full log: https://gist.github.com/konard/513141ac5f144b15e9593f05f86362e8

## What happened

Formal AI planned the work-item read through Codex's `exec_command` tool:

```
exec_command {"cmd":"gh issue view '…/issues/1' --json title --jq .title && echo && gh issue view '…/issues/1' --json body --jq .body"}
```

The call returned the title and body (exit 0). Formal AI then planned **the
same read again, 78 times in a row**, each preceded by "Let me run the
requested command to get that for you.", burning 6.2M input tokens until Codex
compacted the thread. Nothing was implemented.

## Root cause

Codex's shell tool argument key is `cmd`. `progress.rs:466-472` read only
`command`:

```rust
fn command_argument(arguments: &str) -> Option<String> {
    let arguments: serde_json::Value = serde_json::from_str(arguments).ok()?;
    arguments.get("command").and_then(serde_json::Value::as_str).map(str::to_owned)
}
```

so `issue_view_url` returned `None`, `attempted_work_item_reads` never recorded
the URL, and `plan_work_item_read` re-derived the identical call every turn.
Six sibling helpers existed; five already accepted both keys, and
`workspace_change.rs` shared the `command`-only defect.

## The fix

1. **One shared reading** (`rust/src/agentic_coding/tool_result.rs`):
   `command_argument` accepts `command`, `cmd`, `script`, and joined array
   forms; `command_argument_key` resolves the key from a tool definition's own
   schema for callers that hold the request's tool declarations. The seven
   local copies delegate to it.
2. **A parsing-independent invariant** (`planner.rs::stop_repeated_call`): a
   planned call that repeats one this turn already *completed* — byte-identical
   arguments, or the same canonical command/URL under the client's own
   spelling — is replaced by a report of the stuck step and its last result
   (`stuck_step_report` in `data/meta/work-item-steps.lino`). Failed calls keep
   their bounded retry; the second identical failure was already stopped by the
   issue-#1133 guard.
3. The successful read now lands in `fetched_pages` under the `cmd` spelling,
   so `repository_work_item_objective` drives `plan_work_item_execution`
   exactly as on the Claude and Agent paths.

## Verification

`rust/tests/unit/issue_1154_progress_arguments.rs` replays the transcript's
first two Responses items through the server's own Responses conversion and
asserts the next plan is the artifact write, never another `gh issue view`;
a table test covers `{command}`, `{cmd}`, `{script}` and the joined array
form; a third test replays the twice-repeated transcript shape the run
accumulated and pins that no third identical call is planned.
