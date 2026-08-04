# Living-Documentation Audit — link-assistant/formal-ai

Audited at v0.326.0 (main, 2026-08-04). Read-only audit; this file is the edit plan.
Every edit lists its pin-test impact. Pin tests live in `tests/unit/docs_requirements*.rs`,
`tests/unit/docs_requirements/`, `tests/unit/specification/*`, `tests/unit/user_journeys.rs`,
and `tests/unit/issue_538_agentic.rs` (byte-pin of `docs/diagrams/agentic-recipes.md`).

Conventions respected: REQUIREMENTS.md R-numbering (plain rows currently end at **R535**;
newer work uses per-issue `R<issue>-N` blocks); ROADMAP E-epics (#651 umbrella, E69–E77 =
#916–#924). `docs/diagrams/agentic-recipes.md` and committed Agent CLI session JSONs are
generated/byte-pinned — never hand-edited; fix = regenerate via generator.

---

## Root docs

### README.md — verdict: accurate (2 minor findings)

Verified against code: both binaries exist (`formal-ai` in Cargo.toml `[[bin]]`;
`with-formal-ai` at `src/bin/with-formal-ai.rs`); all CLI subcommands quoted
(chat/dataset/serve/proxy/memory/bundle/telegram/with/agent/report/file-legality/
github-logs/environments/clients/import/benchmark/improve/learn/…) exist in
`src/main.rs::Command`; memory subcommand flags (`--events-only`, `--confirm`,
`--backup`, `--storage-capacity-bytes`) exist; wrapper flags (`--no-start-server`,
`--start-server`, `--base-url`, `--protocol`, `--global`, `--undo`, `--all`,
`--non-interactive`, `--summarize` alias `keep-summarization`) exist in
`src/client_integrations.rs`; model aliases match `data/seed/model-aliases.lino`;
API namespaces match `src/server.rs`; compose profiles (`""`,`telegram`,`server`,
`agent`,`all`) match `compose.yaml`; Dockerfile base `konard/box-dind:2.1.1` matches
`Dockerfile:10`; env vars (`FORMAL_AI_API_BEARER_TOKEN`, `FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR`,
`FORMAL_AI_MEMORY_PATH`, `FORMAL_AI_DIALOG_LOG_DIR`, `FORMAL_AI_OPENCODE_DESKTOP_BIN`,
`LINK_ASSISTANT_AGENT_CONFIG_CONTENT`) all exist in `src/`; every relative link target
exists (checked all path-like references); npm/bun/rust-script commands exist;
`verify-formal-ai-dind` exists (`Dockerfile:37`); desktop health-check commands match
`desktop/lib/service-control.cjs:41-43`; hello-world language list and rule list match seed.

Findings:

1. [README.md:990] Summarization split list says the pipeline is
   "split into `mod.rs`, `markdown.rs`, `dialog.rs`, `file.rs`, and `resource.rs`" —
   `src/summarization/` actually contains 13 files (also `context.rs`, `dedup.rs`,
   `gathering.rs`, `identifier.rs`, `importance.rs`, `pipeline.rs`, `recheck.rs`,
   `vocabulary.rs`).
   EDIT: `(split into \`mod.rs\`, \`markdown.rs\`, \`dialog.rs\`, \`file.rs\`, and \`resource.rs\`)`
   → `(split across \`src/summarization/\` — core stages in \`mod.rs\`, with \`markdown.rs\`, \`dialog.rs\`, \`file.rs\`, \`resource.rs\`, \`dedup.rs\`, and further stage modules)`
   Pin impact: none (grep of "split into `mod.rs`" over tests/ has no hits).
2. [README.md:1073] "See REQUIREMENTS.md for the cumulative requirement matrix (now
   alongside VISION.md)" — accurate; no change. (Noted only because the "(now
   alongside …)" parenthetical is an old-move remark; harmless.)
   EDIT: none (optional cleanup only — drop the parenthetical). Pin impact: none.

Missing coverage (README): the issue #903 / PR #925 wrapper behavior shipped
2026-08-04 (caller argv is parsed into a structured request and re-rendered per
client; piped prompts become each client's own prompt argument; `isatty(stdin)`
decides interactive mode). README's `formal-ai with` section predates it but does
not contradict it. OPTIONAL EDIT (add after README.md:297 paragraph):
`Arguments after the tool name are parsed into a structured request (prompt plus options) and re-rendered in each client's own vocabulary, so a flag the wrapper also defines is still forwarded, a piped prompt becomes that client's prompt argument, and interactive mode follows whether stdin is a terminal.`
Pin impact: none.

### VISION.md — verdict: stale (1 real finding + doctrine interplay)

1. [VISION.md:235] "it grew to a 10-case slice and passes **10/10** with a
   `minimum_pass_count` ratchet" — stale. `data/benchmarks/industry-suite.lino`
   now has 13 cases and `minimum_pass_count "13"` (updated_at 2026-07-30).
   EDIT: `it grew to a 10-case slice and passes **10/10** with a` + backtick
   `minimum_pass_count` + ` ratchet`
   → `it grew to a 13-case slice and passes **13/13** with a \`minimum_pass_count\` ratchet`
   (Keep the rest of the sentence; the GSM8K `18` / MATH `11` / BIG-bench `3`
   answers are per-case values and still correct.)
   Pin impact: none (no test pins "10-case"/"passes **10/10**").
2. Doctrine interplay (see "Standing doctrine" section below):
   [VISION.md:158] rule shape 3 "Compiled JS handlers — registered through the
   browser worker …" presents JS handlers as a permanent first-class rule shape,
   which now conflicts with the owner's standing doctrine. EDIT (append to the
   shape-3 sentence): `; under the compiled-logic doctrine this shape is transitional — logic belongs in compiled Rust (native or WASM), and the JS registration surface remains only until the worker absorption (issue #658) completes`
   Pin impact: none ("Compiled JS handlers" not pinned).
   Pinned strings that must survive (docs_requirements.rs): `# Vision`,
   `associative operational space`, `Links Data Store`, `Add-only history`,
   `dynamic type system` — none touched by these edits.

### GOALS.md — verdict: accurate

All claims verified (surfaces, orchestration status paragraph matches
`src/orchestration/`, self-evolution status matches #656/#657 shipped state).
Pinned strings `# Goals`, `smallest useful seed dataset`, `transparent reasoning`,
`chat-first`, `isolated execution` untouched. No edits.
Optional doctrine hook: add one bullet under "Architecture Goals" (see doctrine
section, edit D4-optional).

### NON-GOALS.md — verdict: accurate

Pinned strings `# Non-Goals`, `memoized answer cache`, `GPU-required neural
inference`, `Hidden autonomous actions` untouched. No edits.

### ROADMAP.md — verdict: stale (3 findings)

1. [ROADMAP.md:133] Pillar 18 row says status **Built** ("Rust-to-WebAssembly
   parity with JavaScript reserved for UI/glue | Built | `src/web_engine_core.rs`
   plus the browser worker | Worker logic moved into Rust/WASM by #282 (PR #291)").
   ARCHITECTURE.md:1114-1125 states the opposite, honestly: pillar 18 "describes
   the target, not today's split"; `src/web/worker/*.js` still carries ~27,700
   lines of solver logic (measured: 27,705 across 24 shard files) vs a 1,710-line
   WASM worker crate. The two living docs contradict each other; ARCHITECTURE is
   the one matching reality.
   EDIT (whole row): `| 18 | Rust-to-WebAssembly parity with JavaScript reserved for UI/glue | Built | \`src/web_engine_core.rs\` plus the browser worker | Worker logic moved into Rust/WASM by [#282](https://github.com/link-assistant/formal-ai/issues/282) (PR #291). |`
   → `| 18 | Rust-to-WebAssembly parity with JavaScript reserved for UI/glue | Partial | \`src/web_engine_core.rs\`, \`src/web/wasm-worker/\` own the parity-sensitive primitives (#282, PR #291); \`src/web/worker/*.js\` still carries ~27,700 lines of mirrored solver logic under the \`scripts/check-worker-line-budget.rs\` shrink-only ratchet | Absorbing the remaining worker logic into Rust→WASM is tracked by [#658](https://github.com/link-assistant/formal-ai/issues/658) (R380); see ARCHITECTURE.md §13. |`
   Pin impact: none ("Rust-to-WebAssembly parity" not pinned by any test).
2. [ROADMAP.md:140-141] Pillar 25 "The suite now reports **10/10 passing**" and
   pillar 26 "The benchmark suite passes 10/10" — present-tense, stale (suite is
   13 cases / floor 13; see VISION finding 1).
   EDIT (row 140): `The suite now reports **10/10 passing** with a` → `The suite now reports **13/13 passing** with a`
   EDIT (row 141): `The benchmark suite passes 10/10;` → `The benchmark suite passes 13/13;`
   (Lines 12, 84, 190 are dated audit narration — "the 2026-05-29 fifth pass
   records …" — leave as history.)
   Pin impact: none.
3. [ROADMAP.md:406] "about 19,600 lines across 40 files remain in
   `src/solver_handlers/`" — measured today: 19,621 lines across **46** `.rs`
   files (recursive). Line count fine; file count off.
   EDIT: `about 19,600 lines across 40 files remain in` → `about 19,600 lines across 46 files remain in`
   Pin impact: none.
   (Minor, non-blocking: [ROADMAP.md:128] pillar 13 evidence cites `/v1/graph`,
   which is now the deprecated alias of `/v1/network` (src/server.rs:205-211).
   OPTIONAL EDIT: `\`/v1/graph\`` → `\`/v1/network\``. Pin impact: none.)

### REQUIREMENTS.md — verdict: stale (6 findings, all evidence-cell path drift)

The matrix itself (through the R914 block, rows R914-1…R914-15) is current.
Automated existence check of every path-like reference found exactly these breaks:

1. [REQUIREMENTS.md:307] R150 cites `src/solver_handlers/definition_merge.rs` —
   file no longer exists; the handler was migrated by #699; logic now lives in
   `src/definition_merge.rs` (dispatched via the method registry).
   EDIT: `the \`definition_merge\` specialized handler in \`src/solver_handlers/definition_merge.rs\``
   → `the \`definition_merge\` method in \`src/definition_merge.rs\` (migrated out of \`solver_handlers/\` by #699)`
   Pin impact: none.
2. [REQUIREMENTS.md:398] R193 cites `changelog.d/20260519_140000_issue_133_default_duckduckgo_rrf.md`
   — fragment files are deleted at release when collected into CHANGELOG.md, so
   this evidence path can never exist on main.
   EDIT: `Implemented by \`changelog.d/20260519_140000_issue_133_default_duckduckgo_rrf.md\`, which declares \`bump: minor\``
   → `Implemented by the issue #133 fragment \`20260519_140000_issue_133_default_duckduckgo_rrf.md\` (declared \`bump: minor\`; collected into \`CHANGELOG.md\` at release, as all fragments are)`
   Pin impact: none.
3. [REQUIREMENTS.md:733] R274 cites `changelog.d/20260606_201500_issue_398_semantic_facets.md`
   — same deleted-at-release problem.
   EDIT: `Implemented by \`changelog.d/20260606_201500_issue_398_semantic_facets.md\`.`
   → `Implemented by the issue #398 fragment \`20260606_201500_issue_398_semantic_facets.md\` (collected into \`CHANGELOG.md\` at release).`
   Pin impact: none.
4. [REQUIREMENTS.md:419,424,425,428] R197/R202/R203/R206 cite `src/summarization.rs`
   — the module is now the directory `src/summarization/` (mod.rs + 12 stage files);
   later rows (R345, R355, R501) already use the new paths.
   EDITS (4 rows, mechanical): every `` `src/summarization.rs` `` → `` `src/summarization/mod.rs` ``;
   in R197 also `18 unit tests in \`src/summarization.rs::tests\`` →
   `unit tests in \`src/summarization/mod.rs::tests\``;
   in R203 also `(\`src/solver_handlers/mod.rs::try_summarize_conversation\`)` →
   `(\`src/solver_handlers/conversation_memory/conversation_summary.rs::try_summarize_conversation\`)`
   (verified: the fn is at conversation_summary.rs:61).
   Pin impact: none (no test pins these strings).
5. [REQUIREMENTS.md:249] R129 cites `tests/unit/specification/code_generation.rs` —
   now the directory `tests/unit/specification/code_generation/` (mod.rs,
   single_turn.rs, follow_up.rs, task_catalog.rs).
   EDIT: `\`tests/unit/specification/code_generation.rs\`` → `\`tests/unit/specification/code_generation/\``
   Pin impact: none.
6. [REQUIREMENTS.md:134,136 and ~15 other rows] cite `src/solver_helpers.rs` —
   now the directory `src/solver_helpers/` (ARCHITECTURE.md:942 already uses
   `src/solver_helpers/code.rs`). Because these are many historical evidence
   cells, the minimal correct fix is a targeted replace in the two R86/R88 rows
   plus any other row where the exact file is named (mechanical
   `\`src/solver_helpers.rs\`` → `\`src/solver_helpers/\``, replace_all across
   REQUIREMENTS.md only).
   Pin impact: none (checked: no docs_requirements test pins "solver_helpers.rs").

Notes (no edit): `raw-data/online-research.md`-style relative mentions inside
case-study evidence cells are contextual prose next to the case-study path, not
links — left alone. `docs/REQUIREMENTS.md` at line 174 describes the historical
move ("moving `docs/REQUIREMENTS.md` → `REQUIREMENTS.md`") — correct as written.
`src/dispatch_parity.rs` at line 925 is described as removed — correct.

### ARCHITECTURE.md — verdict: stale (5 findings)

1. [ARCHITECTURE.md:1103] "`src/web/worker/formal_ai_worker_00.js` … `_21.js`" —
   shards now run `_00.js` … `_23.js` (24 files).
   EDIT: `\`src/web/worker/formal_ai_worker_00.js\` … \`_21.js\`` → `\`src/web/worker/formal_ai_worker_00.js\` … \`_23.js\``
   Pin impact: none.
2. [ARCHITECTURE.md:1115-1116] "The WASM bridge (`src/web/wasm-worker/src/lib.rs`)
   is ~500 lines, while `src/web/worker/*.js` still carries roughly 26,700 lines"
   — measured: wasm-worker crate is 1,710 lines total (lib.rs 898, plus
   memory_query_worker.rs 734 and proof_translation_worker.rs 78); worker JS is
   27,705 lines.
   EDIT: `The WASM bridge (\`src/web/wasm-worker/src/lib.rs\`) is ~500 lines, while \`src/web/worker/*.js\` still carries roughly 26,700 lines of solver logic`
   → `The WASM worker crate (\`src/web/wasm-worker/src/\`) is roughly 1,700 lines, while \`src/web/worker/*.js\` still carries roughly 27,700 lines of solver logic`
   Pin impact: none.
3. [ARCHITECTURE.md:1203] "grew to a 10-case slice that passes **10/10**" —
   same staleness as VISION finding 1 (suite is 13/13). This sentence narrates a
   2026-05 audit but reads present-tense.
   EDIT: `grew to a\n10-case slice that passes **10/10** with a \`minimum_pass_count\` ratchet`
   → `grew to a\n10-case slice that passed **10/10** with a \`minimum_pass_count\` ratchet (13 cases / 13-floor today — see \`data/benchmarks/industry-suite.lino\`)`
   Pin impact: none.
4. [ARCHITECTURE.md:1314] References list says "R1 … R444, plus per-issue blocks
   such as R499-1…R499-8" — plain rows now end at R535 and per-issue blocks reach
   R914-15.
   EDIT: `issue-by-issue implementation matrix (R1 … R444, plus per-issue blocks such as R499-1…R499-8).`
   → `issue-by-issue implementation matrix (R1 … R535, plus per-issue blocks such as R499-1…R499-8 and R914-1…R914-15).`
   PIN IMPACT: **must update tests/unit/docs_requirements_issue_451.rs:40**, which
   pins the exact string `"R1 \u{2026} R444"` in ARCHITECTURE.md → change the pin
   to `"R1 \u{2026} R535"` in the same PR.
5. [ARCHITECTURE.md:711-713] §7.1 says "`try_summarize_conversation` in
   `src/solver_handlers/mod.rs` now collects …" — the fn moved to
   `src/solver_handlers/conversation_memory/conversation_summary.rs:61`.
   EDIT: `\`try_summarize_conversation\` in\n  \`src/solver_handlers/mod.rs\` now collects` → `\`try_summarize_conversation\` in\n  \`src/solver_handlers/conversation_memory/conversation_summary.rs\` now collects`
   Pin impact: none.

Everything else verified: pipeline module table, handler-precedence seed story
(`data/seed/handler-precedence.lino`, `src/solver_dispatch.rs`,
`src/method_registry.rs`), dreaming stack (`src/dreaming*.rs`,
`src/storage_policy.rs`, `desktop/lib/dreaming.cjs`,
`FORMAL_AI_DESKTOP_DREAMING`), `/api/formal-ai/v1/network` + deprecated `/graph`
alias (src/server.rs:205-216), SolverConfig knob table, translation pipeline
modules, `solver_handler_units.rs` / `solver_handler_how.rs` /
`solver_handlers_policy.rs` all present, world-model/persistence references
(`src/world_model.rs`, `src/associative_persistence.rs`,
`src/relative_meta_logic.rs`).

### CONTRIBUTING.md — verdict: accurate (1 minor finding)

Verified: `test-agent-cli-e2e` job exists (release.yml:758), driver script
exists, `examples/self-coding/run.sh` exists, `self_coding_session_replays`
test exists (tests/unit/self_coding.rs), all referenced scripts and guards
exist (`check:web-hardcoded-ui` / `check:i18n` in tests/e2e/package.json,
`scripts/audit-total-closure.py`, `scripts/close-total.py`,
`scripts/generate-role-registry.py`, `scripts/self-hosting-metric.rs`,
`scripts/reproduce-issue-538.sh`), file-size caps (Rust 1000; `.lino`/worker JS
1500) match `scripts/check-file-size.rs`, project-structure tree matches the
repository.

1. [CONTRIBUTING.md:382] Convention 1 says every Rust reasoning path "has a twin
   in the browser worker `src/web/formal_ai_worker.js`" — that file is now a
   small loader shim (per ARCHITECTURE.md:1101); the twins live in
   `src/web/worker/formal_ai_worker_*.js`.
   EDIT: `has a twin in the browser worker \`src/web/formal_ai_worker.js\`` →
   `has a twin in the browser worker (\`src/web/formal_ai_worker.js\` loader plus the \`src/web/worker/formal_ai_worker_*.js\` shards)`
   Pin impact: none.
   (Doctrine cross-reference edit D5 below also touches this convention.)

### LEGAL-COMPLIANCE.md — verdict: accurate

All referenced files exist (`data/training/source-registry.json`,
`data/benchmarks/LICENSES.md`, `data/seed/sources-registry.lino`, all
docs/legal/* companions). `data/training/artifacts/` does not exist as a
directory — consistent with the stated empty/fail-closed state ("may exist only
under", CI compares files when present). "Last reviewed: 2026-08-01" is honest.
No edits.

---

## Standing doctrine to codify (owner directive, 2026-08-04)

Doctrine text: **JS code must be used only as interfacing glue and for JSX
(React) UI components. All logic must be in compiled Rust — natively on the
server side, as WebAssembly in the web app — and the same WASM web engine must
be reused in desktop and other surfaces (VS Code, etc.).**

Reality check (verified):
- Today's split: `src/web/wasm-worker/` (1,710 lines Rust→WASM, built by
  `src/web/wasm-worker/build.sh` into `src/web/formal_ai_worker.wasm`) owns
  parity-sensitive primitives; `src/web/worker/*.js` still holds ~27,705 lines
  of mirrored solver logic. `scripts/check-worker-line-budget.rs` already
  enforces a shrink-only ratchet whose header states exactly this doctrine
  ("moved out of `src/web/worker/*.js` and into the Rust→WASM worker, leaving
  JavaScript as UI glue"). Absorption is tracked by issue #658 (R380), blocker
  for #665.
- Web app UI: React (`src/web/app/main.jsx` built to `src/web/app.js`) — the
  JSX/UI part of the doctrine matches the shipped architecture.
- Desktop: Electron starts a loopback native `formal-ai serve` and serves the
  same `src/web/` app, so it reuses the same web engine bundle (JS worker +
  WASM worker) as fallback but routes prompt sends to the native Rust API
  (README.md:759, ARCHITECTURE.md:1127-1131). So "same WASM web engine reused
  in desktop" is true for the bundled engine, with native Rust preferred when
  available — consistent with "logic in compiled Rust" (native there).
- VS Code: web host runs the in-process WASM engine; Node host opt-in starts
  the native server (README.md:809-819, ARCHITECTURE.md:1096-1100). Matches
  the doctrine's direction.
- Existing partial codifications: REQUIREMENTS R194, R249, R380; ROADMAP
  pillar 18; ARCHITECTURE §13 honest-state paragraph. None states the doctrine
  as a standing, enforced requirement covering desktop/VS Code reuse and the
  JSX-only-UI rule.

Doctrine edits:

D1. REQUIREMENTS.md — add a new top-level row continuing the plain sequence
    (next free number after R535 is **R536**), in a new dated section appended
    after the "Issue #914 …" block (end of file), following house style:

    ```markdown
    ## Standing Doctrine: Compiled Logic, Interfacing-Only JavaScript (2026-08-04)

    Stated by the project owner as a standing architectural requirement; it
    strengthens R194/R249/R380 from "as much as possible" to a boundary rule.

    | ID | Requirement | Status / Evidence |
    | --- | --- | --- |
    | R536 | JavaScript must be used only as interfacing glue and for JSX (React) UI components. All logic must be compiled Rust — native on the server side, WebAssembly in the web app — and the same WASM web engine must be reused by the desktop shell and other surfaces (VS Code, etc.) rather than reimplemented. | Partially implemented: `src/web/wasm-worker/` owns the parity-sensitive primitives and `src/web/app/main.jsx` is the JSX UI; `src/web/worker/*.js` still carries ~27,700 lines of mirrored solver logic under the shrink-only ratchet `scripts/check-worker-line-budget.rs`. Desktop serves the same `src/web/` engine bundle and prefers the native `formal-ai serve` process; the VS Code web host runs the in-process WASM engine. Full absorption of the JS worker into Rust→WASM is tracked by [#658](https://github.com/link-assistant/formal-ai/issues/658) (R380); after absorption the JavaScript surface is capped and lint-enforced as UI/glue. |
    ```
    Pin impact: none (pure addition; no pinned string altered). New pins may be
    added later by a docs_requirements test if desired, but none is required.

D2. ARCHITECTURE.md §13 — after the honest-state paragraph (insert after line
    1125, before "Each surface assembles the same `Context` shape"):

    ```markdown
    **Standing principle (2026-08-04, R536).** JavaScript is interfacing glue
    and JSX (React) UI only. All logic is compiled Rust: native in the CLI,
    server, and desktop-managed processes; Rust→WASM in the web app. The same
    WASM web engine is reused — not reimplemented — by the desktop shell and
    the VS Code hosts. The remaining `src/web/worker/*.js` solver logic is a
    transitional mirror under the shrink-only ratchet
    `scripts/check-worker-line-budget.rs`; it may only move into Rust→WASM
    (issue #658, R380), never grow.
    ```
    Pin impact: none (pure addition).

D3. ROADMAP.md pillar 18 row — already covered by ROADMAP finding 1 above
    (status Built → Partial), which is required for the doctrine to be
    non-contradicted by the roadmap. Pin impact: none.

D4 (optional). GOALS.md "Architecture Goals" — add bullet:
    `- Keep JavaScript as interfacing glue and JSX UI only; compile all logic from Rust (native on servers, WebAssembly in the web app) and reuse the same WASM engine across desktop and editor surfaces.`
    Pin impact: none.

D5. CONTRIBUTING.md convention 1 (Mirror parity) — append after the twin-naming
    sentence (line ~386):
    `Mirror parity is the transitional contract while JS worker logic remains: under the compiled-logic doctrine (REQUIREMENTS.md R536), prefer absorbing the path into the Rust→WASM worker over adding a new JS twin, and never grow the worker line budget (\`scripts/check-worker-line-budget.rs\`).`
    Pin impact: none.

D6. VISION.md §Computation Model — covered by VISION finding 2 above (qualify
    rule shape 3 "Compiled JS handlers" as transitional). Pin impact: none.

Contradiction scan result: no other living doc asserts JS as a *permanent* home
for solver logic. README.md:746 already states the target split correctly
("Rust/WASM owns parity-sensitive worker primitives; JavaScript remains
responsible for UI state, seed fetching, browser fetch/CORS orchestration, and
no-WASM fallbacks") — the "no-WASM fallbacks" clause remains accurate as a
compatibility behavior and needs no change.

---

## docs/desktop/, docs/vscode/, vscode/README, docs/testing/, docs/ci-cd/, docs/design/, docs/legal/

### docs/desktop/server-api.md — verdict: stale (2 findings + missing coverage)

1. [line 112] Endpoint table documents `GET /api/formal-ai/v1/graph` (alias
   `/v1/graph`) as the reasoning-graph route. Since issue #664 the canonical
   route is `/v1/network` (src/server.rs:205 routes `"/v1/network" |
   "/api/formal-ai/v1/network"`); `/v1/graph` is a deprecated alias returning a
   `Deprecation` header + successor `link` (src/server.rs:45-47, 210-212;
   vscode/src/lib/config.cjs:103-104). The doc never mentions `/v1/network`.
   EDIT: `| \`GET\` | \`/api/formal-ai/v1/graph\` | \`/v1/graph\` | Reasoning-graph nodes/edges for a trace |`
   → `| \`GET\` | \`/api/formal-ai/v1/network\` | \`/v1/network\`, \`/v1/graph\` (deprecated alias) | Links-network nodes/edges for a trace |`
   Companion EDIT [line 117]: `native formal-ai \`/v1/*\` graph, bundle, links, and memory routes`
   → `native formal-ai \`/v1/*\` network/graph, bundle, links, and memory routes`
   Pin impact: none (only tests/unit/local_surface.rs:124 touches `/v1/graph`,
   as a code comment, not a doc pin).
2. [line 95] `# {"status":"ok","model":"formal-ai"}` — `/health` also returns
   `version` since 2026-07-23 (src/server.rs:196-203 adds
   `"version": env!("CARGO_PKG_VERSION")`).
   EDIT: `# {"status":"ok","model":"formal-ai"}` → `# {"status":"ok","model":"formal-ai","version":"<crate version>"}`
   Pin impact: none.

Missing coverage (add rows/sections; no pinned strings touched):
`GET/POST /{api/formal-ai/,}v1/conversations/{id}[/learn]` (issue #839,
src/server.rs:336-354); `POST /mcp` missing from the §2 table (src/server.rs:288);
§5d tool list is 6 tools but desktop/lib/tool-router.cjs:31-46 now also serves
`web_search`, `web_fetch`, `read_file`, `grep`, `glob`, `list_directory`,
`read_many_files`; desktop bridge methods `setEngine`, updater
(`checkForUpdates`/`installUpdate`), `installVsCodeExtension`,
`runAgentProvider`/`agentEvent`, `dataMigrationStatus`/`replayDataMigration`
(desktop/preload.cjs:6-50) are undocumented.
All other content verified against code (routes/aliases, auth order, 60 rpm /
60,000 tpm, capacity fields, wrapper flags and 12 seeded tools, persistent
config paths, reasoning fields, memory-path defaults).

### docs/desktop/service-control.md — verdict: accurate
Every claim verified (container/volume names, compose profiles, IPC handlers
desktop/main.cjs:514-546, preload bridges, docker probes, health-check triple,
Dockerfile contents). No edits. Pins (3x) untouched.

### docs/vscode/extension.md — verdict: stale (1 finding)
1. [line 127] "50 `node:test` cases across config/bridge/webview-html/chat-view/
   server-process" — now 51 (config 11, bridge 11, chat-view 11, webview-html 10,
   server-process 8; 51st added 2026-07-26, commit 6fa63d33, after doc's last
   edit 2026-07-18).
   EDIT: `50 \`node:test\` cases` → `51 \`node:test\` cases`
   Pin impact: none.
All other claims verified (entry points, 4 commands, 6 settings with defaults
18080 / `konard/box-dind:2.1.1`, web-host no-Node-builtins spec, binary
resolution order, vendored helpers, mirror-tree skips, CI wiring).

### vscode/README.md — verdict: accurate
Commands, settings, `GET /v1/network` (canonical route, correctly named),
Unlicense — all verified. No edits.

### desktop/README.md — confirmed absent (see Missing docs).

### docs/testing/agentic-cli-tools.md — verdict: accurate
Verified: agentic-cli-matrix.yml triggers/jobs incl. `learn` publishing
`client-contract-learning.lino`; 12 matrix rows = 12 seeded tools; 8 case names
in run_leg.sh; port formula base 8900 stride 60; `clients observe/learn` flags;
`proxy` flags; pinned versions in clients.lock; MCP `formal_ai_chat` + `-32601`;
`--globally` alias; t3 `--no-browser` mapping. No edits. Pins (3x) untouched.

### docs/ci-cd/troubleshooting.md — verdict: accurate
scripts/publish-crate.rs, scripts/rust-paths.rs exist; crates.io API check in
scripts/check-release-needed.rs:104; Pages action versions match release.yml.
No edits.

### docs/design/no-hardcoded-natural-language.md — verdict: accurate
All gates exist and run in the release.yml lint job (:402/:445/:468/:476);
`ALLOWED_LITERALS == {"Tool calls"}`; allowlist 1353 rows; `seed::response_for`
(src/seed.rs:287) and worker `answerFor` confirmed. No edits.

### docs/design/rule-synthesis.md — verdict: accurate
`PROGRAM_MODIFIERS` correctly marked Done/absent from src/intent_formalization.rs;
`lower`/`lower_with_rules` at src/program_plan.rs:203/212; seed files exist.
Pinned by tests/unit/docs_requirements.rs:750-771 — untouched. No edits.

### docs/design/self-improvement-loop.md — verdict: accurate
All four API items at src/self_improvement.rs:51/132/167/475; referenced seed +
benchmark files exist. No edits.

### docs/legal/ (all five) — verdict: accurate
source-registry.json `current_state: no-approved-training-sources`,
`last_audited: 2026-08-01` matches every "Reviewed on 2026-08-01" header;
all cross-references exist. `data/training/artifacts/` absent = consistent with
zero approved sources. No edits.

All relative markdown links in this doc group resolve (scripted sweep, zero broken).

---

## docs/configuration/

Pin test for this whole group: **tests/issue_761_docs.rs** — it requires every
seed-registered tool id to have a `## \`<id>\`` heading in agentic-clis.md, the
tools guide to name every `environments.lino` tool, and per-page marker strings
(e.g. server-api.md must contain `cost`, `--agent-mode`, `context_used_fraction`;
languages.md must contain `data/seed/`, `data-only`, `parity`, `English`,
`Russian`, `Chinese`, `Hindi`; output-sessions.md must contain `friendly`,
"```json", `transcript`, `session`, `resume`, `FORMAL_AI_PROXY_LOG`). All edits
below were checked against those needles.

### docs/configuration/README.md — verdict: accurate. No edits.
(Index links all 13 pinned pages plus orchestration.md; install commands match
scripts/install.sh / install.ps1; pinned markers intact.)

### docs/configuration/agentic-clis.md — verdict: stale (2 findings)
1. [lines 107-108] "Restore it with `--global --undo <tool>` (or `--undo <tool>`
   with the standalone `with-formal-ai` wrapper)." — bare `--undo` works with
   `formal-ai with` too: src/client_integrations.rs:176 gates on
   `args.global || args.undo`, and both binaries share `WithFormalAiArgs`
   (src/bin/with-formal-ai.rs). The doc itself uses bare
   `formal-ai with --undo opencode-vscode` at line 181 and
   `formal-ai with --undo opencode-desktop` at line 192.
   EDIT: `Restore it with \`--global --undo <tool>\` (or\n\`--undo <tool>\` with the standalone \`with-formal-ai\` wrapper).`
   → `Restore it with \`--undo <tool>\` (with or without \`--global\`; the standalone \`with-formal-ai\` wrapper accepts the same flags).`
   Pin impact: none — issue_761_docs.rs:88-89 needs the bare substrings
   `--global` and `--undo`, both retained.
2. [lines 24-27] "the seeded completion contract supplies one corrective prompt.
   … A second no-effect result exits nonzero" — stale since 2026-07-31
   (eef340ac "learn which recovery strategy actually produces artifacts"): the
   contract now carries a strategy ladder with `max_attempts 4` and three
   recovery strategies (data/seed/client-completion-contracts.lino:4-11;
   src/client_integrations/completion.rs:175 retries while
   `attempts < max_attempts`).
   EDIT: `the seeded completion contract\nsupplies one corrective prompt. The wrapper resumes the exact native session\nwhen the client exposes one. A second no-effect result exits nonzero with a\nstructured \`completion_state: "incomplete"\` record`
   → `the seeded completion contract\nsupplies a ladder of corrective prompts (one recovery strategy per retry, up\nto the contract's \`max_attempts\`, default 4). The wrapper resumes the exact\nnative session when the client exposes one. Exhausting the ladder with no\neffect exits nonzero with a structured \`completion_state: "incomplete"\` record`
   Pin impact: none.
Everything else verified: per-tool sections match every `tool "<id>"` in
data/seed/client-integrations.lino (enforced by the pin test); the Exa bridge
claim in the `## opencode` section is correct (`OPENCODE_ENABLE_EXA=1` at
client-integrations.lino:140, also set for opencode-vscode/:201 and
opencode-desktop/:240 — the doc mentions it only under opencode, which is
where users see it; no edit required); 15-minute default deadline =
`timeout_seconds` default 900 (src/cli_orchestration.rs:95); `.formal-ai/`
exclusion via `git rev-parse --git-path info/exclude`
(src/client_integrations/completion.rs:370); cursor-agent, claude/qwen/grok/
aider env sets all match seed.

### docs/configuration/browser-demo.md — verdict: accurate. No edits.
### docs/configuration/desktop.md — verdict: accurate. No edits.
(Engine selector / installed-Agent-first behavior confirmed: desktop/main.cjs:599
"Issue #759", desktop/lib/agent-provider.cjs `start-agent` default.)
### docs/configuration/docker.md — verdict: accurate. No edits.

### docs/configuration/languages.md — verdict: missing-coverage (1 edit)
1. [line 3] "English, Russian, Chinese, and Hindi are supported language peers."
   — since 2026-07-30 (c79cf539, issue #706 language-expansion protocol, after
   the doc's 2026-07-19 touch) the supported set is a data ledger:
   data/seed/languages.lino declares en/ru/hi/zh `status full`, **es `status
   partial`** (lines 28-30), and `fallback_policy explicit_gap` (line 3);
   src/language.rs:151 `registered_languages()` reads it (LANGUAGE_LEDGER
   include at :164).
   EDIT: `English, Russian, Chinese, and Hindi are supported language peers.`
   → `English, Russian, Chinese, and Hindi are full-status language peers, and Spanish is registered as a partial peer. The supported set is data, not a Rust enum: it is declared in \`data/seed/languages.lino\` (with \`fallback_policy explicit_gap\`) and read through \`registered_languages()\`.`
   Pin impact: none — issue_761_docs.rs:191-203 needs `data/seed/`,
   `data-only`, `parity`, `English`, `Russian`, `Chinese`, `Hindi`; all retained.
Both quoted guard scripts exist (tests/e2e/scripts/check-multilingual-intent-coverage.mjs,
check-language-change-parity.mjs).

### docs/configuration/memory.md — verdict: accurate. No edits.
(`/v1/memory/since` + `/v1/memory/import` confirmed at src/server.rs:225-228;
pinned markers intact.)

### docs/configuration/modes.md — verdict: accurate. No edits.
(agent-commander/`start-agent` boundary confirmed in
desktop/lib/agent-provider.cjs:18; PR #783 / issue #759 wiring in
desktop/main.cjs:599.)

### docs/configuration/orchestration.md — verdict: accurate. No edits.
(All flags exist in src/cli_orchestration.rs (`--parent`, `--disproved-claim`,
`--evidence`, `--translation-session` requires `synthesize`, `--max-depth`,
`--output-dir`, `--response-language`); default timeout 900 s; live gate
script experiments/issue_703_orchestration/run_live_cli_matrix.sh and
`FORMAL_AI_ISSUE_703_CODEX_UNSANDBOXED` guard at its line 36; six adapters
match the seed registry. Note: this page is not in the issue_761 PAGES pin
list — no pins to preserve.)

### docs/configuration/output-sessions.md — verdict: stale (3 findings)
1. [line 22] The completion-record JSON example omits the `recovery` object
   added 2026-07-31 (eef340ac), one day after the doc's last touch (2026-07-30).
   Actual record (src/client_integrations/completion.rs:227-252, serde_json
   default BTreeMap → alphabetical key order) includes
   `"recovery":{"ledger":…,"max_attempts":4,"strategies_available":[…],"strategies_spent":[…]}`
   between `rawMetadata` and `reason`; defaults from
   data/seed/client-completion-contracts.lino:4-11; ledger path
   `<state root>/formal-ai/completion-recovery.lino`
   (src/client_integrations/completion_learning.rs:36-40, override
   `FORMAL_AI_STATE_DIR`).
   EDIT: in the example JSON, insert between `…"output_tokens":1553}},` and `"reason":"workspace_effect_observed"`:
   `"recovery":{"ledger":"/home/user/.local/state/formal-ai/completion-recovery.lino","max_attempts":4,"strategies_available":["restate_postcondition","name_target_artifact","decompose_into_leaf"],"strategies_spent":[]},`
   Pin impact: none (pin needs the "```json" fence and `session`/`resume`
   markers, untouched).
2. [lines 26-27] `still produced no effect after the bounded\ncorrective attempt` — singular; there is now a ladder (see above).
   EDIT: `after the bounded\ncorrective attempt` → `after the seeded ladder of bounded corrective attempts`
   Pin impact: none.
3. [line 251] Troubleshooting bullet quotes `\`dialog log unavailable\`` as the
   message — the actual rendered message is
   `FORMAL_AI_DIALOG_LOG_DIR is not configured`
   (data/seed/agent-info.lino:100-101 template `"{variable} is not configured"`
   + src/conversation_context.rs:40-45 substituting the env-var name).
   EDIT: `- \`dialog log unavailable\` means the server did not start with`
   → `- \`FORMAL_AI_DIALOG_LOG_DIR is not configured\` means the server did not start with`
   Pin impact: none.
All other content verified (context export flags/sources in src/cli_context.rs,
conversation API routes at src/server.rs:336-354, session/resume table matches
the seed registry, OpenCode SQLite path/read-only mode).

### docs/configuration/server-api.md — verdict: wrong (1 finding)
1. [line 29] "Formal AI performs no paid inference and reports `cost: 0`" —
   the server deliberately emits **no cost field at all**: tests/issue_752.rs:110
   asserts `json.get("cost").is_none()`, and
   tests/integration/issue_751_token_usage.rs:140-166
   (`responses_use_real_timestamps_and_omit_fake_cache_and_cost_metadata`)
   asserts serialized usage never contains `cost`; no `"cost"` key exists in src/.
   EDIT: `AI performs no paid inference and reports \`cost: 0\`; usage fields describe the`
   → `AI performs no paid inference; usage envelopes deliberately omit cost and cache fields, and the usage counts describe the`
   PIN IMPACT: tests/issue_761_docs.rs:176 requires the substring `cost` in
   this page — the replacement retains the word "cost", so **no pin change
   needed** (double-check after applying).

### docs/configuration/t3-code.md — verdict: accurate. No edits.
### docs/configuration/telegram.md — verdict: accurate. No edits.

### docs/configuration/tools.md — verdict: missing-coverage (1 edit, 2 parts)
1. [lines 12-21] The "Complete internal tools registry" table omits `run_agent`
   — `tool tool_run_agent` (name `run_agent`, `mode agent`) ships at
   data/seed/tools.lino:223-226 (added 04a03ba0, 2026-07-30, "permission-gated
   agent orchestration"); tools.md (last touched 2026-07-31) never mentions it.
   The issue_761 contract only cross-checks `environments.lino` tools, so this
   drift is CI-invisible. Also line 12's "do not delegate to an agent harness"
   is contradicted by run_agent.
   EDIT [line 12]: `These are symbolic engine operations and do not delegate to an agent harness:`
   → `These are symbolic engine operations; all but \`run_agent\` stay inside the engine (\`run_agent\` launches one seed-registered agent CLI only after an explicit workspace-scoped grant — see the orchestration guide):`
   EDIT [table]: add row after the Calculation row:
   `| Agent delegation | \`run_agent\` |`
   Pin impact: none (issue_761_docs.rs:120-146 needs `Internal tools`,
   `External tools`, `capability`, `environment`, `specialized`, `bash`,
   `fallback`, `hosted` + every environments.lino tool in backticks — all
   preserved; run_agent is an addition).

### docs/configuration/vscode.md — verdict: accurate. No edits.

---

## Top-level docs/ (USER-JOURNEYS, philosophy, meta-algorithm, tech stack, benchmarks, computer-use, report-issue, upload-memory, diagrams)

### docs/USER-JOURNEYS.md — verdict: accurate (1 typo)
Pinned strings (tests/unit/user_journeys.rs:12-46: section headings,
`What is 8% of $50?`, `Why did you answer that?`, `formal_ai_bundle`,
`operation-vocabulary.lino`) all intact; all quoted commands and env vars
verified against src/main.rs, src/cli_procedure.rs, src/server.rs.
1. [line 58] Persona table names `**Ltoo, the learner**` — every journey uses
   "Lin" (lines 123, 275, 377); typo since the doc's creation (ad9e92a6).
   EDIT: `**Ltoo, the learner**` → `**Lin, the learner**`
   Pin impact: none (no test contains "Ltoo").

### docs/philosophy.md — verdict: accurate
All pins (tests/issue_885_docs.rs:131-152) present; "Implemented today" claims
match src/relative_meta_logic.rs, src/substitution.rs,
src/statement_verification.rs. No edits.

### docs/meta-algorithm.md — verdict: stale + missing-coverage (2 findings)
Pins: tests/unit/specification/*meta_algorithm*.rs (each pins `# Meta-Algorithm`,
its recipe path, test path, topic keyword), tests/unit/docs_requirements_issue_540.rs:83-91,
tests/unit/docs_requirements_issue_656.rs:38-55 — none touched by these edits.
1. [lines 14-16 + table 18-28] "Nine recipes are grounded today … the other
   eight encode" — `data/meta/` holds **12** `*-recipe.lino` files; the three
   missing from the table are all grounded: `computer-use-recipe.lino` (#707,
   tests/unit/specification/computer_use_meta_algorithm.rs),
   `grounded-action-recipe.lino` (#840,
   tests/unit/specification/grounded_action_meta_algorithm.rs),
   `draft-portfolio-recipe.lino` (#704, tests/unit/issue_704.rs).
   EDITS: `Nine recipes are grounded today.` → `Twelve recipes are grounded today.`;
   `the other eight encode` → `the other eleven encode`; ADD three table rows
   after line 28 for computer-use (#707), grounded-action (#840), and
   draft-portfolio (#704), each citing its recipe file and grounding test.
   Pin impact: none ("Nine recipes" not pinned).
2. [lines 811, 814, 815] `formal-ai chat "Using the numbers…"` and
   `FORMAL_AI_COMPUTE_BUDGET=256 formal-ai chat "…"` — `chat` has no positional
   prompt; the prompt is the required `--prompt` flag
   (src/main.rs:79-81, `#[arg(long, env = "FORMAL_AI_PROMPT")]`). The commands
   as written fail with "unexpected argument".
   EDIT (3 occurrences): `formal-ai chat "` → `formal-ai chat --prompt "`
   Pin impact: none ("formal-ai chat" not pinned by any test).
Verified accurate: recursive core 12 steps / 25 functions, agentic recipe counts,
`MAX_TURNS = 12` (src/agentic_coding/driver.rs:43), `ASSUMED_TRUE_PRIOR = 0.6`,
default budget 512 (src/solver.rs:284), `improve --promote` flag set.

### docs/associative-tech-stack.md — verdict: accurate
Grounded by tests/issue_874_docs.rs (cross-checks every named component against
Cargo.toml/package.json); all claims verified. No edits.

### docs/benchmarks.md — verdict: stale (1 number)
Pins: tests/unit/docs_requirements/benchmarks.rs:110-157 (`# Benchmark Catalog`,
fixture-filename index — programmatically asserts every `data/benchmarks/*.lino`
appears — issue ids, suite names, `Apache-2.0`, `Anti-memorization`).
1. [line 18] Industry-slice row floor `| … | 10 |` — fixture floor is now 13
   (data/benchmarks/industry-suite.lino:8 `minimum_pass_count "13"`).
   EDIT: `| \`issue_304_benchmark_suite_reports_pass_fail_counts\` | 10 |`
   → `| \`issue_304_benchmark_suite_reports_pass_fail_counts\` | 13 |`
   Pin impact: none (the "10" is not pinned).
All other floors (4/1440/12/10/16/56), the 12 ratchet test names, and the
external "Honest current numbers" table (2026-08-03, 0.323.0, HumanEval 0/20,
MBPP 0/20, GSM8K 2/20, MATH 0/20, object_counting 0/20, CoEdIT 0/20, SWE-bench
Lite 0/1, EditEval `benchmark_unavailable`) verified exact against
data/benchmarks/external-results.lino and .github/workflows/external-benchmarks.yml.

### docs/computer-use.md — verdict: accurate
12 primitives (`COMPUTER_USE_PRIMITIVES: [ComputerUsePrimitive; 12]`,
src/computer_use/mod.rs:32), CLI flags, `/mcp`, audit env var, archive format
id, seeded/held-out counts, harness scripts in release.yml — all verified.
No edits.

### docs/report-issue.md — verdict: accurate
All flags/defaults match src/cli_report.rs:31-87 and src/cli_context.rs:84-96;
title cap and defaults verified. No edits.

### docs/upload-memory.md — verdict: accurate
Commands match src/main.rs:307-341; `formal-ai-memory.lino` matches
src/storage_policy.rs:35 and src/web/i18n-catalog.lino. Oldest doc in the set
(2026-05-16) but nothing has drifted. No edits.

### docs/diagrams/agentic-recipes.md — verdict: in sync with generator
Line-by-line comparison against `render_document()` in
src/agentic_coding/diagram.rs matches, including the trailing newline; byte pin
tests/unit/issue_538_agentic.rs:340-343 (`include_str!` equality). NO hand
edits ever; if it drifts, regenerate via the generator.

---

## Case-study broken references (historical docs — fix links only, no content rewrites)

- docs/case-studies/issue-1/README.md:49 → `../../REQUIREMENTS.md` resolves to
  docs/REQUIREMENTS.md (missing). EDIT: `../../REQUIREMENTS.md` → `../../../REQUIREMENTS.md`.
- docs/case-studies/issue-14/README.md:10 and :243 → `../../../docs/demo/formal_ai_worker.js`
  / `../../../docs/demo/app.js` (docs/demo/ removed). EDIT: point at
  `../../../src/web/formal_ai_worker.js` / `../../../src/web/app.js` (current homes).
- docs/case-studies/issue-78/README.md:71 and docs/case-studies/issue-140/README.md:61
  → `../issue-44/README.md` (issue-44 case study does not exist). EDIT: replace the
  link with plain text `issue #44` (or link the GitHub issue URL) — target never existed in-tree.
- docs/case-studies/issue-442/README.md:12,34,36,41,177 → `./logs/…` (logs/ never
  committed). EDIT: de-link to plain text noting the logs were not committed.
- docs/case-studies/issue-523/README.md:287 → `./data/deploy-pages-81948748273.log`
  (file absent). EDIT: de-link to plain text.
- docs/case-studies/issue-541/{best-practices.md:117; requirements.md:44,226;
  solution-plans.md:58,234,417} → `../../tests/e2e/tests/issue-541-*.spec.js`
  one `../` short (specs exist at repo-root tests/e2e/tests/).
  EDIT: `../../tests/e2e/tests/` → `../../../tests/e2e/tests/` in those six links.
- docs/case-studies/issue-673/README.md:122 →
  `../../../dev/log/issues/673/pulls/807/full-test-eager-ast-regression.log`
  (dev/log/issues/673 absent). EDIT: de-link to plain text.
Pin impact: none for all of the above (none of these link strings appears in tests/).
Excluded by design: relative links inside `docs/case-studies/*/raw-data/`
(verbatim captures of external repos) and the literal `(<encoded>)` placeholder
in issue-21/README.md:89. All docs/screenshots/ and docs/assets/ references in
live docs resolve.

---

## Missing docs (shipped surfaces with no documentation)

CLI subcommands with no page or mention anywhere under docs/ or README:
- `formal-ai statement-audit` (src/cli_statement_audit.rs)
- `formal-ai learn` (issue #701 auto-learning adoption cycle, src/cli_learn.rs)
- `formal-ai import` (bulk lexeme import, issue #660/R378, src/cli_import.rs)
- `formal-ai shared-dialog` (src/cli_shared_dialog.rs)
- `formal-ai algorithm` (learned execution-algorithm proposals, src/cli_algorithm.rs)

Server/API:
- `/v1/network` (canonical links-network route) absent from docs/desktop/server-api.md;
  `GET/POST /{api/formal-ai/,}v1/conversations/{id}[/learn]` and `POST /mcp`
  missing from its endpoint table (details in that doc's section above).

Desktop:
- No desktop/README.md at all (only build/lib/scripts + main.cjs/preload.cjs live there).
- Desktop bridge surfaces undocumented: engine selection (`setEngine`),
  auto-updater (`checkForUpdates`/`installUpdate`), `installVsCodeExtension`,
  `runAgentProvider`/`agentEvent`, data-migration replay (desktop/preload.cjs:6-50).
- Desktop tool router's newer native tools (`web_search`, `web_fetch`,
  `read_file`, `grep`, `glob`, `list_directory`, `read_many_files`) not in
  docs/desktop/server-api.md §5d.

CI:
- docs/ci-cd/ covers only release-style troubleshooting; desktop-release.yml,
  external-benchmarks.yml, learning-cycle.yml, proactive-failure-report-e2e.yml,
  and task-ladder.yml have no docs coverage.

Meta-algorithm/features:
- draft-portfolio (k-draft) feature (#704, `--draft-count`/`FORMAL_AI_DRAFT_COUNT`)
  has no prose doc anywhere (partially fixed by the meta-algorithm.md
  twelve-recipes edit); grounded-action recipe (#840) likewise.
- The issue #903 / PR #925 `formal-ai with` argv re-rendering behavior is not
  yet described in README/agentic-clis (optional edit R-opt1 below).

Doctrine:
- The compiled-Rust-logic / JS-glue-and-JSX-only / shared-WASM-engine doctrine
  was previously codified nowhere as a standing requirement (fixed by D1/D2
  below).

---

## Final prioritized edit checklist (apply to main in this order)

Priority 1 — wrong statements (docs contradict code):
 1. docs/configuration/server-api.md:29 — remove `cost: 0` claim (keep the word
    "cost" for the issue_761_docs.rs:176 pin). [pin: none, verify needle]
 2. ROADMAP.md:133 — pillar 18 status Built → Partial with honest evidence
    (also doctrine D3). [pin: none]
 3. docs/meta-algorithm.md:811,814,815 — `formal-ai chat "…"` →
    `formal-ai chat --prompt "…"` (3 occurrences; commands currently fail).
    [pin: none]
 4. docs/configuration/output-sessions.md:251 — replace fictional
    `dialog log unavailable` message with the real
    `FORMAL_AI_DIALOG_LOG_DIR is not configured`. [pin: none]
 5. docs/configuration/agentic-clis.md:107-108 — bare `--undo` is not
    wrapper-exclusive. [pin: none; keep `--global`/`--undo` substrings]

Priority 2 — doctrine codification (owner directive):
 6. D1: REQUIREMENTS.md — append "Standing Doctrine" section with new row
    **R536** (text in the doctrine section above). [pin: none]
 7. D2: ARCHITECTURE.md — insert the standing-principle paragraph after line
    1125. [pin: none]
 8. D3: (same as item 2 — ROADMAP pillar 18.)
 9. D5: CONTRIBUTING.md convention 1 — doctrine cross-reference sentence +
    fix `formal_ai_worker.js` twin wording (line 382 finding). [pin: none]
10. D6: VISION.md:158 — qualify "Compiled JS handlers" rule shape as
    transitional. [pin: none]
11. D4 (optional): GOALS.md Architecture Goals bullet. [pin: none]

Priority 3 — stale numbers/counters (present-tense claims vs code):
12. VISION.md:235 — 10-case/10/10 → 13-case/13/13. [pin: none]
13. ROADMAP.md:140 — "10/10 passing" → "13/13 passing". [pin: none]
14. ROADMAP.md:141 — "passes 10/10" → "passes 13/13". [pin: none]
15. ARCHITECTURE.md:1203 — past-tense + "(13 cases / 13-floor today)" note.
    [pin: none]
16. docs/benchmarks.md:18 — industry-slice floor 10 → 13. [pin: none]
17. ROADMAP.md:406 — "40 files" → "46 files". [pin: none]
18. ARCHITECTURE.md:1103 — `_21.js` → `_23.js`. [pin: none]
19. ARCHITECTURE.md:1115-1116 — "~500 lines"/"26,700 lines" →
    "roughly 1,700"/"roughly 27,700". [pin: none]
20. ARCHITECTURE.md:1314 — "R1 … R444" → "R1 … R535" (+ R914 block mention).
    [PIN: must update tests/unit/docs_requirements_issue_451.rs:40
    `"R1 \u{2026} R444"` → `"R1 \u{2026} R535"` in the same commit]
21. docs/vscode/extension.md:127 — "50 `node:test` cases" → "51". [pin: none]
22. docs/meta-algorithm.md:14-16 + table — "Nine recipes"/"other eight" →
    "Twelve"/"other eleven" + 3 new table rows (#707, #840, #704). [pin: none]

Priority 4 — moved/renamed paths in evidence cells:
23. REQUIREMENTS.md:307 — definition_merge handler path → src/definition_merge.rs.
    [pin: none]
24. REQUIREMENTS.md:419,424,425,428 — `src/summarization.rs` →
    `src/summarization/mod.rs` (4 rows; R203 also fixes
    try_summarize_conversation location). [pin: none]
25. REQUIREMENTS.md:249 — specification/code_generation.rs → …/code_generation/.
    [pin: none]
26. REQUIREMENTS.md — `src/solver_helpers.rs` → `src/solver_helpers/`
    (replace_all within REQUIREMENTS.md). [pin: none]
27. REQUIREMENTS.md:398 and :733 — deleted changelog.d fragment refs → describe
    as collected-into-CHANGELOG fragments (2 rows). [pin: none]
28. ARCHITECTURE.md:711-713 — try_summarize_conversation location. [pin: none]
29. CONTRIBUTING.md:382 — worker twin = loader + worker/ shards. [pin: none]
30. README.md:990 — summarization split file list. [pin: none]

Priority 5 — feature drift / missing rows in living reference docs:
31. docs/desktop/server-api.md:112+117 — /v1/network canonical row (+ prose).
    [pin: none]
32. docs/desktop/server-api.md:95 — /health `version` field. [pin: none]
33. docs/desktop/server-api.md — ADD conversations routes + /mcp table row +
    extended tool list + bridge methods (missing-coverage block). [pin: none]
34. docs/configuration/agentic-clis.md:24-27 — recovery ladder wording.
    [pin: none]
35. docs/configuration/output-sessions.md:22 — add `recovery` object to the
    example record. [pin: none]
36. docs/configuration/output-sessions.md:26-27 — ladder wording. [pin: none]
37. docs/configuration/languages.md:3 — ledger-driven set + Spanish partial +
    explicit_gap. [pin: none; keep the four language-name needles]
38. docs/configuration/tools.md:12 + table — run_agent row + harness-delegation
    qualifier. [pin: none]
39. docs/USER-JOURNEYS.md:58 — `Ltoo` → `Lin`. [pin: none]

Priority 6 — case-study link repairs (historical docs, links only):
40. docs/case-studies/issue-1/README.md:49 — `../../REQUIREMENTS.md` →
    `../../../REQUIREMENTS.md`.
41. docs/case-studies/issue-14/README.md:10,243 — docs/demo/* →
    src/web/* current homes.
42. docs/case-studies/issue-78/README.md:71 and issue-140/README.md:61 —
    de-link nonexistent issue-44 case study (plain text or GitHub issue URL).
43. docs/case-studies/issue-442/README.md:12,34,36,41,177 — de-link
    never-committed ./logs/*.
44. docs/case-studies/issue-523/README.md:287 — de-link missing log file.
45. docs/case-studies/issue-541/{best-practices.md:117; requirements.md:44,226;
    solution-plans.md:58,234,417} — add one `../` to the six spec links.
46. docs/case-studies/issue-673/README.md:122 — de-link missing dev/log path.
    [pins: none for 40-46]

Optional (nice-to-have, not required for accuracy):
R-opt1. README.md after :297 — one sentence on the #903 argv re-rendering.
R-opt2. ROADMAP.md:128 — pillar 13 evidence `/v1/graph` → `/v1/network`.
R-opt3. README.md:1073 — drop the stale "(now alongside VISION.md)" parenthetical.

Never edit by hand (byte-pinned/generated):
- docs/diagrams/agentic-recipes.md — currently IN SYNC with
  src/agentic_coding/diagram.rs::render_document() (byte pin
  tests/unit/issue_538_agentic.rs:340-343); regenerate via the generator if it
  ever drifts.
- Committed Agent CLI session JSONs under docs/case-studies/** — byte-pinned by
  tests; regenerate via their recorded recipes (e.g.
  scripts/reproduce-issue-538.sh).
- CHANGELOG.md — release tooling owns it (out of scope per instructions).

Edit count: 46 required checklist items (items 1-46; item 8 duplicates item 2,
so 45 distinct edit operations, of which several are multi-line row rewrites),
plus 3 optional items and 1 pin-test update
(tests/unit/docs_requirements_issue_451.rs:40) required by item 20.
