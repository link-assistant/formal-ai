# Issue 16 Case Study: Multilingual chat, Wikipedia lookup, append-only memory

## Summary

Issue [#16](https://github.com/link-assistant/formal-ai/issues/16) asks the
project to continue the vision work along five concrete axes:

1. Move `docs/demo/*` to `src/web/` so the deployable artefact lives next to
   the other library/CLI/web sources.
2. Support every existing chat message in Russian, Hindi, and Chinese (e.g.
   "Привет", "Кто ты?", "Что такое википедия?") and add "What is X?" prompts
   that succeed in the browser by calling Wikipedia services directly.
3. Add Playwright e2e tests that run locally for pull requests and against
   the live https://link-assistant.github.io/formal-ai deploy.
4. Add **export/import** controls in the web UI: the demo must persist the
   conversation log as Links Notation text and offer round-trip import.
   Storage should be `localStorage` for short documents and IndexedDB for
   larger ones; the persistent representation is **append-only**.
5. Keep working toward an associative-doublets architecture in which both
   the seeded knowledge and the user-curated state live as named links the
   user can read or write through Links Notation or natural language.

This case study collects fresh raw data, distills the requirements, lists
the components we used to satisfy them, and traces the implementation back
to the open issue.

## Collected Data

Fresh GitHub evidence is stored in `raw-data/`:

- `issue-16.json`: full issue body and metadata.
- `issue-16-comments.json`: issue conversation comments (empty at collection
  time — the request is in the issue body).
- `pr-17.json`: pull request metadata for the draft that implements this
  case study.
- `pr-17-conversation-comments.json`: general PR discussion comments.
- `pr-17-review-comments.json`: inline code-review comments.
- `pr-17-reviews.json`: PR review records.
- `recent-merged-prs.json`: recent merged PRs used to match repository
  documentation style.

## Prior Case Studies

The new work is grafted onto previously reviewed evidence:

- [`../issue-1/README.md`](../issue-1/README.md): the formal AI proof of
  concept boundaries.
- [`../issue-4/README.md`](../issue-4/README.md): GitHub Pages deploy and
  CI regression coverage.
- [`../issue-6/README.md`](../issue-6/README.md): demo-mode default
  behavior, countdown feedback, diagnostics gating.
- [`../issue-8/README.md`](../issue-8/README.md): Telegram interface, code
  execution metadata, `lino-arguments` config.
- [`../issue-10/README.md`](../issue-10/README.md): issue-reporting links,
  identity intent, preview removal.
- [`../issue-12/README.md`](../issue-12/README.md): holistic requirements
  synthesis from #1, #4, #6, #8, #10 and the associative-doublets vision.
- [`../issue-14/README.md`](../issue-14/README.md): universal solver loop
  unification and concept lookup handler.

## Online Research

External components and references checked against current docs:

- [Wikipedia REST `page/summary`](https://en.wikipedia.org/api/rest_v1/#/Page%20content/get_page_summary__title_):
  served with CORS so the browser worker can read summaries directly from
  GitHub Pages without a proxy.
- [link-foundation/links-notation](https://github.com/link-foundation/links-notation):
  the portable text format used for the exported event log.
- [link-foundation/lino-objects-codec](https://github.com/link-foundation/lino-objects-codec):
  reference indent style for the exported memory file header.
- [linksplatform/doublets-rs](https://github.com/linksplatform/doublets-rs):
  the Rust doublet store the universal solver continues to target; this
  issue does not yet flip the demo over to a JS port, but the memory log
  is shaped so it can become the source-of-truth for a future
  doublets-web import.
- [link-foundation/link-cli](https://github.com/link-foundation/link-cli):
  the embed target for CLI/Telegram/server modes (already integrated for
  prompts in issue-14; nothing new in #16 here).
- IndexedDB on MDN: confirms append-only semantics are achievable simply
  by never wiring a delete code path through the public API.

## Requirements From The Issue Body

The bullet list below maps each user requirement to the implementation
artefact that addresses it. Numbers refer to the bullets in the Summary
section above.

| # | Requirement | Implementation artefact |
|---|---|---|
| 1 | Move `docs/demo/*` to `src/web/` | Commit `765b2ea` plus the deploy job in `.github/workflows/release.yml` (path: `src/web`). |
| 2 | English/Russian/Hindi/Chinese greetings, identity, unknown, concept lookup | Rust: `src/language.rs` Unicode-block detector, `src/engine.rs` `language_aware_answer_for()` arms, `src/concepts.rs` prefix/suffix patterns + Cyrillic/Devanagari/CJK aliases. JS mirror: `src/web/formal_ai_worker.js` `detectLanguage`, `MULTILINGUAL_ANSWERS`, multilingual concept extraction. |
| 2 | "What is X?" works via Wikipedia in the browser | `src/web/formal_ai_worker.js` `WIKIPEDIA_HOSTS`, `fetchWikipediaSummary`, `tryWikipediaLookup`. Falls back to the English host when the detected language has no article. |
| 3 | Local e2e for PRs | `tests/e2e/playwright.local.config.js` already runs in CI; this PR adds `tests/e2e/tests/multilingual.spec.js` to the local matrix. |
| 3 | Remote e2e against the live deploy | `tests/e2e/playwright.pages.config.js` matrix updated to include the new spec; the existing `test-e2e-pages` job runs it after `deploy-demo`. |
| 4 | Export/import full memory as Links Notation | `src/web/memory.js` (IndexedDB-backed) and `src/web/app.js` topbar buttons. Export downloads `formal-ai-memory.lino`; import accepts the same shape. |
| 4 | Append-only by default; no "Forget X" without an explicit retraction protocol | `src/web/memory.js` deliberately exposes `appendEvent`, `listEvents`, `importEvents`, `exportLinksNotation`, `parseLinksNotation` and **no** delete/forget/clear operation. The e2e suite asserts this surface (`tests/e2e/tests/multilingual.spec.js` → "Memory module exposes no delete/forget operation"). |
| 5 | Associative network knows what the AI is built from | Already seeded in `data/seed/concepts.lino` and `src/concepts.rs` (`concept_universal_solver`, `concept_event_log`, `concept_links_notation`, `concept_doublet`). This PR extends the multilingual aliases so the same answers surface in ru/hi/zh. |

## Architecture Notes

- **Two views of the log.** The append-only IndexedDB store *is* the
  transaction log. `listEvents()` returns the materialized state at "now"
  by replaying records in id order; an import is the equivalent of
  re-playing a snapshot into the same store. A future PR can compute a
  point-in-time projection by truncating the replay at a chosen event id,
  fulfilling the "time machine" axis of #16 without changing the storage
  model.
- **Wikipedia fallback path.** `tryWikipediaLookup` is intentionally last
  in the synchronous-handler chain so the offline `CONCEPTS` table wins
  whenever it can; the network call is reserved for prompts the local
  knowledge base does not cover. The handler returns a structured answer
  with `source:` evidence so the user can audit the citation.
- **Browser/Rust parity.** Every multilingual change lands in *both* the
  Rust solver and the JS worker, so the Telegram bot, CLI, and web UI
  share identical behaviour for the supported prompts. The `unknown`
  reply is translated for ru/hi/zh on both sides; unsupported languages
  fall through to the existing English-with-`language:unknown` evidence
  link.

## Verification

- Cargo: `cargo fmt --check`, `cargo clippy --all-targets --all-features
  -- -D warnings`, `cargo test --all-features` (33 + 2 + 150 = 185 tests
  pass).
- e2e: `cd tests/e2e && npx playwright test --config=playwright.local.config.js`
  (33 tests pass, including 14 new ones covering multilingual prompts,
  the Wikipedia REST fallback, the export/import buttons, the
  Download-bundle export, the Report-issue body referencing the bundle,
  the seed-loaded tool registry, and reasoning/tool_call events
  surfacing in the append-only log).
- Manual: opened `http://localhost:3457` in headless Chromium and
  confirmed Russian/Hindi/Chinese greetings + identity, "Что такое
  Википедия?", and "What is Albert Einstein?" all answered as expected;
  Export memory downloads a `.lino` file with `demo_memory` header and
  Import memory inserts the new events while never deleting earlier ones.

## Follow-Up: Data-Driven Configuration (PR #17 continuation)

After the initial PR merged, the maintainer reopened the PR with a
second pass of requirements: replace every hardcoded constant in the
demo with seed data so the user can reconfigure the agent without
rewriting code, surface the tool catalog the AI is allowed to call,
record every reasoning step and tool invocation in the append-only
log, and let any interface export the full agent state as a single
Links Notation file for issue reports.

These additions are mapped as requirements R97-R100 in
[`../../../REQUIREMENTS.md`](../../../REQUIREMENTS.md). The implementation
artefacts are:

- **Seed-first runtime tables.** `data/seed/multilingual-responses.lino`,
  `data/seed/concepts.lino` (with multilingual `aliases`), and
  `data/seed/tools.lino` define the agent's responses, concept table,
  and tool registry. Mirrored under `src/web/seed/` and merged at boot
  by `src/web/seed_loader.js`. `src/web/formal_ai_worker.js` now
  initialises mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS`
  tables from the seed instead of carrying hardcoded literals.
- **Tool registry UI.** `src/web/app.js` renders the loaded tools in
  the context panel with a `thinking` vs `agent` mode badge (see
  `[data-testid="tool-registry"]`). The catalog currently exposes
  `http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`,
  `read_local_file`, `append_memory`, and `export_memory`; the demo
  worker dispatches `wikipedia_lookup`, `eval_js`, and
  `concept_lookup` end-to-end today and records every invocation as a
  `kind:"tool_call"` event so future surfaces can implement the rest
  against the same data shape.
- **Reasoning + tool-call events.** The solver's `solve()` returns a
  structured `steps[]` array (`impulse`, `formalize`,
  `detect_language`, `match_rule`/`dispatch_handler`/`invoke_tool`,
  `fallback`) and a `toolCalls[]` array. `src/web/app.js`
  appends each one to the append-only memory log alongside the user
  and assistant turns; the schema in `src/web/memory.js` gains
  optional `kind`, `tool`, `inputs`, and `outputs` fields without
  breaking older logs.
- **Bundle export.** `FormalAiMemory.exportBundle({seed, events, info})`
  produces a single `formal_ai_bundle` Links Notation document
  containing the agent's environment metadata, every seed file in
  full, and the entire memory log. The `Download bundle` topbar
  button writes it to disk as `formal-ai-bundle.lino`. The prefilled
  "Report issue" body ends with an `Attach full state` section that
  asks the reporter to drag the bundle into the issue, so the
  maintainer can fully reconstruct the agent's state from a single
  attachment.

## Follow-Up: Universal Seed and Single-File Round-Trip (PR #17 reopen)

The PR was reopened a second time with [comment 4458370500](https://github.com/link-assistant/formal-ai/pull/17#issuecomment-4458370500),
which asked for four additional invariants:

1. `data/seed/` must be the canonical knowledge surface for **every**
   user-facing interface (Rust library, CLI, HTTP server, Telegram bot,
   browser demo) — not just the web demo.
2. `REQUIREMENTS.md` should live next to `VISION.md` in the repository
   root so the architecture story and the requirement matrix are
   reviewed together.
3. More behavior should be lifted out of code constants into `.lino`
   data so the agent stays reconfigurable end-to-end.
4. The merged bundle must be a true single-file import/export: one
   `formal_ai_seed_bundle` document must round-trip back to the same
   per-category split files that drive the rest of the loader.

These are tracked as requirements R101-R104 in
[`../../../REQUIREMENTS.md`](../../../REQUIREMENTS.md). The
implementation artefacts are:

- **Universal seed loader (`src/seed.rs`).** Every `data/seed/*.lino`
  file is `include_str!`-embedded into the binary, so the Rust solver,
  the `formal-ai` CLI, the `serve` HTTP route, and the `telegram`
  webhook all read from the same data the browser fetches from
  `src/web/seed/`. `scripts/sync-seed.sh` keeps the web mirror in sync.
- **REQUIREMENTS / VISION colocated.** `REQUIREMENTS.md` is now in the
  repository root next to `VISION.md`; cross-references and case
  studies were updated to point at the new path.
- **Intent routing as data.** Greeting / identity / hello-world /
  concept-lookup / unknown intents now live in
  `data/seed/intent-routing.lino` with four explicit match semantics:
  `keyword` (exact prompt), `phrase` (exact multi-word prompt),
  `token` (substring), and `combo` (all tokens present). The Rust
  matcher in `engine::matches_intent_route` and the browser worker's
  `matchesIntentRoute` consume the same schema. Regression-guarded by
  `solver_helpers::hello_world_request_is_not_routed_to_greeting` —
  before the split, the prompt "Write me hello world program in Rust"
  matched the greeting keyword "hello" via substring and got routed to
  the greeting reply.
- **Single-file bundle round-trip.** `seed::merged_bundle()` already
  emitted one `formal_ai_seed_bundle` document; the new
  `seed::parse_bundle()` and `seed::bundle_from_files()` make the
  round-trip symmetric, with mirrors `FormalAiSeed.parseBundle` and
  `FormalAiSeed.loadFromBundle` in `src/web/seed_loader.js`. Covered
  by `seed::tests::bundle_round_trips_through_parse_bundle` and
  `seed::tests::parse_bundle_recovers_intent_routing_via_inner_parser`.

## Follow-Up: Self-Aware Environments and Cross-Surface Memory Migration (PR #17 reopen 3)

The PR was reopened a third time with [comment 4459020108](https://github.com/link-assistant/formal-ai/pull/17#issuecomment-4459020108).
The maintainer asked for four further invariants on top of the
universal-seed work above:

1. `src/web/seed/*` must move fully under `./data/seed/*` — the seed
   itself should be self-aware of every environment it supports, and
   if `src/web/seed/*` is only a deployment artefact it must be
   gitignored so contributors cannot accidentally diverge the browser
   mirror from the canonical seed.
2. The agent's append-only memory must migrate between every
   surface — CLI ↔ browser ↔ HTTP ↔ Telegram — using the same
   Links Notation files, not bespoke per-surface formats.
3. Every capability exposed by the CLI/server must also be reachable
   from the `formal_ai` library, so downstream embedders can build
   their own surfaces against the same primitives.
4. Continue keeping `VISION.md`, `REQUIREMENTS.md`, and the case study
   in lockstep with the implementation.

These are tracked as requirements R105–R108 in
[`../../../REQUIREMENTS.md`](../../../REQUIREMENTS.md). The
implementation artefacts are:

- **`src/web/seed/` is a deploy artefact.** `.gitignore` now lists
  `src/web/seed/` so the directory is regenerated from `data/seed/`
  by `scripts/sync-seed.sh` rather than tracked. The local Playwright
  config (`tests/e2e/playwright.local.config.js`) and the GitHub
  Actions `test-e2e-local` + `deploy-demo` jobs each invoke the sync
  before the web bundle is served or deployed, so there is no
  "edit `src/web/seed/`" path that can drift away from canon.
- **Seed declares every environment.** `data/seed/environments.lino`
  enumerates the six environments the agent runs in (`browser`,
  `rust_library`, `cli`, `http_server`, `telegram`,
  `docker_microservice`) with each one's runtime, seed path, memory
  store, export commands, and tool surface. The Rust accessors
  `seed::environment_directory()` and `seed::environment_records()`
  expose the same data to embedders; `FormalAiSeed.extractEnvironmentDirectory`
  in `src/web/seed_loader.js` mirrors it on the browser. The CLI
  surfaces it as `formal-ai environments`, and the `migration` block
  in the file documents the four named cross-surface flows
  (`browser_to_cli`, `cli_to_browser`, `browser_to_browser`,
  `cli_to_cli`).
- **Cross-surface memory migration.** `src/memory.rs` implements
  `MemoryStore` (append-only), `export_links_notation`,
  `parse_links_notation`, `export_bundle`, and
  `extract_memory_from_bundle` over the same `demo_memory` schema
  that `src/web/memory.js` already speaks. `src/main.rs` adds three
  command groups — `formal-ai memory export|import|show`,
  `formal-ai bundle export|import`, and `formal-ai environments` —
  each of which accepts `--path <file>` or `-` for stdin/stdout so
  the flows in `environments.lino` work without temp files.
  `seed::parse_bundle` now tolerates both bundle dialects (the flat
  `formal_ai_seed_bundle` emitted by the Rust solver and the nested
  `formal_ai_bundle` emitted by the browser's `Download bundle`
  button), so a bundle generated in any surface round-trips back into
  any other surface.
- **Library-first availability.** `src/lib.rs` re-exports
  `MemoryStore`, `MemoryEvent`, `export_memory_links_notation`,
  `parse_memory_links_notation`, `export_memory_bundle`,
  `extract_memory_from_bundle`, `seed_files`, `merged_bundle`,
  `parse_bundle`, `environment_directory`, `environment_records`,
  `intent_routing`, `language_rules`, `multilingual_responses`,
  `prompt_patterns`, `response_for`, `seed_concepts`, and
  `agent_info` from the crate root. Every action the CLI binary or
  HTTP server performs is one library call away, so a downstream
  embedder can build a Tauri app, a VS Code extension, or another
  bot transport without reimplementing the storage or the seed.

Verification for this round:

- `cargo fmt --check`, `cargo clippy --all-targets --all-features
  -- -D warnings`, `cargo test --all-features` — all green.
- Manual CLI smoke test:
  `formal-ai memory import --path examples/memory-import.lino`,
  `formal-ai memory show`,
  `formal-ai bundle export --path examples/bundle.lino`,
  `formal-ai bundle import --path examples/bundle.lino --into examples/restored-memory.lino`
  round-trip a multilingual transcript without losing events.
- `formal-ai environments` prints the six environments and the four
  migration flows declared in `data/seed/environments.lino`.
