# Changelog archive: 0.105.0 to 0.175.0

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

## [0.175.0] - 2026-06-01

### Fixed
- Fixed GitHub repository extraction prompts so repository URLs without schemes route correctly, avoid false configuration capability answers, and keep JSON formatting directives attached to the extraction task.

## [0.174.0] - 2026-06-01

### Fixed
- Resolved relational apple-box arithmetic word problems by reducing box facts into calculator expressions with step-by-step reasoning in Rust and the browser worker.

## [0.173.0] - 2026-06-01

### Fixed
- Answer compound-interest prompts that ask for step-by-step calculation and a
  follow-up EUR conversion instead of falling through to the unknown fallback.

## [0.172.0] - 2026-06-01

### Fixed
- Agent-mode Wikipedia research prompts now split quoted patent comparisons into focused searches and strip follow-up instructions from web-search queries.

### Fixed
- Routed the Russian `SPEC dirven development` how-to prompt through procedural planning with a `dirven` to `driven` typo correction instead of the unknown fallback.

## [0.171.0] - 2026-06-01

### Fixed

- Kept agent-mode research table follow-up prompts tied to the prior web-search step instead of falling through to the unknown response.

## [0.170.0] - 2026-05-31

### Fixed
- Route the budget-calculator composite prompt to a Python `write_program` blueprint instead of reducing it to web search or an unsupported template.

## [0.169.0] - 2026-05-31

### Changed
- Documented local-server setup for Codex, Claude Code, OpenCode, and Link Assistant Agent in the README and server API guide.

## [0.167.0] - 2026-05-31

### Added
- Add a white-box self-improvement loop that proposes learned Links Notation seed rules from accumulated unknown traces and gates adoption behind the coding-modification benchmark ratchet.

## [0.166.0] - 2026-05-30

### Fixed
- Reduced response-level "Report issue" prompts to unresolved unknown turns and included the focused reasoning trace in missing-rule report URLs.

### Changed
- Softened multilingual unknown fallback copy so reporting is framed as a last-resort seed-extension path.

## [0.165.0] - 2026-05-30

### Added

- Added an issue #362 multilingual multi-turn coding-modification benchmark with
  a deterministic `minimum_pass_count` ratchet and download-on-test provenance
  for CanItEdit, HumanEvalFix, and EDIT-Bench.

## [0.164.0] - 2026-05-30

### Fixed
- Added an issue #361 cross-runtime parity harness that verifies the browser
  worker mirrors the Rust core for the issue #349 reverse-sort follow-up,
  including the unknown-path rule synthesis trace and the no-active-program
  guard.

## [0.163.0] - 2026-05-30

### Fixed
- Added full diagnostic traces for synthesized write-program follow-ups, including route attempts, coreference binding, modifier detection, rule construction, verification, and program-plan lowering.

## [0.162.0] - 2026-05-30

### Added
- Added unknown-path rule construction and verification traces for resolvable program-modification follow-ups.

## [0.161.0] - 2026-05-30

### Added
- Generalized `write_program` modifiers so operation-vocabulary slugs referenced
  by program-plan rules are discovered as modifiers instead of being hard-coded.
- Added reverse-sorted file-listing program variants, including the composed
  path-argument plus reverse-sort variant across supported template languages.

### Fixed
- Reverse-sort follow-ups to file-listing programs now lower to reverse-sorted
  program output instead of reusing the ascending file-listing variant.

## [0.160.0] - 2026-05-30

Fixed
- Route bare program-result follow-ups back to the active generated program artifact across English, Russian, Hindi, and Chinese.

## [0.159.0] - 2026-05-30

### Added

- Added the issue #356 rule-synthesis design for constructing verified
  substitution rules over Links Notation, plus a docs traceability test that
  pins the core contract for #357, #358, and #359.

## [0.158.0] - 2026-05-30

### Added
- VS Code extension (`vscode/`) that embeds the committed `src/web/` chat UI inside a Webview around the same HTTP/web boundary as the browser, the HTTP server, and the Electron desktop shell — no forked UI (issue #353).
- Dual-host packaging from one manifest: a Node host (`src/extension.node.cjs`, `shell: "VS Code"`) that starts an opt-in loopback `formal-ai serve` process, routes chat through `POST /v1/chat/completions`, and can drive Docker-sandboxed code execution; and a Web Worker host (`src/extension.web.cjs`, `shell: "VS Code Web"`) for `vscode.dev` / `github.dev` that stays on the in-process WebAssembly engine and imports no `node:*` builtins.
- Reusable pure extension libraries (`vscode/src/lib/`): `config.cjs` (settings → `desktopStatus` mapping), `bridge.cjs` (host-agnostic, default-deny `FormalAiDesktop` dispatcher), `webview-html.cjs` (Webview sandbox reconciliation — `<base href>`, strict nonce CSP, same-origin blob Worker bootstrap, main-thread/worker `fetch` and `importScripts` seed rebasing, and the `postMessage` bridge), `chat-view.cjs` (shared `WebviewView` provider), and `server-process.cjs` (Node-only `formal-ai serve` discovery / health-wait / spawn). Each takes its effectful dependencies by injection so it is unit-testable without a live VS Code host.
- Six `formal-ai.*` settings (server enabled/host/port, docker image, default tool grants, default agent mode) and four commands (Open Chat, Toggle Local Server, Sync Memory, Open Network View); the extension declares `virtualWorkspaces` and `untrustedWorkspaces` support because the in-process agent is safe everywhere while the server/Docker features only run in trusted desktop windows.
- `vscode` environment declared in the canonical seed (`data/seed/environments.lino`) with `browser_to_vscode` and `vscode_local_sync` flows, plus a strengthened `environment_directory_declares_every_supported_surface` unit test.
- VS Code spec test (`tests/unit/specification/vscode_surface.rs`, 13 cases) that pins the dual-host file contracts and exercises the shared engine endpoints (`/v1/chat/completions`, `/v1/graph`, full-bundle memory round-trip) to prove "all the same features", and a Playwright e2e spec (`tests/e2e/tests/issue-353.spec.js`) asserting the VS Code surface labelling for both hosts.
- `npm run vscode:dev` / `vscode:package` / `vscode:smoke` / `vscode:test` root scripts, with the VS Code node test suite wired into the CI lint job; `.cjs` files now count as code changes in `detect-code-changes.rs` so extension-host edits trigger lint/test/changelog.
- Architecture docs (`docs/vscode/extension.md`), a Marketplace README (`vscode/README.md`), a README VS Code section, and an issue-353 case study (`docs/case-studies/issue-353/`).

### Changed
- The web app's desktop status label is now surface-aware: `desktopSurfaceLabel(status)` returns "VS Code" when the host shell matches `/code/i` (so both `"VS Code"` and `"VS Code Web"` read as *VS Code*), otherwise "Desktop". The Electron shell is unaffected.

## [0.157.0] - 2026-05-30

### Added
- Added an ignored regression test and runnable example that reproduce issue #349's Russian reverse-sort follow-up for issue #355.

## [0.156.0] - 2026-05-30

### Fixed
- Answer prompts asking which dollar exchange rate is used for calculations by delegating the USD/RUB lookup to `link-calculator`.

## [0.155.0] - 2026-05-30

### Fixed
- Recognize Russian "Привет давай знакомиться!" and equivalent get-acquainted prompts as identity self-introduction requests instead of unknown prompts.

## [0.154.0] - 2026-05-30

### Added
- `/download` landing page for formal-ai Desktop (`src/web/download/`), modelled on
  vk-bot-desktop: OS auto-detection with macOS/Windows/Linux tabs, a release grid
  fed from the GitHub Releases API, in-browser SHA-256 checksum verification
  against `SHA256SUMS.txt`, build-provenance guidance, and macOS Gatekeeper notes.
  It honours the existing theme and locale switching (en/ru/zh/hi) and ships a CSP
  (issue #347, R1/R2/R7).
- Cross-platform desktop release pipeline (`.github/workflows/desktop-release.yml`)
  with explicit electron-builder `artifactName` templates, `SHA256SUMS.txt`,
  `BUILD-PROVENANCE.txt`, SLSA build-provenance attestation, and release-asset
  upload for macOS, Windows, and Linux (R1).
- Playwright e2e coverage for `/download` (`tests/e2e/tests/issue-347.spec.js`) and
  CI-generated theme/locale screenshots committed under `docs/screenshots/issue-347/`
  (R2).
- `docs/desktop/server-api.md`: how to enable the opt-in local OpenAI-compatible
  server (`formal-ai serve`) and point the `codex`, `agent`, and `claude` CLIs at
  it, with bearer-token auth and the in-process-by-default contract (R3/R4).
- `docs/case-studies/issue-347/` case study (requirements, prior-art survey,
  CI/CD-template comparison) plus a `ROADMAP.md` documenting the R5c/R5d/R6
  implementation (R8/R9/R10).
- Local-database sync (R5c): `src/memory_sync.rs` (`SyncStore` + union-by-id
  merge) and `GET /v1/memory`, `GET /v1/memory/since`, `POST /v1/memory/import`
  endpoints, with `desktop/lib/memory-sync.cjs` reconciling the browser
  (IndexedDB) log with the native store while server mode is on.
- Local-execution routing (R5d): `desktop/lib/tool-router.cjs`, a default-deny,
  permission-gated tool dispatcher. `http_fetch` / `url_navigate` /
  `read_local_file` are served by the local process; `eval_js` / `code_exec` /
  `shell` run inside the `konard/box-dind:2.1.1` Docker sandbox with logs
  captured. Denied calls (the default) return a structured refusal and nothing
  executes; Docker absence refuses rather than running unsandboxed.
- Links-Notation REST envelopes + LinksQL (R6): `GET /v1/bundle`, `GET /v1/links`
  (the knowledge graph as a `knowledge_graph` document), and `POST /v1/links/query`
  returning a `links_query_result` envelope, backed by the read-only LinksQL
  evaluator in `src/links_query.rs` (`MATCH (a)-[r]->(b) WHERE … RETURN …`).
- First-party Anthropic→OpenAI adapter (R4): `POST /v1/messages` (`src/anthropic.rs`)
  translates the Anthropic Messages protocol to the existing solver and back,
  including SSE streaming, so `claude` targets the local server via
  `ANTHROPIC_BASE_URL` without a third-party proxy.

### Fixed
- Web app (`src/web/app.js`): declared the R5c `syncDesktopMemoryNow` callback
  before the effect that lists it as a dependency. React evaluates a hook's
  dependency array during render, so referencing the later `const … =
  useCallback(…)` hit its temporal dead zone and threw
  `ReferenceError: Cannot access 'syncDesktopMemoryNow' before initialization`,
  crashing the whole component before it could mount. Added a static guard
  (`tests/e2e/scripts/check-web-tdz.mjs`, wired into the lint job as
  `check:web-tdz`) that fails CI if any hook dependency array references a
  `useCallback`/`useMemo` const declared later in the same component.

### Changed
- The desktop shell (`desktop/main.cjs`) now runs the **in-process** reasoning
  agent by default and only starts the local OpenAI-compatible server when
  `FORMAL_AI_DESKTOP_SERVER` is set — matching the `/download` page copy and the
  in-process-by-default requirement. The web app routes chat to the local server
  only when the Electron bridge reports it is ready, otherwise it stays in-process
  (R3/R4/R5a/R5b).

## [0.153.0] - 2026-05-29

### Added
- Composite `write_program` **blueprints**: a request the verified template
  catalog cannot resolve to a single alias (e.g. "make an HTTP GET request,
  parse the JSON, compute the mean and median, and output the results with error
  handling and comments") no longer dead-ends on `write_program_unsupported`.
  The blueprint synthesizer (`src/coding/blueprint.rs`) decomposes the prompt
  into capabilities (http_request, json_parse, statistics, output_results,
  error_handling, comments — each matched in English, Russian, Hindi, and
  Chinese), matches a recipe (`http_json_stats`), and returns a real, idiomatic
  program for Rust, Python, or JavaScript together with a numbered decomposition
  plan, the required libraries, and how-to-run instructions.
- Honest execution contract for blueprints: because composite programs need
  external libraries and network access the offline sandbox cannot provide, the
  blueprint is always reported as **"not run"** and never claims it "compiled and
  ran". The decomposition is recorded as `program_blueprint:` trace links and a
  `response:write_program:blueprint:<recipe>:<language>` evidence link.
- Case study `docs/case-studies/issue-340/` with timeline, requirements,
  root-cause analysis, solution plans, and an existing-components review.
- Two independent compositional axes: a blueprint program is now a *projection*
  of its decomposed capabilities rather than a single frozen string.
  - `comments` axis — when the request asks for comments the documented program
    is emitted; otherwise whole-line documentation (and a leading Python
    docstring) is stripped.
  - `error_handling` axis — optional defensive blocks are wrapped in
    `// region:error_handling … // endregion:error_handling` markers (`#` for
    Python/Ruby): the Rust empty-input guard, the Python `raise_for_status` +
    empty-list guard, and the JavaScript `!response.ok` + empty-array guard. The
    marker lines are always stripped from output; the region body is kept only
    when the request asks for error handling.
  The axes are orthogonal, so one recipe yields the full cross-product of four
  distinct, still-compilable programs (`documented`, `comments_only`,
  `errors_only`, `stripped`) — reasoning from the decomposition instead of
  memoizing one answer (`NON-GOALS.md`). Verified by unit tests in
  `src/coding/blueprint_tests.rs`, mirrored in the JS worker, and compile-checked
  offline via `examples/issue_340_emit_variants.rs` (each emitted Python/JS
  variant passes `py_compile` / `node --check`).
- `BlueprintComposition` setting ("Program composition"): switches the synthesis
  strategy between `Composed` (default — project the program from the decomposed
  capabilities) and `Documented` (always emit the fully annotated program with
  every region and comment). Exposed as a dropdown in the demo UI, toggleable by
  natural language ("documented programs", "полная документация", …), persisted
  in preferences, forwarded to the worker, localized across all four lino-i18n
  locales (en/ru/hi/zh), and reported in the self-facts inventory as
  `relation "blueprint_composition"` across the Rust core, JS worker, and app.js
  local fallback.

### Changed
- Browser worker parity (R7): `src/web/formal_ai_worker.js` mirrors the
  blueprint synthesizer byte-for-byte, so the GitHub Pages WASM/JS demo answers
  composite program requests identically to the Rust core. A `vm`-sandboxed
  parity experiment (`experiments/issue-340-worker-parity.mjs`) asserts both
  engines agree across English/Russian Rust, Python, and JavaScript variants,
  that both the `comments` and `error_handling` axes compose identically in both
  engines, that the `Documented` strategy keeps every region/comment, that the
  active composition is reported in the self-facts, and that partial requests
  (no statistics) stay honestly unsupported.

## [0.152.0] - 2026-05-29

### Added
- Recursive `fibonacci` coding task in the coding catalog (Rust catalog, `.lino`
  seed, and the WASM/JS worker), so prompts like "Write a Python function that
  calculates the Fibonacci sequence recursively" generate a verified program
  that prints F(10) = 55 (issue #334).
- Natural-language "word problem" normalizer that resolves "(the) N-th Fibonacci
  number" references, rewrites spelled-out operators ("and multiply it by" →
  `*`), and drops trailing instruction sentences, so "calculate the 10th
  Fibonacci number and multiply it by 8% of 500" reduces to `55 * 8% of 500`
  = 2200 (issue #334).

### Fixed
- The shared `no_std` arithmetic evaluator (used by the CLI fallback, the
  compiled WASM worker, and the JS worker fallback) now understands "N% of M"
  percentage-of phrases, rewriting `8% of 500` to `( 8 * 500 / 100 )` so the
  GitHub Pages WASM demo evaluates `55 * 8% of 500` to 2200 instead of returning
  "unparseable". A bare `%` not followed by "of" still parses as modulo
  (issue #334).
- Coding prompts containing "number" or "program" are no longer misread as a
  unit-incompatibility conversion: unit tokens such as "mb" and "gram" now match
  only on word boundaries instead of as substrings of "nu**mb**er" /
  "pro**gram**" (issue #334).

### Added
- Software-project follow-up handler so a decomposed agent step such as "test it
  by scraping wikipedia.org and show me the top 10 most frequent words" stays
  bound to the active project dialogue. It formalizes a `software_project_followup`
  meaning (parent request, follow-up kind, target site, expected output) with
  `generated_code`, `test_execution`, and `network_access` approval gates instead
  of running the test. Verification/execution/demonstration verbs are recognized
  across all supported languages (en, ru, hi, zh), and the handler is mirrored in
  both the Rust solver and the browser worker (issue #341).

### Fixed
- A software-project test/run/verify follow-up no longer misroutes to a
  `wikipedia` concept lookup (online) or the unknown-intent opener (offline)
  after the first plan turn (issue #341).

## [0.151.0] - 2026-05-29

### Added
- Syntax highlighting for chat code blocks via a dependency-free, highlight.js-compatible tokenizer (`src/web/syntax-highlight.js`) covering rust, python, javascript/typescript, go, c, cpp, java, csharp, ruby, bash, and json (issue #330).
- A copy button on every rendered code block that copies the raw source to the clipboard with "Copied!" feedback.
- A "Copy as Markdown" button on each chat message that copies the whole message content (Markdown fences preserved).
- Localized strings for the new copy buttons in all four locales (en/ru/zh/hi).
- End-to-end Playwright tests proving highlighting renders and both copy buttons work against a freshly built `src/web`.
- Runnable example (`examples/issue-330-code-highlighting/`) with run/test instructions and a deep case study in `docs/case-studies/issue-330/`.
- Code answers now teach a novice: every generated program is followed by a localized "How it works" explanation and step-by-step "How to test it yourself" instructions (install the toolchain, save the file, compile, run, compare the output) in en/ru/hi/zh (issue #330).
- When the dialog already walked the user through running code, a follow-up code edit omits the verbose setup steps and shows a concise "test it the same way" note instead, detected from prior assistant turns in the conversation history.
- Four new deterministic coding tasks broaden the catalog beyond hello-world and list-files — FizzBuzz, factorial of 5, string reversal, and the sum from 1 to 10 — each with a verified fixed output and templates for all ten supported languages, reachable in en/ru/hi/zh (issue #330).
- The JavaScript demo worker (`src/web/formal_ai_worker.js`) now mirrors the full Rust catalog (the four new tasks, all ten languages with their setup/run/check metadata) and the novice "How it works"/"How to test" guidance, keeping the in-browser engine in lockstep with the Rust engine.

### Changed
- Reorganized the coding-task support into a cohesive `src/coding/` module — a `catalog/` submodule (`types.rs` for the records, `languages.rs`/`tasks.rs` for the catalog tables, `templates_core.rs`/`templates_extended.rs` for the per-language templates, and `mod.rs` for the lookups) plus `guidance.rs` for the novice "How it works"/"How to test" guidance — replacing the misleadingly named `src/engine_hello_world.rs` and `src/engine_program_guidance.rs`. The module covers general coding tasks across all ten supported languages, not only hello-world, and every file stays well under the repository's per-file line limit (issue #330).

## [0.150.0] - 2026-05-29

### Added
- Added `ROADMAP.md`, an implementation-progress tracker that maps every `VISION.md` pillar to its real `src/` status, the closed planning batches, and the planning epic that closes each remaining gap (issue #244).
- Added the issue #244 case study under `docs/case-studies/issue-244/`: a deep analysis (`README.md`), a structured code audit (`raw-data/code-audit.md`), summarized online prior-art research (`raw-data/online-research.md`), the full body and acceptance criteria of every planning epic (`proposed-issues.md`), and the raw issue/PR/CI snapshots.

- Opened the 14 vision-implementation planning issues (E1–E14, [#246](https://github.com/link-assistant/formal-ai/issues/246)–[#259](https://github.com/link-assistant/formal-ai/issues/259)), each linked to #244 and labeled `enhancement`, and recorded their numbers in `ROADMAP.md` and the case-study "Created Planning Issues" table.
- Added the 2026-05-26 post-implementation audit after E1-E14 were merged, preserving closed-issue/merged-PR/deferred-marker snapshots under `docs/case-studies/issue-244/raw-data/`.
- Opened the remaining follow-up batch E15-E20 ([#278](https://github.com/link-assistant/formal-ai/issues/278)–[#283](https://github.com/link-assistant/formal-ai/issues/283)) for native doublets storage, symbolic probabilistic ranking, desktop packaging, associative packages/permissions, Rust/WASM parity, and generalized skill compilation.
- Opened the reasoning-focused batch E21-E27 ([#298](https://github.com/link-assistant/formal-ai/issues/298)–[#304](https://github.com/link-assistant/formal-ai/issues/304)) after a third-pass audit on issue #244 feedback: reasoning under unknowns instead of a canned fallback, intent formalization as Links Notation (dropping the fixed catalogue), parametric `write a program` intents, `link-cli`-style `replace x y` / `when n do m` substitution rules over link CRUD, natural-language access to memory/APIs/code execution, a general code-modifying/executing agent, and permissively-licensed industry benchmark datasets.
- Opened the synthesis-focused batch E28-E32 ([#313](https://github.com/link-assistant/formal-ai/issues/313)–[#317](https://github.com/link-assistant/formal-ai/issues/317)) after a fourth-pass audit on issue #244 feedback: the universal 11-step loop is the verified single main path, but the synthesis step still resolves seeded answers instead of deriving them by composing decomposed sub-results over the links network (the imported industry benchmark suite passed 0/5 at the time; it now passes 10/10 after E28-E32 merged — see the 2026-05-29 entry). The batch adds a general link-native synthesis substrate, derived math/word-problem and counting answers, general program synthesis from spec + tests, general text manipulation over link structure, and a ratcheting benchmark suite — each bound by an anti-memorization rule (pass counts must rise via derivation, with paraphrased/renumbered held-out variants passing only when composed, never recalled).
- Embedded the hand-drawn universal problem-solving algorithm diagram in `README.md` with a stage→11-step-loop mapping table that points at `src/solver.rs`, `src/solver_unknown_reasoning.rs`, and `src/intent_formalization.rs`.

### Changed
- Reconciled stale documentation with the real state of the code: `ARCHITECTURE.md` §17 now references the `REQUIREMENTS.md` matrix as R1 … R251 (was R1 … R149) and links `ROADMAP.md`, and `REQUIREMENTS.md` gains an Issue #244 vision-planning section (R246–R251).
- Refreshed `ROADMAP.md`, `VISION.md`, `ARCHITECTURE.md`, `REQUIREMENTS.md`, and the issue #244 case study so they record E1-E27 as closed/merged (PRs #305-#311) and scope the then-remaining gap — generality of the synthesis step — to the E28-E32 batch, instead of describing the original 69-test planning backlog. (The 2026-05-29 entry records E28-E32 as merged and the benchmark suite at 10/10.)

### Added
- Implemented E33 ([#326](https://github.com/link-assistant/formal-ai/issues/326)): a single shared, data-driven multilingual operation vocabulary (`data/seed/operation-vocabulary.lino`, loaded by both the Rust core via `seed::operation_vocabulary()` and the browser worker via `seed_loader.js`). The text-manipulation handler now canonicalises every transform (uppercase, lowercase, reverse words, extract email, count occurrences, count unique words, deduplicate lines, sort lines, replace) against this table instead of matching hardcoded English literals, so a request triggers equally from native `en|ru|hi|zh` phrasing. Adding a new surface form or language is now a seed-data edit, never a code change. Covered by cross-language specs in `tests/unit/specification/text_manipulation.rs` and unit tests in `src/seed/operation_vocabulary.rs`.
- Opened the parity batch E33-E34 ([#326](https://github.com/link-assistant/formal-ai/issues/326)-[#327](https://github.com/link-assistant/formal-ai/issues/327)) after a fifth-pass audit on the issue #244 PR feedback ("all Rust and JavaScript logic are in sync", "all languages are supported equally"); E34 tracks porting the E28-E31 derivation paths into the JavaScript browser worker so it mirrors the Rust core.

### Changed
- Synced the issue #244 tracking docs to the post-E32 state: `ROADMAP.md`, `VISION.md`, `ARCHITECTURE.md` §16, `REQUIREMENTS.md`, and the case study now record E28-E32 ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317)) as closed/merged (PRs #319-#323), the synthesis step as deriving rather than seeding answers, and the industry benchmark suite as passing **10/10** with a `minimum_pass_count` ratchet (was the stale "0/5"). Vision pillars 24-26 move to **Built**.
- Recorded the fifth-pass parity audit in `docs/case-studies/issue-244/README.md`, scoping the remaining vision gap to cross-language and cross-runtime parity.

## [0.149.0] - 2026-05-29

### Added
- Mirror Rust synthesis, program synthesis, and text manipulation parity cases in the browser worker with a shared Rust/JS parity fixture.

### Fixed
- Prevent browser-worker synthesis prompts from falling through to the unknown or legacy template paths when the Rust core can derive the answer.

### Added
- Added a shared multilingual operation vocabulary so text manipulation and program synthesis recognize operation verbs across English, Russian, Hindi, and Chinese.

## [0.148.0] - 2026-05-29

### Added
- Response-language preference (`last message language` default, `preferred selected language`, or `UI language`) in the web app, with new `settings.responseLanguage` / `settings.preferredLanguage` i18n entries for all four locales.
- `list_files_arg` `write_program` task (list files at a path supplied on argv) with templates for all ten catalog languages.
- Conversation-context recovery for follow-up program modifications: a follow-up such as "make the program accept a path as an argument" now reuses the language and task from the prior turn instead of failing with `missing`/`missing`.
- Data-driven program-modification pipeline (`src/program_plan.rs`, mirrored in the browser worker) that represents the request as a Links Notation plan and lowers it through the substitution engine using rules defined as data in `data/seed/program-plan-rules.lino` (e.g. `path_argument` rewrites `list_files` → `list_files_arg`). Adding a new `(modifier → task-variant)` rewrite is pure rule data, proven by data-driven tests in both the Rust core and the JS worker. The lowered plan is surfaced as a `write_program_plan:` evidence link (Issue #324 R4/R6).
- Case study `docs/case-studies/issue-324/` with timeline, root-cause analysis, solution plans, and a universal dynamic problem-solving roadmap.

### Fixed
- `write_program` answers (intro, unsupported message, and execution report) are now rendered in the detected response language for Russian, Hindi, and Chinese instead of always English. Applied in both the Rust engine and the browser worker so the GitHub Pages demo stays in parity.

## [0.147.0] - 2026-05-28

### Added
- Expanded the industry benchmark fixture with held-out variants and a recorded pass-count ratchet.

## [0.146.0] - 2026-05-28

### Added
- Added formalized text-manipulation routing backed by composed substitution rules for transforms, rewrites, extraction, counting, line operations, and multi-step text workflows.

## [0.145.0] - 2026-05-28

### Added
- Added Python function synthesis with isolated verification for HumanEval/MBPP-style write-program prompts.

## [0.144.0] - 2026-05-28

### Added
- Added deterministic synthesis traces and anti-memorization coverage for GSM8K-style word problems, algebra substitution, and category-filtered object counting.

## [0.143.0] - 2026-05-28

### Added
- Added link-native synthesis over solved sub-impulse links for algebra substitution, remainder-sale word problems, and object counting.

## [0.142.0] - 2026-05-27

### Added
- `write_program` now answers "list the files in the current directory" requests
  for every catalog language (Rust, Python, JavaScript, TypeScript, Go, C, C++,
  Java, C#, Ruby). The Rust template uses `std::fs::read_dir`, matching what
  general assistants return for issue #312. The task is recognized in English,
  Russian, Hindi, and Chinese prompts.
- CJK-aware token and phrase matching in the program intent detectors
  (`engine_hello_world.rs`, `intent_formalization.rs`, and the web worker), since
  Chinese has no inter-word spaces and could not be matched by whitespace-split
  tokenization.
- Case study `docs/case-studies/issue-312/` documenting the timeline, the full
  list of requirements, root-cause analysis, the solution plan, and online
  research into how `read_dir`-based file listing is idiomatically written.

### Fixed
- A concrete `write_program` request (recognized task + language with a matching
  template) now takes precedence over the specialized handlers. Previously a
  prompt naming a language could be intercepted by `concept_lookup` and answered
  as an encyclopedia definition ("Rust") instead of returning the requested
  program.
- The JS-fallback `normalizePrompt` in the web worker now preserves the
  Devanagari block (U+0900–U+097F, including combining marks), restoring parity
  with the Rust `normalize_prompt` so Hindi prompts are matched identically on
  both code paths.

## [0.141.0] - 2026-05-26

### Added
- Add a curated permissive benchmark slice for HumanEval, MBPP, GSM8K, MATH, and BIG-bench with provenance notes and a deterministic pass/fail reporting test harness.

## [0.140.0] - 2026-05-26

### Added
- Add a bounded agent workspace runtime that can create, modify, delete, and inspect files through logged sandbox actions.

## [0.139.0] - 2026-05-26

### Added
- Natural-language API and code execution requests now go through agent-mode and associative-package permission gates, with auditable tool parameters, results, and execution status links.

## [0.138.0] - 2026-05-26

- Added the parameterized `write_program(language, task)` intent for seeded program generation, replacing per-language hello-world routing with language/task template parameters.
- Extended the catalog with `count_to_three` templates and unsupported-parameter responses so missing languages or missing language/task combinations fail explicitly.
- Updated Rust and browser-demo behavior-rule/tool metadata to advertise the single `write_program` path.

### Added
- Added a data-driven substitution-rule engine for link-pattern `replace x y` rules, conditional `when ... do ...` composition, CRUD event triggers, and trace-link records.

## [0.137.0] - 2026-05-26

### Added
- Added Links-Notation intent formalization with an impulse-id cache and routed the Rust solver from the formalized intent record.

## [0.136.0] - 2026-05-26

### Added

- Added traced unknown-prompt reasoning that records knowns, unknowns, candidate sources, and gather attempts before using the legacy fallback.

## [0.135.0] - 2026-05-26

### Fixed
- Localized behavior-rule list, detail, and dialog rule-update responses for supported UI languages.
- Added multilingual coverage for the reported "list your rules" phrasing and tightened chat markdown rendering for rule lists.

## [0.134.0] - 2026-05-26

Issue #288: add a seeded concept entry for `ложная тотальность` / `false
totality` so Russian manual-mode prompts such as `Что такое ложная
тотальность?` resolve through local concept lookup instead of the
unknown-intent fallback.

## [0.133.0] - 2026-05-26

### Changed
- Moved browser-worker stable id generation, unknown-answer opener selection, and intent-route matching semantics behind the Rust/WASM core so multilingual browser answers stay aligned with the native solver.

### Added
- Extended the natural-language skill compiler with a deterministic structured subset for typed inputs, procedure steps, generated tests, handler stubs, and explicit package/tool permissions.

### Fixed
- Recognize the Russian assistant-name command `Теперь тебя зовут ...` in the web demo and persist the configured name instead of falling through to the unknown-rule answer.

Issue #286: add a seeded concept entry for `антирежим` / `antiregime` so
Russian manual-mode prompts such as `Что такое антирежим?` resolve through
local concept lookup instead of the unknown-intent fallback.

## [0.132.0] - 2026-05-26

### Added
- Added reusable associative packages with dependency validation, Links Notation import/export, trigger replay, package permission checks for tool calls, and graph visibility for package handler/trigger/permission links.

## [0.131.0] - 2026-05-26

### Added

- Added an Electron desktop wrapper that starts the local Rust HTTP API, reuses the existing web chat, and exposes desktop API, graph, memory, and permission status.

## [0.130.0] - 2026-05-26

### Added
- Added link-native symbolic probability evidence with deterministic Bayesian-style and Markov-style ranking over formalization and answer candidates.
- Added probability evidence replay into traces and link-store memory, including cached-source provenance and offline-mode handling.

## [0.129.0] - 2026-05-26

### Changed
- Make `doublets-rs` the default native link-store backend while preserving Links Notation import/export as the recovery and migration projection.

## [0.128.0] - 2026-05-26

### Fixed
- Issue #272: recognize Russian prompts such as `А в чём ты можешь быть полезен` as capability questions instead of returning the unknown-intent teaching fallback.

## [0.127.0] - 2026-05-26

### Fixed
- Issue #262: recognize the Russian acknowledgement `ого, чето начал соображать:)` as a courtesy response instead of returning the unknown-intent teaching fallback.

## [0.126.0] - 2026-05-26

### Added
- Added deterministic natural-language skill compilation into reusable Links Notation packages with trigger-rule replay and `cache_hit` evidence.

## [0.125.0] - 2026-05-26

### Added
- Graduated the issue #258 trace-surface coverage for non-blocking graph-adjacent chat, Telegram trace links, code-answer execution status, and default-off diagnostics prose.

## [0.124.0] - 2026-05-26

### Added
- Graduated the OpenAI compatibility checks for configured bearer-token authentication and tool/function-call refusal unless agent mode is enabled.

## [0.123.0] - 2026-05-26

### Added
- Graduated agent-mode isolation and chat bounded-autonomy checks for explicit opt-in, sandbox disclosure, visible action logs, surfaced failures, destructive-action confirmation, time budgets, secret hygiene, and privilege revocation.

## [0.122.0] - 2026-05-26

### Added
- Graduated the links-network invariants for dynamic Type/SubType chains, source-backed facts, answer trace links, and ordered reasoning steps.

## [0.121.0] - 2026-05-26

### Added
- Graduated transparent-state chat queries for network snapshots, concept links, diagnostics, why explanations, retraction policy, Links Notation export, and user fact filtering.

### Fixed
- Kept personal fact-list prompts on the append-only memory query path instead of routing them as generic web-search requests.

## [0.120.0] - 2026-05-25

### Added
- Added a delegated relative-meta-logic / SMT-style decision procedure for propositional tautologies and linear arithmetic constraints.

## [0.119.0] - 2026-05-25

### Changed
- Graduated the issue #252 code-generation specification tests for top-10 hello-world seeds, execution Links Notation, isolation disclosure, sorting algorithms with tests, semantic code translation, and execution-failure traces.

## [0.118.0] - 2026-05-25

### Fixed
- Graduated the E6 translation-via-Links checks by preserving canonical meaning links and translated surface events in translation traces.

## [0.117.0] - 2026-05-25

### Added
- Graduated the public-knowledge source-cache provenance specification so source URL, fetched_at, content hash, refresh, cache-hit, conflict, flush, and offline-policy behavior are active tests.

### Fixed
- Offline external lookup attempts now emit an auditable `policy:offline` evidence link instead of only recording a skipped search.

## [0.116.0] - 2026-05-25

### Added
- Added temperature-based formalization selection with deterministic softmax
  guessing, ambiguity policy events, and clarifying-question handling.

## [0.115.0] - 2026-05-25

### Added
- Added a Wikidata P/Q-id prompt formalization engine with scored anchors,
  Wiktionary/Wikipedia/raw fallbacks, and solver evidence links for unresolved
  terms.

## [0.114.0] - 2026-05-25

### Changed
- Graduate the universal reasoning-loop acceptance tests and record candidate, validation, simplification, and trace events for finalized handler answers.

## [0.113.0] - 2026-05-25

### Added

- Introduced a `LinkStore` abstraction for `.lino` memory, event-log replay, and optional native doublets-rs mirroring.
- Graduated the issue 246 links-network specification tests for doublet reducibility, stable IDs, schema versioning, append-only history, concept uniqueness, and malformed Links Notation rejection.

## [0.112.0] - 2026-05-25

Issue #242: recover malformed English meaning questions such as
`what i digress mean?`, route them through the existing concept/Wikipedia/
Wikidata/Wiktionary lookup chain, and list dictionary page sources in the
source registry and connectivity diagnostics. Extend meaning prompt coverage
across the supported `en`, `ru`, `hi`, and `zh` language patterns.

## [0.111.0] - 2026-05-25

### Added
- Added confirmed, backup-aware memory purge/reset operations across the Rust library, CLI, and browser demo.
- Added multilingual browser controls and reset phrases for permanent deletion and full memory reset.
- Added issue #196 case-study evidence and regression coverage for destructive memory maintenance.

## [0.110.0] - 2026-05-24

### Fixed
- Russian proof requests such as `привет. докажи что простых бесконечно` now
  resolve to the formal Euclid infinitude-of-primes proof instead of the
  generic proof-plan fallback.
- English, Russian, Hindi, and Chinese prime-infinitude prompts now share a
  coverage-checked proof test matrix so localized phrasing cannot regress to a
  generic plan or capability response.

## [0.109.0] - 2026-05-24

### Fixed

- Russian requests for a Playwright script, including the common `Playright` typo, now return a Playwright starter example or ask for the target URL/actions when guess probability is low instead of falling through to `unknown` (issue #135).

## [0.108.0] - 2026-05-24

### Fixed
- Recognize general "what facts do you know?" prompts, including the reported Russian phrasing, and answer with local, internet, memory, and self-fact sources instead of the unknown fallback.
- Recognize LLM/OpenAI/API architecture follow-ups and explain the deterministic Links Notation runtime instead of falling through to unknown.
- Extend the same self-awareness coverage to reported Russian self-introduction, world-model, working-principle, project-purpose, and conversation-topic prompts from issues #137, #139, #141, #142, #147, #148, #155, and #237, including assistant-name configuration status in self-facts.

### Added
- Issue #223: pandas `DataFrame.join` method questions now return a scoped official-docs summary instead of the unknown fallback, with diagnostics linking the pandas docs source.

## [0.107.0] - 2026-05-23

### Fixed
- Answer Russian creator prompts like `кто тебя создал?` from the built-in Formal AI origin fact instead of falling through to the unknown fallback.

## [0.106.0] - 2026-05-23

### Added

- Answer assistant-name prompts in supported languages and add a configurable assistant name setting for the web demo.

## [0.105.0] - 2026-05-22

### Fixed
- Translate the Russian phrase `Найти синонимы или примеры согласования` as `Find synonyms or examples of agreement` instead of returning an English placeholder.
- Report unknown translation gaps explicitly with `translation_gap` evidence instead of rendering bracketed language placeholders such as `[en] ...` or `[ru] ...`.

### Changed
- Enforce PR language-facing test coverage for every supported language: English, Russian, Hindi, and Chinese.

### Fixed
- Issue #232: Answer Russian definition-style Wikipedia disambiguation pages such as `Существо` with their listed meanings instead of falling through to the Wikidata `Animalia` alias.
- Extend the Issue #232 regression to English, Russian, Hindi, and Chinese, with a CI coverage guard that fails if the definition-style disambiguation matrix loses a supported language.
