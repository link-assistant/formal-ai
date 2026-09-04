# Reproducing the issue #661 agent-CLI E2E failure (issue #1069)

`E2E Tests (agent CLI ↔ formal-ai)` failed on every run of this branch at

```
- name: "Run agent CLI E2E — repository statement audit (issue #661)"
  run: experiments/agent_cli_e2e/run_issue_661_statement_audit.sh
```

exiting 1 within about four seconds and printing nothing. The silence is the
script's: it asserts `[[ -f $REPORT ]]` first, and the report *was* there, so
the first line to say anything was `grep -q '^repository_statement_audit'` --
which fails without output.

## What actually happened

The audit command writes `statement-audit.lino` itself. On the turn after it
succeeded, the planner emitted a second tool call -- a `write` of its own
rendered status line to that same path -- and the agent CLI carried it out,
landing the narration on top of the report.

## The two artefacts here

`run-keep.sh` is `experiments/agent_cli_e2e/run_issue_661_statement_audit.sh`
with its cleanup changed to `cp -a "$work" /tmp/e2e661-work` before `rm -rf`,
so the workspace, the agent stream and the traced request bodies survive the
run. It must stay at this directory depth: the script resolves the repository
with `ROOT="$(cd "$(dirname "$0")/../.." && pwd)"`.

```bash
RUSTUP_TOOLCHAIN=1.98.0 cargo build --release --bin formal-ai
PORT=8884 BIN=target/release/formal-ai experiments/issue_1069_e2e_661/run-keep.sh
cat /tmp/e2e661-work/statement-audit.lino
```

`probe_661.rs` reduces the same three-turn conversation to one `plan_chat_step`
call, so the planner can be questioned without a server or the CLI. Copy it to
`tests/probe_661.rs` and run `RUSTUP_TOOLCHAIN=1.98.0 cargo test --test
probe_661 -- --nocapture`; it panics on purpose to print the plan for each tool
set. Before the fix, `["bash"]` returned `Final(...)` while any tool set that
also advertised `write` returned a `write` call to `statement-audit.lino`.

## The fix

`later_route_delivering` declines a delivery when a route below already
produces the requested file, but `probe_settled_routes` only recognised a
*write* call naming it, never a *command* naming it with `--output`. Reading a
planned command's destination the same way `src/agentic_coding/driver.rs`
already does restores the decline. The regression test lives in
`tests/unit/issue_661_agentic_statement_audit.rs`.
