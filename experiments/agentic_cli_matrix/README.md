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
| `install_client.sh` | Installs one pinned client (`npm`/`npm-native`/`pipx`/`tarball`/`appimage`/`script`). |
| `lib.sh` | Shared harness: stack management, fixtures, proxy assertions, headless and PTY drivers. |
| `run_leg.sh` | Runs every case for one client. |
| `run_matrix.sh` | Runs several legs one after another on a single machine (CI runs them in parallel). |
| `replay.sh` | Replays the committed transcripts offline; needs `jq` and nothing else. |
| `recorded/` | Committed transcripts, one directory per client and case. |

Nothing here hardcodes a client list. `lib.sh` reads each client's shape from
`formal-ai clients --format json`, which is generated from
`data/seed/client-integrations.lino` — the same registry `formal-ai with` uses —
so a leg cannot drift from the wrapper it exists to prove. A client added to the
seed without a pin, a CI leg and a documented row fails
`tests/unit/issue_671_matrix_coverage.rs`.

## The four leg shapes

Not every client answers a prompt, so not every leg can assert on one. Handing a
client assertions its integration can never satisfy produces a red leg that
means nothing — or, worse, a green one. `matrix_client_shape` picks:

| Shape | Clients | What the leg proves |
| --- | --- | --- |
| `cli` | codex, opencode, agent, gemini, claude, qwen, grok, aider | The full case list: prompt in, answer out, headless *and* through a PTY. |
| `server` | t3code | The client starts, serves its UI, and carries our base URL. |
| `gui` | opencode-vscode, opencode-desktop | Same, windowed, under Xvfb. |
| `mcp` | cursor | We are the **tool server**, not the model: `/mcp` JSON-RPC `initialize`, `tools/list`, `tools/call` and an unknown-tool refusal, plus the assertion that the CLI still needs its own vendor credentials. |

The `mcp` shape is read from the registry (`default_protocol "mcp"`), not from a
name list. Before it existed the cursor leg was entirely red for a reason that
was not a defect: the wrapper writes `.cursor/mcp.json` and Cursor's own model
drives, so there is no base URL to redirect and no prompt this harness can put
through it — without `CURSOR_API_KEY` the CLI exits 1 with "Authentication
required" before any turn.

`server` and `gui` legs cannot assert on traffic either: a GUI sends nothing
until a human types. `matrix_assert_launch_configured` walks the launched
process tree and requires our base URL in a process's environment or in the
config file that environment names (`OPENCODE_CONFIG` → a JSON file, `CODEX_HOME`
→ a directory holding `config.toml`). The assertion this replaced — "something
reached the proxy" — was satisfied by the harness's own `/health` readiness
probe, so it could never fail. Its search is deliberately narrow for the same
reason: scanning *any* process whose command line mentions the client matched an
unrelated `claude` process on the developer's host, and `$HOME` and the harness's
own artifacts directory are excluded because grepping them recursively matched
leftovers.

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

## Running several legs locally

```bash
experiments/agentic_cli_matrix/run_matrix.sh              # every locked client
experiments/agentic_cli_matrix/run_matrix.sh codex claude # just these
```

Each leg's base port is its position in `clients.lock` — `8900 + index * 60` —
which is the same formula the workflow's `base_port` values use.
`tests/unit/issue_671_matrix_coverage.rs` asserts the two agree, because a leg
that quietly shares a port with another leg ends up asserting against a
stranger's server.

Two knobs matter on a single machine and never in CI, where each leg owns a
runner:

- `MATRIX_ARGS_<CLIENT>` — extra flags for one client, e.g.
  `MATRIX_ARGS_CODEX=--dangerously-bypass-approvals-and-sandbox` on a kernel
  that cannot run Codex's bubblewrap sandbox.
- `MATRIX_ISOLATED_NPM=1` — install each npm client under its own prefix
  (`~/.formal-ai-matrix/clients/<client>`) instead of one shared global tree.
  Hoisting every client into one tree made grok die on
  `TypeError: ansiStyles.color.ansi is not a function`, its `slice-ansi`
  resolving to another client's `ansi-styles` major. That is a packaging
  accident of the host, and a leg must not report it as a defect in the client
  or in our server. `run_matrix.sh` puts these prefixes on `PATH` first.

`t3code` installs through `npm-native` rather than `npm`: it builds `node-pty`
during install, and `bun add -g` skips lifecycle scripts (with `--trust` it
still fails, because bun's bundled Node 20 cannot build node-pty at all —
`webidl.util.markAsUncloneable is not a function`). Half-installed, `t3` starts,
runs its 32 database migrations and *only then* exits 1 with
`NodePtyModuleLoadError`, which reads like a failure of our server. The
installer therefore requires Node ≥ 22 — t3's own `engines` field — and CI adds
`actions/setup-node` for that leg. If a stale bun shim is still on `PATH`, remove
`~/.bun/bin/t3` and `~/.bun/install/global/node_modules/t3`; it shadows the good
install.

`aider` installs through `pipx` and declares `Requires-Python >=3.10,<3.13`, so
`install_client.sh` pins the interpreter to the newest `python3.12`/`3.11`/`3.10`
it finds. Without that pin, `pip` resolves *backwards* to aider 0.16.0 from 2024
rather than failing — a leg installing a two-year-old CLI under the name of a
pinned modern one is worse than no leg.

## Recording and replaying

```bash
MATRIX_RECORD=1 experiments/agentic_cli_matrix/run_matrix.sh claude grok aider
experiments/agentic_cli_matrix/replay.sh          # offline, jq only
```

`MATRIX_RECORD=1` copies each case's transcript into `recorded/<client>/<case>.jsonl`
with the request/response bodies stripped, so a re-record does not churn on temp
paths and session ids. `replay.sh` re-asserts the transcript-level invariants of
every committed transcript with no CLI, no server, no network and no vendor
credentials — which is what makes claude, grok and aider, the integrations
PR #648 shipped without ever running, actually replayable. It cannot re-derive
the CLI's own rendered output; those assertions exist only in the live leg, and
the split is stated so nobody reads a green replay as a green matrix.

Replay is shape-aware, and reads the shape from the transcript's file name so it
still needs nothing but `jq`: a `cli` transcript must carry bounded model rounds
naming `formal-ai`, an `mcp` transcript must carry the three `/mcp` round trips
and must *not* name our model (if it ever does, the client stopped driving its
own and the leg is no longer testing the MCP surface), and a `launch` transcript
legitimately holds only startup traffic — its real assertion, the process
configuration, is live-only and cannot be archived.

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

## Gotcha: `pipefail` plus `grep -q` reports a match as a failure

Every log assertion goes through `matrix_log_matches` / `_ci` / `_re`, which use
process substitution rather than a pipe. The obvious spelling is wrong here:

```bash
set -uo pipefail
matrix_strip_ansi "$log" | grep -qF -- "$needle"   # 141 when the needle is found early
```

`grep -q` exits at the first match and closes the pipe, `sed` is killed by
SIGPIPE, and `pipefail` promotes that to the pipeline's status. The bug is
therefore length-dependent — it only bites when the match is far from the end of
a long log — which is what made it read as a client defect: the `agent` TUI
rendered `ALPHA_MARKER_11111` three times into a 31 KB log and its leg still
reported "client output never contained" it, while short-log clients passed.
It also disarmed the negative assertions (a real `bwrap:` match returned 141, so
`&& matrix_fail` never fired) and made every `await:` step spin for the full
`MATRIX_TUI_AWAIT` instead of returning as soon as the answer rendered.

## Gotcha: never reuse a proxy log path

`formal-ai proxy` holds an `append` handle open for its whole lifetime.
Truncating or deleting a live log sends later rows to an unlinked inode instead
of resetting the file, and the assertions then read an empty transcript.
`matrix_start_stack` gives every case a fresh log and a fresh proxy.
