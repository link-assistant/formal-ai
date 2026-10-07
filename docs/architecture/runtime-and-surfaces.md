# Architecture: Configuration, Event Log, Surfaces, Evidence, and Testing

Part of the [architecture overview](../../ARCHITECTURE.md) (§11–§15). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 11. Configuration

All configuration lives in `SolverConfig` and is persisted with the agent
session. The knobs:

| Knob | Type | Default | Effect |
| --- | --- | --- | --- |
| `guess_probability` | f32 in `[0, 1]` | `0.8` | 0 = strongly prefer asking under ambiguity, 1 = always guess. |
| `context_sensitivity` | f32 in `[0, 1]` | `0.6` | how strongly recent messages bias formalization. |
| `questioning_rigor` | f32 in `[0, 1]` | `0.4` | how strict the clarifying question is. |
| `max_decomposition_depth` | usize | `4` | bound on recursive decomposition. |
| `agent_mode` | bool | `false` | unlock destructive / autonomous actions. |
| `diagnostic_mode` | bool | `false` | include trace/intent/evidence chips in the answer prose. |
| `offline` | bool | `false` | refuse external lookups (also `FORMAL_AI_OFFLINE`). |
| `cache_ttl_seconds` | u64 | `5_184_000` | TTL for `source_cache` entries (≈ 60 days). |
| `temperature` | f32 in `[0, 1]` | `0.7` | softmax temperature for interpretation selection. |
| `follow_up_probability` | f32 in `[0, 1]` | see `SolverConfig::default` | how often the proof engine invites the user to refine proof inputs before final execution. |
| `definition_fusion_by_default` | bool | see default | plain definition prompts use cross-language fusion before concept lookup (also `FORMAL_AI_DEFINITION_FUSION`). |
| `associative_project_promotion` | bool | see default | repository questions prefer known Link Assistant / Link Foundation / LinksPlatform projects first. |
| `recursion_mode` | `RecursionMode` | `Down` | which directions of the meta core's recursion are traced (`down`/`up`/`both`); trace-only (R338). |
| `selection_mode` | `SelectionMode` | `Off` | whether the registry-resolved method per atomic leaf is recorded (`off`/`record`); trace-only (R339). |
| `skill_mode` | `SkillMode` | `Off` | whether the skill/curriculum ledger is accumulated (`off`/`accumulate`); proposal-only (R342, also `FORMAL_AI_SKILL_MODE`). |
| `execution_surface` | `ExecutionSurface` | see default | embedding surface used for environment-aware self-description. |
| `blueprint_composition` | `BlueprintComposition` | see default | how composite-program blueprints project their recipe template (issue #340). |
| `probability_policy` | `ProbabilityDecisionPolicy` | paper baseline | `CU`/`TU`/`TC`/`SS` knobs governing how symbolic probability evidence ranks candidates. |
| `forced_response_language` | `Option<&'static str>` | `None` | forces one replay's response language and guards recursion (issue #556, R337-adjacent). |

The same prompt + same config produces the same answer. Random choices are
seeded from the impulse content hash. The defaults live in
`SolverConfig::default` in `rust/src/solver.rs`; the trace-verbosity knobs
(`recursion_mode`, `selection_mode`, `skill_mode`) change neither routing nor
the answer.

---

## 12. Append-Only Event Log

Every event written by the pipeline carries:

- a content-addressed `id` (FNV-1a 64-bit);
- a `kind` naming what was recorded — representative kinds include `impulse`,
  `sub_impulse`, `candidate`, `validation`, `cache_hit`, `source`,
  `source_refresh`, `agent_action`, `trace`, and `error`. The vocabulary is
  open rather than fixed: `EventLog::append` takes any `&'static str`, and many
  kinds use a `prefix:detail` convention (`policy:offline`,
  `formalization:*`, `trace:execution_failure`, `summarization:mode`,
  `meta_algorithm_amendment`, …), which is what the evidence-link namespace
  projects;
- a `payload` that varies by kind (Links Notation snippet).

The in-process `Event` struct (`rust/src/event_log.rs`) is deliberately minimal —
`id`, `kind`, `payload`. Nesting depth, language, and surface are carried by
the payload and the surrounding trace rather than by dedicated fields.

The log is the system of record. The user-facing `answer` field is a
projection. The Links Notation trace is the canonical export form.

`memory::export_full_memory` exports the full bundle (seed + events +
preferences + environment metadata) as one `formal_ai_bundle` Links Notation
file. `memory::import_full_memory` round-trips it back, including known
migrations. Destructive memory maintenance commands reuse the same full-bundle
format for backups before physical deletion or reset.

---

## 13. Surfaces

The same `FormalAiEngine` answers prompts in every surface:

- **Rust library** — `formal_ai::FormalAiEngine::answer` /
  `formal_ai::solve_with_history`.
- **CLI binary** — `formal-ai chat`, `formal-ai memory ...`,
  `formal-ai bundle ...`, operator commands such as
  `formal-ai github-logs ...`, `formal-ai telegram`, `formal-ai serve`.
- **HTTP server** - a local gateway with OpenAI routes under
  `/api/openai/v1`, Anthropic under `/api/anthropic/v1`, Gemini under
  `/api/gemini/v1beta`, Vertex under `/api/vertex/v1`, and native formal-ai
  routes under `/api/formal-ai/v1`. The legacy `/v1/chat/completions`,
  `/v1/responses`, `/v1/messages`, and `/v1/network` aliases remain for existing
  desktop and CLI configs; the older `/v1/graph` alias still resolves but is
  flagged deprecated in favor of `/v1/network`.
- **Desktop shell** — `desktop/main.cjs` starts the same local
  `formal-ai serve` API on loopback, serves the existing `js` chat, and
  exposes a preload bridge for API, links-network, full-memory, and permission
  status.
- **Telegram bot** — `POST /telegram/webhook` (webhook) or
  `formal-ai telegram` (long polling).
- **Prepared Telegram Docker image** — releases publish the root `Dockerfile`
  to GitHub Container Registry as `ghcr.io/link-assistant/formal-ai:latest`.
  The repository root `compose.yaml` runs that image with only
  `TELEGRAM_BOT_TOKEN` required, while `FORMAL_AI_DOCKER_IMAGE` lets operators
  point the same compose file at a local build or mirror. The image builds the
  Docker-in-Docker Telegram image: it builds the Rust binary, copies it into
  `konard/box-dind:2.10.2`, keeps
  `/usr/local/bin/dind-entrypoint.sh` as the entrypoint, and defaults to
  `formal-ai telegram --mode polling`. Commands that need nested execution
  use the bundled `$ --isolated docker --auto-remove-docker-container --`
  wrapper from `start-command`, which records logs under
  `/tmp/start-command/logs/`.
- **One-click / one-line services** — the same prepared image runs two managed
  containers: the Telegram bot (`formal-ai-telegram`, the default command) and
  the OpenAI-compatible API server (`formal-ai-server`, `formal-ai serve` for
  agentic mode, published on `127.0.0.1:8080`). The desktop app starts and stops
  both with one click via the testable `desktop/lib/service-control.cjs` module
  (an injected `runDocker` runner, exposed over IPC as
  `formalAiDesktop:serviceStatus` / `startService` / `stopService`); a server
  reproduces the identical containers with `docker compose --profile all up -d`
  (or the documented raw `docker run` arguments). Each service uses its own
  inner-Docker volume (`formal-ai-telegram-docker`, `formal-ai-server-docker`)
  because two DinD daemons cannot share one `/var/lib/docker`. See
  [docs/desktop/service-control.md](../desktop/service-control.md).
- **Reasoning projection for client protocols** — the solver's ordered
  `thinking_steps` remain the canonical internal trace. The OpenAI Chat surface
  also emits the rendered trace as `message.reasoning_content` and streaming
  `delta.reasoning_content`; the OpenAI Responses surface emits a
  `type:"reasoning"` output item and `response.reasoning_summary_*` stream
  events; the Anthropic adapter emits `thinking` blocks and `thinking_delta`
  events only for requests that enable extended thinking. This keeps
  thinking-capable CLIs on their native protocol fields without inventing a
  separate display contract.
- **VS Code extension** — `vscode/`, shipped for both hosts: the desktop
  (Node) host drives the local `POST /v1/chat/completions` route, while the
  web/`vscode.dev` host runs the in-process WASM engine. Marketplace and Open
  VSX publication is tracked by issue
  [#666](https://github.com/link-assistant/formal-ai/issues/666).
- **Browser demo** — `js/worker/formal_ai_worker.js` (a small loader shim) plus
  the solver logic it `importScripts`-loads from
  `js/worker/formal_ai_worker_00.js` … `_23.js`, alongside the WebAssembly
  worker built from `js/wasm-worker/src/lib.rs`.

Rust/WASM owns deterministic domain primitives that must match the native
solver byte-for-byte: prompt normalization, language detection, arithmetic
evaluation, stable FNV-1a ids, unknown-answer opener selection, intent-route
matching semantics, web-search provider constants, request evidence, and
reciprocal-rank fusion. JavaScript keeps the browser-only responsibilities: UI
state, seed-file fetch/parsing, network/CORS orchestration, DOM integration,
and compatibility fallbacks when WASM cannot be instantiated.

**The browser boundary is not yet narrow, and this is the honest current
state.** The WASM worker crate (`js/wasm-worker/src/`) is 2,156 lines,
while `js/worker/` still carries the mirrored solver logic in 35
JavaScript modules, every one under a shrink-only ceiling recorded in
`data/meta/worker-line-budget/` (the 35 ceilings sum to 30,179 lines as of
this writing) and enforced by `scripts/check-worker-line-budget.rs` toward a
3,000-line end-state target — the cross-runtime parity (E34) and
issue #349/#408 handlers were mirrored into JavaScript rather than absorbed
into WASM. Pillar 18 ("Rust-to-WebAssembly parity with JavaScript reserved for
UI/glue") therefore describes the target, not today's split. The
WASM-absorption epic [#658](https://github.com/link-assistant/formal-ai/issues/658)
closed on 2026-07-18 (PR #691); what carries the absorption now is the
shrink-only ratchet toward that end-state target, which in turn unblocks the
npm-published engine in issue
[#665](https://github.com/link-assistant/formal-ai/issues/665).

**Standing principle (2026-08-04, R536).** JavaScript is interfacing glue
and JSX (React) UI only. All logic is compiled Rust: native in the CLI,
server, and desktop-managed processes; Rust→WASM in the web app. The same
WASM web engine is reused — not reimplemented — by the desktop shell and
the VS Code hosts. The remaining `js/worker/*.js` solver logic is a
transitional mirror under the shrink-only ratchet
`scripts/check-worker-line-budget.rs`; it may only move into Rust→WASM,
never grow, and the checker's 3,000-line `TARGET_TOTAL_LINES` is the end
state.

Each surface assembles the same `Context` shape so the pipeline answers
identically. The desktop app intentionally stays a wrapper: it sends prompts
through the local chat-completions route, links network inspection to the native
links-network route, and uses the browser memory import/export path for
`formal_ai_bundle` round-trips.

---

## 14. GitHub Evidence Collection

Issue #115 adds the first concrete operator workflow for turning external
development traces into local, reviewable memory. `rust/src/github_logs.rs` builds
deterministic GitHub CLI capture plans and can execute them into a case-study
directory. `scripts/mine-hive-mind-dataset.rs` wraps that command with the
focused Hive Mind defaults used by the issue #115 case study.

The collector records:

- repository metadata;
- recent issues, pull requests, and workflow runs;
- selected issue bodies and issue comments;
- selected PR bodies, discussion comments, inline review comments, reviews,
  and diffs;
- selected GitHub Actions run metadata and full logs;
- a `manifest.json` that preserves every command used to produce each file.

This is not a reasoning engine by itself and is intentionally not registered
as a seed agent tool. It is the ingestion boundary for real-world traces from
systems such as `link-assistant/hive-mind`, so later solver work can operate
over observed issue text, PR feedback, work-session summaries, CI outcomes,
and run logs instead of undocumented anecdotes.

---

## 15. Testing Architecture

Tests live under `rust/tests/unit/specification/` and follow three patterns:

1. **Active test** — pins a current implementation behavior. Always green on CI.
2. **Tracked requirement test** — `#[ignore = "tracked requirement: ..."]`. Documents a failing
   expectation without blocking CI. Run with `cargo test --include-ignored`.
3. **Matrix test** — `for (prompt, expected) in [..]` table-driven. Used
   for 5–10 input variations per category per language (issue #103). Today
   we get to 5–10 variations per language with zero new dependencies; for
   external-file catalogs the cleanest upgrade is `datatest-stable` + YAML.

The test module split is intentional: each surface or capability gets its
own file under `rust/tests/unit/specification/`, so a contributor adding a new category
adds one file (or extends one matrix) without touching the rest.
