# Issue #1075 — the sidecar boundary, before and after

Captured by `experiments/agent_cli_e2e/run_issue_1075.sh` against the real
Agent CLI and the real Claude Code CLI, on the release matrix's own hello-world
task (`Give me hello world program in Rust`, `main.rs`, `Hello, world!` —
`.github/workflows/release.yml:1091`).

The server runs in a container with only its own `/sidecar` mounted; each client
runs on the host in a fresh directory. A client's directory therefore does not
exist from where the server stands, which is the condition that made the
September 4 sessions fail. Two directories on one host are *not* enough to
reproduce it: the pre-fix code followed any declaration it could `is_dir()`, so
a visible workspace was honoured and nothing went wrong.

## `issue-1075-before-fix.txt` — `origin/main`

```
FAIL[agent]:  main.rs missing from /tmp/tmp.VEab1Xm2u5
FAIL[claude]: main.rs missing from /tmp/tmp.28nznrKbJX
FAIL: the server offered clients paths inside its own directory:
"/sidecar/main.rs"
"Error: ENOENT: no such file or directory, open '/sidecar/main.rs'"
```

Both clients were told to write the program into the server's own directory.
Both did exactly as they were told, both failed — Agent with `ENOENT`, Claude
with `EACCES: permission denied, mkdir '/sidecar'` — and both task workspaces
stayed empty. That is the issue's Scala row, reproduced twice.

## `issue-1075-after-fix.txt` — this branch

```
PASS[agent]:  main.rs written inside the client's workspace
PASS[agent]:  the program compiles and prints exactly: Hello, world!
PASS[claude]: main.rs written inside the client's workspace
PASS[claude]: the program compiles and prints exactly: Hello, world!
PASS: the server's directory gained no authored file
PASS: no emitted path was rooted in the server's directory
```

The greeting is `rustc` on the file each run actually produced, not a transcript
assertion.

## Codex

`codex exec` is not driven here. Pointed at this server per the README's Codex
CLI section (`wire_api = "responses"`, `/api/openai/v1`), it declares its own
`<cwd>` correctly and the server answers in a single round with the program in
prose, emitting no tool call and writing no file. Nothing addresses the sidecar,
so there is no boundary for this replay to check. The Codex row of the issue —
`codex_apps__github.create_file` chosen over `apply_patch` — is covered by
`tests/release_tool_grounding.rs::a_workspace_file_is_created_in_the_workspace_and_not_on_a_server`,
which advertises Codex's own tool shapes with a GitHub connector beside them.

## Re-capture

```
cargo build --release --bin formal-ai
experiments/agent_cli_e2e/run_issue_1075.sh
```
