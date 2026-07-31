# Verified computer use without vision

Formal AI can turn ten seeded natural-language computer-use requests into
deterministic plans over twelve named primitives:

`fs.read`, `fs.write`, `fs.list`, `fs.move`, `shell.run`, `http.fetch`,
`http.post`, `dom.query`, `dom.extract`, `archive.pack`, `archive.unpack`, and
`process.status`.

The planner is intentionally non-visual. It operates on files, structured JSON
and HTML, and recorded HTTP fixtures. A request that needs pixels, a rendered
page, or a GUI-only application returns the named `gui_rendering`
`capability_gap`; it does not claim that a visual action happened.

## Native CLI

Both agent mode and effect confirmation are explicit:

```bash
formal-ai computer-use \
  --prompt "Filter active customers into a report" \
  --agent-mode \
  --confirm-effects \
  --replay
```

The JSON result contains the isolated workspace, the plan, each primitive's
arguments and output, and three events for every step: `precondition`, `effect`,
and `postcondition`. `--replay` executes the same plan in a fresh workspace and
independently compares the verified outputs. Omitting either permission flag
fails before a workspace effect.

The complete multilingual prompt set is in
[`data/seed/computer-use-tasks.lino`](../data/seed/computer-use-tasks.lino).
The tasks cover file transforms, structured GET/POST and extraction, report
generation, directory operations, archives, and isolated process status.

## Server and external agents

Start the server with computer-use tools enabled:

```bash
formal-ai serve --host 127.0.0.1 --port 8080 --agent-mode
```

The MCP endpoint at `http://127.0.0.1:8080/mcp` advertises
`formal_ai_<primitive>` tools alongside `formal_ai_chat`. Each primitive schema
requires a plan id, step id, precondition, and postcondition. Without
`--agent-mode`, calls return a structured policy refusal.

Set `FORMAL_AI_COMPUTER_USE_AUDIT_PATH` to an operator-owned JSONL path to
persist one complete step record per MCP call:

```bash
FORMAL_AI_COMPUTER_USE_AUDIT_PATH=/tmp/formal-ai-computer-use.jsonl \
  formal-ai serve --host 127.0.0.1 --port 8080 --agent-mode
```

The issue-707 acceptance harness connects the real Link Assistant Agent CLI to
both the OpenAI-compatible endpoint and MCP, executes all ten requests, restarts
the server, and executes all ten again:

```bash
cargo build --bin formal-ai
experiments/agent_cli_e2e/run_issue_707.sh
```

## Permissions and isolation

Every primitive has its own `tool:computer:<primitive>` permission. Native
plans receive those grants only after agent mode is selected; desktop calls
must receive the corresponding explicit grant from the renderer. State-changing
operations additionally require `confirmed: true`.

All native paths are relative, reject parent traversal and absolute paths, and
resolve beneath a fresh per-plan temporary workspace. `shell.run` accepts only
the structured `count_lines`, `filter_csv`, and `unique_csv` operations; it does
not pass user text to a shell. Desktop computer-use paths are separately
confined beneath the application's user-data `computer-use/<plan_id>` directory;
they never resolve against the repository workspace used by the older desktop
file tools. Native benchmark HTTP accepts only committed `fixture://` resources,
while the permission-gated desktop executor can use HTTP(S). GET and POST
outputs record method, URL, status, cache path, and SHA-256 provenance.

The deterministic archive format is `formal-ai-archive-v1`. It is designed for
verified task replay, not as a general replacement for ZIP or tar.
