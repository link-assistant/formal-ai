# Changelog archive: 0.242.0 to 0.306.1

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

## [0.306.1] - 2026-07-26

### Added

- A standalone associative technology stack guide that links each component
  and explains its runtime, architecture, or development role (#874).

## [0.306.0] - 2026-07-26

### Fixed
- The planner could not recognise the result of its own tool call once the client's schema renamed the argument (issue #671). Codex's `exec_command` takes `cmd` where the planner plans `command`, and Gemini's `read_file` takes an absolutised `absolute_path` where the planner plans `path`, so `formal-ai with --non-interactive codex "read the file alpha.txt"` re-planned the identical call 281 times and never terminated. Recorded tool results are now matched through the same alias set `protocol_responses` projects with, and a shell result is stripped of its transport envelope (Codex's `Chunk ID` / `Wall time` / `Process exited with code` preamble) before it is quoted back. The same request now converges in two model rounds. A hand-written `curl` never reproduced this, because it uses the canonical key.
- A request to *read* a file could plan a *write* that destroyed it (issue #671). `show me the contents of the file beta.md` planned `write(beta.md, "of the")`, because the marker-led branch of the write parser accepted any content-lead surface — "contents" is one — without also requiring a write verb. A marker that precedes the file clause now needs the same write cue the destination-led branch already demanded, and a recovered payload with no alphanumeric character at all (the `opencode` leg recovered a lone `"`) is rejected rather than written.
- Tool arguments naming a file are absolutised when, and only when, the client's own schema says they must be (issue #671). The `agent` leg answered `Error: File not found: /alpha.txt` and the `qwen` leg `File path must be absolute, but was relative: alpha.txt`, because the projection absolutised only Gemini's literally-named `absolute_path` property. The requirement is advertised — in the property name, the property description or the tool description — so it is read rather than hardcoded per client, and a client that accepts relative paths keeps the request's own spelling.
- An absolutised path is resolved against the *client's* working directory, not the server's (issue #671). The two share one whenever the CLI is launched from the same place, which is why the matrix never saw it; the issue-#715 Agent CLI E2E starts `formal-ai serve` in the repository and each CLI in a fresh temporary workspace, and the report the task derives landed in the repository root while the CLI looked for it in its own directory. Every client declares where it runs — `agent` and `opencode` as `<env>  Working directory: …`, `codex` as `<environment_context><cwd>…</cwd>`, `gemini` as a `**Workspace Directories:**` list — so the declaration is read from the request, with the server's own directory kept as the fallback and a directory that is not on this machine ignored.
- `formal-ai serve` answers `HEAD` on `/`, `/health` and the protocol roots. Claude Code probes reachability with `HEAD` and reported the server as unavailable when it 405'd.
- The Gemini protocol path accepts the utility model ids the vendor CLI hardcodes instead of rejecting them with a 400, and `formal-ai proxy` recovers the model id from the request path when the body carries none, so a Gemini-shaped exchange is no longer logged with a null `request_model`.

### Changed
- The tool-free reader's two answer headings are grounded meanings rather than English typed into the engine (R379): `data/seed/multilingual-responses.lino` carries `supplied_file_contents` and `supplied_file_first_line` for all four supported languages, and `src/agentic_coding/file_read.rs` reads them back through `seed::response_for` in the request's own language.
- The Gemini headless-`-p`-advertises-no-`functionDeclarations` constraint (#620) was lifted upstream in `@google/gemini-cli@0.51.0`, and the matrix is what found out: the `constraints` assertion failed loudly, exactly as issue #671 asked. It is inverted rather than deleted — the leg now fails if `read_file` stops being advertised — and the `read-file` case proves a real headless tool call round-trips.

### Added
- A multi-CLI agentic end-to-end matrix (issue #671): `.github/workflows/agentic-cli-matrix.yml` runs one leg per client `formal-ai clients` knows about — codex, t3code, opencode, opencode-vscode, opencode-desktop, agent, cursor, gemini, claude, qwen, grok, aider — driving the **real** CLI, headless and through a real PTY, against a local `formal-ai serve --agent-mode` with `formal-ai proxy` recording every exchange. Our server is the model provider, so no leg needs vendor credentials, and every recorded `proxy.jsonl` is uploaded — including on green legs, so `claude`, `grok` and `aider` finally have replayable sessions.
- `experiments/agentic_cli_matrix/` holds the harness: `clients.lock` pins every client's version, `install_client.sh` installs one, `lib.sh` manages the server/proxy stack and the assertions, and `run_leg.sh` runs the case sequence. Per-leg shape is read from the seed registry rather than hardcoded, and a case that exceeds its model-round budget fails as a tool-call loop instead of as a slow leg.
- Cases cover the issue-#650 defect surface (`/responses` instructions, empty interactive messages, summarization requests, the `--globally` alias), the issue-#713 interactive-only launch blockers, and issue #746's hosted `web_search` advertisement. Documented upstream limitations — Gemini's headless `-p` advertising no `functionDeclarations` (#620), the missing headless approval handshake in codex/gemini/qwen (#511) — are assertions that fail when upstream lifts them, never skips.
- Every log assertion in the harness reads the client's output through `matrix_log_matches` instead of `matrix_strip_ansi | grep -q`. Under `set -o pipefail` the pipe form reports 141 — a failure — whenever `grep -q` exits early enough to SIGPIPE `sed`, so it announced a *present* marker as missing on long logs (the `agent` and `codex` TUI legs), silently disarmed the negative checks such as the client-sandbox diagnosis, and made every `await:` step spin for its full timeout rather than returning when the answer rendered. The interactive case now runs the same sandbox diagnosis the headless read does, so a host kernel that cannot start the client's own sandbox is named identically in both.
- `formal-ai clients [--format text|json]` prints the seed-baked registry of supported CLI clients and what each one can do, so the matrix and the `with` wrapper cannot drift apart.
- Every client is covered by the leg shape its integration actually has, so no client is skipped and none is handed assertions it can never satisfy: `cli` (prompt in, answer out), `server` (t3code serves a web UI), `gui` (opencode-vscode, opencode-desktop, under Xvfb) and `mcp` (cursor, where we are the *tool server* and the client's own model drives). The `mcp` leg exercises `/mcp` directly — `initialize`, `tools/list`, a `tools/call` that must reach the real solver, and an unknown tool name that must be refused with JSON-RPC `-32601` — and asserts Cursor's own credential requirement as an upstream constraint. The launch legs prove the wrapper's configuration reached the *running application* by reading the launched process tree, because a GUI issues no model request until a human types; the earlier "something reached the proxy" check was satisfied by the harness's own `/health` probe and could never fail.
- `replay.sh` validates each committed transcript according to its shape: a `cli` transcript must carry bounded model rounds naming `formal-ai` (in the body or, for Gemini, in the request path), an `mcp` transcript must carry the three `/mcp` round trips and must not name our model, and a `launch` transcript legitimately holds startup traffic only.
- `tests/unit/issue_671_matrix_coverage.rs` fails the build if a client is added to `data/seed/client-integrations.lino` without a pinned version, a CI leg and a documented row; `tests/unit/issue_671_planner_tool_alias.rs` covers the projected-argument regression directly.
- Client verification behavior now lives beside each integration in `data/seed/client-integrations.lino`. The workflow and local runner generate their leg plan from those contracts instead of repeating client identities or branching on names; seeded fields cover surface, file delivery, headless/interactive invocation, tool constraints, launch readiness, extensions, sandboxes and vendor-auth boundaries.
- Successful real-client sessions now feed `formal-ai clients observe` and `formal-ai clients learn`. Reusable behavior requires two independently worded observations, stable tools are inferred by intersection, and every proposed seed amendment carries transcript evidence and remains `awaiting_human_review`. The CI learner publishes that deterministic review artifact after all generated legs pass.
- A real Agent CLI reference run executes the learning command through Formal AI and writes the exact report. That run exposed a general planner bug that treated “its exact stdout” as literal file content; command-output requests now plan the explicit command, redirect its output to the safe relative target, and read the target back before completing.

## [0.305.1] - 2026-07-26

### Fixed

- Route multilingual process-list requests through explicit Agent permissions,
  use `tasklist` on Windows, and learn successful trusted research for otherwise
  unknown browser requests.

## [0.305.0] - 2026-07-26

### Added

- Add a meanings-driven grounded-action recipe with stateful local discovery, definition follow-up, comparison, and report journeys.

## [0.304.1] - 2026-07-25

### Fixed

- Honor excluded CI paths on direct pushes instead of unconditionally running
  tests, coverage, and end-to-end jobs.

## [0.304.0] - 2026-07-25

### Added
- `formal-ai report body` renders the complete issue-report document — the same
  six sections the web reporter emits — from an exported conversation, with the
  full Links Notation context attached inline or as a gist (#839). One shared
  builder (`src/issue_report.rs`, mirrored for the browser by
  `src/web/app/issue-report.js` and kept honest by a parity test) now formats
  reports for the web, CLI, desktop, Telegram, and VS Code surfaces.
- `formal-ai context session` prints the harness session identifier the current
  shell is inside, so a report can export the session the user is actually in.
- New guide [`docs/report-issue.md`](../report-issue.md) documenting the
  report document, the CLI flags, the source semantics, and the gist-visibility
  choice.

### Fixed
- `report issue` now exports the real harness session instead of a hash of the
  conversation's first message. The HTTP server records the
  `x-formal-ai-dialog-id` header of every request, so the exported session is the
  caller's own and two conversations that open with the same sentence no longer
  collide (#839, #838).
- A named `--source` that cannot be exported now fails loudly instead of
  silently degrading to a different capture. Issue #838 was filed with 271 KB of
  base64 HTTP proxy traffic in place of the conversation while the run reported
  success; the server's conversation record is now a separate artifact from the
  proxy trace.
- Oversize contexts are trimmed by whole Links Notation records with an explicit
  `... omitted N records ...` marker instead of `tail -c 12000`, which cut
  mid-record by construction. The complete context is attached as a gist
  (`secret` by default, an explicit documented choice).
- The generated report script verifies every program it calls with `command -v`
  before it runs, and its scratch template expands to a real filename — #838's
  gist was named `formal-ai-report.XXXXXX.lino`.
- Report titles quote what the conversation was about rather than defaulting to
  `Formal AI agentic session report`: the trailing report request is dropped,
  and the first and last remaining user turns are quoted when they fit.

## [0.303.0] - 2026-07-24

### Added

- Added a conservative legal/compliance policy, explicit contribution-rights
  rules, and a fail-closed training/distillation source registry with
  per-source provenance, privacy, terms, attribution, naming, and approval
  requirements.
- Added exact-version guidance for Llama 3.3 and Mistral 7B, OpenRouter/provider
  term checks, personal-data and prohibited-use controls, an EU AI Act
  assessment, disclaimer limits, and an issue #834 evidence-backed case study.

## [0.302.3] - 2026-07-24

### Fixed

- Export agentic report context locally, report GitHub command failures
  truthfully, and reject stale Formal AI servers before launching a client.

## [0.302.2] - 2026-07-23

### Fixed
- Agentic step narration now explains each action in natural language instead of
  echoing the shell command that OpenCode already prints, and drops the robotic
  "so I can verify the next step before continuing" tail (#819). Local-path finds,
  web searches, and report prompts are worded distinctly across all supported
  languages (en, ru, hi, zh), with the empty-result case explained in beginner
  terms.

## [0.302.1] - 2026-07-23

### Fixed

- Explain empty local file and folder searches in beginner-friendly language,
  including client placeholders such as `(no output)`.
- Let report confirmations select and execute multiple destinations in one
  interaction.
- Keep temporary Formal AI server diagnostics out of wrapped fullscreen TUIs
  while retaining them in a durable session log.

## [0.302.0] - 2026-07-22

### Added
- Add confirmation-gated full-context agentic reports, a conversation-context API and CLI, deterministic JSON/OpenCode-to-Links-Notation adapters, and verbose-by-default diagnostics with an explicit `--silent` opt-out.

## [0.301.1] - 2026-07-22

### Fixed

- Route local Desktop, home, and workspace path discovery through a bounded
  client-side `find` command instead of web search, including multilingual
  action/scope/kind phrases and fuzzy remembered names.

## [0.301.0] - 2026-07-20

### Added
- Budget-driven random and evolutionary search in solution synthesis (issue #662, F4). When reuse and rule reasoning produce no candidate, step 7 now samples and evolves compositions of the known numbers against the step-6 generated tests until it reaches the target or exhausts the compute budget.
- `compute_budget` knob on `SolverConfig`, wired through the `FORMAL_AI_COMPUTE_BUDGET` environment variable and the `--compute-budget` CLI flag, counting candidate evaluations.
- Atomic `search:` trace events recording each generation — `search:problem:{target,numbers,ops}`, `search:budget`, `search:test:{each_number_once,only_operators,evaluates_to}`, `search:random:{sampled,best_diff}`, `search:evolutionary:{generation,best_diff}`, `search:candidate:{phase,evaluations,expression}`, `search:solution`, `search:exhausted:{evaluations,budget,best_diff}`; on budget exhaustion the honest unknown-reasoning reply keeps the search evidence attached. Each event carries a single slug/value pair, so no user-facing prose is hardcoded in the trace (R379).
- A self-authored, search-only benchmark source (base + held-out variant) in the industry suite, raising `minimum_pass_count` from 10 to 12.
- `search:skill` proposal-only auto-learning event: a solved composition is recorded as a `candidate_skill` in status `proposed`, never promotable without review (`search:skill:promotable` is always `0`), mirroring the skill-accumulation ledger (R21/R340, C3/R13).
- Grounded meta-algorithm recipe `data/meta/budget-search-recipe.lino`, pinned by `tests/unit/specification/budget_search_meta_algorithm.rs` and documented in `docs/meta-algorithm.md`, so the nine-step stage always describes how the running code was produced.

### Changed
- Search recognition is now grounded entirely in the seed lexicon (issue #386): the operand framing, search cue, and target marker are read by semantic role from `data/seed/meanings-search.lino` instead of hardcoded per-language phrase tables, so the reach-a-target class spans en/ru/hi/zh (and any language whose surfaces are added to the seed) without touching Rust.
- The operator toolbox is derived from the seed `arithmetic_operator_word` vocabulary via `seed::Lexicon::arithmetic_operators`, generalising the search from `+ - *` to include division and modulo (with a division/modulo-by-zero guard) the moment the seed lists them.
- The budget-search reply is looked up from the seed knowledge base by the prompt's detected language (intent `budget_search_solution`, localized en/ru/hi/zh in `data/seed/multilingual-responses.lino`) instead of a hardcoded English string, so a Russian puzzle is answered in Russian (R379: data is the interface).

### Added
- Every formalized statement now carries an explicit, inspectable
  probability weight (issue #661, R384). For a multi-interpretation prompt
  (e.g. the copula-ambiguous "apple is a fruit", which splits into P31
  instance-of vs P279 subclass-of), the formalization step appends a
  `statement_weight` link per accepted interpretation, whose values are the
  softmax posteriors already used for temperature selection and sum to 1
  across candidates. The weights live in the trace (evidence links), never in
  the plain reply, so diagnostics stay default-off.
- Contradictory standing requirements are now detected and warned about
  (issue #661, R384). When a newly formalized directive conflicts with a
  retained one — same subject, opposite polarity, e.g. "always answer in
  Russian" then "never answer in Russian" — the solver appends a
  `requirement_contradiction` event and replies with a warning that names
  both statements, their weights, and a proposed resolution (retract one via a
  superseding requirement, split the meanings, or scope each to a different
  context). The resolution reuses the append-only retraction protocol
  (`policy:add_only_history`), and the warning is rendered in the prompt's
  language (en/ru/hi/zh). Polarity markers are recognised across all four
  languages, so widening coverage is a marker-table edit rather than new
  control flow. The check runs before the contextual handlers so a
  contradictory language directive is flagged instead of being silently
  replayed.
- A generalized `formal-ai statement-audit` command now snapshots repository
  prose, code comments, and structured facts; gives every recognized statement
  an evidence-adjusted probability and stable source location; verifies path
  claims against the Git index; and exports contradictions, issue candidates,
  provenance, and learned associations as deterministic Links Notation.
  Original-source captures contribute Relative Meta-Logic mass while preserved
  unoriginal reposts contribute zero mass. A committed case study and release
  test replay the fixture both directly and through the real
  `@link-assistant/agent` CLI.

### Added

- Canonical `GET /v1/network` endpoint (and `/api/formal-ai/v1/network`) for the
  links-network projection of the event log, returning the same nodes/edges,
  `?trace=` filtering, `?format=dot` export, and 404-on-unknown-trace behavior.
- `scripts/check-associative-terminology.rs` hygiene lint that blocks *new*
  graph-named public API routes and Rust modules/files, wired into the release
  workflow's Lint job. It allowlists the deprecated `/v1/graph` alias, the
  Wikidata `knowledge_graph` engine, and the codecov coverage badge / external
  graph-API citations.

### Changed

- The associative surface is now consistently described as a *links network*,
  not a "graph": renamed `src/self_source_graph.rs` → `src/self_source_links.rs`
  and `src/agentic_coding/source_graph.rs` → `source_links.rs`, swept the web,
  desktop, and VS Code UI strings to "links network view", and updated
  `ARCHITECTURE.md`, `README.md`, and `REQUIREMENTS.md` terminology (fixing the
  stale `src/graph.rs` reference in R81).

### Deprecated

- `GET /v1/graph` (and `/api/formal-ai/v1/graph`) is now a deprecated alias of
  `/v1/network`. It still returns the identical payload, but the response is
  flagged deprecated (a `deprecation: true` header plus a `link` header pointing
  at the `/v1/network` successor) so existing desktop / VS Code / CLI clients
  keep working while migrating.

### Changed
- Retired the `SPECIALIZED_HANDLERS` precedence remnant into data-driven routing: the dispatch ordering now lives in `data/seed/handler-precedence.lino` and is joined with the Rust function pointers at dispatch-build time, with a permutation assertion guarding against silent handler drops or duplicates.

### Added
- Handler-precedence auto-learning report: Formal AI re-derives the specialized-handler precedence itself through its own Agent CLI, ranking the persisted precedence rationale (`data/meta/issue-663-handler-precedence-learning.lino`) into a human-review-gated proposal whose committed evidence is byte-for-byte reproducible by the in-process renderer.

### Fixed
- General-change write routing no longer claims a request whose recovered payload is only a non-referential subject (a bare pronoun such as "it"/"this"): "save it to FILE" pointed back at content a keyword recipe must still compose, so the generic write probe now declines and lets that recipe author the real artifact instead of writing the literal word "it".

### Fixed
- Scoped read-file grounding to the current user turn so stale reads from earlier turns no longer influence write operations (issue #755)

### Added

- `data/meta/links-network-terminology-recipe.lino` — the grounded meta-algorithm
  recipe for issue #664. It records every ordered step, handler function, module
  rename, lint allowlist entry, CI wiring, and pinning test that produced the
  links-network terminology cleanup, together with a `generalization` note that
  turns "this public surface still says graph, make it a links network" into a
  reusable procedure.
- `tests/unit/specification/links_network_terminology_meta_algorithm.rs` — the
  grounding test that loads the recipe and asserts the live source still matches
  it (functions exist, renames landed and old names are gone, allowlist entries
  are present in the lint, the lint is wired into CI, and the pinning tests
  exist). CI now fails if the recipe and the code drift apart, so the cleanup is
  a reproducible artifact of the meta-algorithm rather than a one-off hand edit.
- A "links-network terminology meta-algorithm" section in `docs/meta-algorithm.md`
  documenting the recipe and how to run its grounding test.

### Added

- A per-message control that reveals a withheld answer immediately (issue #672,
  F3). The animation budget stays a global preference, but a single message can
  be settled without changing it for every future answer. Reduced-motion users
  are unaffected: they never see the control because there is nothing to skip.
- Reasoning-step hierarchy editing in Diagnostics mode (issue #672, F4).
  Right-clicking a step offers "Bump to high level" / "Demote to sub-step" /
  "Restore the original level" in all four locales. Edits are appended to an
  event log and the visible hierarchy is a projection of it, so the trace the
  solver reported is never rewritten — each step keeps the solver's own label on
  `data-solver-level` beside the user's `data-level-override`.
- A desktop notice for profile migrations, with a replay button (issue #672,
  F2). `dataMigrationStatus` and `replayDataMigration` carry the result of the
  startup migration to the renderer, which names the profile the data came from
  and what moved. Failures surface as errors rather than as claimed successes,
  and a clean install is never interrupted.

### Changed

- The desktop profile migration now also copies `Cookies`, `Service Worker`,
  `WebStorage`, and `WebSocketStorage` (`DATA_VERSION` 2, issue #672 F2), so a
  user whose data moved to the pinned profile no longer arrives logged out. An
  existing v1 profile is topped up with only the new subtrees. `Cache` and
  `Code Cache` are deliberately excluded — they are derived, large, and unsafe
  to carry between Chromium builds.

- The per-message UI strings moved out of `src/web/i18n-catalog.lino` into
  `src/web/i18n-catalog-messages.lino`, the same way the permission strings were
  split earlier, so each catalog stays under the Links Notation line limit that
  the F3/F4 keys pushed the single file past. The loader merges all three files
  per locale and the catalog, parity, and coverage guards all watch the new file.

### Fixed

- `desktop/scripts/*.test.mjs` was written but never executed by any workflow.
  The Lint job now runs it, so the profile-migration code has a gate.

### Tests

- `tests/e2e/tests/issue-672-theme-snapshots.spec.js` (issue #672, F1) snapshots
  the computed colours of the five widgets issue #541's R1 fixed, across
  light/dark/auto and both the web and desktop surfaces, and runs in CI. Three of
  those widgets previously had no automated theme coverage at all.
- `tests/e2e/tests/issue-541-permissions-cold-start.spec.js` (issue #672, F5)
  re-runs the #541 R9 grant-all journey with the desktop provider behind a real
  `page.exposeFunction` boundary, so the replayed task and the granted tools are
  asserted on the payloads that actually left the browser context.
- The F3 control and the F4 menu are asserted per supported language (en, ru,
  zh, hi) on the labels the browser renders, and each of those tests performs
  the edit, so a locale cannot be labelled correctly and broken behaviourally.

### Added
- Opt-in, default-off per-dialog JSONL request/response recording through `FORMAL_AI_DIALOG_LOG_DIR`, allowing exact agentic sessions to be reconstructed without enabling body logging globally (issues #781 and #800).
- A reusable native-client research harness covering Agent, OpenCode, Claude Code, and Codex against the same deterministic MCP evidence source.

### Fixed
- Narrate each web-research action before executing it, fetch sources in separate turns, and synthesize only successfully fetched evidence with its exact URL.
- Prefer executable open-world research tools over local grep or hosted-only tools, including MCP children advertised through Responses namespaces.
- Preserve MCP namespace identity for client dispatch and normalize recognized Codex tool-result envelopes only for planning while retaining their exact raw transport content.

### Fixed
- Stopped a malformed `Formal-AI-Evidence` record on an already-merged commit from permanently blocking every release; the pull-request gate stays strict while release recording now reports the commit and leaves it unattributed (issue #810)
- Made the macOS ad-hoc signing hook report its entry banner and ignore-predicate counters synchronously, so an aborted `electron-builder` run can no longer discard the diagnostics that identify why the bundled browser runtime was signed (issue #810)

### Fixed
- Added default-off debug output to the macOS ad-hoc signing script so failing desktop release runs can be diagnosed from CI logs (issue #804).

### Added
- A workspace-wide self-AST census (issue #673): `data/meta/self-ast/` now holds one census document per `src/` module plus an index, replacing the single pinned module as the algorithm's view of its own source. Modules under `src/agentic_coding/` are censused at full AST fidelity through the meta-language links network; the rest carry a signature-level census (items, symbols, spans). Every document records its fidelity marker.
- `formal_ai::self_ast_census` with `WorkspaceCensus::compile`/`resolve`, per-module `ModuleCensus`, and a pure `drift_report` so a stale, missing, or orphaned census document is detected without touching the filesystem.
- `cargo run --example regenerate_self_ast_census` regenerates the census incrementally — only the documents whose modules changed are rewritten — and `cargo run --example dump_self_ast_census [reference]` prints the index or a resolved module.

### Changed
- The general planner resolves edit targets through the census index (`resolve_census_target`), so a request naming a bare module file (`method_registry.rs`) is planned against its real workspace path instead of a hardcoded one.

### Fixed
- Replaced the contradictory `always() && !cancelled()` job guards in the CI/CD pipeline with `!cancelled()`, so cancelling a run actually stops the dependent jobs (issue #808).
- Excluded the bundled Chrome for Testing runtime from macOS code signing via `electron-builder`'s `mac.signIgnore`, so ad-hoc desktop builds no longer fail with "unsealed contents present in the root directory of an embedded framework" (issue #808).
- Routed the macOS ad-hoc signing diagnostics to stderr and added an unconditional hook-entry line, so the sign trace actually reaches CI logs; the per-file trace stays default-off behind `FORMAL_AI_MACOS_SIGN_DEBUG` (issue #808).

- Set `CSC_FOR_PULL_REQUEST` for ad-hoc macOS packaging, so `electron-builder` no longer skips code signing on pull-request runs and the packaged app keeps its `CodeResources` envelope (issue #808).

### Added
- Desktop and VS Code extension packaging now run on pull requests that touch `desktop/`, `vscode/` or the release workflow, exercising the full six-target build matrix, macOS signing and the artifact smoke tests in dry-run mode with every publishing step skipped (issue #808).
- Pull-request job building the Docker image and running `scripts/verify-docker-runtime.sh` inside it, so a broken `Dockerfile` can no longer surface only after the crate has been published (issue #808).
- Pull-request job validating `Formal-AI-Session` / `Formal-AI-Evidence` commit trailers with the same measurement the release uses, so a malformed trailer is rejected before it can fail Auto Release on `main` (issue #808).
- Pull-request fresh-merge simulation (`scripts/simulate-fresh-merge.sh`, adopted from the pipeline templates), so a pull request cannot pass against a stale merge preview and then break `main` (issue #808).
- Secrets scan on pull requests (`scripts/check-secrets.sh`, secretlint recommended preset) covering the files a change touches (issue #808).
- Resilient Buildx setup composite action retrying the BuildKit image pull with a registry-mirror fallback, replacing the raw `docker/setup-buildx-action` uses (issue #808).

### Fixed
- Stopped the self-hosting ratchet from deadlocking every release: the metric no longer counts captured CI transcripts (`*.log`, `*.jsonl`, `*.diff`, `*.patch`, `*.stderr`, `*.stdout`) or dependency lockfiles, which made up 94.20% of the measured range and turned "commit your evidence" into a guaranteed regression, and enforcement moved from `record_release` on `main` — where every contributing commit is already immutable — to a differential pull-request gate where the commits can still be amended (issue #812).
- `cargo clippy` now runs with `-D warnings`. Every lint in `[lints.clippy]` is set to `warn`, so the job printed findings and still exited 0 (issue #812).
- `auto-release` and `manual-release` now gate on `Secrets Scan` and both E2E suites, not just `[lint, test, build]`; a red secrets scan on `main` could previously publish the crate, the Docker image and the GitHub Release anyway (issue #812).
- The desktop `finalize` job runs under `!cancelled()` instead of `always()`, so a cancelled run can no longer clobber a complete `SHA256SUMS.txt` with a partial one via `gh release upload --clobber` (issue #812).
- The desktop packaging and VS Code jobs now simulate the fresh merge on pull requests, the same way `lint` and `test` already did, so packaging is validated against the merge result rather than a stale merge preview (issue #812).
- `scripts/check-file-size.rs` excluded `.github/workflows/**` by accident: its `.git` exclusion was matched as a substring. Directory exclusions are now matched per path component, GitHub Actions workflows are measured (warn at 1500 lines, fail at 2000), and quoted third-party workflows under `docs/case-studies/**` stay exempt (issue #812).
- `node --test $(ls … | grep -v …)` in the desktop library test step fell back to Node's own test discovery if the glob ever stopped matching — a green step running none of the intended tests. The file list is now built explicitly and an empty list fails (issue #812).
- Pinned `secretlint` to an explicit version in `scripts/check-secrets.sh`; `npx --yes -p secretlint` resolved to whatever `latest` was when the job ran (issue #812).
- Quoted `$BASE_REF` at every expansion in `scripts/simulate-fresh-merge.sh`, which upstream leaves bare on two of three lines (issue #812).

### Added
- `actionlint` (with `shellcheck`) now lints every workflow definition, and `shellcheck` lints the shipped `*.sh` scripts. Nothing validated either before, so a mistyped `needs.<job>` reference or a malformed expression — which fails *open*, silently skipping the guarded step under a green check — could only be found by pushing (issue #812).
- `METRIC_VERSION` on self-hosting ledger rows, so a change to how the share is measured starts a new comparison epoch instead of silently invalidating recorded history; rows from different epochs are never compared (issue #812).

### Fixed (second pass, found by running the fixed pipeline)
- The new release gates compared `needs.<job>.result != 'failure'`, which a job killed by its own `timeout-minutes` would pass: GitHub reports a timeout as **cancelled**, not failed. The gates now enumerate the acceptable results (`success` or `skipped`), so a timed-out secrets scan or E2E suite cannot release (issue #812).
- Raised the `test` job budget from 15 to 25 minutes. Run 29767811026 reported the job as failed with all 1953 tests passing — the suite finished 1.1 s before the cap killed the job during teardown, and run 29749095334 on `main` had already done the same. The step now emits a `::warning` once the suite passes 70% of the budget, so the margin cannot be eaten again silently (issue #812).
- Raised the `coverage` (15 → 25) and `lint` (10 → 15) budgets for the same reason: measured against the last eight `main` runs, `coverage` peaked at 14.1 minutes (94% of its cap) and `lint` had grown from ~3.3 to 7.8 minutes as checks accumulated. `test-e2e-local` (63%) and `docker-build` (23%) have real headroom and were left alone (issue #812).

## [0.300.0] - 2026-07-19

### Added
- Accept normalized `web-capture shared-dialog` JSON in the shared-dialog converter, including structured provider failure diagnostics.

### Changed
- Ground agentic web research in up to three independently fetched sources and preserve a citation for each captured result.

### Added

- Constraint-satisfying option networks (`option_network`): research can now record what an answer must supply, what each discovered candidate supplies, and every *minimal* set of candidates that jointly satisfies the requirement — including options made of two separate items, such as a conversion adapter plus the part it adapts. Options are listed cheapest first, with a provenance ladder (authentic, official-compatible, generic-compatible) breaking ties. The network projects onto `world_model::Context`, so the still-open part of a question is an ordinary `ContextDiff`.

### Changed

- Web research now deepens across rounds instead of stopping after one search and fetch. Each round searches only for the aspects of the question no fetched page supports, skips sources already read, and stops when nothing is left open, when a refinement would repeat the previous search, or when the round budget is spent.
- Evidence reading (`option_evidence`): candidates and their prices are now read straight out of fetched page text. The constraints supply the units to look for, so no attribute name is ever matched against prose and the same code reads a Russian spec sheet and an Indian listing. An attribute the page does not state is left open rather than guessed.

### Fixed

- The research loop no longer spends an extra round on questions it has already answered. Deepening now requires a single identifiable gap in a several-aspect question, because token coverage varies with the language a source is written in and a looser rule re-searched answered questions in Hindi and Russian.

- Fixed macOS desktop ad-hoc signing so the bundled Playwright browser keeps its valid framework seal on both architectures.
- Made direct WASM and all desktop Rust builds reject warnings, with platform-specific code and intentional partial modules scoped correctly.
- Bundled VS Code extension runtime dependencies instead of shipping thousands of `node_modules` files.

## [0.299.0] - 2026-07-19

### Added
- `formal-ai import lexemes` — a deterministic bulk semantics importer that
  generalises the one-off `scripts/ground-meanings.rs` into a reusable pipeline
  (issue #660, R378). It reads a `concepts` document of `<slug> <Qid>` pairs,
  pulls each concept's four project-language labels (en/ru/hi/zh) from the
  committed Wikidata entity cache, and emits grounded meaning blocks whose
  surfaces denote their meaning and carry `part_of_speech`/`grammatical_number`
  facets. `--offline` (the default) reads only the committed cache, so a run
  reproduces the committed batch byte-for-byte; live population is gated behind
  `FORMAL_AI_LIVE_API` and honours the bounded-cache policy `min(1%, 512)`.
- The importer validates on import: every generated block is parsed back through
  the real seed loader and must parse, denote its meaning, and carry both facets;
  a concept that fails is refused, persisted as a replayable `import_rejected`
  event, and leaves the previous shard set unchanged. Each accepted surface
  carries its exact Q-record JSON field, language scripts are checked with a
  deterministic same-record alias fallback, obsolete importer-owned shards are
  removed, and the CLI reports requested/accepted and expected/emitted coverage.
- A bulk batch of common concrete nouns grounded from Wikidata, growing the seed
  by 208 grounded meanings and 832 surfaces across en/ru/hi/zh, each backed by a
  committed `data/cache/wikidata/entity/<Qid>.{json,lino}` record.
- A generalized, human-review-gated learning report derives reusable importer
  amendments from persisted observations. CI executes the task through two real
  Agent CLI clients with Formal AI as their model provider and requires the
  resulting reports to be byte-identical.

Added `formal-ai with t3code` (alias `t3`) with isolated Codex configuration,
OpenAI/Anthropic protocol selection, persistent setup/undo, and T3 Code setup docs.

### Fixed

- Normalize client tool-result envelopes into localized, format-aware answers while retaining raw outputs for follow-up questions and durable memory.

### Fixed
- Count each Unicode scalar as one token across OpenAI, Anthropic, and Gemini usage metadata, sum all visible input message content, and return real response timestamps without fake cache or cost fields.

### Added
- Advertise context capacity from free disk space and on-disk shared-memory usage across OpenAI, Codex, Gemini, Vertex, and Anthropic client metadata, with a configurable average UTF-8 byte width.

### Fixed
- Route Grok Build through the local OpenAI-compatible endpoint with a temporary API key, and persist its native user settings only when requested globally.

### Added
- Support `formal-ai with cursor` in interactive and headless modes by launching `cursor-agent` with isolated or global Cursor MCP configuration backed by the authenticated local `/mcp` endpoint.

### Added
- Print the concrete session artifact created by each `formal-ai with` run,
  together with a resume command when supported and the configured proxy log;
  preserve temporary client homes when they contain the reported artifact.

### Added

- Use one secure, persistent per-user `memory.lino` by default across the CLI,
  server, dreaming worker, desktop, VS Code desktop host, Telegram, API, and
  Agent CLI containers, with `FORMAL_AI_MEMORY_PATH` as the explicit override.

### Fixed

- Route shared agentic CLI tools through a seed-backed capability registry, prefer specialized local tools over shell fallbacks, and project arguments recursively onto each advertised JSON schema.

### Added

- Add a persistent desktop engine selector that defaults to an installed Agent
  CLI, offers only detected Agent/Codex/Claude passthroughs, and keeps the native
  out-of-box engine available.
- Stream agent-commander JavaScript API events into the shared desktop chat UI
  while routing every engine through the local Formal AI server and memory.

### Fixed

- Recognize seed-defined source-first translation commands such as `<source> - translate to <target>` in both the native solver and browser worker.
- Route the reported formal-system proposition through one language-neutral meaning with English, Russian, Hindi, and Chinese renderings and round-trip coverage.
- Prevent long Unicode translation terms from panicking when cache filenames are truncated.

### Added

- Add a discoverable end-to-end configuration guide for agent clients, modes,
  tools, shared memory, APIs, session debugging, languages, and every user
  surface, synchronized with the client integration seed registry.

### Added
- Add an isolated and persistent `formal-ai with opencode-vscode` target for the official `sst-dev.opencode` extension.

### Added

- Added a distinct `formal-ai with opencode-desktop` integration with isolated
  one-shot configuration, platform-aware executable discovery, persistent
  setup/undo, and `--all` support.

### Fixed

- Agentic web research now answers with the sentences of a fetched page that bear on the question, ranked by symbolic token overlap and cited with the source URL, instead of returning the whole scraped page verbatim (#771).
- Sentence splitting now ends a sentence at the Devanagari danda `।` and double danda `॥`. Hindi prose does not use a full stop, so without this a Hindi page was a single statement and anything that ranks or trims sentences — the web-research extract above among them — degraded to returning the whole document (#771).
- The web-research extract ranks Chinese pages by shared characters when word-boundary tokenization finds no overlap. Chinese writes without spaces, so bag-of-words cosine scored every sentence 0.0 and the extract fell back to the head of the page instead of the part that answered the question (#771).
- Issue reports filed from an agentic session render each turn as a bold role label followed by a blockquote, so a multi-line turn can no longer escape its list item and leak its own headings and lists as top-level issue content. The transcript is also bounded per turn and overall, keeping the body inside GitHub's issue size limit (#771).

### Fixed
- `scripts/install-node-dependencies.sh` matched reviewed npm deprecation
  warnings by exact `name@version`. Transitive versions float without any change
  on our side, so when `archiver-utils` resolved `glob` from `7.2.3` to
  `10.5.0` the warning stopped matching, was treated as an unexpected
  diagnostic, and failed the `.vsix` packaging job plus every desktop build even
  though `npm install` itself succeeded (issue #796). Reviewed deprecations are
  now matched by package name, so a version float can no longer break CI, and
  each one carries an accurate upstream tracking URL — `glob` reaches both
  workspaces through `@link-assistant/web-capture -> archiver -> archiver-utils`,
  not through vsce or electron-builder as previously implied. Unreviewed
  diagnostics still fail the build.
- `scripts/self-hosting-metric.rs` read commit trailers through git's
  `%(trailers:key=...)` placeholder, which only parses the last paragraph of a
  message. A release commit that separated `Formal-AI-Session` and
  `Formal-AI-Evidence` with a blank line therefore reported only the evidence
  trailer, and the resulting "must record both" error failed the whole Auto
  Release job after the version had already been computed. Trailers are now read
  from the full commit body, so their placement no longer decides whether a
  compliant commit is recognised.

### Added
- `INSTALL_NODE_DEPENDENCIES_VERBOSE=1` traces how each npm stderr line is
  classified, so an unexpected diagnostic can be diagnosed from CI logs without
  a local reproduction. Off by default.

## [0.298.1] - 2026-07-18

### Fixed
- Route explicit shell commands verbatim, honor strict client schemas, and cover multilingual file, VCS, build, and local-search intents.

## [0.298.0] - 2026-07-18

### Added
- `scripts/check-hardcoded-language.rs` (rust-script) gate that scans `src/` for user-facing prose string literals and fails the build on any literal missing from the committed allowlist, or on an allowlist row whose literal no longer occurs (issue #659, R379)
- `scripts/hardcoded-language-allowlist.txt`: sorted, tab-separated inventory of today's hardcoded natural-language debt, one `<path>\t<text>` row per literal, regenerated with `--write`
- "Check hardcoded natural language" step in the `release.yml` lint job and a matching local-checks entry in `CONTRIBUTING.md`

### Changed
- Migrated the duplicated English fallback answers in `src/engine_responses.rs` into the grounded `data/seed/multilingual-responses.lino` records, now read via `seed::response_for`, proving the R379 burn-down loop (allowlist shrinks as prose moves into meanings)

### Fixed

- Route OpenAI hosted tools, Anthropic server tools, Gemini function declarations, and Google-hosted tools through the shared capability router, and serialize hosted OpenAI web searches as native `web_search_call` output instead of an unsupported client-side function call.

## [0.297.5] - 2026-07-18

### Fixed
- Restored the `wasm32-unknown-unknown` build of the `no_std` WASM worker by
  splitting the forced-language seam in `src/language.rs` into cfg-gated
  backends — a `thread_local!` cell on native builds and a single-threaded
  `static` cell on `wasm32` — while preserving the `FORCED_LANGUAGE` seam
  behaviour across every supported language (`en`, `ru`, `hi`, `zh`).

### Added
- Two CI enforcement guards for the WASM-worker migration (issue #658, R380):
  `scripts/check-worker-line-budget.rs` ratchets the combined
  `src/web/worker/*.js` line count down toward the 3,000-line UI-glue target so
  the JS mirror can never silently regrow, and `scripts/check-wasm-worker-size.rs`
  keeps the shipped `.wasm` under its size budget. The lint job now rebuilds the
  WASM worker and runs both guards so `no_std` regressions cannot slip through.

### Fixed
- Configure Codex with a generated Formal AI model catalog so sessions recognize the model metadata without warnings.

## [0.297.4] - 2026-07-18

### Fixed
- Route multilingual file reads, URL fetches, web searches, writes, directory listings, and code searches by semantic intent and typed object instead of narrow English phrasing.

## [0.297.3] - 2026-07-17

### Added
- Route permission-free desktop and VS Code web search through headless Playwright across Google, Bing, and DuckDuckGo with reciprocal-rank fusion, render JavaScript-heavy fetches through web-capture, and expose specialized common file/agent tools with write and shell effects still permission-gated.

## [0.297.2] - 2026-07-17

### Fixed
- Project agentic tool calls onto each client tool's advertised JSON Schema, including required qwen fetch, search, shell, and absolute-path fields, and exclude qwen startup reminders from user intent routing.

## [0.297.1] - 2026-07-17

### Fixed
- Restored docs.rs generation by excluding the broken upstream Lindera build script from the documentation-only dependency profile, while retaining the complete meta-language runtime in normal builds.
- Added fail-closed CI coverage for both the docs.rs profile and the existing `/docs/api` GitHub Pages publication path.

## [0.297.0] - 2026-07-17

### Added

- Measure the Formal AI-authored share of every release from committed session
  evidence, publish it in release notes, and preserve a monotonic trailing
  metric in `data/meta/self-hosting-ledger.lino`.

### Added

- Derive a review-gated auto-learning report for the self-hosting metric,
  ranking the attribution observations behind the metric contract as an
  associative links network. Two external Agent CLIs execute the task against
  `formal-ai serve` as their own model provider — Formal AI running issue #657's
  task using Formal AI, with no external model — and must derive a byte-identical
  report.

### Changed

- Generalize the four auto-learning modules into one `LearningReport` descriptor
  table. Identity now lives in the descriptor and derivation in a single
  renderer, so a new report is a row rather than a copied module; the planner
  routes through the table instead of a branch per report.

### Fixed

- Stop rendering every learning report under issue #686's identity and patching
  it back out per-report. The patch failed silently when it matched nothing, so
  a report could claim the wrong issue while ranking a different network.

## [0.296.4] - 2026-07-17

### Fixed
- Regenerate the fragment-to-release map inside every release commit so later pull requests no longer inherit a stale reconstruction check.

## [0.296.3] - 2026-07-17

### Fixed
- Validate fail-closed Rust API documentation before release, remove “demo” branding from production workflow jobs, and suppress advisory file-size noise for files that do not grow.

## [0.296.2] - 2026-07-17

- Make agentic code generation and contextual follow-up changes use real client workspace tools across every catalog language. Follow-ups now execute auditable, bounded normal-algorithm programs with ordered/leftmost/restart/terminal semantics, empty-string creation and deletion, multi-rule and arbitrary-path support, no partial write on exhaustion, structural multilingual literal slots, and review-gated associative learning verified through built-in and OpenCode Agent CLI replays.
- Add `links_substitution_query`: the [link-cli](https://github.com/link-foundation/link-cli) substitution query language, as the meta-language representation between a natural-language request and a harness's read/write tools. `(matching pattern) (substitution pattern)` carries link-cli's CRUD-by-substitution shorthands — `() (("new"))` creates, `(("old")) ()` deletes, `(("old")) (("new"))` updates, `(("x")) (("x"))` reads — lowers to a bounded, Turing-complete normal algorithm, round-trips through a canonical renderer, and is published in every mutation trace alongside each rule's CRUD effect. Queries are also accepted directly as requests and lower identically on every harness vocabulary.
- Substitute over links as well as text sequences. The substitution model is the operand-independent part, so the query language is written once and its operands are read two ways: quoted character sequences, or `(source target)`/`(index: source target)` doublets with link-cli's `$i`/`$s`/`$t` variables binding across slots. `parse_link_substitution_query` reads link-cli's documented queries verbatim — `() ((1 1))` creates, `((1 1)) ()` deletes, `((1: 1 1)) ((1: 1 2))` updates, `(($i: $s $t)) (($i: $s $t))` reads all links without modifying them — and executes them under the same ordered, restart-at-rule-zero, step-bounded Markov control model, which is what carries Turing completeness across to the associative store.
- Fix the published mutation trace not being readable as the Links Notation it is written in, so a trace for a change to real code — anything carrying both a quote and a paren, such as `println!("Hello, world!");` — now parses instead of failing on an unclosed group. Links Notation escapes a quote by doubling it and picks a delimiter the value does not already contain, so a value is now carried as `pattern 'println!("Hello, world!");'` rather than backslash-escaped. The trace renderer delegates to the codec's own escaper instead of keeping a private copy of the rules, and `tests/issue_715_notation.rs` holds it there by parsing every published trace with the same codec the library encodes with.
- Fix text-manipulation and document requests losing their operands after an ASCII apostrophe, so prompts such as `It doesn't matter, replace "cat" with "dog" in this text: "cat naps"` no longer fall through to the unknown intent. Both handlers kept private copies of the literal-slot reader that predated the general one in `normal_markov`; the copies are deleted and `quoted_segments`/`quoted_segment_spans` is now the single implementation behind every caller.
- Answer link substitution queries from `formal-ai memory query`, so the link half of the meta language is reachable from the surface that owns the links. `parse_link_substitution_query` and `matched_links` were public API with no caller in the product, which left link-cli's own syntax unable to reach link-cli's own operand domain; a turn that *is* a query now routes to it ahead of the natural-language recognisers, and `(($i: $s $t)) (($i: $s $t))` reads every link of the memory projection back in the notation it was asked in. Link-level writes are refused with a message naming the reason rather than silently doing nothing: the doublet view is a one-way projection of memory events, so an edited link has no inverse back to the event it came from. Prose that merely opens with a parenthesis still falls through to natural language.
- Read Links Notation's own quote escaping in the seed parser, fixing a live corpus corruption that predated this issue. The notation escapes a delimiter by *doubling* it, and `strip_comment` already read it that way, but the value decoder never learned the rule: a value carrying its own delimiter had no closing quote on its line, failed to decode, and fell back to raw text with the quotes still in it. Five files under `data/` already write that form — `the subject''s name` in `data/cache/wikidata/property/P138.lino` read back as `'the subject''s name'` — and now decode. The change is additive, not a migration: doubled delimiters had no valid meaning before, so the backslash dialect the rest of the corpus uses is still read alongside it.
- Route every Links Notation renderer through one encoder, so a record carrying code is written in the notation it claims to be. Seven files had each grown a private `escape_lino_value` implementing the same C-style escape with subtle variations, and several value slots were interpolated with no escaping at all; substitution rule sets, associative packages, skill packages, behavior rules, self-facts, intent formalizations, and formalization candidates now share `format_lino_value`. It borrows the rule rather than restating it — the codec's always-quoting encoder is private, so a one-field record is formatted with the public `format_indented_ordered` and the field taken back off it — which means it cannot drift from the notation, because it is the notation's encoder. `tests/unit/issue_715_renderer_artifacts.rs` checks every renderer against both readers of the same document: the real grammar and the repository's own parser.
- Carry the above through the `tests/source/` mirror, the hand-copied second library that exists to reach private functions. Nothing enforces that it matches `src/`, and it had rotted where nobody looked: 51 of 143 mirrored modules differ from their source by more than two lines. Every module this change touches whose copy *can* compile is brought back into line, including `normal_markov`, whose copy was left behind by an earlier commit on this branch. One cannot, and saying so is the point: `intent_formalization`'s copy still holds the private `escape_lino_value` deleted above, because bringing it forward needs `cue_lexicon`, which reads its data through `include_str!("../data/meta/cue-lexicon.lino")` — a path that resolves under `tests/` from the mirror and cannot resolve at all. A faithful copy is not merely absent there, it is unrepresentable, so the mirror can only ever cover the part of `src/` that neither reads a file relative to itself nor depends on something that does. Nothing tests that copy — there is no `source_tests/intent_formalization/` — so no test asserts the old escaper's behaviour and the rot is dead weight rather than a false pass; but dead weight is what the next reader copies from, and the only thing that would have caught it is the `--check` this mirror has never had.
- Execute issue #715's own auto-learning task through two real external Agent CLIs, closing the one evidence row that cited only the in-process harness. The derived report names its promotion gate `normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass`, but no external Agent CLI had ever run the task — the gate asserted a pass that did not exist, and the in-process harness is precisely the one that cannot show capability routing surviving the wire. `experiments/agent_cli_e2e/run_issue_715_learning.sh` now drives `@link-assistant/agent` and `opencode` against the same task and diffs the two derived reports byte for byte, which turns "all harnesses supported in the similar way" into an assertion: a harness is supported only if it derives the *same* artifact. All three harnesses produce an identical 3961-byte report, so a harness contributes its tool vocabulary and nothing semantic. The script also asserts the report never promotes itself, and is wired into the `E2E Tests (agent CLI ↔ formal-ai)` CI job so the parity is enforced continuously rather than captured once.
- Fix eleven CI step names silently losing the issue number they exist to carry. An unquoted `#` opens a YAML comment, so `- name: Run agent CLI E2E — declarative new-file phrasing (issue #712)` reached the runner as `… (issue` — every E2E step advertised a dangling open paren where its issue reference should be, which is precisely the link a reader follows when the step goes red. The names are now quoted.
- Fix the writer and the reader of the same document disagreeing about what an escape means, so a rewrite of real code survives being written down. `sanitize_lino_value` escapes `\r`, `\n` and `\t` for the line-based `seed::parser`, but the decoder only ever learned `\n`, and neither escaped the backslash itself — so the pair was not invertible in either direction. A tab was written and read back as the two characters `\` and `t`, and a value carrying a backslash was read as an escape it never wrote: rewriting `println!("\n")` returned a *real newline* where the Rust source holds two characters. Neither is exotic for the subject of this issue — a Makefile recipe line is required to begin with a tab, Go is tab-indented, and a substitution rule round-tripped this way silently stops matching the code it was derived from. `SubstitutionRuleSet::from_links_notation` reads rules back through exactly this path. The backslash is now escaped first (it must be: sanitizing *introduces* backslashes), the decoder learned `t` and `r`, and its catch-all stays so LaTeX like `\ldots` still passes through. `tests/source/source_tests/links_format/tests.rs` holds the two functions to being inverses over the cases that motivated them rather than asserting the escape table by eye.
- Fix `scripts/build-views.py` emitting a backslash it never escaped, which the above would otherwise have turned into corruption. The generator interpolates each gloss straight into a quoted slot, so `\rightarrow` in `data/view/en/graph.lino` reached a decoder that now has an `r` arm and would have read a carriage return. Doubling it at the emit site — not in `_gloss_clean`, whose output the merge and keyword logic compare against — makes the decoder's `\\` arm return the one backslash the gloss holds; the regeneration moves exactly three lines in two files. The escaper is deliberately minimal and documented as such: a faithful one would have to reproduce the codec's delimiter choice, and doing that in a second language is precisely what the generator already gets wrong elsewhere — it rewrites Wiktionary's own quotation marks into apostrophes because it cannot escape them, so *hello* is defined as `'Hello!' or an equivalent greeting.` where the source says `"Hello!"`. Fixing that means writing these values through the codec rather than adding a tenth hand-rolled escaper, and is left as its own change.
- Fix the auto-learning report being unreadable whenever it reported on real code. An eighth private escaper survived the unification above, in the one renderer whose document the seed parser never reads back — so the grammar was its only reader, and the grammar is exactly the reader a backslash escape defeats. A `text` field carrying a quote made the whole report fail to parse, which is to say the auto-learning loop could not report on the subject of this issue. Separating the two jobs the encoder had been doing is what makes the fix free: `format_lino_value_verbatim` quotes and nothing else, while `format_lino_value` keeps sanitizing newlines for the documents `seed::parser` reads back a line at a time. The report takes the verbatim path, so its values keep their newlines and the committed issue-686 Agent CLI session stays byte-reproducible. `tests/unit/issue_715_renderer_artifacts.rs` asserts the field *survives* the grammar rather than that the document merely parses — a distinction the probes in `experiments/` had to establish, because the same escape elsewhere parses fine while the field silently disappears from the tree.

## [0.296.1] - 2026-07-16

### Fixed
- Stop the auto-release job from failing with "cannot rebase: Your index contains
  uncommitted changes" when a concurrent release lands on `origin/main` mid-job.
  The release now rebases onto the remote while the tree is still clean, before
  the version bump is written and staged.
- Only rebase when `origin/<branch>` actually has commits the release job lacks.
  Being ahead of the remote no longer reports "Local branch is behind remote".
- Create the release tag only after the release commit reaches the remote, so a
  `pull --rebase` retry can no longer leave the tag on an orphaned commit.

### Fixed
- Stop the release jobs from crashing the runner with "No space left on device"
  during the Docker build, which published the crate to crates.io but produced
  no image and no GitHub Release. The disk reclaim that issue #523 added to the
  Pages job now also runs in the auto-release and manual-release jobs, and lives
  in the shared `scripts/free-runner-disk.sh`.
- Warn when a runner is still nearly full after the reclaim, so the next
  occurrence leaves a diagnosable annotation instead of a job that dies with no
  failed step and no downloadable log.

### Fixed
- Stop the desktop release from reporting success when only some targets built.
  `BUILD-PROVENANCE.txt` listed all six builders unconditionally, so a run where
  most of the matrix failed still published a green, authoritative-looking
  `SHA256SUMS.txt` claiming builds that never happened. The manifest now lists
  only the builders that produced a fragment, names the missing targets, and the
  run fails after publishing the partial manifest.
- Verify the Linux and Windows artifacts before uploading them. Only the macOS
  artifacts were smoke tested, so the other four targets shipped with nothing
  checking that they were produced under the expected names and non-empty.
- Attach the SLSA build provenance before publishing assets to the release,
  rather than after, so assets are never downloadable without an attestation.
- Deduplicate concurrent desktop releases on the automatic (`workflow_run`) path.
  The concurrency group read `release.tag_name`/`inputs.tag`, neither of which
  that event carries, so it fell through to the always-unique `run_id` and
  concurrent runs for the same tag raced on `gh release upload --clobber` and on
  the consolidated `SHA256SUMS.txt`.

### Fixed
- Write `CHANGELOG.md` during a release in the exact shape the reconstruction
  check expects. The release spliced each new section in before the first
  `## [` line, but the preceding lines already ended with the blank line that
  follows the insert marker and the entry opened with another newline, so every
  release left a doubled blank line; `lines()` also dropped the file's trailing
  newline, which `join` never restored. Both defects went unnoticed because the
  check only runs when the lint job's path filter fires, which release commits
  do not trigger, so `main` turned red on the next unrelated pull request and
  the artifacts were refreshed by hand instead. Applied to both the automatic
  (`version-and-commit.rs`) and manual (`collect-changelog.rs`) release paths.

### Fixed
- Stop the issue #656 traceability test from asserting that a changelog fragment
  exists forever. Fragments are consumed by the release that ships them, so the
  test began failing on every run once the v0.296.0 release deleted the fragment
  it pinned. It now follows the entry across its lifecycle: a fragment before
  release, a `CHANGELOG.md` section after one.

## [0.296.0] - 2026-07-16

### Added
- Add a benchmark-gated promotion protocol (issue #656): `formal-ai improve --promote`
  executes canonical coding-modification, industry, and promotion-unit gates from
  fresh process output and, under `--apply --confirm`, creates a clean local review
  branch and materializes accepted `.lino` seed edits through Formal AI's Agent
  task path. Proposal-supplied runners/results, unsafe paths, malformed evidence,
  and failed commands are rejected; no push occurs. The promotion event chain and
  rejected changes round-trip through bundle export/import.

### Fixed
- Attest desktop and VS Code release artifacts directly so LF checksum manifests cannot break Windows provenance.
- Enforce rustdoc warnings, least-privilege workflow permissions, bounded desktop jobs, and fail-closed classification of known dependency diagnostics.
- Keep file-authoring Agent CLI requests from being misrouted into duplicate GitHub issue creation.

## [0.295.2] - 2026-07-16

### Fixed
- Route typed generated-source artifacts and ordered compiler/run commands through the write and shell tools advertised by an agentic CLI instead of scraping rendered answer labels or describing execution performed in a server-private fixture. Follow-up output edits now update the source before it is written, failures stop the command sequence, and Chat Completions, Responses, Anthropic Messages, and Gemini use the same routing behavior.
- Prevent HTTP API requests from executing agent actions in Formal AI's embedded temporary workspace; the client harness remains the auditable execution boundary.
- Persist issue #716 observations and evidence-linked architectural amendments in the associative auto-learning substrate, and produce a human-review-gated client-execution report through Formal AI and the real Agent CLI.

### Tests
- Add issue #716 presentation-independence, all-catalog-language, API-surface, auto-learning, and real Agent CLI E2E coverage that verifies `main.rs` is written and the harness receives both Rust compile and execution commands.

## [0.295.1] - 2026-07-15

### Fixed
- Route URL-navigation wording to advertised fetch tools, broader web-research wording to search tools, common file-update verbs to edit tools, and declarative `new file: …, contents: …` requests to write rather than read (issue #712).

## [0.295.0] - 2026-07-15

### Fixed
- Consume and stage changelog fragments after a successful release collection, preventing later releases from republishing stale notes.
- Reconstruct `CHANGELOG.md` from Git release history so each of the 391 released fragments appears exactly once.

## [0.294.0] - 2026-07-15

### Fixed
- Route agentic “Report issue” actions through an advertised shell tool to `gh issue create`, and enable OpenCode's documented Exa-backed `websearch` tool in ephemeral Formal AI sessions.
- Preserve client-executed tool inputs and outputs as durable memory evidence after the final API turn, including unnamed OpenAI tool results and Anthropic/Responses translations, so the associative and dreaming loops can learn from work performed by an Agent CLI.
- Parse Gemini `functionCall`/`functionResponse` history, retain call ids, and continue the shared multi-turn planner after a Gemini client executes a tool.

## [0.293.0] - 2026-07-15

### Fixed

- Keep subcommand-only and value-taking prompt flags out of empty interactive `with-formal-ai` launches, with PTY launch coverage for every supported CLI.

## [0.292.0] - 2026-07-15

### Added

- **Agentic mode now acts on simple natural-language requests** instead of
  falling to the "I could not determine…" blurb (issue #687). When Formal AI is
  driven as an agentic backend (e.g. OpenCode over the OpenAI-compatible server),
  the deterministic planner now recognises three new request classes and emits
  the appropriate tool calls for the harness to run:
  - **Factual / research questions** the symbolic engine cannot answer locally
    ("When are the next elections in the USA?", "What is the current population of
    Japan?", "Learn about it.") are routed to the client's **web-search** tool,
    then the surfaced source is **fetched** and the answer read from it
    (`src/agentic_coding/web_research.rs`). Whether a prompt warrants web research
    is decided by *asking the engine* — we search precisely what it cannot resolve
    from its own knowledge base — so it generalises across phrasings rather than
    matching fixed strings.
  - **"Report [this] on GitHub"** in natural language is turned into a real
    `gh issue create` shell tool call against the Formal AI repository, and the
    created issue URL is surfaced back to the user
    (`src/agentic_coding/report_issue.rs`). Agentic mode has no Formal AI web UI,
    so the top-bar "Report issue" button was previously unreachable.
  - **Conversational / meta questions** ("What we were talking about?") are
    answered from the message history with no tool call
    (`src/agentic_coding/conversation_recall.rs`).

### Changed

- The agentic `Progress` scan now also captures **web-search output**
  (`Progress::search_output`) so the research recipe can pick the source URL the
  search surfaced and fetch it before answering.

## [0.291.0] - 2026-07-15

- Fix Windows desktop provenance attestation by using the LF-safe current attestation action, update deprecated CI actions, remove recurring false-positive workflow warnings, and prevent docs-only final commits from hiding earlier code changes from CI.
- Make file-edit plans read their target before editing so read-before-write Agent CLIs can execute Formal AI's requested patch.

## [0.290.0] - 2026-07-15

### Added
- Usage-weighted associative persistence for issue #686
  (`src/associative_persistence.rs`): an `AssociativeMemory` that keeps a
  persistent version of meta-language expressions saved in an associative links
  network. Each expression is a content-addressed node (`stable_id`, so one meaning
  is one node) in an embedded `SubstitutionGraph`; the store counts usages (reads)
  and changes (writes) per expression and derives an independent usage signal from
  each node's incoming and outgoing link degree. A single `retention_score` (reads
  + writes + in-degree + out-degree, under configurable `RetentionWeights`) drives
  an LFU-style policy so the most used, most changed, and most connected knowledge
  persists longest; `eviction_order` / `evict_least_used` / `retain_most_used`
  forget the lowest-scored first, and `forget` removes an expression together with
  its incident links. Everything serializes to Links Notation, `from_context`
  ingests an issue #649 world-model `Context` preserving statement ids, and the
  whole policy is deterministic (no clocks, no randomness). Durable
  `MemoryEvent::write_count` now round-trips through native serialization, sync,
  substitutions, link projection, and the browser mirror; automatic dreaming
  rebuilds this associative view and uses the complete score for real eviction.
  Event ingestion also preserves qualifiers and validation warnings, normalizes
  evidence aliases, and supports bounded multi-hop recall. A derived persisted
  memory scenario executes through Formal AI and the real external Agent CLI.
  Covered by the issue-686 persistence, dreaming, and agentic regression suites.
- Design case study for issue #686 under `docs/case-studies/issue-686/`: a deep
  analysis mapping persistence, read/write counting, incoming/outgoing-link-degree
  usage, and links-only retention onto the associative stack, with cited online
  research (the Wikontic paper's full transferable symbolic pipeline,
  AriGraph, LFU/LRU cache replacement, reference counting, degree centrality), a
  per-requirement solution plan and prior-art survey, requirement rows R445–R458 in
  `REQUIREMENTS.md`, and the `tests/unit/docs_requirements_issue_686.rs`
  traceability test.

## [0.289.0] - 2026-07-14

### Fixed
- OpenAI Chat Completions: accept an assistant tool-call turn with an explicit `"content": null` (the standard OpenAI shape emitted by Qwen Code) instead of returning `400 invalid chat request: data did not match any variant of untagged enum MessageContent`. `#[serde(default)]` only covered an absent `content` key; a small deserializer now maps an explicit `null` to the default empty content. (#682)

## [0.288.0] - 2026-07-14

### Fixed
- Route natural-language file-creation requests ("create/write/save/generate a file …") to the `write` tool instead of emitting a `read` on the not-yet-existing target (issue #681). Write intent now beats read intent across every supported language via a general `has_file_write_intent` gate, and the write planner recognises the `named …` + `with the content …` phrasing.

## [0.287.0] - 2026-07-14

### Fixed
- Corrected documentation that had drifted from the codebase: `ARCHITECTURE.md` no longer claims a nonexistent `Event::Impulse` enum variant or `parent_id`/`language`/`surface` event fields, lists all 18 `SolverConfig` knobs instead of 9, drops four event kinds that no longer exist, renumbers a duplicated section 4.4, counts five rule shapes instead of four, documents the VS Code surface, and states honestly that ~26,700 lines of solver logic still live in `src/web/worker/*.js` (issue #658) rather than implying the JavaScript boundary is already narrow.
- `CONTRIBUTING.md` no longer carries template boilerplate: the title and clone URL name `formal-ai` instead of `rust-ai-driven-development-pipeline-template`, the project-structure tree reflects the real repository, and the line-limit rule distinguishes the 1000-line Rust cap from the 1500-line `.lino`/worker-JS caps.
- `docs/meta-algorithm.md` records the previously undocumented recursive-core recipe (issue #559) and corrects the procedural how-to record counts (11 roles, 8 functions, 6 stages, 4 parity pairs) to match the grounding suite.
- `docs/ci-cd/troubleshooting.md` invokes `rust-script scripts/publish-crate.rs` instead of a `node scripts/publish-crate.mjs` file that does not exist.
- `docs/testing/agentic-cli-tools.md` and the generated `docs/diagrams/agentic-recipes.md` now state their real scope instead of implying the multi-CLI CI matrix (issues #625/#671) and the full planner router set are already covered.
- Fixed the stale `SolverConfig::selection_mode` doc comment, which described `Legacy`/`Registry`/`Compare` variants that R344 replaced with `Off`/`Record`.

## [0.286.0] - 2026-07-14

### Changed
- Synchronized `VISION.md`, `GOALS.md`, `NON-GOALS.md`, `ROADMAP.md`, `ARCHITECTURE.md`, and `docs/USER-JOURNEYS.md` with the 2026-07-14 full-history requirement audit (issue #651): the roadmap now tracks requirement-level status (done / partial / not done), stale "current PR" headings reference their merged PRs, and the vision records the self-evolution frontier (world models #702, orchestration #703, portfolios #704, prediction #705, any-language #706, computer-use #707).
- Updated the doc-traceability pins in `tests/unit/docs_requirements*` to match the corrected wording.

### Added
- `docs/case-studies/issue-710/` preserving the raw audit reports over all 329 closed issues, 317 merged PRs, 31 open issues, and 17 open PRs, plus the konard/problem-solving methodology digest.

## [0.285.0] - 2026-07-14

### Added
- Tool-call emission in `formal-ai serve` is now **intent-based** rather than
  phrasing-gated (issue #680). When a client advertises a web-search, web-fetch, or
  write/edit tool, a request expressing that intent in *any* phrasing — across en, ru,
  hi, and zh — routes to the matching `tool_call` instead of a prose description. The
  routing holds over all three wire surfaces the target CLIs use (OpenAI Chat
  Completions, OpenAI Responses, and Gemini `generateContent`), and only fires when the
  matching capability tool is actually advertised, so a request that cannot be honoured
  still falls through to the prose answer.
- A file-creation intent that names a relative target file and literal content now
  routes to the advertised write tool in any phrasing/language. The write intent is
  recognised entirely from the seed lexicon (the new `file_write_*` roles in
  `data/seed/meanings-file-write.lino`) rather than from hardcoded English or Russian
  phrasings (CONTRIBUTING §2), and is probed before the file-read router so
  "create file X containing Y" is a *write*, not a read of X.
- A file-modification intent that names a target file plus an old→new replacement
  ("In greeting.txt, change hello to goodbye", "Replace foo with bar in notes.txt",
  «замени привет на пока в файле заметки.txt») now routes to the advertised edit tool,
  whatever the CLI calls it (`edit`, `replace`, `apply_patch`, `str_replace`). The
  new `Capability::Edit` recovers the `(target, old, new)` triple entirely from the
  seed lexicon (the new `file_edit_*` roles in `data/seed/meanings-file-edit.lino`),
  emits every common argument-key alias so one plan drives any CLI's edit tool, and is
  probed after the create-file write router and before the file-read router so an edit
  is never mistaken for a write or a read.
- A semantic shell request that never names the command — expressing an *intent* such as
  "Print the current working directory", "How much disk space is free?", or "What is my
  username?" — now routes to the advertised run tool carrying the concrete command
  (`pwd`, `df -h`, `whoami`) instead of a prose answer. The intent→command table,
  including multilingual cue phrases and per-intent argument recovery (`wc -l Cargo.toml`,
  `mkdir build`), lives in the new `data/seed/shell-intents.lino`, so coverage is retuned
  by editing seed data rather than the planner (CONTRIBUTING §2). It runs as a fallback
  after the named-command (#676) and directory-listing routers, so existing shell
  behaviour is unchanged, and only fires when a run/shell tool is advertised.

### Fixed
- The Russian navigation verb "загрузи" (load) is no longer misclassified as an
  `http_fetch`; it stays with `url_navigate`, while "скачай" (download the bytes)
  remains the fetch verb, so bare-domain navigation prompts resolve to an HTTPS link
  without fetch advice (issue #680).
- The general write router no longer mistakes a sentence-ending word for a target file:
  a token whose only dot is a terminal `.`/`!`/`?` ("… add the plural to томат.") is no
  longer treated as a dotted filename, so stored recipe requests are not hijacked
  (issue #680).

## [0.284.0] - 2026-07-13

### Added
- Add a replayable Hive Mind → Agent CLI → Formal AI self-coding scenario and CI-pinned evidence.

## [0.283.0] - 2026-07-13

### Added

- Added an issue #482 Nemotron 3 Ultra training-data sample suite: a
  no-full-download Hugging Face row sampler, a compact 10-row CC-BY-4.0 legal
  training-data fixture, benchmark/catalog provenance, and unit ratchets that
  verify sampler output, row provenance, digests, and `length=1` ingestion.

## [0.282.0] - 2026-07-13

### Added
- The assistant now honours being named in conversation. After "Now your name is
  Ineffa" (or "I'll call you Ada", "you are called …") it acknowledges the name and
  recalls it when later asked "what is your name", using dialog-local memory with no
  server state — mirrored in the browser worker (issue #676).
- Reasoning traces now open with a human, first-person narrative of what the
  assistant understood and decided ("You asked how I'm doing, so I told you and
  offered to help.") instead of an identical per-intent category template. The
  concrete steps remain beneath it as an expandable, recursive "robotic detail"
  layer. Applied to the API/CLI reasoning field (what agentic clients such as
  OpenCode render) and mirrored in the web thinking preview across en/ru/zh/hi
  (issue #676).

### Fixed
- Agentic planner now runs any seed shell token (`pwd`, `git`, `cargo`, …) named in a
  prompt, not just `ls`. `execute pwd`, `run git status`, and their many phrasings map
  to the real command (issue #676).
- Natural-language file-listing requests such as "give me a list of files in current
  folder" resolve to `ls` across many more phrasings (issue #676).
- Self-healing now triggers on natural self-directed repair requests such as "Can you
  fix it yourself?", "debug yourself", or "heal yourself", while ordinary "fix this
  file" requests still stay out of the repair loop (issue #676).
- "How are you?" small talk now gets its own warm wellbeing reply instead of the
  generic greeting. A dedicated `wellbeing` intent is matched before `greeting`
  (first-match-wins), so "how are you", "как дела", "आप कैसे हैं", and "你好吗" reply
  with an actual answer across en/ru/hi/zh — mirrored in the browser worker (issue
  #676).

## [0.281.0] - 2026-07-13

### Added
- Compose deterministic, capability-tagged Agent CLI plans for safe file-oriented change requests that are not encoded as pinned recipes.

## [0.280.0] - 2026-07-13

### Added
- Symbolic world models & contexts for issue #649 (`src/world_model.rs`): a
  first-class `Context` (a links network plus dependent statements), a
  `WorldModel` holding the per-dialogue `current`, `target`, and shared `general`
  contexts, and an `Action` modeled as STRIPS-style add/delete link edits. The
  module exposes the current→target `difference` (add / remove / conflicting
  links), predicts an action's consequences without mutating the model
  (`Context::predict` = apply-to-a-clone + recalculate + diff), recalculates every
  dependent statement's relative-meta-logic probability to a bounded fixpoint when
  the world changes (JTMS-style cascade over `Dependency` justification edges), and
  merges/splits contexts (ATMS-style). Reuses the existing `SubstitutionGraph`,
  the `relative_meta_logic` kernel, and `stable_id` content addressing; covered by
  `tests/unit/issue_649_world_model.rs`.
- Design case study for issue #649 under `docs/case-studies/issue-649/`: a deep
  analysis mapping the current-state / target-state world models, context
  merge/split, dependent statements, and action-consequence prediction onto the
  associative stack (links networks, `SubstitutionGraph`, the relative-meta-logic
  kernel, symbolic probability), with cited online research (STRIPS/PDDL,
  JTMS/ATMS, AGM belief revision, the JEPA world-model literature), a
  per-requirement solution plan and prior-art survey, requirement rows R428–R434
  in `REQUIREMENTS.md`, and the `tests/unit/docs_requirements_issue_649.rs`
  traceability test.

## [0.279.0] - 2026-07-13

### Fixed

- Keep Responses API instructions separate from the latest user request, make
  `formal-ai with` interactive/headless mode selection uniform across all
  supported tools, handle inline compaction prompts, and accept `--globally`.

## [0.278.0] - 2026-07-12

### Added
- Make `formal-ai with` auto-start a temporary agent-mode server, disable supported client summarization by default, isolate one-shot configuration, and support Claude Code, Qwen Code, Grok Build, and Aider.

## [0.277.0] - 2026-07-12

Issue #540 adds default-on dreaming maintenance planning for memory. The new
`formal-ai memory dream` command reports recomputable duplicate cleanup,
low-use cache/intermediate eviction under a 20% free-space target, and
storage-migration needs without mutating memory unless `--apply --confirm` is
used. The desktop shell now schedules the plan-only task in the background at
low priority.

Issue #540 dreaming now learns and generalizes, not just garbage-collects. While
idle it recalculates which topics the user interacts with most, remembers the
durable requirements the user has stated on them so he never has to repeat
himself, and generalizes each requirement into a meta-algorithm amendment baked
into memory as retained, never-forgotten learning (`meta_algorithm_amendment`).
Because an amendment can reproduce the specific task/test-run records it covers,
those specifics are forgotten first under storage pressure (the new
`ForgetCoveredSpecific` action) while the generalization is kept forever. The
dreaming meta-algorithm is now recorded as grounded data in
`data/meta/dreaming-recipe.lino`, pinned to the live source by
`tests/unit/specification/dreaming_meta_algorithm.rs`.

The follow-up completes that loop: structured amendments are now read by future
chat and Responses requests; coverage requires exact replay; repeated task
structures and multilingual data cues feed learning; real filesystem pressure,
incoming bytes, and persisted consent govern minimal cleanup; and core plus
desktop workers run only while idle and yield to foreground work. A complete
Formal AI Agent CLI gap-audit session is preserved with the issue case study.

# Issue #540: verified organic learning and runtime regression tests

- Amendments now form for any topic with stated requirements, so organic
  chat-only memory stores (raw messages plus durable task events) learn rules
  even before reproducible specifics exist.
- Refinement folds back only explicit `Learned standing requirement (...)`
  projection marker lines; free-form prose that merely quotes a requirement
  (such as solver fallback text) no longer pollutes rules.
- New regression tests: coverage revocation on rule change, eviction fallback
  for unverifiable records, the full organic record→dream→apply loop through
  the production chat path, refinement resurrection, durable failure records,
  numeric-pattern trial synthesis, multilingual task-kind gating, and the core
  dreaming runtime (idle gate, mid-run yield, `FORMAL_AI_DREAMING` opt-out,
  serve() wiring, locked atomic writes, desktop `PRIORITY_LOW`).

## [0.276.0] - 2026-07-09

### Added
- Recognize a language-agnostic "learn from this data source" directive so the
  reported issue #499 prompt is routed to a new `learn_from_source` intent instead
  of `intent: unknown`. Recognition is data-driven from a seed-declared
  learnable-source registry (`data/seed/learning-sources.lino`) shared by the chat
  handler and the Agent CLI planner, and the same teaching directive drives
  Formal AI's own Agent CLI learning recipe end-to-end (pinned session plus a live
  external-CLI E2E step in CI).

## [0.275.0] - 2026-07-09

### Added

- Added the issue #498 Google Trends catalog pipeline: parse a Trends RSS snapshot, expand the top 10 searches into multilingual prompt variants, answer every prompt through `FormalAiEngine`, and render the reviewable catalog at `data/meta/google-trends-catalog.lino`.
- Added a `google_trends_catalog` Agent CLI recipe with a pinned session under `docs/case-studies/issue-498`, plus raw Trends/GitHub evidence and tests that keep the seed, generated catalog, recipe routing, and documentation traceable.

### Added

- Closed the issue #498 auto-learning loop: `trending_learning_report()` re-answers every Google Trends catalog prompt, separates the ones the engine already routes from the *learning frontier* it cannot yet resolve, and hands that frontier to the human-gated issue #558 self-improvement learner. Because trending searches are open-domain questions, the learner honestly adopts nothing; the proposal-only result is rendered at `data/meta/google-trends-learning.lino`.
- Added a `google_trends_learning` Agent CLI recipe (`GOOGLE_TRENDS_LEARNING_TASK`) with a pinned session under `docs/case-studies/issue-498`, plus tests that keep the frontier split, proposal-only run, recipe routing, and documentation traceable byte-for-byte.

### Fixed

- Made the live Agent-CLI ↔ formal-ai E2E harness (`experiments/agent_cli_e2e/run_agent_cli.sh`) resilient to the third-party `@link-assistant/agent` CLI's non-deterministic early exit: the deterministic server plans the same next step every time, but the external CLI occasionally stops after the first tool round without writing the file, so the harness now retries the whole invocation up to `ATTEMPTS` (default 5) times and still enforces every hard assertion on a genuine, complete round-trip.

## [0.274.0] - 2026-07-07

### Documentation

- Added the issue #558 auto-learning case study, PR #601 gap analysis,
  requirements matrix, online research notes, and phased self-learning solution
  plan.
- Corrected the root issue #538 requirement status so delivered Agent CLI,
  diagram, and self-AST slices are no longer described as missing follow-ups.

### Added
- Issue #558 auto-learning: a closed, human-gated self-healing loop (`src/self_healing.rs`) that composes a failure trace, a verified source↔links round-trip, a benchmark-gated candidate lesson, and a terminal human-review outcome into one auditable `RepairCase`.
- `SourceRoundTrip::for_pinned_target` proves a real module survives a byte-for-byte `source → links → source` round-trip (the first genuine Links-to-source direction, not just a census).
- Fifth agentic recipe (`src/agentic_coding/self_heal.rs`): the self-healing loop is reachable through the agentic interface (Codex / OpenCode / Gemini / Agent CLI or the in-repo driver), emitting the repair case as `data/meta/self-healing-case.lino`. Adoption stays a human decision — nothing is auto-written.

### Added
- Issue #558 auto-learning (R558-04/R558-05): the **entire** source code of Formal AI is now translatable to the links / meta language and back. `build.rs` embeds every owned `src/*.rs` file (`OWNED_SOURCE_FILES`) so the whole tree is present in our data, and `src/self_source_graph.rs` content-addresses all of it and proves every owned module round-trips byte-for-byte through the sole CST/AST engine (`SourceGraph::owned`, exhaustive lossless proof).
- Sixth agentic recipe (`src/agentic_coding/source_graph.rs`): the whole-repository source↔links projection is reachable through the agentic interface, emitting a read-only Links Notation projection document (`self-source-graph.lino`). Nothing writes source back — the recompile-itself guardrail stays human-gated.
- `project_source_graph` example prints the exhaustive whole-repository projection for review.

### Added
- Issue #558 auto-learning (R558-03): `src/learning_ledger.rs` is the single, human-gated promotion protocol that terminates the self-healing loop. `LearningLedger::promote` records a `RepairCase` as a durable *approved learning record* only when **both** the benchmark gate is green **and** a human approves, and refuses every other case with a specific reason (`TestsNotGreen`, `NoReviewableProposal`, `SourceNotFaithful`, `HumanDeclined`, `AlreadyPromoted`). A repeated failure is then answered from the ledger instead of re-derived — the concrete payoff of "auto learning".
- Seventh agentic recipe (`src/agentic_coding/ledger.rs`): the promotion ledger is reachable through the agentic interface, emitting the approved learning record as a Links Notation document (`learning-ledger.lino`). The document records an already-approved decision, so nothing new is adopted and the recompile-and-reattach guardrail stays human-gated.
- `dump_learning_ledger` example prints the canonical approved ledger; `data/meta/learning-ledger.lino` is the generated, byte-for-byte-pinned artifact.

### Added
- Issue #558 auto-learning (R558-08): `src/self_explanation.rs` answers "how does Formal AI work?" grounded in the system's *own* source, data, and tests rather than prose docs. Each topic cites real artifacts; every `CitationKind::Source` citation resolves its `content_id` from the owned manifest and *panics* if the path is not an owned source file, so a fabricated citation cannot be constructed. The rendered Links Notation is anchored to the whole-source manifest content id that the source-to-links round-trip proves lossless.
- Eighth agentic recipe (`src/agentic_coding/explain.rs`): the grounded self-explanation is reachable through the agentic interface, emitting `how-formal-ai-works.lino`. Like the source-graph recipe it commits no byte-pinned artifact because the citation ids track the whole source tree.
- `explain_formal_ai` example prints the canonical grounded explanation.

### Added
- Issue #558 auto-learning (R558-07): `src/change_request.rs` turns a natural-language "change Formal AI itself" request into a reviewable pull request through the *same* human-gated repair loop the ledger uses. A request plus a target module becomes a `ChangeRequest` — a normalised requirement, a proposed test name, and an ordered patch plan whose target is grounded against the owned manifest (`ChangeRequest::for_module` *panics* on any path the repository does not ship, so a request can never target fabricated source). `ChangeRequest::review` merges the change only when a `BenchmarkGateReport` is green *and* an explicit `HumanApproval` is granted, refusing every other case (`TestsNotGreen` / `HumanDeclined`); neural inference stays a NON-GOAL, and the patch is a deterministic plan a human or Agent CLI executes, not generated code.
- Ninth agentic recipe (`src/agentic_coding/change_request.rs`): the user-driven self-change is reachable through the agentic interface, emitting `requested-change.lino`. Like the source-graph and explain recipes it commits no byte-pinned artifact because the target's manifest content id tracks the whole source tree.
- `request_change` example prints the canonical change request and demonstrates the accept/decline review gate.

### Added
- Issue #558 auto-learning (R558-02): `src/repair_strategy.rs` is the *general* front of the failure-to-repair loop. The self-healing slice repairs a single canonical failure by synthesising a solver method; this generalises it. `RepairStrategy::classify` reads an arbitrary `UnknownTrace` — the same trace the self-healing loop reasons about — and, purely deterministically from the trace's own prompt and event signals, maps it onto exactly one of the three targets issue #558 names (`RepairTarget::SolverMethod` / `DataRecord` / `Test`), so the loop is *total* — every failure is classified. For each it composes the grounded repair plan (rationale, proposed change scoped to the target class, and the automated verification that must be green before human promotion). It stays proposal-only and human-gated; neural inference stays a NON-GOAL — the classification and plan are deterministic functions of the trace, and the "change" is a plan a human or Agent CLI executes, never generated code applied automatically.
- Tenth agentic recipe (`src/agentic_coding/repair_strategy.rs`): the general classifier is reachable through the agentic interface, emitting the three canonical strategies (one per target class) as `repair-strategies.lino`. Unlike the source-graph, explain, and change-request recipes it commits a byte-pinned artifact (`data/meta/repair-strategies.lino`), because the document depends only on self-contained canonical traces, not the whole source tree — asserted byte-for-byte against a fresh render like the self-healing repair case.
- `classify_repair` example prints the grounded, human-gated repair strategy the classifier composes for each of the three failure classes.

### Added
- Issue #558 auto-learning (R558-06): `src/rebuild_plan.rs` closes the final loop — *"recompile and reattach the improved code to the UI."* `RebuildPlan::for_accepted_change` derives a plan purely from an already-accepted change (a green benchmark gate AND a human approval), so a rebuild can never precede acceptance. It grounds every reattached UI artifact against the real repository bytes and the owned manifest (`Cargo.toml`, `src/main.rs`, `src/web/formal_ai_worker.js`, `src/web/index.html`), and emits a strictly ordered, observable, reversible five-step pipeline (recompile → regenerate worker → reattach → hot-swap → verify). The regenerated `formal_ai_worker.wasm` is deliberately absent from the grounded inputs — it is the pipeline's *output*, referenced by the steps. The plan stays proposal-only and human-gated; nothing rebuilds or hot-swaps automatically, and neural inference stays a NON-GOAL — the plan is a deterministic, content-addressed function of the accepted change that a human or Agent CLI executes.
- Eleventh agentic recipe (`src/agentic_coding/rebuild_plan.rs`): the rebuild-and-reattach plan is reachable through the agentic interface as `rebuild-and-reattach.lino`. Like the source-graph, explain, and change-request recipes it asserts a *live* document (never a byte-pinned artifact), because the plan depends on the whole owned source tree; it keys on `reattach` so it stays disjoint from the source-graph recipe, which owns `recompile`.
- `rebuild_and_reattach` example prints the grounded, human-gated recompile-and-reattach pipeline the plan composes for the accepted canonical change.

### Added

- Added a lazy, configurable issue #527 question-generation API with grammar and meaning classification plus an answer stream through `FormalAiEngine`. The generator is language-agnostic: `QuestionGenerationConfig::for_language` and `question_lexicon_summary_for_language` drive the same enumeration, frequency-tiering, and classification over English, Russian, Hindi, and Chinese vocabulary seeded in `data/seed/question-generation-lexicon.lino`, so no language-specific code path exists.
- Added the `question_catalog` agentic recipe (the eleventh) that drives Formal AI through its own Agent CLI to enumerate questions smallest-first, classify them, answer the meaningful ones, and record the reviewable catalog in Links Notation (`data/meta/question-catalog.lino`), grounded in the `data/seed/question-generation-lexicon.lino` frequency-tier vocabulary. Answered questions form a case/whitespace-insensitive recall table (`QuestionCatalog::answer_for`) that never mutates the human-gated learning ledger.

## [0.273.0] - 2026-07-04

### Added
- Added issue #526 round-trip translation quality requirements, natural/code translation regression coverage, and case-study documentation.

### Changed
- Reworked code translation (`translate_program`) to route through a language-neutral code meta language (`CodeMeaning` / `formalize_code_meaning` / `render_code_meaning`) instead of direct `(source, target)` pairs, so it stays at `N` formalizers + `N` renderers and pairs like Python → JavaScript or Rust → Go translate through one shared meaning.

## [0.272.0] - 2026-07-04

### Fixed

- Aligned one-shot `with-formal-ai codex` invocations and documentation with the
  direct Codex examples by applying `--sandbox read-only` alongside
  `--skip-git-repo-check`.

## [0.271.0] - 2026-07-04

### Documentation

- Added an agentic CLI testing guide with fixture markers, logging proxy
  provenance checks, phrasing matrices, and CI e2e assertions for `codex`,
  `opencode`, `gemini`, and `agent`.

## [0.270.0] - 2026-07-04

### Fixed

- Agent-mode OpenAI-compatible tool planning now routes local file-reading
  prompts to `read`/`bash` tool calls instead of treating filenames such as
  `beta.md` as URLs or falling through to non-agentic answers.

## [0.269.0] - 2026-07-04

### Fixed
- Routed natural-language current-directory listing prompts, such as "what files are in this folder?", to agent-mode shell tool calls with `{"command":"ls"}` instead of falling through to the unknown-answer response.

## [0.268.0] - 2026-07-04

### Added
- Added `formal-ai proxy`, a built-in logging reverse proxy that forwards HTTP traffic to a Formal AI server and appends JSONL provenance/routing summaries.

## [0.267.0] - 2026-07-03

### Fixed
- Fixed Codex Responses compatibility by matching shell tool-call arguments to the advertised `cmd` schema, returning `slug` in OpenAI-compatible model metadata, and allowing `with-formal-ai codex` to start outside Git worktrees.

## [0.266.0] - 2026-07-03

### Added
- Added `with-formal-ai agent` support with inline Agent CLI config and
  persistent `~/.config/link-assistant-agent/opencode.json` setup.

## [0.265.0] - 2026-07-03

### Fixed

- Isolated one-shot `with-formal-ai gemini` invocations from cached Gemini CLI
  OAuth settings by selecting API-key auth in a temporary Gemini home and
  enabling workspace trust.

## [0.264.0] - 2026-07-03

### Added
- Added deterministic OCR/text market-price claim extraction for the document verification path, including ETH aliases across supported languages and source-backed relative-meta-logic assessments.
- Mirrored market-price contradiction checks in the browser worker and added an e2e regression for the image/OCR flow.
- Preserved issue #493 evidence under `docs/case-studies/issue-493`, including the screenshot, OCR output, market-data captures, and before/after regression logs.

### Fixed
- Preserved full multi-line OCR/text samples during document verification so factual claims after the first line are checked instead of being dropped from the statement plan.
- Flagged `ETH in 2024: $1,700` as contradicted using captured Binance ETHUSDT 2024 daily klines.

## [0.263.0] - 2026-07-03

### Added

- Route the whole class of externally verifiable questions to web research by
  reasoning about the referent instead of memorising topic vocabulary: any
  interrogative that names a referential external entity — a brand written with
  interior capitalisation such as `ChatGPT`, `OpenAI`, `iPhone`, or `TypeScript`
  — and that the solver cannot resolve from local memory now routes to the
  source-gathering research plan. The structural rule fires identically for
  English, Russian, Hindi, and Chinese prompts and across any topic (pricing,
  release dates, hardware specs, features), so the Russian annual-discount prompt
  for Claude Max and ChatGPT Pro is handled as one instance of the general class
  rather than by a product-specific answer or a stored word list.

## [0.262.0] - 2026-07-03

### Fixed

- Answer previous-user-question recalls such as Russian `что я спрашивал` from the user's earlier request instead of the assistant's previous reply.

## [0.261.0] - 2026-07-03

### Added
- Expose solver thinking traces through OpenAI Chat `reasoning_content`, OpenAI Responses reasoning summary output/events, and Anthropic thinking documentation.

### Added
- Added `formal-ai with` and the standalone `with-formal-ai` wrapper for running or permanently configuring Codex, OpenCode, and Gemini against a local Formal AI server from seed-backed client integration templates.

## [0.260.0] - 2026-07-03

### Fixed

- Covered the legacy `/v1/responses` route with a real loopback HTTP streaming
  regression test so Codex-style Responses SSE clients must receive
  `response.completed`.

### Documentation

- Documented a copy-paste Codex 0.142+ configuration and `codex exec "hi"`
  command for driving Formal AI through the Responses wire API.

## [0.259.0] - 2026-07-03

### Added

- Added `/api/<protocol>/...` gateway routes for OpenAI, Anthropic, Gemini,
  Vertex, and formal-ai native APIs, with per-protocol model discovery.

### Fixed

- Added named OpenAI Responses SSE events for streaming Responses clients,
  including the final `response.completed` event.

## [0.258.0] - 2026-07-02

### Fixed
- Renamed the advertised model id to `formal-ai` and accepted seed-backed aliases such as `@link-assistant/formal-ai`.

## [0.257.0] - 2026-07-02

### Fixed
- Added real loopback HTTP regression coverage for OpenAI Chat Completions `stream:true` responses so the SSE stream must use `chat.completion.chunk` frames with `choices[].delta.content`, and documented a verified OpenCode `hi` setup.

## [0.256.0] - 2026-07-02

### Fixed

- Agentic Chat Completions now emits `bash` / `shell` / `run_command` tool calls
  for natural-language `ls` directory-listing requests when agent mode is enabled,
  so the Link Assistant Agent CLI can execute and return the listing.

### Added

- `formal-ai serve --agent-mode` as the documented command-line opt-in for
  OpenAI-compatible agent clients, alongside the existing `FORMAL_AI_AGENT_MODE=1`
  environment variable.

## [0.255.0] - 2026-07-02

### Added

- Grammatical detail for the tomato **and potato** meanings (issue #538): every
  surface (`tomato`/`tomatoes`, `помидор`/`помидоры`, `томат`/`томаты`,
  `potato`/`potatoes`) now pins its part of speech and grammatical number
  (singular/plural) in the seed data, and the previously missing plurals `томаты`
  and `potatoes` were added.
- New `grammatical_number` semantic facet kind plus `WordForm::grammatical_number()`,
  `WordForm::part_of_speech()`, and `WordForm::denotations()` accessors.
- Grounded, multilingual `grammatical_number` / `singular` / `plural` meanings
  (Wikidata `Q104083` / `Q110786` / `Q146786`) lexicalised in en/ru/hi/zh, with
  cached Wikidata data for offline grounding-closure tests.
- The meaning-detail change is produced by **driving Formal AI through its own
  in-repo Agent CLI** (`src/agentic_coding/`), with the committed seed asserted
  byte-for-byte equal to the driver output. A concept registry generalises the
  recipe, proven by driving tomato and potato with two *differently worded*
  requests. The Agent-CLI sessions that solved the task are committed
  (`docs/case-studies/issue-538/agent-cli-session*.json`), and
  `scripts/reproduce-issue-538.sh` regenerates the change on a clean checkout.
- Generated agentic-recipe **mermaid diagrams**, split into parts
  (`docs/diagrams/agentic-recipes.md`), rendered from the planner's own recipe
  table by `src/agentic_coding/diagram.rs` — a non-lexeme axis (issue #538
  R15/R16) proving the Agent-CLI method generalises beyond meaning data. The Agent
  CLI writes the document from a *third* differently worded request; the document
  and its session JSON are reproduced byte-for-byte under test.
- Self-inspection **CST/AST census** recipe (`src/agentic_coding/self_ast.rs`,
  issue #538 R13): the meta algorithm parses one of its own Rust modules (the
  deterministic planner) through the repo's sole CST/AST engine — the
  link-foundation `meta-language` links network — and stores the abstract-syntax
  node census in our data as Links Notation (`data/meta/self-ast.lino`). The
  census logic is general (works on any Rust source, proven by tests over several
  sources), the Agent CLI drives it from a *fourth* differently worded request
  (`docs/case-studies/issue-538/agent-cli-session-self-ast.json`), and the
  committed artifact is reproduced byte-for-byte under test.
- `formal-ai agent --session-json <path>` to capture a replayable Agent-CLI
  session as JSON.
- Case study `docs/case-studies/issue-538` with a requirements decomposition,
  per-requirement solution plan, online research, and a `refusal-anti-pattern.md`
  recording the rejected "ship a slice, defer the rest" reasoning.
- Real Agent CLI ↔ formal-ai E2E round-trip test
  (`experiments/agent_cli_e2e/run_agent_cli.sh`) that boots `formal-ai serve`
  and drives it with the **external** `@link-assistant/agent` CLI over the
  OpenAI-compatible endpoint — no mocks. Wired as the new `test-agent-cli-e2e`
  CI job in `.github/workflows/release.yml` (running all four recipe axes —
  tomato, potato, diagrams, and the self-AST census — against the real server),
  and the real captured console log is committed at
  `docs/case-studies/issue-538/agent-cli-e2e-run.log` so the round-trip evidence
  is inspectable, not synthesised.

### Changed

- Made the Agent-CLI-driven, no-deferral development workflow the **standing
  rule** in `CONTRIBUTING.md`: from this task forward Formal AI changes are
  produced by driving the Agent CLI (never hand-editing, never deferring to
  follow-ups), with the tool extended when it cannot yet do the work. Added
  four further standing rules covering the real Agent-CLI E2E requirement,
  hardcoded cases only in tests, real captured logs in case studies, and small
  atomic commits.
- Fixed a TOCTOU race in `AgentWorkspace::for_prompt` (parallel runs with the
  same prompt shared a deterministic temp dir) via a per-instance unique
  workspace id.
- Split the meaning-lexicon seed parser into `src/seed/meanings/parse.rs` so both
  `meanings.rs` and the new module stay under the Rust file-size guard after the
  grammatical-detail additions (mirrors the existing `roles.rs` split).
- Reworked the meaning-detail recipe (`src/agentic_coding/meaning_detail.rs`) to
  **derive** every enriched surface from real, checked-in Wikidata lexeme JSON
  (parsed by a general serde_json algorithm) instead of hardcoded answer tables:
  the singular form is anchored to the lexeme's lemma and the plural is paired by
  matching non-number grammatical features, so the logic is general for any case
  paradigm and references no hardcoded case id. Hardcoded strings now live only in
  tests; the four seed blocks are reproduced byte-for-byte from the source JSON.

## [0.254.0] - 2026-07-01

### Added
- Generalized the document-originality handler into a full verification class: authenticity, factual-accuracy, and veracity requests (not only plagiarism/uniqueness) now route to the same grounded workflow across English, Russian, Hindi, and Chinese.
- Weighed every extracted statement with relative-meta-logic (github.com/link-foundation/relative-meta-logic): statements start from an assumed-true prior, are raised by trusted original-first sources, lowered by contradicting originals, and unoriginal reposts are ignored — recorded deterministically in the append-only event log.
- Grounded each statement with a dedicated fact-check web-search query, mirrored byte-for-byte into the Web app worker so the browser matches the Rust engine.

### Fixed
- Routed multilingual text-attachment originality and plagiarism checks through a grounded attachment workflow instead of falling back to unknown.
- Included sampled text/plain attachment content in Web app solver context so browser uploads can be inspected by deterministic handlers.
- Folded Telegram document attachments into the shared attachment-context builder so forwarded files reach the same originality/verification handler.
- Classified `.lino` seed and language-resource changes as code in `detect-code-changes.rs` so editing files the language-change-parity guard watches (e.g. `src/web/i18n-catalog.lino`) now triggers lint/test instead of silently skipping them.

## [0.253.0] - 2026-07-01

### Added
- Issue #556: generalized the response-language follow-up beyond repository lookups to the whole class of "re-answer the previous request in another language" turns. A bare follow-up such as "I do not understand English, write in Russian" now replays the previous request through the entire solver with the target language forced at a single detection seam, so capabilities, identity, project lookups, and other localizable answers all re-render in any seeded language (English, Russian, Hindi, Chinese) — and reverse back to English on request.
- Grounded the follow-up in a machine-readable meta-algorithm recipe (`data/meta/response-language-followup-recipe.lino`), pinned by `tests/unit/specification/response_language_meta_algorithm.rs`, so the eight recursive-reasoning steps, seed roles, Wikidata groundings, handler functions, forced-language seam, and Rust↔JS parity targets can never silently drift from the live source. Documented in `docs/meta-algorithm.md`.
- Added round-trip translation tests (issue #526) proving English↔Russian/Hindi/Chinese vocabulary survives a source→meta-language→target→source cycle with both meaning and surface preserved.

### Fixed
- Issue #556: repository lookup language-change follow-ups now rerender the previous GitHub lookup in the requested seeded response language instead of falling through to unknown.

## [0.252.0] - 2026-06-30

### Added
- Natural-language access to the entire associative memory (issue #529). Queries now read across all stored memory events and projected memory links, and `formal-ai memory query` performs Turing-complete read+write control: appending new memory and applying substitutions that rewrite every matching stored value in place (not just recording intent). The WASM/browser app reaches parity: the JS worker recognizes the same multilingual append and substitution directives, and the browser persists them by appending memory events and rewriting matching stored values in IndexedDB. All paths are driven by the multilingual seed lexicon across English, Russian, Hindi, and Chinese.

### Fixed
- Asking "what was written in the previous message?" (and its Russian, Hindi, and Chinese equivalents) now recalls the previous message instead of returning an unknown intent, in both the Rust runtime and the browser JS worker (issue #529).

## [0.251.0] - 2026-06-30

### Fixed
- Recognize Russian calendar event prompts that use spoken-hour wording such as "на 10 часов" and route them to calendar event creation.

## [0.250.0] - 2026-06-30

### Fixed

- Guard the reported Russian hackathon dialog so `Где посмотреть актуальные хакатоны?` and the follow-up `Найди мне хакатоны` stay on the `web_search` route instead of the unknown fallback.

## [0.249.0] - 2026-06-30

### Fixed
- Route current public-event questions such as `Какие хакатоны сейчас проходят?` to web search instead of the unknown-intent fallback.

## [0.248.0] - 2026-06-30

### Added
- The Rust solver can now answer natural-language queries over prior dialog turns and persisted `.lino` memory, such as "When did I mention Rust?" or "Find Rust in another conversation", through the `conversation_recall` intent.
- The local HTTP surfaces (`/v1/chat/completions`, `/v1/responses`, and `/v1/messages`) now scan `FORMAL_AI_MEMORY_PATH` when a natural-language recall query asks about memory outside the current request history.
- The CLI now includes `formal-ai memory query --prompt ...` for direct natural-language recall over a saved `demo_memory` or `formal_ai_bundle` file.

## [0.247.0] - 2026-06-29

### Fixed

- Route unresolved bare term prompts such as `cursor` to web search after local knowledge sources miss instead of returning the unknown fallback.

## [0.245.0] - 2026-06-29

### Fixed
- Route multilingual event-listing prompts such as "Найди мне хакатоны" to web search instead of the unknown-intent fallback.

## [0.244.0] - 2026-06-29

### Fixed
- Routed telegraphic install how-to prompts such as `how install cursor` through official-documentation-first procedural discovery instead of the unknown fallback.

## [0.243.0] - 2026-06-28

### Fixed
- GitHub repository traffic questions now answer from official GitHub traffic documentation instead of falling to `intent: unknown`, including the reported Russian prompt.

## [0.242.0] - 2026-06-28

### Fixed
- Route multilingual topic-interest prompts to web search instead of the unknown-intent fallback.
