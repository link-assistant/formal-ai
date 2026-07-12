# Code Audit: Implementation Status vs Vision (2026-07-12)

Read-only audit of the repository at branch `issue-651-0f33c6af0d5a`
(crate version 0.278.0) performed for issue
[#651](https://github.com/link-assistant/formal-ai/issues/651). This is the
ground truth behind the requirement statuses in
[`ROADMAP.md`](../../../ROADMAP.md) and the proposed issue batch in
[`proposed-issues.md`](proposed-issues.md).

## 1. Documentation landscape

- `ROADMAP.md` (323 lines): tracks 26 vision pillars, **all marked Built**;
  records E1–E34 planning epics all closed/merged, plus issue #349
  (reverse-sort), #408 (text/code edit), #538 (detailed meanings), #526
  (translation quality) roadmaps. Contains a "Verification Contract" requiring
  PRs to sync REQUIREMENTS.md + ARCHITECTURE.md + ROADMAP.md together.
- `REQUIREMENTS.md` (1108 lines): per-issue requirement matrix through issue
  #647. The only rows not "Implemented" are the issue #538 follow-ups
  **R378–R386** (see §6 below) plus scattered "tracked as follow-up" notes
  (R271 source-response importers, R282 grounding breadth).
- `ARCHITECTURE.md` (1186 lines): pipeline status table (§2) marks all 9
  pipeline steps Implemented; step 9 (skill compilation) qualified as
  "Implemented for deterministic trigger/response skill packages".
- `docs/USER-JOURNEYS.md`: J1–J11 supported today; "Potential Future Journeys"
  **F1–F6** are the explicit not-done list: F1 visual link network beside
  chat, F2 compiled NL skills (partially scaffolded), F3 cloud memory sync,
  F4 search/evolutionary solving, F5 WebVM browser execution, F6 shared
  associative packages.

## 2. Capability status by pillar

### Server / protocol surface — Built

Evidence: `src/server.rs` (route table), `src/protocol.rs`,
`src/protocol_responses.rs`, `src/responses_stream.rs`, `src/anthropic.rs`,
`src/gemini.rs`, `src/proxy.rs`.

- **OpenAI Chat Completions**: `POST /v1/chat/completions`
  (+ `/api/openai/v1/...`), with SSE streaming
  (`chat_completion_sse_response`) — Built; integration test
  `tests/integration/openai_chat_streaming.rs`.
- **OpenAI Responses API**: `POST /v1/responses` with the full SSE event
  sequence (`response.created` / `response.in_progress` / … in
  `src/responses_stream.rs`), Codex-compatible `cmd`-argument mapping in
  `src/protocol_responses.rs` — Built;
  `tests/integration/openai_responses_streaming.rs`.
- **Anthropic Messages**: `POST /v1/messages`, in-process tool-aware
  translation to chat completions incl. `tool_use`/`tool_result` blocks and
  Anthropic SSE (`src/anthropic.rs`) so the `claude` CLI works via
  `ANTHROPIC_BASE_URL` — Built.
- **Gemini + Vertex**: `GET /api/gemini/v1beta/models`, `generateContent` and
  `streamGenerateContent` (incl. `/api/vertex/v1/projects/.../publishers/google/models/...`
  paths) via `handle_dynamic_protocol_route` — Built.
- Extras: `/v1/graph`, `/v1/links`, `/v1/links/query` (LinksQL),
  `/v1/memory{,/since,/import}`, `/v1/bundle`, `/telegram/webhook`, `/health`.
- **`src/proxy.rs`** is a separate `formal-ai proxy` logging proxy (JSONL
  request/response/tool-call capture), not a protocol adapter — Built;
  `tests/integration/logging_proxy.rs`.
- Multi-protocol coverage pinned by `tests/integration/multi_protocol_api.rs`;
  `src/bin/with-formal-ai.rs` wraps external agent CLIs.

### Agent / agentic coding — Built (bounded/deterministic by design)

- `src/agent.rs`: isolated temp workspace, path validation, **allowlisted
  commands only** (with built-in `cat`/`ls` fallbacks), no host env
  inheritance, 2s default time budget, every action projected into Links
  Notation events.
- `src/agentic_coding/` (issue #468): deterministic `planner.rs` classifies
  advertised tool names into capabilities (`Search/Fetch/Read/Write/Run`) with
  capability-class permission grants (`tool:capability:*`), emits tool calls /
  consumes results / observes errors over any agentic CLI; `driver.rs` runs
  the full loop offline; `file_read.rs` (issue #627) handles direct-read and
  list-then-read recipes. Numerous recipe modules (formalize, diagram,
  self_ast, source_graph, change_request, repair_strategy, rebuild_plan,
  ledger, self_heal, meaning_detail, google_trends, question_catalog).
- **Caveat**: the planner is recipe/task-driven (explicit `is_*_task`
  recognizers with pinned `*_TASK` constants), not an open-ended planner —
  REQUIREMENTS R388 itself records this as "recipe-driven Agent CLI boundary".
  This is deliberate (neural inference is a NON-GOAL) but means "agentic
  coding" ≠ general autonomous coding.
- `src/coding/` holds program blueprints/CST/catalog; `src/code_editing.rs` is
  narrow scoped post-processing (hello-world output replacement).

### Self-improvement / self-inspection — Built as proposal-only, human-gated slices

- `src/self_improvement.rs` (#364): unknown-trace → candidate seed-rule
  proposals, gated by the coding-modification benchmark; never writes back to
  `data/seed/`.
- `src/meta_self_improvement.rs` (#559 R340): detects drift between the
  algorithm-as-data (`data/meta/recursive-core-recipe.lino`) and
  algorithm-as-code (`meta_core::record_meta_core` stages), proposes a recipe
  update; default mode `Off`, never auto-writes.
- `src/self_source_graph.rs` (#558): whole-repository Rust source ↔ links
  projection, source embedded at build time via `build.rs`
  (`OWNED_SOURCE_FILES`); cheap content-addressed manifest + full CST/AST
  round-trip view.
- `src/self_healing.rs` (#558): composes failure trace → source mapping →
  candidate lesson → benchmark gate → human review into an auditable
  `RepairCase`; **proposal-only**.
- `src/dreaming*.rs` (#540): idle-time memory maintenance planner +
  generalization of user requirements into `meta_algorithm_amendment`
  records; `dreaming_application.rs` injects retained amendments into future
  solves; `dreaming_runtime.rs` runs default-on background learning, deletion
  default-off without consent.
- Net: the "auto-learning / recompile itself" vision (issue #558) is
  **Partial** — every stage exists but promotion is intentionally
  human-gated; issue #558's follow-up scope (R385, "arbitrary auto-learning")
  is not yet done.

### Web / WASM / desktop / VS Code — Built shells; WASM parity Partial

- Web demo: `src/web/` — `index.html`, React app (`app/main.jsx` → `app.js`),
  IndexedDB memory mirror (`memory.js`), i18n catalogs, OCR bundle, e2e
  Playwright suites in `tests/e2e/`.
- **WASM worker (`src/web/wasm-worker/src/lib.rs`)**: `no_std` crate that
  includes only `language.rs`, `arithmetic.rs`, `web_engine_core.rs`,
  `web_search_core.rs` → `formal_ai_worker.wasm`. **The bulk of solver logic
  still lives in JavaScript**: `src/web/worker/formal_ai_worker_00..21.js`,
  ~26,658 lines across 22 files. REQUIREMENTS **R380 explicitly marks this
  Partial** ("absorbing the remaining `src/web/worker/*.js` logic is the
  tracked follow-up"). This is the biggest code gap vs. the "Rust-to-WASM
  parity, JS reserved for UI/glue" pillar (ROADMAP pillar 18 says "Built",
  which refers to behavioral parity; implementation-language parity is
  Partial).
- Desktop: `desktop/` Electron app (v0.212.0, `agent-commander` +
  `electron-updater`, electron-builder targets for Linux/mac/Windows, GitHub
  publish, auto-update, 10 node test scripts + smoke) — Built, released by
  `.github/workflows/desktop-release.yml`.
- VS Code: `vscode/` extension v0.154.0, dual desktop+web host, opt-in local
  `formal-ai serve`, permission-gated tools, network view; **not on the
  Marketplace yet** (README says install manually via `.vsix`) — Built but
  unpublished. The R383 "interactive step-by-step debugging view
  (chat/data/mermaid/Rust/JS panes)" is **Open** (only exploratory notes under
  `docs/vscode/`).

### Store — doublets-rs is the real default

- `Cargo.toml`: `default = ["doublets-native"]`, `doublets = "0.4.0"` +
  `platform-mem`; `src/link_store.rs` documents doublets-rs as the default
  native physical store with `.lino` as the deterministic export/import
  projection and `--no-default-features` falling back to the `MemoryStore`
  projection. Browser mirrors via IndexedDB (`src/web/memory.js`). Matches
  ROADMAP pillar 1 / E15 (#278).
- No contradicting persisted table/graph data structures found: "table" hits
  in `src/` are lookup/precedence tables in code (e.g. `SPECIALIZED_HANDLERS`
  precedence table behind the formalized router — ROADMAP pillar 20 notes
  this remnant) and the generated `src/arithmetic_word_tables.rs`, which is
  itself **generated from seed `.lino` meanings** with a parity test —
  associative-first is respected.

### Benchmarks — Built with ratchets

`data/benchmarks/`: 4 suites, all with `minimum_pass_count` ratchets asserted
in CI:

- `industry-suite.lino` (HumanEval/MBPP/GSM8K/MATH/BIG-bench slice): 10
  cases, floor **10/10**.
- `coding-modification-suite.lino`: floor **4** (multilingual multi-turn
  coding modification).
- `text-manipulation-suite.lino` (issue #408): floor **1,440** generated
  profile checks (48 sources × 30 variations).
- `procedural-howto-suite.lino`: floor **12**.

## 3. Test suite shape

- `tests/{unit,integration,e2e,source}`; `tests/unit/` has 77 entries incl.
  `specification/` with 77 spec files; `tests/integration/` has 25 files
  (protocol streaming, agent mode, self-healing, issue reproductions);
  `tests/e2e/` is Playwright (3 configs).
- ~2,066 `#[test]` annotations under `tests/`.
- **Only 2 `#[ignore]` tests**, both with legitimate reasons (not deferred
  requirements):
  - `tests/unit/issue_558_source_graph.rs:275` — exhaustive whole-repo
    CST/AST parse, "minutes in debug".
  - `tests/unit/specification/coding_modification_benchmarks.rs:115` —
    network benchmark behind `FORMAL_AI_BULK_BENCHMARK=1`.
- The original 69 `#[ignore = "tracked requirement"]` tests were all
  graduated (per the ROADMAP audit history).

## 4. "Graph"/"table" terminology audit (issue #651 associative-only check)

~50 `src/*.rs` files mention "graph", but essentially all are **naming, not
architecture**:

- `GET /v1/graph` endpoint (`src/server.rs::handle_graph_request`) — serves
  the *visual network view* of links; the VS Code/desktop UIs call it
  "link-graph network view". The name contradicts the "only links networks"
  vocabulary but the data is doublet links.
- `src/self_source_graph.rs`, `src/agentic_coding/source_graph.rs` — both are
  source↔links projections; doc comments consistently say "links / meta
  language". Pure naming.
- Others are Wikidata/Wiktionary "knowledge graph" citations, the seeded
  user-facing "graph" concept (issue #161), and the codecov badge URL.
- Terminology-purity candidates to rename: `/v1/graph` → `/v1/network` (with
  a compatibility alias), `self_source_graph.rs`/`source_graph.rs` →
  `*_source_links`, and UI "network view"/"link-graph" strings.
- `src/arithmetic_word_tables.rs` is generated from seed `.lino` meanings, so
  it is acceptable as a build artifact but renameable.

## 5. Release / version-bump mechanics

Single pipeline in `.github/workflows/release.yml` (plus
`desktop-release.yml` for Electron artifacts):

1. **In the PR**: add a changelog fragment
   `changelog.d/YYYYMMDD_HHMMSS_description.md` with frontmatter
   `---\nbump: patch|minor|major\n---` and `### Added/Changed/Fixed/...`
   sections. The `changelog` job runs `scripts/check-changelog-fragment.rs`
   and **fails code-changing PRs without a fragment** (docs-only PRs exempt
   via `detect-code-changes.rs`).
2. **Never bump versions manually**: the `version-check` job runs
   `scripts/check-version-modification.rs` and fails PRs that touch
   `Cargo.toml`/`package.json` versions — versions are only modified by the
   pipeline.
3. **On push to `main`** (after lint/test/build pass), the `auto-release`
   job: `get-bump-type.rs` reads fragments → `check-release-needed.rs`
   verifies against crates.io/GitHub releases/Docker Hub → 
   `version-and-commit.rs` bumps `Cargo.toml`, collects fragments into
   `CHANGELOG.md`, commits and tags → `publish-crate.rs` (crates.io) →
   GHCR/Docker Hub images → `create-github-release.rs`.
4. Manual fallback: `workflow_dispatch` with `release_mode` (incl.
   `changelog-pr`) and explicit `bump_type`/`description`.
5. Desktop (`desktop/package.json` 0.212.0) and VS Code
   (`vscode/package.json` 0.154.0) are versioned independently of the crate
   (0.278.0), released via `desktop-release.yml` / vsix scripts.

## 6. Concrete gaps found

1. **R380 Rust→WASM worker (biggest code gap)**: ~26.7k lines of solver
   logic in `src/web/worker/*.js` vs. a thin `no_std` WASM worker covering
   only language detection/arithmetic/search cores. Behavioral parity is
   pinned by shared fixtures; implementation-language parity is Partial.
2. **R378 bulk semantics import** — Partial: curated tomato/potato entries
   reproduced via Agent CLI recipes; batch importer generalizing
   `scripts/ground-meanings.rs` not built.
3. **R379 hardcoded-string audit** — Partial: no CI lint over `src/` for
   user-facing literals outside the lexicon (design doc exists at
   `docs/design/no-hardcoded-natural-language.md`).
4. **R383 interactive VS Code debugging view** — Open: only exploratory
   notes in `docs/vscode/`.
5. **R384 fully-inspectable meta algorithm** — Partial: method registry +
   self-AST exist; probability-weighted statement formalization and
   contradiction warnings/repair not implemented.
6. **R385/issue #558 arbitrary auto-learning** — Partial by design: all loop
   stages exist (`src/self_healing.rs`) but are proposal-only/human-gated;
   no automatic promotion protocol.
7. **USER-JOURNEYS F1–F6 not fully delivered**: cloud memory sync (F3),
   budget-driven random/evolutionary search (F4), WebVM execution (F5),
   shareable associative packages export/import between instances (F6);
   F1 (visual network) largely exists via `/v1/graph` + network views — the
   doc's ○ marks look stale relative to pillar 13 "Built".
8. **Naming contradictions**: `/v1/graph`, `self_source_graph.rs`,
   `source_graph.rs`, "link-graph network view" UI strings vs. the "only
   links networks / links notation / meta language" principle.
9. **VS Code extension not on the Marketplace** (README admits this);
   desktop/vscode/crate version triple (0.278.0 / 0.212.0 / 0.154.0) can
   confuse release notes.
10. **`SPECIALIZED_HANDLERS` precedence table** still sits behind the
    formalized intent router (acknowledged in ROADMAP pillar 20) — a remnant
    of the fixed-catalogue era.
11. **Recipe-bounded agentic mode**: agentic_coding recognizers are pinned
    per-task (`is_*_task` + canonical constants), so agentic capability
    generalizes only to encoded recipes (self-acknowledged in
    `docs/case-studies/issue-558/pr-601-gap-analysis.md` per R388).
12. README claims check out against code (protocol namespaces, installer
    targets, WASM worker, desktop/VS Code shells all exist); no
    unimplemented README claims found beyond the Marketplace caveat it
    already discloses.

For roadmap status marking: the honest Partial rows are **R378, R379, R380,
R384** and **R383 (open)**, plus USER-JOURNEYS F2–F6 as Open/Partial, the
associative-terminology cleanup, VS Code Marketplace publication, general
(non-recipe) agentic planning, and the self-coding promotion protocol.
