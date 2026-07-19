# Issue 758: capability-first agentic tool routing

## Reproduction and root cause

The agentic planner previously recognized only six broad capabilities and then
classified concrete tool names with ad hoc substring rules. That made local
`grep`/`codesearch` names look like web search, left glob/list/todo/subagent and
multi-file tools unreachable, and allowed the client’s alphabetical tool order
to decide between semantically equivalent aliases.

The protocol boundary already removed undeclared top-level argument aliases,
but it did not recursively constrain nested arrays and objects. Strict clients
could therefore reject otherwise correct todo or multi-edit calls.

## Implemented behavior

- `data/seed/agentic-tool-capabilities.lino` is the shared source of capability
  aliases and multilingual intent cues.
- Routing matches intent to a capability first, then chooses the highest-priority
  advertised alias from the registry. A specialized tool wins; shell is used
  only for capabilities with a meaningful fallback.
- The shared set covers web search/fetch, read/write/edit, shell, grep, glob,
  directory listing, todo planning, subagent delegation, multi-file reads and
  edits, memory, image viewing, and user questions.
- Argument projection retains only advertised fields, maps semantic aliases,
  fills required fields, enforces scalar types and enums, and recursively
  projects nested object/array schemas.

## Verification

The initial four-group unit regression failed before the implementation and is
preserved in the development evidence as `red-test.log`. The committed tests
cover every reported search alias, all newly reachable shared capabilities,
specialized-over-shell precedence, strict schema projection, and English,
Russian, Hindi, and Chinese intent parity.

The real Agent CLI E2E creates a unique local marker in `src/fixture.rs`, boots
the release-mode `formal-ai serve --agent-mode` server, and asks the external
`@link-assistant/agent` CLI to search the local code. It passes only when:

1. the server plans the local `grep` tool while `codesearch` and `websearch` are
   also advertised;
2. schema projection emits only the client’s declared `pattern` property;
3. the actual tool result and final CLI response contain `src/fixture.rs`; and
4. no planned call uses `websearch`.

The harness shadows `gh` with a failing fixture binary, so an unrelated routing
regression cannot mutate GitHub while this test runs. The successful release
run is captured in `agent-cli-e2e-run.log`, with the raw server trace in
`raw-data/formal-ai.log`.

Reproduce it with:

```sh
cargo build --release --bin formal-ai
experiments/agent_cli_e2e/run_issue_758.sh
```

Focused automated checks:

```sh
cargo test --test unit issue_758
cargo test --test integration issue_758
```
