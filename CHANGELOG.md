# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->




































## [0.36.0] - 2026-05-16

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- `try_incompatible_units` handler: queries that mix dimensionally incompatible units
  (e.g. meters vs kilobytes) now return `intent:unit_incompatibility` with a clear
  symbolic explanation instead of falling through to `intent:unknown` (fixes #43).
- Five new `reasoning_paths` tests covering the Russian prompt from the bug report
  (`"Сколько метров в килобайте?"`), the English equivalent, evidence-link emission,
  and regression guards for greetings and arithmetic.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Fixed

- Inappropriate or vulgar prompts (e.g. Russian mat) now receive a polite policy refusal (`intent: policy_inappropriate_content`) with a language-matched response instead of the generic "intent: unknown" fallback. Applies to Russian, Hindi, Chinese, and English content. Fixes issue #39.

### Added
- Russian "назови " prefix recognized as a `concept_lookup` intent trigger (issue #30). The prompt "назови цвет" previously returned `intent: unknown`; it now resolves to `concept_lookup` and returns a definition of the color concept.
- `concept_color` seed record in `data/seed/concepts.lino` with full multilingual support (English, Russian, Hindi, Chinese), Wikidata anchor Q1075, and per-language localized blocks citing Wikipedia in each language.
- Two regression tests in `tests/unit/mvp/multilingual.rs` pinning down the reporter's exact prompt: `russian_nazovi_prefix_routes_to_concept_lookup` and `russian_nazovi_tsvet_answer_references_color`.
- `DEFAULT_CONCEPT_PREFIXES` fallback in `src/web/formal_ai_worker.js` updated to include "назови " so the browser worker mirrors the Rust pipeline when the seed has not yet been loaded.

### Fixed
- **Issue #44 — Topbar "Report issue" generates misleading title when session contains unknown-intent responses.** `createIssueTitle` and `createIssueReportBody` now fall back to the last `intent: unknown` assistant message as the effective focus when the user clicks the topbar button (no per-message `focusMessage`). This ensures the generated GitHub issue title reads `Unknown prompt: <prompt>` and the dialog body marks the relevant message as `(reported message)`, matching the behaviour already seen when clicking the per-message "Report missing rule" link.

### Fixed
- **Issue #50 — "шабат шалом!" not recognised as a greeting.** Added `шалом` as a greeting keyword and `шабат шалом` as a greeting phrase to `intent-routing.lino`, `greetings.lino`, and `prompt-patterns.lino`. The agent now routes these Hebrew-origin greetings (common in Russian-speaking communities) to the `greeting` intent and responds in Russian instead of returning the unknown-intent fallback.

### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).

### Fixed
- `scripts/version-and-commit.rs` now updates the workspace-package entry in `Cargo.lock` in the same release commit that bumps `Cargo.toml`. Previously every release left `Cargo.lock` stale, forcing follow-up "sync Cargo.lock" commits and producing avoidable merge conflicts on every open PR.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Russian prompts such as "покажи как ты работаешь?" now correctly resolve to `intent: meta_explanation` instead of falling back to `intent: unknown` (#51)

### Added
- Multilingual responses for the `meta_explanation` intent (English, Russian, Hindi, Chinese) so the agent explains how it works in the user's language
- Pattern recognition for "how do you work" / "show me how you work" style queries in English, Russian, Hindi, and Chinese
- Prompt patterns for `meta_explanation` intent in the routing seed

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

### Fixed
- Follow-up prompts such as "how it works?" or "how does it work?" after a concept lookup no longer return `intent: unknown`. A new `try_how_it_works` handler recognises these elaboration patterns, extracts the topic from an inline subject ("how does Wikipedia work?") or from the prior assistant reply, and either re-runs a concept lookup or returns a meaningful fallback. Five new regression tests cover the bare form, the explicit-subject form, the multi-turn history form, and the evidence-link audit requirement (issue #52).

### Fixed
- Russian phonetic transliterations "хелло" and "ворлд" are now recognized as valid hello/world tokens, and Russian language names "питоне" (Python), "расте" (Rust), and "джаваскрипт" (JavaScript) are now matched as language aliases. Previously, prompts like "Напиши хелло ворлд на питоне" fell through to `intent: unknown` (issue #53).

### Added
- Opinion question intent (`opinion_question`) that handles prompts like "Do you think space is continuous or discrete?" with a deterministic explanation instead of the generic unknown-intent error
- `try_opinion_question` handler in `solver_handlers.rs` detecting opinion/belief phrasings across multiple patterns
- Tests pinning the opinion question intent for the exact prompt from issue #42 and five related phrasings

### Fixed
- Issue #42: Opinion-style questions such as "Do you think space is continuous or discrete?" now return a helpful deterministic explanation instead of the confusing "I do not have a learned symbolic rule" fallback

## [0.35.0] - 2026-05-16

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Fixed

- Inappropriate or vulgar prompts (e.g. Russian mat) now receive a polite policy refusal (`intent: policy_inappropriate_content`) with a language-matched response instead of the generic "intent: unknown" fallback. Applies to Russian, Hindi, Chinese, and English content. Fixes issue #39.

### Added
- Russian "назови " prefix recognized as a `concept_lookup` intent trigger (issue #30). The prompt "назови цвет" previously returned `intent: unknown`; it now resolves to `concept_lookup` and returns a definition of the color concept.
- `concept_color` seed record in `data/seed/concepts.lino` with full multilingual support (English, Russian, Hindi, Chinese), Wikidata anchor Q1075, and per-language localized blocks citing Wikipedia in each language.
- Two regression tests in `tests/unit/mvp/multilingual.rs` pinning down the reporter's exact prompt: `russian_nazovi_prefix_routes_to_concept_lookup` and `russian_nazovi_tsvet_answer_references_color`.
- `DEFAULT_CONCEPT_PREFIXES` fallback in `src/web/formal_ai_worker.js` updated to include "назови " so the browser worker mirrors the Rust pipeline when the seed has not yet been loaded.

### Fixed
- **Issue #44 — Topbar "Report issue" generates misleading title when session contains unknown-intent responses.** `createIssueTitle` and `createIssueReportBody` now fall back to the last `intent: unknown` assistant message as the effective focus when the user clicks the topbar button (no per-message `focusMessage`). This ensures the generated GitHub issue title reads `Unknown prompt: <prompt>` and the dialog body marks the relevant message as `(reported message)`, matching the behaviour already seen when clicking the per-message "Report missing rule" link.

### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).

### Fixed
- `scripts/version-and-commit.rs` now updates the workspace-package entry in `Cargo.lock` in the same release commit that bumps `Cargo.toml`. Previously every release left `Cargo.lock` stale, forcing follow-up "sync Cargo.lock" commits and producing avoidable merge conflicts on every open PR.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

### Fixed
- Follow-up prompts such as "how it works?" or "how does it work?" after a concept lookup no longer return `intent: unknown`. A new `try_how_it_works` handler recognises these elaboration patterns, extracts the topic from an inline subject ("how does Wikipedia work?") or from the prior assistant reply, and either re-runs a concept lookup or returns a meaningful fallback. Five new regression tests cover the bare form, the explicit-subject form, the multi-turn history form, and the evidence-link audit requirement (issue #52).

### Fixed
- Russian phonetic transliterations "хелло" and "ворлд" are now recognized as valid hello/world tokens, and Russian language names "питоне" (Python), "расте" (Rust), and "джаваскрипт" (JavaScript) are now matched as language aliases. Previously, prompts like "Напиши хелло ворлд на питоне" fell through to `intent: unknown` (issue #53).

## [0.34.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).

### Fixed
- `scripts/version-and-commit.rs` now updates the workspace-package entry in `Cargo.lock` in the same release commit that bumps `Cargo.toml`. Previously every release left `Cargo.lock` stale, forcing follow-up "sync Cargo.lock" commits and producing avoidable merge conflicts on every open PR.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

### Fixed
- Follow-up prompts such as "how it works?" or "how does it work?" after a concept lookup no longer return `intent: unknown`. A new `try_how_it_works` handler recognises these elaboration patterns, extracts the topic from an inline subject ("how does Wikipedia work?") or from the prior assistant reply, and either re-runs a concept lookup or returns a meaningful fallback. Five new regression tests cover the bare form, the explicit-subject form, the multi-turn history form, and the evidence-link audit requirement (issue #52).

### Fixed
- Russian phonetic transliterations "хелло" and "ворлд" are now recognized as valid hello/world tokens, and Russian language names "питоне" (Python), "расте" (Rust), and "джаваскрипт" (JavaScript) are now matched as language aliases. Previously, prompts like "Напиши хелло ворлд на питоне" fell through to `intent: unknown` (issue #53).

## [0.33.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).

### Fixed
- `scripts/version-and-commit.rs` now updates the workspace-package entry in `Cargo.lock` in the same release commit that bumps `Cargo.toml`. Previously every release left `Cargo.lock` stale, forcing follow-up "sync Cargo.lock" commits and producing avoidable merge conflicts on every open PR.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

### Fixed
- Follow-up prompts such as "how it works?" or "how does it work?" after a concept lookup no longer return `intent: unknown`. A new `try_how_it_works` handler recognises these elaboration patterns, extracts the topic from an inline subject ("how does Wikipedia work?") or from the prior assistant reply, and either re-runs a concept lookup or returns a meaningful fallback. Five new regression tests cover the bare form, the explicit-subject form, the multi-turn history form, and the evidence-link audit requirement (issue #52).

## [0.32.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

## [0.31.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

### Fixed
- Added a visible hint message in the composer when demo mode is active, explaining how to stop the demo and type a custom message. Fixes confusion on Android and other mobile browsers where the demo-status text and button labels are hidden.

## [0.30.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

## [0.29.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.

## [0.28.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

## [0.27.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Added
- **Issue #27 — Mobile-responsive topbar and slide-out drawer.** Topbar buttons (Report issue, Export memory, Import memory, Diagnostics, Chat/Agent, Demo) now render an emoji icon next to their label; under the 820 px breakpoint the label collapses and only the emoji is shown while the full action name stays available via `aria-label` and `title`. A hamburger toggle (`☰` / `✕`) surfaces below the same breakpoint and slides the conversations / prompts / tools sidebar in over the chat as a drawer with a tappable backdrop, so the chat surface keeps the full viewport on phones. Three new Playwright tests under `Issue #27: mobile layout` pin the behavior at a 390×780 viewport.
- **Issue #27 — Chat / Agent mode toggle.** A new topbar toggle decomposes a single user message into sequential steps (`;` / `then` / Russian and Chinese equivalents) and executes each step through the existing deterministic solver, rendering the aggregated plan + per-step result as Markdown. Single-step prompts keep the plain Q&A surface. Covered by three e2e tests.
- **Issue #27 — Deterministic summarize skill.** A logical (no neural-network) summarizer projects the live conversation onto turn counts, languages, intents, concepts, calculations, hello-world programs, and unanswered questions, then renders a Markdown report. Triggered by EN / RU / HI / ZH phrasings (`summarize`, `резюме беседы`, `सारांश`, `总结`, …). The trigger now runs before normalization bails on non-Latin scripts, and `conversationHistory()` carries intent / evidence so per-turn classification survives into the summary. Four new e2e tests.
- **Issue #27 — Conversations sidebar.** Every appended event is now tagged with `conversationId` / `conversationTitle`, so the append-only log can be grouped per chat thread on read. The sidebar lists known threads (most-recent first), click-to-switch hydrates the transcript from IndexedDB, the `+ New conversation` button mints a fresh thread, and a new `currentConversationId` preference restores the last active conversation on reload. Three e2e tests under `Issue #27: conversations sidebar`.
- **Issue #27 — Random greeting variations.** Variant entries for the four greeting languages live alongside the canonical `text` in `data/seed/multilingual-responses.lino`; the JS seed loader returns `{ text, variants }` per response while the Rust parser ignores `variant` siblings so the shared seed stays compatible. A preference toggle exposes the random-variant selection and pins the canonical greeting when disabled.
- **Issue #27 — Natural-language Export/Import memory.** Typing `Export memory` / `Export your memory` (and ru / hi / zh translations) into the chat now triggers the Export memory toolbar action and produces an assistant acknowledgement; same for `Import memory` → Import memory dialog. The phrase recognizer normalizes casing / punctuation / whitespace before matching against a curated seed list so the chat surface and the sidebar stay in lock-step.
- **Issue #27 — Natural-language cross-conversation recall.** Typing recall prompts like `When did I ask about Rust?`, `search my conversations for Wikipedia`, `find Donald Trump in another conversation` (and ru / zh equivalents — `Когда я спрашивал про Илона Маска?`, `найди Илон Маск в другой беседе`, `我什么时候问过 Rust`, `在对话中搜索 Donald Trump`) now scans the IndexedDB event log, groups matching turns by `conversationId`, and renders a Markdown report with timestamps and excerpts. A scope suffix (`in another conversation` / `в другой беседе` / `在其他对话中`) restricts the search to threads other than the current one; otherwise all conversations are scanned. Two new Example prompts surface the skill in the sidebar, and four new e2e tests under `Issue #27: cross-conversation recall` pin the EN/RU phrasings, the cross-conversation scope filter, and the no-match path.
- `tests/e2e/playwright.adhoc.config.js` — Playwright config that listens on port 3499 to sidestep stale dev servers occupying 3456.
- `docs/case-studies/issue-27/` — raw issue + PR snapshots used as input to the case study.
- `docs/screenshots/desktop-topbar.png`, `docs/screenshots/mobile-topbar-closed.png`, `docs/screenshots/mobile-drawer-open.png` — visual regression baselines for the new responsive topbar.

### Fixed
- **Issue #27 — `Кто такой Илон Маск?` Wikipedia lookup.** Russian Wikipedia biographies use the `Surname, Given names` slug form (e.g. ru.wikipedia.org redirects `Илон_Маск` to `Маск,_Илон` and the REST summary endpoint 404s on the former). `wikipediaTermVariants` now appends the swapped two-word form so the lookup hits the canonical biography slug. Covered by a new e2e test that returns 404 for every variant except `Маск,_Илон` and asserts the chat renders the biography.
- **Issue #27 — Sidebar accordion regression coverage.** The VS Code-style sidebar (expanded sections share height via `flex: 1 1 0` and each body has `overflow: auto`) is now pinned by three Playwright tests under `Issue #27: sidebar accordion`: equal-height expanded sections, independent scroll on each body, and collapsing a section grows its siblings.

### Changed
- **Issue #27 — Demo cycle iterates the Example prompts list.** `createDemoTurns` now reads directly from `EXAMPLE_PROMPTS` and advances persistent `demoGreetingCursor` / `demoFeatureCursor` indices, so each demo cycle visits a new greeting + feature prompt in deterministic rotation (instead of repeatedly picking from a tiny hard-coded `DEMO_LANGUAGES` / `DEMO_GREETINGS` subset). Export / Import are filtered out since they would trigger file downloads. Each demo user message now exposes the source label via `data-demo-label` for regression testing.
- Replaced the `Download bundle` button with an alias of `Export memory` so the action stays available under both names while collapsing the duplicate toolbar entry, per the issue body.
- `tests/e2e/tests/demo.spec.js`: the `quick prompts sidebar` test now selects `Rust` by label instead of by absolute index because the example-prompts list now leads with multilingual greetings; the `unknown prompts …` test now uses a nonsense prompt that cannot resolve so the unknown-intent path is actually exercised.
- The Chat/Agent toggle test asserts on the `.btn-label` span because the topbar buttons now render `btn-icon` + `btn-label` so the label can collapse on the mobile breakpoint.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

## [0.26.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.
- New `data/seed/concept-contexts.lino` registry of disambiguating contexts (`context_machine_learning`, `context_signal_processing`, `context_neural_network`, `context_programming`) anchored by Wikidata Q-IDs (Q2539, Q1058710, Q192776, Q80006) with per-language `label "en|ru|hi|zh"` blocks and multilingual alias lists. The IIR record references the registry via a new `context_links "..."` field so contexts are wired as links rather than restated inline (issue #20 follow-up R8/R13).
- Per-language `localized "en|ru|hi|zh"` blocks on `ConceptRecord` carrying the native term, native aliases, native-Wikipedia summary, and source URL (e.g. `Фильтр с бесконечной импульсной характеристикой … или IIR-фильтр` from ru.wikipedia.org). The solver and JS worker prefer the prevailing-language block when rendering the response, so the original maintainer prompt `что такое iir в ml` now returns a Russian-language definition citing the Russian Wikipedia article (issue #20 follow-up R9/R10/R11).
- New `concept_lookup_in_context_no_alias` response template variant: when the user already typed the localized context label, the body renders as `В контексте Машинное обучение IIR …` instead of `«Машинное обучение» (Машинное обучение)`. The variant is picked by template id from `multilingual-responses.lino` so future languages add a row without Rust changes.
- `wikidata "Q…"` field on every concept and new `wikidata:Q…` evidence link so the cross-language join key used by `link-assistant/human-language` and `link-assistant/meta-expression` is part of the public trace (issue #20 follow-up R13).
- Additional Rust integration tests in `tests/unit/mvp/multilingual.rs` pinning down native-language body content (Russian, English, Hindi, Chinese), the `«ml» (Машинное обучение)` rendering, the ru.wikipedia.org source citation, the no-alias-duplication template, and the Wikidata anchor in `evidence_links`.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed. The ranker resolves context phrases through the new registry: a record can declare `context_links "context_machine_learning|…"` and `record_has_context` will follow the registry rather than require every alias inline.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` and `wikidata:*` evidence labels, picks the `_no_alias` template when needed, and renders both plain and in-context bodies from the localized block selected by `detectLanguage(prompt)`.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

## [0.25.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

## [0.24.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.
- Fixed live GitHub Pages Playwright navigation so tests preserve the `/formal-ai/` repository subpath and wait for seeded manual-mode data before multilingual prompts.

## [0.23.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

### Fixed
- Hardened the GitHub Pages demo release gate so live e2e tests wait for the exact deployed commit and load cache-busted static assets.

## [0.22.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

### Changed
- **Export memory** now produces the full, self-contained agent state by default on every interface (issue #18). The browser **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — seed files (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, version, and the full append-only event log — instead of only the in-session events. `formal-ai memory export` matches: it defaults to the full bundle and accepts `--events-only` to opt back into the legacy `demo_memory` shape (R109, R113).

### Added
- `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`, `FormalAiMemory.importFullMemory(text)`, and `FormalAiMemory.suggestMigrations({ imported, current })` in `src/web/memory.js`, mirrored on the Rust side by `memory::export_full_memory`, `memory::import_full_memory`, `memory::suggest_migrations`, `BundleInfo`, and `ParsedBundle` (re-exported from `formal_ai`'s crate root) so embedders writing their own surface get the same defaults (R109, R110, R111, R113).
- Header-agnostic imports: both `formal-ai memory import` / `formal-ai bundle import` (CLI) and `handleImportMemory` (browser) auto-detect `formal_ai_bundle` and legacy `demo_memory` documents so older exports keep round-tripping (R110).
- Seed-version migration suggestions on import. When the imported bundle's `agent_info.version` differs from the running app, the browser memory-status indicator and the CLI (`Migration: <message>` via `eprintln!`) tell the user what changed in `data/seed/` so they can take action without reading code (R111).
- Rewritten **Attach full memory** block in the prefilled "Report issue" body: explicitly tells users that the export is the full memory of the agent, walks them through wrapping it in a `.zip` on macOS / Windows / Linux (GitHub does not yet accept `.lino` attachments), and reminds them to redact sensitive content (R112).
- `docs/case-studies/issue-18/` with raw GitHub data, root-cause analysis, requirement-to-implementation mapping (R109-R114), and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js`: `Export memory downloads a full formal_ai_bundle by default` asserts `formal_ai_bundle` / `seed_files` / `preferences` in the downloaded file; `Import memory accepts a formal_ai_bundle and reports seed migrations` exercises the auto-detect path and the migration-suggestion surface; the report-issue test now also asserts the `.zip` and redaction guidance (R114).
- Rust unit coverage in `src/memory.rs`: `full_memory_round_trip_preserves_seed_preferences_and_events`, `import_full_memory_accepts_legacy_demo_memory`, `suggest_migrations_flags_seed_version_drift`, `suggest_migrations_flags_legacy_only_import`, `suggest_migrations_is_quiet_when_versions_match`.

## [0.21.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.

## [0.20.0] - 2026-05-15

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

### Added
- Added arithmetic, concept-lookup, conversation-recall, and explicit-JavaScript-execution handlers to the universal solver so the same loop now answers "what is 2 + 2?", "what is Wikipedia?", "what is my name?", and "please run this javascript: ..." prompts (issue #14).
- Added `solve_with_history`, `ConversationRole`, and `ConversationTurn` to the library API so every surface can carry conversation memory through the same solver loop.
- Added `data/seed/concepts.lino` with offline records for Wikipedia, Wikidata, Wiktionary, Links Notation, doublet links, the universal solver, the event log, WebAssembly, and Rust; each record cites its source for auditability.
- Added `tests/unit/mvp/reasoning_paths.rs` (R85–R88) with 24 tests pinning the new handlers and proving every answer is a projection of the append-only event log rather than a memoized constant.
- Added `examples/universal_solver_tour.rs` which walks every specialized handler through the same `FormalAiEngine::answer` entry point used by the library, CLI, HTTP server, Telegram bot, and demo.

### Changed
- Re-implemented `docs/demo/formal_ai_worker.js` as a universal-solver port: the hardcoded prompt→answer table is gone and every reply now walks greeting, identity, arithmetic, concept-lookup, recall, JavaScript-execution, hello-world, and unknown-fallback handlers that mirror `src/solver.rs`. JavaScript snippets requested with "please run this javascript" are actually executed in the worker sandbox; output, return value, and errors are reported with execution-status evidence.
- Persisted the demo's "Demo on/off" and "Diagnostics" toggles in `localStorage` using a Links Notation encoding (`docs/demo/preferences.js`) so the UI state survives reloads in the same format that grounds the solver's knowledge.

## [0.19.0] - 2026-05-14

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

### Added
- Added the issue #12 holistic case study, root vision/goals/non-goals documents, and a unit test that keeps the documentation set present and traceable.

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.

## [0.18.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

### Added
- Added a Telegram webhook endpoint backed by the symbolic engine, plus execution metadata for hello-world code answers and verification coverage for private and public chat updates.

### Added
- Added a `formal-ai telegram` CLI subcommand that defaults to Telegram long polling (`getUpdates`) and keeps the existing webhook server as `--mode=webhook`, configured through `lino-arguments` so flags, environment variables, and `.lenv`/`.env` files all feed the same parser.
- Introduced `TelegramPollingConfig`, `parse_get_updates_response`, the `TelegramTransport` trait, and a curl-backed default transport so the polling loop is fully unit-tested without a network.

## [0.17.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

### Added
- Added demo chat issue-report links that prefill GitHub issues with dialog history and browser metadata.
- Added a symbolic identity response for "Who are you?" style prompts.

### Removed
- Removed the unused demo composer preview control.

## [0.16.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

### Changed

- Start the web demo in interactive demo mode by default, add a live next-dialog countdown, and hide diagnostic trace details unless diagnostics mode is enabled.

## [0.15.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

### Fixed
- Deploy the demo through the GitHub Pages workflow artifact path so workflow-sourced Pages deployments fail accurately and expose the deployed URL to Pages e2e tests.

## [0.14.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.

## [0.13.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.

## [0.12.0] - 2026-05-12

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

## [0.11.0] - 2026-05-09

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

## [0.10.0] - 2026-05-09

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

## [0.9.0] - 2026-05-03

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

## [0.8.0] - 2026-05-01

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

## [0.7.0] - 2026-04-14

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

## [0.6.0] - 2026-04-13

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

## [0.5.0] - 2026-04-13

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

## [0.4.0] - 2026-04-13

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

## [0.3.0] - 2026-04-13

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

## [0.2.0] - 2026-03-11

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

## [0.1.0] - 2025-01-XX

### Added

- Initial project structure
- Basic example functions (add, multiply, delay)
- Comprehensive test suite
- Code quality tools (rustfmt, clippy)
- Pre-commit hooks configuration
- GitHub Actions CI/CD pipeline
- Changelog fragment system (similar to Changesets/Scriv)
- Release automation (GitHub releases)
- Template structure for AI-driven Rust development