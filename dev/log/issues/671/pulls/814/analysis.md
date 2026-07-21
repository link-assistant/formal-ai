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

## 7. CI findings on this pull request

Run `29792607769` (`CI/CD Pipeline`, head `6b550267`) reported two failures.
Both logs are in `ci-logs/` beside this document.

| Job | Log | Cause | Fix |
| --- | --- | --- | --- |
| Lint and Format Check | `ci-logs/lint-and-format-88517577507.log:1403` | `actionlint` shellcheck `SC2086` — `echo … >> $GITHUB_PATH` unquoted in `.github/workflows/agentic-cli-matrix.yml:174-175` | quoted `"$GITHUB_PATH"` |
| Self-Hosting Evidence Check | `ci-logs/self-hosting-evidence-88517378931.log:1353` | the branch carried no `Formal-AI-Session` / `Formal-AI-Evidence` trailers, so merging it projected the release share down from 17.14% to 15.34% | trailers added to the six Formal-AI-authored commits, pointing at this document |

The bootstrap commit `3c730d4b` ("Initial commit with task details", two lines,
authored by the solver harness rather than by Formal AI) is deliberately left
unattributed — `CONTRIBUTING.md` is explicit that an honest lower number beats a
trailer on work Formal AI did not author.

## 8. CI findings on the next head (`d51c6aa8`)

Runs `29793014327` (`CI/CD Pipeline`) and `29793014368` (`Agentic CLI Matrix`)
reported four distinct failures. Every log below is archived in `ci-logs/`.

| Job | Log | Cause | Fix |
| --- | --- | --- | --- |
| Lint and Format Check | `ci-logs/lint-and-format-88518829964.log` | five `clippy` findings in library code introduced by this branch (`redundant_pub_crate`, `ptr_arg`, `too_long_first_doc_paragraph`, `assigning_clones`, `doc_markdown`) — and a sixth, `needless_pass_by_value`, that only surfaces in the test target once the library compiles | `894125bd` |
| E2E (codex) | `ci-logs/matrix-codex-88519013924.log:328` | `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted` — codex executes every command inside its **vendored** `bwrap`, and Ubuntu 24.04 runners deny it the unprivileged user namespace via `kernel.apparmor_restrict_unprivileged_userns=1` | `d2f1c9c1` |
| E2E (gemini) | `ci-logs/matrix-gemini-88519013928.log:343` | gemini-cli 0.51.0's `isHeadlessMode()` forces headless whenever `CI`/`GITHUB_ACTIONS` is set, so the `interactive` case never got a TUI to drive | `d2f1c9c1` |
| E2E Tests (agent CLI ↔ formal-ai) | `ci-logs/agent-cli-e2e-88518829962.log:2147` | `agent: code-rewrite-learning-report.lino was never written` — the issue-#671 absolutisation resolved the planned relative path against the *server's* directory, so the write landed in the repository while the CLI ran in its own temporary workspace | `305835fe` |

`Matrix summary` carries no cause of its own: it is the aggregation job, red because
the codex and gemini legs were.

### The path fix, in full

Absolutising is right (clients reject relative paths outright), but the base
directory was wrong. Reproduced locally: the server emitted
`{"filePath": "/tmp/gh-issue-solver-.../code-rewrite-learning-report.lino"}`
while the agent CLI's workspace was `/tmp/tmp.ATy70pqOD9`. Both the matrix and
the in-process harness share a directory with the server, which is why every
existing test passed.

The client is the one that knows where it runs, and each says so in its own
prose — all four patterns below are copied from recorded request bodies:

| client | declaration |
| --- | --- |
| `agent`, `opencode` | `<env>\n  Working directory: …` |
| `codex` | `<environment_context>\n<cwd>…</cwd>` |
| `gemini` | `- **Workspace Directories:**` followed by one indented path per line |

`protocol::client_working_directory` reads them, `response_arguments_for_tool`
resolves against the result, and the server's own directory stays as the
fallback the matrix runs under. A declaration naming a directory that is not on
this machine (a transcript replayed elsewhere) is ignored rather than planned
against. Four cases in
`tests/integration/issue_671_absolute_path_projection.rs` cover the three
declaration shapes and that fallback; each fails against the previous head.

`experiments/agent_cli_e2e/run_issue_715_learning.sh` then passes locally on
both harnesses — `both harnesses derived a byte-identical review-gated report
over 7 chat rounds` — and the refreshed transcripts under
`docs/case-studies/issue-715/agent-cli-learning/` record the write landing in
the CLI's own workspace.

## 9. The file-size gate on `4f343d9c`

With clippy green, `Lint and Format Check` reached a step it had never run
before on this branch and failed there
(`ci-logs/lint-and-format-88530419664.log:2637`):

```
Found files exceeding the line limit:
  src/server.rs: 1018 lines (exceeds Rust limit of 1000)
  src/proxy.rs: 1064 lines (exceeds Rust limit of 1000)
```

Both files entered this branch under the limit (977 and 945 on `main`) and were
pushed over it by the fixes above. Each was split along the seam it already
had, so no behaviour moved:

| new module | what moved | before → after |
| --- | --- | --- |
| `src/proxy/summary.rs` | response summarisation for `proxy.jsonl` — four vendor shapes, whole-body and SSE — leaving `proxy.rs` the transport | 1064 → 669 (+409) |
| `src/server/http_io.rs` | the blocking HTTP/1.1 listener: read a head, read `content-length` bytes, write one `connection: close` response | 1018 → 859 (+171) |

`src/proxy/summary.rs` is a private module, so its items are `pub` rather than
`pub(crate)` — clippy's `redundant_pub_crate` rejects the latter there. The
self-AST census was regenerated for the new module layout
(`cargo run --example regenerate_self_ast_census`, 5 documents rewritten), which
`tests/unit/issue_673_self_ast_census.rs` requires.

Every other job on `4f343d9c` was already green: the Agentic CLI Matrix run
`29796997177` passed all 14 legs including `E2E (codex)`, `E2E (gemini)` and
`Matrix summary`, and CI/CD run `29796997192` passed
`E2E Tests (agent CLI ↔ formal-ai)`, `Self-Hosting Evidence Check`,
`Docker Image Build & Runtime Check`, `Test (ubuntu-latest)` and
`Code Coverage`. `Lint and Format Check` was the only red job.

## 10. The hardcoded-language gate on `5ee22e04`

With the file-size gate satisfied, `Lint and Format Check` reached the *next*
step it had never run on this branch and failed there
(`ci-logs/lint-and-format-88536249710.log:2688`):

```
New hardcoded user-facing strings found in src/ (not in the allowlist):
  src/agentic_coding/file_read.rs: "Contents of `{path}`:\n\n{body}"
  src/agentic_coding/file_read.rs: "First line of `{path}`: {}"
```

Both literals are this branch's own: they are the fence-free renderer added for
the tool-free `aider` leg. R379 ("data is the interface",
`docs/design/no-hardcoded-natural-language.md`) blocks new prose in `src/`, and
the allowlist is a burn-down inventory that may only shrink — so the two
sentences were migrated instead of allowlisted. They now live in
`data/seed/multilingual-responses.lino` as the `supplied_file_contents` and
`supplied_file_first_line` intents in all four supported languages, read back
through `seed::response_for` with the request's own language and an English
fallback, and their new value tokens were grounded with
`python3 scripts/close-total.py` (`unresolved_distinct: 0`).

`the_headings_are_seeded_in_every_supported_language` in
`tests/integration/issue_671_supplied_file_bytes.rs` pins the migration: each
intent must exist for `en`/`ru`/`hi`/`zh`, keep both placeholders, carry no
fence (a fenced answer is an edit instruction to a whole-format client), and be
the heading the served answer actually renders.

Everything else on `5ee22e04` was green — the Agentic CLI Matrix run
`29798969655` passed all 14 jobs, and CI/CD run `29798969664` passed every job
but this one.

## 11. Green, and one runner-side flake

On `4e86fb91` the Agentic CLI Matrix run `29800923688` is green in all 14 jobs
and CI/CD run `29800923586` is green in every job except
`Docker Image Build & Runtime Check`, whose failure is the runner's disk, not
this branch (`ci-logs/docker-image-88541902866.log:2111`):

```
#21 ERROR: write /blobs/sha256/f2e6b6…: no space left on device
ERROR: failed to build: failed to solve: rpc error: code = Unknown desc = io: read/write on closed pipe
```

The same job passed on the two previous heads of this branch with the same
`Dockerfile`, so it was re-run rather than debugged — and the re-run
(`88544585614`) built and passed the runtime contract, taking run `29800923586`
to `success` with every job green.

The last local gate this branch had never exercised is also covered now:
`npm run --prefix tests/e2e check:language-test-coverage` requires a
language-facing seed change to be covered by tests naming every supported
language, so `the_headings_are_seeded_in_every_supported_language` asserts each
record through its own wording (`Содержимое`, `की सामग्री`, `的内容`) rather
than looping over language codes — which also fails if a translation is a copy
of the English record.
