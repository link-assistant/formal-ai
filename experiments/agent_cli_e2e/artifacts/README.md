# Issue #1075 — the sidecar boundary, before and after

Captured by `experiments/agent_cli_e2e/run_issue_1075.sh` against the real Agent
CLI, on the release matrix's own hello-world task
(`Give me hello world program in Rust`, `main.rs`, `Hello, world!` —
`.github/workflows/release.yml:1091`).

The server runs in a container with only its own `/sidecar` mounted; the client
runs on the host. The client's directory therefore does not exist from where the
server stands, which is the condition that made the September 4 Scala session
fail. Two directories on one host are not enough to reproduce it: the pre-fix
code followed any declaration it could `is_dir()`, so a visible workspace was
honoured and nothing went wrong.

## `issue-1075-before-fix.log` — `origin/main`

```
FAIL: main.rs missing from /tmp/tmp.cJf022pAP7
FAIL: the server offered the client paths inside its own directory:
"/sidecar/main.rs"
"Error: ENOENT: no such file or directory, open '/sidecar/main.rs'"
```

The program was addressed into the server's own directory. The client did as it
was told, failed, and the task workspace stayed empty — the issue's Scala
failure, reproduced.

## `issue-1075-after-fix.log` — this branch

```
PASS: main.rs written inside the client's workspace
PASS: the server's directory gained no authored file
PASS: no emitted path was rooted in the server's directory
PASS: the program compiles and prints exactly: Hello, world!
```

Re-capture with:

```
cargo build --release --bin formal-ai
experiments/agent_cli_e2e/run_issue_1075.sh
```
