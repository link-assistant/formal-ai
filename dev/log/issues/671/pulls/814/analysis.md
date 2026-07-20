# Issue #671 — Multi-CLI agentic end-to-end matrix in CI

- Session: `issue-671-claude-20260720`
- Agent: formal-ai (Claude Opus 4.8) via `/solve`
- Issue: <https://github.com/link-assistant/formal-ai/issues/671>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/814>
- Harness: `experiments/agentic_cli_matrix/`
- Workflow: `.github/workflows/agentic-cli-matrix.yml`

Every claim below is reproducible from the repository root. Where something is
inferred rather than observed, it says so.

## 1. What the issue asked for

`docs/testing/agentic-cli-tools.md` prescribed a verification sequence for real
third-party CLI clients against `formal-ai serve --agent-mode`, but shipped as
prose only. PR #648 closed #647 with `claude` "intentionally not run" and
`grok`/`aider` "inferred from the shared adapters"; hands-on testing then found
four defects (#650). The issue asks for the guide's "CI Shape" section to become
a real job matrix — one leg per client, no vendor credentials, upstream
constraints encoded as assertions rather than skips.

Comments on the issue add three binding requirements: the matrix must drive the
**interactive/TUI** path too (#713 — two launch-blocking interactive-only bugs
survived 160 `--non-interactive` runs), API-level `curl` checks are insufficient
because a real TUI advertises tools differently (#746 — the Codex TUI advertises
a hosted `{"type":"web_search"}`), and every client the repository supports must
be covered uniformly, including the GUI rows added by PR #788.

## 2. What was built

`experiments/agentic_cli_matrix/` drives the **real** binary of every client in
`data/seed/client-integrations.lino`, headless and through a real PTY, against a
local `formal-ai serve --agent-mode` with `formal-ai proxy` (PR #631) recording
every exchange. Our own server is the model provider, so no leg needs vendor
credentials.

Client shapes are read from `formal-ai clients --format json` rather than
hardcoded, so a leg cannot drift from the `formal-ai with` wrapper it exists to
prove:

| Shape | Clients | What the leg proves |
| --- | --- | --- |
| `cli` | codex, opencode, agent, gemini, claude, qwen, grok, aider | prompt in, answer out, headless *and* through a PTY |
| `server` | t3code | starts, serves its UI, carries our base URL |
| `gui` | opencode-vscode, opencode-desktop | same, windowed, under Xvfb |
| `mcp` | cursor | we are the *tool server*: `initialize`, `tools/list`, `tools/call`, unknown-tool refusal |

`tests/unit/issue_671_matrix_coverage.rs` fails the build if a client is added
to the seed registry without a pinned version, a CI leg and a documented row.

## 3. Defects the matrix found in our own code

Each was invisible to the hand-written `curl` checks that preceded it.

1. **The planner could not recognise its own projected tool result.** Codex's
   `exec_command` takes `cmd` where the planner plans `command`; Gemini's
   `read_file` takes `absolute_path` where the planner plans `path`. A single
   read request re-planned the identical call 281 times and never terminated.
   Fixed by matching recorded results through the same alias set
   `protocol_responses` projects with. Regression:
   `tests/unit/issue_671_planner_tool_alias.rs`.
2. **A read request could plan a write that destroyed the file.** `show me the
   contents of the file beta.md` planned `write(beta.md, "of the")`.
3. **Tool paths were absolutised only for Gemini's literally-named property.**
   Now driven by what the client's own schema advertises.
4. **`formal-ai serve` 405'd on `HEAD`**, so Claude Code reported the server as
   unreachable.
5. **Gemini utility model ids were rejected with 400**, and `formal-ai proxy`
   logged a null `request_model` for Gemini-shaped paths.

## 4. Defect the matrix found in itself

`set -o pipefail` plus `matrix_strip_ansi | grep -q` returns 141 whenever `grep`
exits before `sed` finishes, so a marker *found early in a long log* was reported
as missing. The failure was length-dependent, which is why it read as a client
defect: the `agent` TUI rendered `ALPHA_MARKER_11111` three times into a 31 KB
log and its leg still reported "client output never contained" it. It also
disarmed the negative assertions (a real `bwrap:` match returned 141, so
`&& matrix_fail` never fired) and made every `await:` step spin for its full
timeout. Every log assertion now goes through `matrix_log_matches` / `_ci` /
`_re`, which use process substitution and keep `grep`'s own exit status.

## 5. Verification

Full serial run of every leg on one host:

```
== all legs OK: codex t3code opencode opencode-vscode opencode-desktop agent \
   cursor gemini claude qwen grok aider ==
```

Offline replay of the committed transcripts (`jq` only, no CLI, no server, no
network, no credentials):

```
bash experiments/agentic_cli_matrix/replay.sh
== replay OK ==
```

Local CI: `cargo fmt --all -- --check` clean, `cargo clippy --all-targets
--all-features -- -D warnings` exit 0, `cargo test --all` 1971 passed, 0 failed.

`claude`, `grok` and `aider` — the three integrations PR #648 shipped without
ever running — now each have a recorded, replayable session under
`experiments/agentic_cli_matrix/recorded/`.

## 6. Upstream constraints, as assertions

- Gemini's headless `-p` advertising no `functionDeclarations` (#620) — the
  matrix is what discovered this was **lifted** in `@google/gemini-cli@0.51.0`:
  the assertion failed loudly, exactly as the issue asked. It was inverted, not
  deleted.
- No headless approval handshake in codex/gemini/qwen (#511).
- Cursor still requires its own vendor credentials (`mcp` leg).
