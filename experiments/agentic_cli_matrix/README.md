# Multi-CLI agentic end-to-end matrix (issue #671)

The executable form of the "CI Shape" section of
[`docs/testing/agentic-cli-tools.md`](../../docs/testing/agentic-cli-tools.md).
One leg per client `formal-ai clients` knows about, each driving the **real**
third-party CLI against a local `formal-ai serve --agent-mode` with
`formal-ai proxy` recording every exchange.

Our own server is the model provider, so no leg needs vendor credentials.

## Why the real CLIs, and not `curl`

An API-level check passes while the TUIs are broken. A real client advertises
tools differently than a hand-written request does — the Codex TUI offers web
search as a hosted `{"type":"web_search"}` tool (#746) — and interactive-only
launch bugs (#713) are invisible to `--non-interactive` runs; 160 such runs
missed two launch blockers. So every leg drives the actual binary, headless
*and* through a real PTY, and asserts on the streamed output.

Issue #671's own first finding came from exactly this: `formal-ai with
--non-interactive codex "read the file alpha.txt"` never terminated, because
Codex's `exec_command` takes `cmd` while the planner plans `command`, and the
planner could not recognise its own projected tool result. 281 identical model
rounds. A `curl` using the canonical key passed the whole time.

## Layout

| File | Purpose |
| --- | --- |
| `clients.lock` | `<client-id> <installer> <spec>` — the pinned version of every client. |
| `install_client.sh` | Installs one pinned client (`npm`/`pipx`/`apt`/`appimage`/`script`). |
| `lib.sh` | Shared harness: stack management, fixtures, proxy assertions, headless and PTY drivers. |
| `run_leg.sh` | Runs every case for one client. |

Nothing here hardcodes a client list. `lib.sh` reads each client's shape from
`formal-ai clients --format json`, which is generated from
`data/seed/client-integrations.lino` — the same registry `formal-ai with` uses —
so a leg cannot drift from the wrapper it exists to prove. A client added to the
seed without a pin, a CI leg and a documented row fails
`tests/unit/issue_671_matrix_coverage.rs`.

## Running a leg locally

```bash
cargo build --release --bin formal-ai
experiments/agentic_cli_matrix/install_client.sh codex
CLIENT=codex experiments/agentic_cli_matrix/run_leg.sh
```

Artifacts land in `artifacts/<client>/<case>/`: `proxy.jsonl` (the recorded
exchanges), `serve.log`, and the CLI's own output. Read the transcript with:

```bash
jq -c '{path, request_model, request_tools, status, response_tool_calls}' \
  experiments/agentic_cli_matrix/artifacts/codex/read-file/proxy.jsonl
```

Useful knobs: `BASE_PORT` (default 8900; each case takes `BASE_PORT + 10n` and
the next port for its proxy), `CASE_TIMEOUT`, `ARTIFACTS`, `BIN`.

Some clients run their shell tool inside their own sandbox. Codex uses
bubblewrap, which needs unprivileged user namespaces; inside a container that
lacks them, `cat alpha.txt` returns
`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted` *as the command's
output*, and the server quotes that back as the file's contents.
`matrix_assert_client_sandbox_worked` names this rather than letting it read as
a missing marker. To run the leg anyway on such a host:

```bash
CLIENT=codex MATRIX_CLIENT_ARGS=--dangerously-bypass-approvals-and-sandbox \
  experiments/agentic_cli_matrix/run_leg.sh
```

CI leaves `MATRIX_CLIENT_ARGS` empty, so the real sandboxed path is what the
matrix exercises.

## Cases

See the table in
[`docs/testing/agentic-cli-tools.md`](../../docs/testing/agentic-cli-tools.md#cases-and-the-defect-each-one-guards)
for what each case guards. Two harness rules are worth repeating here:

- **Bounded rounds are an assertion, not a timeout.** `matrix_assert_bounded_rounds`
  fails the moment a case exceeds its round budget, so a tool-call loop reports
  as a loop rather than as a slow leg.
- **Upstream limitations are assertions, not skips.** When a Gemini release
  starts advertising `functionDeclarations` headlessly, the `constraints` case
  fails and tells us to delete the assertion and add real coverage. A skip would
  stay silent forever.

## Gotcha: never reuse a proxy log path

`formal-ai proxy` holds an `append` handle open for its whole lifetime.
Truncating or deleting a live log sends later rows to an unlinked inode instead
of resetting the file, and the assertions then read an empty transcript.
`matrix_start_stack` gives every case a fresh log and a fresh proxy.
