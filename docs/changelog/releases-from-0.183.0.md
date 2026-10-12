# Changelog archive: 0.183.0 to 0.241.0

Releases in this range, newest first. Newer releases are in [CHANGELOG.md](../../CHANGELOG.md).

## [0.241.0] - 2026-06-28

### Fixed
- Answer supported-language behavior-rule count questions such as `Сколько всего правил?` with the built-in, dialog-local, and total rule counts instead of falling through to the unknown fallback.

## [0.240.0] - 2026-06-28

### Fixed
- Behavior-rule follow-up prompts can now ask for the rule count or a brief localized recap after listing the rules, including the reported Russian prompts.

## [0.239.0] - 2026-06-28

- Concept lookup now honors data-driven response-language markers, so prompts
  such as "Tell me about Telegram Ads in Russian" render the known concept in
  the requested language instead of treating the language phrase as context.

## [0.238.0] - 2026-06-28

### Fixed
- **Issue #485 - multilingual elided `how <action> ...` prompts now route to procedural how-to.** The Rust solver and browser worker recognize seeded weak leads such as Russian `как ...` when the following action is approved by the procedural action lexicon, preserving greeting-prefixed compound answers instead of falling through to unknown.

### Fixed

- Replaced live crates.io/docs.rs status badges in GitHub release notes with
  static release-version artifact badges that still link to the exact published
  crate and documentation pages.
- Restored the README project badge block and added regression coverage plus
  issue #492 case-study evidence.

## [0.237.0] - 2026-06-28

### Fixed
- **Issue #481 - telegraphic `how order ...` prompts now route to procedural how-to.** The Rust solver and browser worker accept the weak `how ...` lead only when the following action is approved by the seed lexicon, so `how order 3d print in nan chang vietnam?` produces the normal source-backed discovery plan without broadly claiming arbitrary `how <word>` prompts.

## [0.236.0] - 2026-06-28

### Fixed

- Added a multilingual seeded concept for neural-network inference so `что такое нейросетевой инференс?` resolves through concept lookup instead of the unknown fallback.

## [0.235.0] - 2026-06-28

### Fixed
- Resolve unseeded English and Russian authorship prompts, such as War and Peace questions, through Wikidata `P50` instead of returning `unknown`.

## [0.234.0] - 2026-06-28

### Fixed

- Added a seed-backed fact answer for Russian Spider-Man film release-order prompts so they resolve to `fact_lookup` instead of `unknown`.

### Fixed

- Added a seed-backed Air India infant stroller baggage allowance answer so the reported Russian prompt resolves to `fact_lookup` instead of `unknown`.
- Corrected the Spider-Man film fact's Wikidata anchor and checked in the required Wikidata cache records for the new and corrected facts.

### Fixed
- Issue #477: Russian prompts like `Что такое кубаторит?` now resolve through the seeded concept dictionary entry for `кубаторить / кубатурить` instead of falling through to `unknown`, with Academic.ru source evidence.
- Corrected the Spider-Man film release-order fact's Wikidata anchor from stale `Q79054` to `Q2307877` and checked in the matching cache snapshot.

## [0.233.0] - 2026-06-27

### Fixed
- Resolve Rust creator follow-up questions by rewriting seeded pronoun references back to the Rust fact lookup path.

## [0.232.0] - 2026-06-26

### Fixed

- Solve train meeting relative-speed prompts with verification-tagged reasoning in Rust and the browser worker instead of falling through to `unknown` or `calculation_error` (issue #460).

## [0.231.0] - 2026-06-26

### Fixed
- **Issue #464 - clock-time duration prompts now route to the calculator.** The web worker and Rust solver now handle `17:30 - 14:00` and elapsed-time wording such as `If a train leaves at 14:00 and arrives at 17:30, how long is the trip?`, returning `3 hours, 30 minutes` instead of `unknown`.

## [0.230.0] - 2026-06-26

### Fixed
- Route composite crypto portfolio tracker prompts to a Python blueprint with mocked prices, alert logic, and a Markdown dashboard instead of generic search or missing-template fallbacks.

## [0.229.0] - 2026-06-26

Fixed
- Route class-based program requests such as "Write a Python class" through `write_program`, and answer the smart travel planner prompt with a Python `TravelPlanner` blueprint instead of falling through to search or an unsupported template.

## [0.228.0] - 2026-06-26

### Fixed
- Fixed Russian follow-up code requests like `На php не получится написать?` so they inherit the advertised Hello World task and return the cached PHP example instead of `unknown`.

## [0.227.0] - 2026-06-26

### Fixed

- Issue #457: route Rust self-source metrics and response-comparison prompts to
  a curated `write_program` blueprint instead of the missing-template fallback.

## [0.226.0] - 2026-06-26

### Fixed

- Kept terse research result follow-up prompts tied to the prior search attempt instead of defining the word "result" after a failed browser search.

## [0.225.0] - 2026-06-25

### Added
- Added repository-file formalization and summarization helpers that record file
  metadata, meta-language parser evidence, and recursive Markdown embedded
  grammar summaries.
- Generalized summarization from files to any repository resource, including
  folders: `RepositoryEntry`, `formalize_repository_resource`, and
  `summarize_repository_resource` summarize a directory tree by the recursive
  decompose → summarize → compose meta-algorithm loop, with recursion depth
  bounded by the summarization mode ladder and link-native `repository_directory`
  evidence.

## [0.224.0] - 2026-06-25

### Fixed
- Route verbless "records/financials/statistics about a subject" prompts such as `Financial records for boeing after crisis with icas system` to web search instead of the unknown-prompt report, with multilingual coverage (en/ru/hi/zh).

## [0.223.0] - 2026-06-25

### Fixed

- Issue #441: Russian definition prompts that start in Cyrillic and ask about
  Latin technical terms, such as `Что такое vulkan layer`, now keep
  `language:ru` instead of being misclassified as English and falling through to
  the unknown-intent answer. The browser worker mirror now follows the same
  detection rule.

## [0.222.0] - 2026-06-24

### Fixed
- Browser worker and Rust seed coverage now recognize length-vs-mass unit questions such as `Сколько метров в килограмме?` as `unit_incompatibility` instead of falling back to `unknown`.

## [0.221.0] - 2026-06-24

### Fixed
- **Issue #446 — large integer exponents in the web calculator were truncated.** Arithmetic fallback evaluation now keeps integer exponentiation exact, so prompts such as `10^100` render the full integer instead of `1e+1`.

## [0.220.0] - 2026-06-24

### Fixed
- **Issue #445 — compound courtesy/question prompts were treated as one unknown.** The solver now decomposes unresolved independent prompt parts, responds to greetings first, and then answers the following question segment while preserving existing specialized decomposition for algebra and list-style synthesis.

## [0.219.0] - 2026-06-24

### Fixed
- Deploy GitHub Pages from the resolved release commit so the website and API docs advertise the same version as the latest release.
- Retry `rust-script` installation in CI so transient crates.io HTTP failures do not fail unrelated workflow jobs.

## [0.218.0] - 2026-06-24

### Added
- Added the issue 559 planning case study for generalizing the meta algorithm architecture.
- Expanded the case study with a recursive link-native solver plan, Voyager design mapping, and upstream dependency audit.
- Deepened the case study into a spine plus companion documents (alignment, critical review, options comparison, recursive core, evidence pipeline), with a critical check (CR1–CR12), strategic re-check resolving conflicts C1–C7, a canonical vocabulary mapping onto existing VISION/REQUIREMENTS terms, option comparisons with `SolverConfig`-knob comparison harnesses, and proposed requirement rows R330–R335.

### Added
- Issue #559 (Phase 1A): an explicit, link-serializable problem frame. Every prompt now produces a `ProblemFrame` (`src/meta_frame.rs`) that wraps the formalized intent and enumerates every detected `Need` (questions, requirements, tasks) found across sentences and coordinating clauses. The frame is emitted as a trace-only `problem_frame` loop event and serialized to Links Notation via `format_lino_record`, making the meaning record first-class without changing routing or answers. Tracked by REQUIREMENTS.md R330.

### Added
- Issue #559 (Phase 1B): the recursive, bounded downward pass of the general meta algorithm. Every problem frame is now decomposed into a `WorkUnit` tree (`src/meta_frame.rs`) — each unit is either a direct-method leaf, an irreducible single need, or split into children, always stopping at `SolverConfig::max_decomposition_depth` so the recursion is terminating. The tree is serialized to Links Notation and emitted as trace-only `work_unit` / `work_unit:enter` / `work_unit:exit` loop events, so the recursive core is observable without changing routing or answers. The reasoning-loop guarantee is widened beyond arithmetic to prove each handler family still emits candidate/validation events when reached as a recursion leaf. Tracked by REQUIREMENTS.md R332.

### Added
- Issue #559 (Phase 2): the need-satisfaction ledger. Every problem frame now produces a `NeedLedger` (`src/meta_frame.rs`) with exactly one row per detected need, each carrying an explicit status derived from the recursive work-unit tree — a need that maps to a known method is satisfiable, while a need with no recognized method is recorded as `blocked` rather than silently dropped. This makes "address every detected need" structural rather than prose (R8). The ledger is serialized to Links Notation and emitted as trace-only `need_ledger` / `need:status` loop events, changing neither routing nor answers. Tracked by REQUIREMENTS.md R333.

### Added
- Issue #559 (Phase 3): the method registry as first-class link data. The catalogue of handlers each atomic work-unit leaf can route to — the ordered `SPECIALIZED_HANDLERS` table plus the five contextual overrides — is now derived from the live dispatch code (`src/method_registry.rs`, `MethodRegistry::from_dispatch`) and serialized to Links Notation, so the meta algorithm can read and reason about its own methods rather than having them locked away in Rust. The registry is recorded as a trace-only `method_registry` loop event, changing neither routing nor answers, and a grounding test pins every derived method name against `src/solver_dispatch.rs` so the data can never drift from the handlers that actually run. Tracked by REQUIREMENTS.md R331.

### Added
- Issue #559 (R335): the recursive meta core now describes *itself* as grounded link data. `data/meta/recursive-core-recipe.lino` enumerates the eight ordered steps that turn any message into a solved, link-native knowledge base — formalize the impulse, build the problem frame, decompose recursively into a bounded work-unit tree, account for every need in a ledger, catalogue the resolving methods, resolve each atomic leaf through the single ordered dispatch, record evidence, and project the answer — and pins each step to the live function that implements it. A grounding test (`tests/unit/specification/recursive_core_recipe.rs`) asserts the source still defines every named function, so the core's self-description can never drift from the code that runs. This is the concrete sense in which the meta algorithm can reason about itself: its own algorithm exists as data the engine can read. Tracked by REQUIREMENTS.md R335.

### Added
- Issue #559 (R334): the evidence pipeline. The meta core now joins its separate link artifacts — the problem frame, the recursive work-unit tree, the need-satisfaction ledger, and the method registry — into one end-to-end `SolutionEvidence` record (`src/solution_evidence.rs`). For every detected need it traces the full chain `frame need → work-unit leaf → ledger status → catalogued method`, with `accounted_for` (every need has a connected, non-pending status) and `fully_resolved` (every need is satisfied) flags, so "ensure every detected need is addressed in the response" is a single auditable fact rather than four projections a reader must reconcile by hand. The ledger rows gained additive `unit_id`/`route` links to support the join. The evidence is serialized to Links Notation and emitted as a trace-only `solution_evidence` loop event, changing neither routing nor answers. Tracked by REQUIREMENTS.md R334.

### Added
- Issue #559: a runnable example (`cargo run --example issue_559_meta_core`) that emits the meta core's Links Notation artifacts — problem frame (R330), recursive work-unit tree (R332), need-satisfaction ledger (R333), method registry (R331), and solution evidence (R334) — for a single routed need, a conjunction, and an unroutable need, offline with no network or neural inference.
- Issue #559: a deep, data-grounded case-study analysis (`docs/case-studies/issue-559/implementation-results.md`) recording what shipped against the plan, walking the real emitted artifacts, and surfacing a genuine route↔method vocabulary gap the unified evidence projection exposes (the routing label `write_program` does not match any registered handler name, so `resolved_to_method` honestly trails `trail_count`). The verbatim run is captured at `docs/case-studies/issue-559/raw-data/meta-core-artifacts.txt`.

### Added
- Added route→method aliases as first-class link data (`data/meta/route-method-aliases.lino`, `src/route_method_alias.rs`) so the meta core can resolve a meta-language intent slug that is coarser or finer than the handler serving it — for example `write_program` → `write_script` — to a catalogued method (issue #559, R336).

### Changed
- The solution evidence join now resolves each need's route through `MethodRegistry::method_for_route` (direct match, then alias), recording `method_via_alias` provenance, so the program-writing need in a request like "translate apple to Russian and write a hello world program in Python" reports a resolving method instead of appearing unaddressed.

### Added
- Added white-box recursive reasoning to the meta core (`src/meta_reasoning.rs`, R337): every work unit now carries a human-readable thought in both directions — the downward thought (what span was observed, why it was decomposed or judged atomic, and which method an atomic leaf resolves to) and the upward thought (how the unit's answer is composed from its solved children). The reasoning is a parallel tree to the work-unit tree, serialized to Links Notation and emitted as the trace-only `work_unit_reasoning` / `work_unit_reasoning:steps` events, so the box is inspectable by users and developers — the reasoning, not just the predicate, is visible (issue #559).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists nine ordered steps, adding the white-box reasoning step and pinning `WorkUnitReasoning::for_unit` and `record_work_unit_reasoning` to their source.

### Added
- Added the upward construction pass to the meta core (`src/meta_construction.rs`, R338): the construction half of the recursion. A post-order (bottom-up) walk of the work-unit tree composes each answer from leaf to root — every leaf is a base case constructed directly from the method that resolves its route (via the same `method_for_route` bridge the evidence join uses), and every parent is a recursive case composing its already-constructed children in source order. Serialized to Links Notation and emitted as the trace-only `upward_construction` / `upward_construction:steps` events, so both directions of the recursion — decompose and compose — are inspectable link data (issue #559).
- Added the `RecursionMode` knob (`Down` | `Up` | `Both`), surfaced as `SolverConfig::recursion_mode` and the `FORMAL_AI_RECURSION_MODE` env override. The default `Down` reproduces the pre-knob trace exactly, so the upward pass is always an explicit opt-in and the default solver behavior is unchanged (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists ten ordered steps, adding the upward construction step and pinning `UpwardConstruction::for_unit` and `record_upward_construction` to their source.

### Added
- Added the method-selection trace to the meta core (`src/selection.rs`, R339): for every atomic work-unit leaf, `MethodSelection::for_unit` names the method the single data-driven registry authority resolves (`MethodRegistry::method_for_route`, alias-aware — e.g. `write_program` resolves to `write_script` through its route→method alias), or marks the leaf `unresolved` when no method serves it, and counts resolved vs. unresolved leaves. Serialized to Links Notation and emitted as the trace-only `selection` event, this makes the dispatch the registry performs auditable per request.
- Added the `SelectionMode` knob (`Off` | `Record`), surfaced as `SolverConfig::selection_mode` and the `FORMAL_AI_SELECTION_MODE` env override. The default `Off` records nothing and leaves both routing and the answer unchanged, so the trace is an explicit opt-in (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) lists the method-selection step, pinning `MethodSelection::for_unit` and `record_selection` to their source.

### Added
- Added the gated meta self-improvement loop (`src/meta_self_improvement.rs`, R340): the meta algorithm now reads *itself* — the recursive-core recipe (the algorithm encoded as Links Notation) against the live `record_meta_core` pipeline (the algorithm as code) — and emits the *updated* algorithm as link-encoded output. It detects drift between the recipe's `meta_function` citations and the `record_*` stages the pipeline actually runs, proposing the additions and stale-citation removals that reconcile them (`MetaSelfImprovement::from_repo().propose()` → `MetaRecipeProposal::to_links_notation`). It is gated and proposal-only: the default `SelfImprovementMode::Off` proposes nothing and it never writes the recipe back, so adoption stays a human review step and behaviour is unchanged (issue #559).

### Changed
- Adopted the loop's first real finding: the self-describing recipe (`data/meta/recursive-core-recipe.lino`) now cites `record_solution_evidence` (and lists the `solution_evidence` event), so it describes every stage the pipeline runs and the loop reports the live sources as self-consistent.

### Added
- Lifted the meta core's hardcoded natural-language recognition cues into reviewable link data (`data/meta/cue-lexicon.lino`, `src/cue_lexicon.rs`, R341). The arithmetic operators, web-search verbs, the fourteen text-manipulation operations, the calendar fallback verbs, and the other intent cues that used to live as inline Rust string literals in `src/intent_formalization.rs` are now `cue_set` records, each declaring how it is matched (`token` whitespace-bounded word / CJK substring, `substring` raw contains, or `prefix` starts-with). The Rust call sites (`append_prompt_relevants`, `looks_arithmetic`, `looks_like_text_manipulation`) read the cue strings from the data and keep only the structural glue (digit presence, AND/OR composition, which input each set is tested against). A grounding test pins every consulted set to the data with its expected mode and proves routing is unchanged, so adding a trigger word for an existing handler family is now a data edit rather than a Rust change. Behaviour is identical: the data reproduces exactly the lists it replaced (issue #559).

### Added
- Added a proposal-only skill-accumulation ledger as the twelfth step of the recursive meta core (`src/skill_ledger.rs`, R342). Each request's solution evidence is distilled into learning the next request can reuse — the deterministic analog of an agent that grows a skill library and a curriculum: every detected need that was satisfied by a catalogued method becomes a proposed, reusable `CandidateSkill` (a named capability the solver demonstrably has, captured with the span that demonstrated it), and every blocked need becomes a `CurriculumItem` recording the gap to close rather than a silent failure. Accumulation is proposal-only and gated: a candidate skill is born `proposed` and may only be promoted to `stable` once its `PromotionGate` is satisfied — tests *and* a benchmark delta — so the promotable count is always zero at trace time and no skill is ever auto-promoted without review (C3). The default `off` mode (env `FORMAL_AI_SKILL_MODE`) records nothing, so the ledger changes neither routing nor the answer (R13); `accumulate` emits it as the trace-only `skill_ledger` event. The recipe (`data/meta/recursive-core-recipe.lino`) describes the new stage, keeping the meta self-improvement loop self-consistent (issue #559).

### Added
- Made the recursive-core recipe *executable as data*, not just a checked description (`src/recipe_interpreter.rs`, R343). Each trace-recorded step in `data/meta/recursive-core-recipe.lino` now binds to the recorder primitive it drives via a `records` field, and `RecipeProgram` parses the recipe into an ordered program and runs it — invoking those primitives in the order the data declares, threading the intermediate artifacts (problem frame, work-unit tree, need ledger, method registry, solution evidence) exactly as the hand-written pipeline does, and mirroring its mode gates. The headline guarantee is parity: `RecipeProgram::reproduces_pipeline` proves the event log produced by executing the recipe is identical, event-for-event, to the one `meta_core::record_meta_core` produces for the same input across every recursion/selection/skill mode combination, and the recipe's recorder order equals the live pipeline's actual stage order. A misordered dependency or unknown binding surfaces as an error rather than silent divergence. This makes the algorithm-as-data and the algorithm-as-code provably the same algorithm — the foundation for eventually driving the pipeline from the recipe itself — while staying trace-only: it changes neither routing nor the answer (issue #559).

### Changed

- Issue #559 (R344): completed the **total migration to the data-driven method
  registry as the sole dispatch authority**. An interim corpus-wide parity
  certificate first proved the registry resolved the *entire route vocabulary the
  system can ever emit* — every registered method name (each a self-resolving
  route), every route→method alias (R336, including the `write_program` intent),
  and every classifier route slug — as a behavior-preserving replacement for the
  legacy hardcoded mapper, with **zero contradictions**. With that proof in hand,
  the legacy authority and the parity scaffolding were **removed outright**:
  `src/dispatch_parity.rs` and `intent_formalization::specialized_handler_name`
  are gone, leaving `MethodRegistry::method_for_route` (alias-aware) as the only
  route→method resolver and the only live dispatch path
  (`src/meta_method_dispatch.rs`). The closure invariant the certificate
  guaranteed now lives directly against the live registry — grounded in
  `MethodRegistry::from_dispatch`, `route_method_alias::aliases`, and
  `seed::intent_routing` — pinned by
  `tests/unit/specification/method_registry.rs::the_registry_is_the_sole_authority_that_closes_over_the_route_corpus`:
  no route resolves to an unregistered method, and every method-name and alias
  route resolves.

### Changed

- Issue #559: replace the solver-local specialized-handler loop with the live
  registry-backed meta method dispatcher. `MethodRegistry` now supplies the
  prelude, specialized, and contextual method ordering used by
  `meta_method_dispatch::try_dispatch`, which is now the sole dispatch authority
  (the legacy route mapper and its parity scaffolding were removed outright once
  the registry was proven a behavior-preserving replacement — see R344). Selected
  handler answers now re-project their
  evidence and Links Notation after the `method` event is recorded, so responses
  expose the selected registry method directly.

## [0.217.0] - 2026-06-21

### Added

- **Dedicated install landing pages for every interface (issue #554).** The site
  chooser now links to three new pages rendered by the shared
  `src/web/site-chrome.js`: `/vscode/` for the VS Code extension, `/cli/` for the
  command-line tool, and `/telegram/` for the Telegram bot. Each page carries
  copy-paste install commands (with one-click copy buttons), ordered manual
  steps, and direct links to the raw installer and the latest release.
- **A universal one-line installer** — [`scripts/install.sh`](../../scripts/install.sh)
  (POSIX `sh`) and [`scripts/install.ps1`](../../scripts/install.ps1) (PowerShell) —
  with a target for every interface: `desktop`, `vscode`, `cli`, `telegram`
  (installs the CLI that powers the bot), and `all`, from a single command
  (`curl -fsSL …/install.sh | sh -s -- <target>`). The VS Code page documents the
  manual-only ".vsix" flow ("VS Code Extension only" mode) while the extension is
  still off the Marketplace.
- **One-click VS Code extension install from the desktop app.** Settings now
  offers *Install VS Code extension*: the Electron shell
  (`desktop/lib/vscode-install.cjs`) detects an installed `code`/`code-insiders`/
  `codium`/`cursor`/`windsurf` CLI, downloads the published `.vsix` from the
  latest GitHub release, and runs `code --install-extension … --force` — all
  exposed through the `formalAiDesktop:installVsCodeExtension` IPC bridge.
- The desktop release CI now builds and uploads the `formal-ai-vscode-*.vsix`
  asset so the installers and the one-click flow have a release artifact to fetch.

### Changed

- The shared chooser (`src/web/site-chrome.js`) gained a sectioned-content
  renderer (`section-<id>`, `command-<testid>`, `copy-<testid>`) used by the new
  install pages; the existing landing/docs/download pages are unchanged.
- The root landing chooser now surfaces six destinations (web app, docs,
  download, VS Code, CLI, Telegram) instead of three.

## [0.216.0] - 2026-06-21

### Added
- Add shared-dialog conversion for ChatGPT shared-page HTML and Markdown transcripts, plus CLI replay export to `demo_memory`.

### Fixed
- Preserve multi-line memory event content through Links Notation export/import and answer captured shell-loop prompts with readable one-line commands.

## [0.215.0] - 2026-06-21

### Fixed
- Thinking preview now fades the **whole** collapsed reasoning stack with a single
  container-level scroll gradient instead of masking each step line separately, so
  two stacked steps read as one continuously-scrolling surface (issue
  link-assistant/formal-ai#550, problem 1).
- Naturalized thinking detail is no longer clipped at 120 characters; the cap was
  raised to 600 in both `truncate_thinking_detail` (Rust core) and the
  `thinkingDetailText` browser helper, so realistic single-step detail renders in
  full while staying bounded (issue link-assistant/formal-ai#550, problem 2).
- A pending assistant message that only shows thinking steps now keeps the full
  message-body width instead of collapsing to a fixed 116px box, removing the
  sudden width jump when the answer body starts streaming (issue
  link-assistant/formal-ai#550, problem 3).
- The desktop **services** and **update** panels (and the services error text) now
  have complete dark-theme rules, so every surface, token input, action button and
  status line is readable in dark mode instead of falling back to light styling
  (issue link-assistant/formal-ai#550, problem 4).
- Every top-bar control — source/download/report/memory buttons, the sidebar and
  mobile-menu toggles, and the mode/diagnostics toggles — now shares one hover
  treatment and one keyboard focus ring, so hover/focus feedback is consistent
  across the whole header rather than partial (issue link-assistant/formal-ai#550,
  problem 5).

### Changed
- The web stylesheet now defines a single semantic design-token palette (`--fa-*`)
  per theme instead of hand-duplicating every colour across the light base and both
  dark layers. New surfaces and controls inherit correct theming by consuming a
  token, which removes the duplication that was the shared root cause of the dark
  `services` panel (problem 4) and the partial top-bar hover (problem 5). The change
  is value-preserving — every token equals the previous colour — so no rendered
  output changed (issue link-assistant/formal-ai#550).
- The eleven top-bar controls now render through a single reusable `ToolbarButton`
  React component, so their markup, classes, accessibility attributes and overflow
  priority stay uniform by construction rather than being copied per button (issue
  link-assistant/formal-ai#550). Together with the `--fa-*` tokens this delivers the
  reusable-component and design-system substance of the requested Chakra UI
  direction; Chakra's Emotion CSS-in-JS runtime remains intentionally unadopted
  because its runtime `<style>` injection is incompatible with the app's strict
  `style-src 'self'` Content-Security-Policy (issue link-assistant/formal-ai#479).

## [0.214.0] - 2026-06-21

### Added
- Added packaged desktop auto-update checks, in-app update notifications, and user-triggered update installation.

### Fixed
- Fixed the desktop shell version badge so packaged apps show the Electron app version instead of `vdev`.

## [0.213.0] - 2026-06-20

### Fixed
- Run granted desktop and VS Code `shell` tool calls on the host machine by default, while keeping Docker isolation available for explicit sandboxed shell requests and code execution.

## [0.212.0] - 2026-06-20

### Fixed
- Dark theme now reaches every primary widget. The topbar mode-status badge, the collapsed-sidebar toggle, the mobile drawer section headings, and the per-step "tool"/"agent" mode badges in the reasoning trace had hard-coded light colors with no `[data-theme="dark"]` or `@media (prefers-color-scheme: dark)` counterparts, so their light palette bled through when the rest of the UI went dark. Each missing override is now in place using the codebase's existing dark palette, so theme switching looks consistent end-to-end (issue #541).
- Desktop conversations now survive app upgrades. The userData directory is pinned to a stable, productName-independent name (`formal-ai`) so a rebrand or package rename can no longer orphan a profile, and on first launch any legacy profile (e.g. `formal-ai Desktop`) is **non-destructively** migrated forward — the renderer's IndexedDB conversation store and `localStorage` preferences are copied into the pinned directory without ever deleting the originals, then version-stamped for future schema migrations (issue #541).
- Desktop app no longer reports "Docker unavailable" when Docker Desktop is installed and running. The `docker` binary is now resolved across well-known install locations (`/usr/local/bin`, `/opt/homebrew/bin`, `/Applications/Docker.app/...`, Windows `Program Files`, NixOS), fixing the GUI-launch PATH gap, and availability is re-probed on a short TTL so a daemon started after the app opened is detected without a restart (issue #541).
- Collapsed reasoning preview now shows the current step in full (at least one whole paragraph) instead of clipping it to a single ellipsised line, so the thinking trace is actually readable while collapsed (issue #541).
- Demo mode no longer touches user conversations. Switching demo on from inside a real conversation now spawns a dedicated, sidebar-invisible demo conversation that holds the scripted turns; switching demo off restores the user's conversation exactly as they left it; and clicking any conversation in the sidebar auto-disables demo mode so the user is never left looking at a demo thread they did not choose. The dedicated demo conversation is reused within a session, so the "last example" survives an off/on toggle without leaking into the user's threads (issue #541).
- The desktop permission panel now offers a single primary action — **Grant all and switch to Agent mode** — directly above the per-tool rows, so users who asked the assistant to run a terminal command (e.g. "run \`ls ~\` in terminal") can replay the request in one click instead of being asked to re-type it. When a command is waiting for permission, the button copy upgrades to "Grant all, switch to Agent mode, and run pending task" and clicking it grants all six desktop tools, flips the mode toggle to Agent, and replays the queued shell command through `executeTerminalCommand`. Without a pending task, the same button still grants and flips mode in one step (issue #541).

### Changed
- Reasoning steps now read as plain human language at every detail level. The "Thinking detail" setting defaults to **Standard** (the 50% midpoint), which shows only the high-level reasoning phases and folds the mechanical sub-steps out of view so newcomers are not overwhelmed. Even at maximum **Detailed** granularity the trace no longer leaks the internal symbolic Links tuple (`(@USER OP:… ?term)`) or jargon like "symbolic form" — the formalization step is projected to a plain task noun (e.g. "greeting", "calculation", "search") in all four UI languages. The raw symbolic formalization remains available in Diagnostics mode for maintainers (issue #541).

### Added
- Minimum message animation time setting (Settings → "Minimum thinking animation"). Reasoning steps now reveal one-by-one and the answer body fades in only after the trace has played out, so the deterministic engine's instant answers still feel considered. Defaults to 2 seconds; set it to 0 for immediate display. Honours `prefers-reduced-motion` (issue #541).
- `FORMAL_AI_DESKTOP_DEBUG` environment variable enables verbose desktop diagnostics (Docker binary resolution and probe results) to help diagnose environment-specific problems.
- `FORMAL_AI_DOCKER_BIN` environment variable overrides the resolved `docker` binary path.

## [0.211.0] - 2026-06-20

### Added
- Case study for issue #511 under `docs/case-studies/issue-511/`: deep analysis of
  the `unknown` answer for terminal-command prompts (e.g. ``Выполни `ls ~` в
  терминале``), a full requirement inventory (R1–R20), per-requirement solution
  plans that reuse the existing permission-gated tool router, Docker sandbox,
  OpenAI-compatible local server, and `src/agentic_coding/` loop, and a sequenced
  implementation epic (E1–E8) for agent/full-auto mode driven by
  `link-assistant/agent` through `link-assistant/agent-commander`.
- The epic milestones are filed as live GitHub issues (via `gh`) and linked as
  sub-issues of #511: E1–E8 (#513–#520). The case-study docs reference the created
  issue numbers so the plan stays in sync with the tracker.
- Re-verified the integration against the latest upstream versions
  (`@link-assistant/agent` v0.24.0, `agent-commander` js_0.8.0 / rust_0.2.6): the
  Agent-CLI permission gap (`link-assistant/agent#271`) is **resolved** by
  `link-assistant/agent#272` (v0.24.0), which adds a native, enforceable
  `--permission-mode auto|plan|readonly|ask`, an OpenCode-compatible `--permission`
  JSON policy, and a per-command JSON approval protocol (JS + Rust). Both residual
  `agent-commander` gaps filed last round are now **closed**:
  `link-assistant/agent-commander#39` (map `--read-only`/`--plan-only` for the
  `agent` tool onto its native `--permission-mode`, shipped js_0.7.0 / rust_0.2.5)
  and `link-assistant/agent-commander#40` (uniform per-command approve-each relay,
  `--approve-each` / `--permission-mode ask`, shipped js_0.8.0 / rust_0.2.6). No open
  `agent-commander` issues remain, so the #511 plan is fully implementable today.
- Documented the **default-backend decision**: the desktop agent path defaults to
  `@link-assistant/agent` through `agent-commander`, because per the agent-commander
  approve-each parity (`docs/common-concepts.md`) only `agent` (scope `session`) and
  `claude` (scope `tool-input`) can relay per-command JSON approvals — and `agent` is
  the only org-owned backend with a clean session-wide `once|always|reject` grant;
  `codex`/`gemini`/`qwen`/`opencode` lack a relayable headless approval handshake
  (documented upstream-CLI limitation, not a bug). The case-study docs and raw-data
  snapshots (incl. a new `external-agent-commander-common-concepts.md`) were refreshed
  to these latest versions.
- Reconciled the issue #511 plan with the merged E1 work from PR #525 and the latest
  `main` service-control changes: E1 is now recorded as implemented, the remaining
  sequence starts at E2, and E3/E5 explicitly reuse the prepared GHCR image,
  `compose.yaml`, and desktop one-click service-control stack instead of duplicating
  server/container lifecycle code.

### Added
- Terminal-command intent (`tryTerminalCommand`) in both the Rust solver
  (`src/solver_terminal.rs`) and the JS worker (`src/web/formal_ai_worker.js`).
  Prompts that ask to run a shell command — fenced/backtick commands, "run … in
  terminal" / «выполни … в терминале» phrasings, or an explicit leading shell
  token like `ls`/`git status` — now resolve to an `agent_suggestion` response
  that names the detected command, explains Agent mode, and offers to switch and
  grant the `shell` capability, instead of falling through to `unknown`
  (visible fix for #511, issue #513). Localized for en/ru/hi/zh.
- Three-way `Chat` / `Agent` / `Full Auto` mode radio group in the web toolbar
  and drawer, replacing the binary agent toggle. A new `mode` preference is
  persisted and the legacy `agentMode` boolean is derived from it
  (`mode !== "chat"`) for back-compat. The topbar status label now reflects the
  active mode.

### Changed
- Toolbar/drawer mode controls expose `data-testid="mode-radio"` /
  `mode-option-<mode>` and a `mode-status` label; existing e2e specs were
  updated from the old `agent-toggle` selector accordingly.
- The terminal-command response prose is no longer hardcoded in either engine.
  The four-language bodies now live in `data/seed/multilingual-responses.lino`
  under the `agent_suggestion` (Chat mode) and `agent_suggestion_active` (Agent
  mode on) intents, with a `{command}` placeholder. Both `src/solver_terminal.rs`
  (via `seed::response_for`) and the JS worker (via `answerFor`) look the
  template up and fill in the detected command, so the natural-language wording
  is sourced from seed data rather than living in code (addresses #513 review
  feedback).
- The terminal-command *trigger* vocabulary is no longer hardcoded either. The
  terminal/shell phrases, run verbs, Chinese run verbs, and leading shell tokens
  now live in the new `data/seed/terminal-commands.lino`. The Rust solver parses
  it via `src/seed/terminal_commands.rs`
  (`seed::terminal_command_vocabulary`), and the JS worker embeds a
  byte-identical inline mirror kept in lockstep by
  `experiments/issue-513-sync-worker-terminal.mjs` (the same convention as the
  operation vocabulary, #386). A `--check` mode guards the parity in CI.
- Every new terminal-command vocabulary token (shell tokens, `command-line`,
  the `agent_suggestion*` intents and their `response_*` templates) is grounded
  as a first-class meaning so the total reference-closure audit
  (`scripts/audit-total-closure.py`) stays at zero. The
  `data/seed/closure-generated-*.lino` shards were regenerated via
  `scripts/close-total.py`; the generation is idempotent.
- E2E Playwright configs now set reasonable per-test, whole-suite
  (`globalTimeout`), assertion (`expect.timeout`), and navigation/action caps in
  both `tests/e2e/playwright.local.config.js` and `playwright.pages.config.js`
  so a hung worker, server, or deployment aborts promptly instead of wedging CI
  (addresses #513 review feedback on iterating faster).

### Added
- Added per-tool desktop permission grants, first-run Agent/Full Auto onboarding,
  and Agent-mode per-command shell approval prompts so desktop tools can be
  granted or declined independently while preserving default-deny behavior.

### Changed
- Routed the desktop permission, command-approval, and one-click services UI
  strings in `src/web/app.js` through the i18n catalog (`t(key, params)`) so
  they translate with the active UI language (en/ru/zh/hi) instead of rendering
  hardcoded English introduced in PR #528.

### Added
- Added a strict CI guard, `tests/e2e/scripts/check-web-hardcoded-ui-strings.mjs`
  (`check:web-hardcoded-ui`), that fails the build when a user-facing prose
  string literal is passed as a child of an `h(...)` render call in
  `src/web/app.js`, plus catalog keys and `check:i18n` coverage for the new
  permission/command/services strings. Documented the rule in CONTRIBUTING.md and
  `docs/design/no-hardcoded-natural-language.md` so the regression cannot recur.

### Changed
- Split the web UI translation catalog so each source file stays under the
  Links Notation line limit enforced by `scripts/check-file-size.rs`. The
  desktop tool-permission and Services strings now live in
  `src/web/i18n-catalog-permissions.lino`, while the core UI strings remain in
  `src/web/i18n-catalog.lino`. The loader (`src/web/i18n.js`) fetches both files
  and merges their per-locale keys, and `check:i18n` plus the language-parity
  guards validate the merged catalog.

### Added
- Auto-start the desktop local OpenAI-compatible server when Agent or Full Auto mode is entered, reusing a healthy running server and exposing its `apiBase` for provider configuration.

### Added
- Added the desktop AgentProvider seam with an in-process default provider, an
  opt-in agent-commander provider for the `agent` backend, and tests guarding
  read-only execution plus direct host `agent`/`claude`/`codex` spawns.

### Added
- Added the installable Formal-AI Agent environment container flow: the prepared image now bundles `@link-assistant/agent` and `agent-commander`, the desktop Services panel can install and health-check `formal-ai-agent`, and Compose exposes the matching `agent` profile.

### Added
- Added an Agent CLI NDJSON adapter that maps assistant text, tool start/result,
  and error events onto the existing chat answer and diagnostics rendering path.

### Added
- Added cold-start desktop e2e coverage for the issue #511 `ls ~` journey,
  including first-run onboarding, three-way mode switching, per-command denial
  and approval, the hermetic in-process provider path in CI, and a
  `FORMAL_AI_E2E_AGENT_COMMANDER=1` gated real commander-provider variant.

### Fixed
- Routed read-only commander-provider requests for the default `agent` backend
  through `agent-commander --read-only`, using the shipped upstream mapping
  instead of the old `--approve-each` workaround.

### Documentation
- Finalized the issue #511 Agent CLI + agent-commander best-practices write-up
  and upstream closeout status for issue #520.

### Documentation
- Updated the issue #511 case study and PR #512 description to reflect that all
  eight implementation milestones (E1–E8, #513–#520) are merged into the parent
  branch: rewrote the "why a plan, not the whole feature" and "acceptance
  criteria" sections to state the feature ships in this PR, with every acceptance
  criterion met and pinned by tests.

## [0.210.0] - 2026-06-17

### Added

- Publish the Telegram Docker-in-Docker image to GHCR on release and document the one-line `docker run` / `docker compose up` startup path.
- Add root `compose.yaml` for the prebuilt Telegram bot image with `TELEGRAM_BOT_TOKEN` as the only required setting.
- One-click start/stop of both prepared services — the Telegram bot and the OpenAI-compatible API server — from the desktop app, with live Docker status polling (`desktop/lib/service-control.cjs` over IPC).
- Opt-in `server` profile in `compose.yaml` so a server reproduces the identical containers with one line (`docker compose --profile all up -d`); each Docker-in-Docker service gets its own inner-Docker volume so the bot and server can run together.
- New `docs/desktop/service-control.md` documenting both the one-click desktop and one-line server paths in detail.

## [0.209.0] - 2026-06-17

### Fixed

- CI: the **Deploy Demo to GitHub Pages** job no longer crashes with
  `No space left on device`. The job stopped restoring the multi-gigabyte
  `target/` cache shared with the `lint`/`test` jobs (it now caches only the
  Cargo registry under a dedicated `*-cargo-docs-*` key) and proactively frees
  unused pre-installed SDKs from the runner before building the API docs. Disk
  usage is now logged with `df -h` around the cleanup for future diagnosis.
  (#523)

## [0.208.0] - 2026-06-17

### Added
- Added a default assistant thinking preview that shows a collapsed current step, a faded previous step, an expandable localized summary list, and a configurable thinking-detail setting while preserving raw reasoning diagnostics behind the diagnostics toggle.
- Added first-class solver thinking metadata derived from the append-only event log and exposed it through Links Notation, Chat Completions, Responses, and the desktop HTTP chat path.
- Made thinking steps concrete by default: a shared naturalizer turns each reasoning event into a human-readable sentence that names the real content (the prompt, the detected language, the chosen route, the computed `expr = result`, the looked-up entity, the composed answer) instead of a generic label, and surfaces the same concrete reasoning on the CLI `--thinking` output, the OpenAI-compatible and Anthropic APIs, the browser, and the Telegram bot via a native collapsed-by-default expandable blockquote.

### Changed
- Promoted thinking to a first-class concern in a dedicated `thinking` module (step model plus naturalizer) so it is shared across every surface rather than embedded in the engine internals.

## [0.206.0] - 2026-06-16

Fix macOS desktop release signing by re-sealing ad-hoc `.app` bundles with
`codesign` before DMG upload, and document the `v0.205.0` CI failure that left
Linux/Windows assets present but macOS assets absent.

## [0.204.0] - 2026-06-15

### Fixed

- **Desktop apps are actually built and available on `/download` (issue #479).**
  The automated release tags a *child* `chore: release vX.Y.Z` commit whose
  CI run carries the *parent* SHA, so the desktop-release resolve step (which
  required a tag pointing at `workflow_run.head_sha`) never matched and zero
  desktop assets were uploaded — every release since the path went live showed
  "Not available in latest release". The resolve script now targets the latest
  published release with a defensive exact-SHA tier and an idempotency guard,
  and emits grouped verbose diagnostics (`[desktop-release-resolve]` logs) so
  the resolution decision is auditable for future triage; the
  `desktop-release` workflow no longer gates on full-pipeline
  `conclusion == 'success'` (the release is published early, so a later job
  failure used to suppress the whole desktop build); and
  `scripts/wait-for-pages-deployment.sh` is now marker-authoritative
  (`deployment.json`'s SHA proves the matching stamped build is live, since
  GitHub Pages deploys atomically) so the E2E Pages probe stops timing out and
  failing the pipeline. Landing/docs assets are cache-busted with
  `?v=__FORMAL_AI_ASSET_VERSION__` like `/app/`.

- **macOS install screenshots are real captures, not synthetic renders
  (issue #479).** The `/download` macOS Gatekeeper figures are now genuine
  macOS 15 (Sequoia) captures from the sibling app `konard/vk-bot-desktop`,
  which ships the identical `electron-builder` ad-hoc signing flow, replacing
  the previously generated images the maintainer rejected as fake. The
  synthetic generator and HTML fixture are removed; provenance is documented
  in `src/web/download/assets/screenshots/README.md`.

### Added

- **Source code is a big hero button on the landing page (issue #479).** The
  landing surfaces the source repository as a prominent `.source-cta` call to
  action (translated for every supported locale) instead of a small footer
  link.

## [0.203.0] - 2026-06-15

### Changed
- Raised the Rust toolchain MSRV to 1.96 (latest stable) and updated the Docker builder image to `rust:1.96-slim`, matching the `web-search` and `web-capture` crate MSRVs.
- Updated all Rust workspace dependencies to their latest versions (`clap` 4.6, `doublets` 0.4.0, `link-calculator` 0.19.0, `meta-language` 0.45, plus transitive updates).
- Updated web bundle dependencies to the latest versions (`react`/`react-dom` 19.2.7, `marked` 18.0.5, `dompurify` 3.4.10) and rebuilt `src/web/vendor.bundle.js`; pinned Bun to 1.3.14.
- Updated desktop (`electron` 42, `electron-builder` 26) and VS Code (`@vscode/test-web` 0.0.80, `@vscode/vsce` 3.9.2) dependencies to the latest versions.
- Refreshed the issue #410 case study to reflect that the upstream `web-search`/`web-capture` readiness blockers are resolved (`web-search` published at npm 0.10.3 / crates.io 0.3.1 with full provider parity; `web-capture` at npm 1.10.9 / crates.io 0.3.31).

### Fixed
- Resolved Clippy lints newly reported by the latest stable toolchain (1.96) so `cargo clippy --all-targets --all-features` stays clean under `-Dwarnings`: added `const fn` where derivable, switched to `Option::is_none_or`, `std::iter::repeat_n`, `f64::midpoint`, and `u*::is_multiple_of`.
- Preserved code-block enhancements (highlighting and copy buttons) under React 19 by memoizing rendered markdown by message content; React 19 compares `dangerouslySetInnerHTML` by object identity, which otherwise re-assigned `innerHTML` and wiped the out-of-band DOM enhancements on unrelated re-renders.

## [0.202.0] - 2026-06-15

### Added

- A **deterministic agentic planner** (`src/agentic_coding/planner.rs`) — the
  server's "brain" for issue #468's *"solve such tasks in agentic mode"*
  framing. It is a pure function of the conversation so far and the tool names an
  agentic CLI advertised, driving a small state-machine recipe
  (`web_search → web_fetch → write_file → run_command → final`) that formalizes
  «Сказка о рыбаке и рыбке» into a Links Notation knowledge base. Steps whose
  tool the CLI did not advertise are skipped, and tool *errors* are observed (an
  errored fetch is ignored and the formalizer falls back to the canonical
  synopsis), so the loop always completes with a stable, all-nine-primitive
  document. No sampling, no hidden state — the same history always yields the
  same plan, keeping neural inference a NON-GOAL.

### Changed

- The OpenAI-compatible chat endpoint (`create_chat_completion_with_solver`) now
  **emits `tool_calls`** with `finish_reason: "tool_calls"` when agent mode is on
  and a formalization task is in flight, closing the core gap that the server
  could never *request* a tool — it previously hard-coded `finish_reason: "stop"`
  on every turn. A `tool`-role result feeds back into the planner on the next
  request until the recipe is exhausted, at which point the server answers with
  the knowledge base inline (`finish_reason: "stop"`). Unrecognised requests
  still fall through to the ordinary symbolic solver, so non-agentic behaviour is
  byte-for-byte unchanged. `ChatMessage` gained OpenAI `tool_calls` /
  `tool_call_id` / `name` fields and `ToolCall` / `FunctionCall` types so tool
  requests and results round-trip through the wire format (issue #468).

### Added

- An **in-repo agentic driver** (`src/agentic_coding/driver.rs`) that plays the
  role of an external agentic CLI against our own OpenAI-compatible server,
  closing issue #468's *"our Formal AI system should have enough skills … to
  actually call all the tools from any agentic CLI, understand errors from
  tools, … do web fetch and web search, to actually complete the task"*. It
  advertises the four-tool set (`web_search`, `web_fetch`, `write_file`,
  `run_command`), and on every `tool_calls` turn the server emits it **executes**
  each call — search/fetch against an offline corpus, file writes and commands in
  a single reused, sandboxed [`AgentWorkspace`] — feeds each result back as a
  `tool` message, and loops until the server returns the finished knowledge base.
  The loop is bounded by a hard turn cap, so unbounded reasoning stays a NON-GOAL
  and no network or neural inference is ever involved. Exposed as
  `run_agentic_task` / `run_agentic_task_in` returning a `DriverOutcome` with the
  full tool-call transcript.
- An **offline, deterministic web corpus** (`src/agentic_coding/corpus.rs`) that
  resolves `web_search` / `web_fetch` tool calls against a fixed page set: a
  search that surfaces the canonical Викитека page for «Сказка о рыбаке и рыбке»
  and a fetch that returns the canonical synopsis (the formalizer's fallback
  text), plus a 404 path for unknown URLs so the driver exercises the
  *"understand errors from tools"* requirement with no live network.
- A new **`agent` CLI subcommand** (`formal-ai agent [--task …] [--transcript]`)
  that drives the whole offline loop and prints the resulting Links Notation
  knowledge base, with `--transcript` showing every executed tool call.
- An `issue_468_agentic_loop` example that runs the driver end to end and prints
  the transcript plus the final knowledge base.

### Changed

- `AgentWorkspace` gained a `last_command_result()` accessor so a long-lived
  workspace reused across a tool-call loop can observe each command's output
  between steps, before `finish` consumes it.
- The default associative packages now include a permission-only
  `pkg_agentic_coding` package granting the client-executed `web_fetch`,
  `write_file`, and `run_command` capabilities, so the full agentic loop passes
  the server's tool-permission gate. `agent_mode` remains the real guard (every
  tool is still refused unless it is explicitly enabled), and capabilities the
  package does not name (e.g. `local_shell`) stay denied.

### Added

- A **grounded meta-algorithm recipe for the agentic-coding loop**
  (`data/meta/agentic-coding-recipe.lino`), following the issue #444 pattern. It
  names every part the deterministic loop is made of — the plan constants
  (`SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH`), the four advertised tools
  and their capabilities and permissions, the `search → fetch → write → run →
  final` state-machine stages, the fourteen handler functions, the nine protocol
  primitives the product realises, the `MAX_TURNS` cap, and the CLI/example/test
  exposure surfaces — plus the eight ordered steps that generalise to a new task.
- A **grounding test** (`tests/unit/specification/agentic_meta_algorithm.rs`)
  that loads the recipe and asserts the live source still matches every entry, so
  the recipe can never silently drift from the code (CI fails if it does).

### Changed

- `docs/meta-algorithm.md` now documents the agentic-coding meta-algorithm as a
  second grounded recipe alongside the procedural how-to one, including the state
  machine, the eight ordered steps, and the grounded-record table.

### Added

- The deterministic **agentic-coding loop now drives the Anthropic Messages
  (`/v1/messages`) and OpenAI Responses (`/v1/responses`) surfaces**, not just
  Chat Completions. The maintainer's framing for issue #468 was that the system
  must "call all the tools from any agentic CLI"; `claude` speaks Anthropic
  Messages and `codex` speaks OpenAI Responses, so both now emit native tool
  requests (`tool_use` content blocks / `function_call` output items) and
  *understand* fed-back tool results delivered in each protocol's own idiom (an
  Anthropic `tool_result` block carried on a `user` message, an OpenAI
  `function_call_output` item) so the loop advances rather than restarting.
- `AnthropicMessagesRequest` and `ResponsesRequest` now accept `tools` and
  `tool_choice`, translated into the shared OpenAI tool shape so a single
  deterministic planner backs all three surfaces.
- New public types for the Responses tool mirror: `ResponseFunctionToolCall` and
  the `ResponseOutputItem` enum (`Message` | `FunctionCall`), with
  `ResponseObject::output_messages()` and `ResponseObject::function_calls()`
  accessors.
- A focused test module (`tests/unit/agentic_surfaces.rs`, 9 tests) pins the
  mirror: tool emission in agent mode, tool-result feed-back advancing the loop,
  the final knowledge-base answer once the recipe is exhausted, refusal without
  agent mode, SSE `input_json_delta` streaming for `tool_use`, and symbolic
  fall-through for non-agentic tasks.

### Changed

- The chat, Anthropic, and Responses surfaces now share one `agentic_outcome`
  decision (refuse / plan / fall-through) so the agent-mode gate, per-tool
  permission gate, and planner behave identically everywhere; the symbolic
  fall-through still preserves `evidence_links`.
- `AnthropicMessage.content` is now a list of typed content blocks
  (`AnthropicContentBlock::Text` | `ToolUse`) instead of a single text block, and
  `ResponseObject.output` is now a list of `ResponseOutputItem`s. Wire-format JSON
  is unchanged for the text-only case.

### Changed
- Rewrote the issue #468 case study (`docs/case-studies/issue-468/README.md`) and
  `REQUIREMENTS.md` rows **R306–R319** to describe the shipped `src/agentic_coding/`
  agentic-coding loop — the deterministic planner (server brain), the in-repo driver
  and offline corpus (client), the two permission gates, and the
  nine-primitives-as-links formalizer — replacing the earlier text that described a
  removed typed-struct draft.

### Added
- `docs/desktop/server-api.md` §4e documents the multi-surface agentic tool-calling
  loop: how each agentic CLI (`codex` via Responses, `opencode` + `agent` via Chat
  Completions, `claude` via Anthropic Messages) drives `formal-ai serve`, how the
  server emits the next tool call and consumes the fed-back result, and the
  `agent_mode` + `pkg_agentic_coding` gating — with external CLIs as front-ends,
  never embedded in the engine.
- A traceability test (`issue_468_agentic_coding_case_study_is_traceable` in
  `tests/unit/docs_requirements_issue_468.rs`) pins `REQUIREMENTS.md` rows
  R306–R319, the case study `README.md`, `formal-protocol-mapping.md`, the
  `server-api.md` §4e agentic-loop section, and the two worked examples
  (`examples/issue_468_agentic_loop.rs`, `examples/issue_468_formalize_text.rs`)
  to the live implementation.

### Fixed
- Desktop app downloads are available again on `/download` for every platform. The desktop-release workflow resolved the parent commit SHA, but the auto-release tag is created on the child "chore: release" commit, so the exact-SHA match never succeeded and v0.187.0–v0.201.0 shipped zero desktop assets. Resolution is now two-tier (exact SHA, then the latest published release / auto-release child commit), with verbose logging for future diagnosis (issue #479).
- Refreshed the obsolete `/download` desktop app-preview and page screenshots.
- Replaced a manual `(len + 1) / 2` ceiling in the lexicon matcher with `div_ceil`, satisfying clippy's `manual_div_ceil` lint under newer stable toolchains.

### Added
- macOS Gatekeeper install screenshots on the `/download` page, mirroring the vk-bot-desktop walkthrough.
- A landing-page chooser at `/` that links to the web app (`/app/`), the documentation hub (`/docs/`), and the desktop download (`/download/`), wired to the shared theme + UI-language preference store and localized into en/ru/zh/hi.
- A documentation hub at `/docs/` and a generated Rust API reference at `/docs/api/`, built with `cargo doc` during the GitHub Pages deploy.
- End-to-end coverage for the new landing and documentation pages, plus CI guards for the new static and deploy invariants.

### Changed
- The interactive web app moved from `/` to `/app/`, served with `<base href="../">` so its shared site-root assets still resolve under both the GitHub Pages path prefix and the desktop static server; the desktop wrapper and in-site back-links now target `/app/`.
- The release pipeline's concurrency is now main-safe, so a release run on `main` is never cancelled mid-flight.

## [0.201.0] - 2026-06-14

### Added
- Recognize document-generation requests ("Сделай мне пдф файл …", "make me a
  PDF/document/report with …") and answer with the universal algorithm's formal
  plan — scope, gather, classify, assemble, export — localized to the prompt
  language, instead of falling through to the unknown response.
- Updated `meta-language` to 0.45.0 (raising the crate `rust-version` to 1.77)
  and exposed its document-format concept layer through the natural-language
  document workflow: TXT, Markdown, HTML, PDF, and DOCX conversion now routes
  through `LinkNetwork::reconstruct_text_as`, reports target fidelity fallbacks,
  and records DOCX package-layer evidence when the upstream OPC profile is
  available.

### Fixed

- List-files program answers now render language-aware sample output, so Python examples no longer show Rust fixture files like `Cargo.toml` or `main.rs` (issue #440). Browser responses also separate the "not run" status from the copy instruction and use a light code-block palette when the app is in light mode.

## [0.200.0] - 2026-06-14

### Fixed

- Issue #444: a bare elaboration follow-up after a "how to …" answer (e.g.
  "Can you give me specific instructions?") no longer dead-ends at the
  unknown-intent opener. It now rebinds to the procedure recovered from the
  prior turn and answers as `procedural_how_to` in the original language.

### Added

- New `procedural_elaboration` seed meaning (en/ru/hi/zh) and
  `try_procedural_how_to_followup` handler, mirrored in the browser worker
  (`tryProceduralHowToFollowup`), keeping Rust ↔ JS parity.

### Added

- **External trusted services are available and opt-out-able (issue #444).**
  The procedural how-to handler may now consult wikiHow, the Stack Exchange
  network, the MediaWiki family (Wikibooks, Wikiversity, Wikivoyage), and
  GitHub READMEs/docs in addition to Wikipedia and Wikidata. Every external
  source is declared in `data/seed/sources-registry.lino` under an
  `external_trusted` group with its own `settings_key` and `default_enabled true`
  (opt-out model), and the web settings UI exposes a section to toggle each one.
- **Procedural how-to / instruction-following benchmark slice (issue #444).**
  `data/benchmarks/procedural-howto-suite.lino` adds self-authored representative
  cases in the style of six widely-used instruction-following benchmarks
  (IFEval, Super-NaturalInstructions, Self-Instruct, OASST1, BIG-bench, MMLU),
  each with a paraphrased held-out variant for anti-memorization, ratcheted by
  `tests/unit/specification/procedural_howto_benchmarks.rs`. Topics span apology
  letters, meal planning, gardening, bicycle repair, pour-over coffee, and
  nutrition labels so the routing is exercised across diverse domains.
- **Central benchmark catalog (issue #444).** `docs/benchmarks.md` indexes every
  benchmark suite the repository has ever touched (issues #103, #304/#317, #362,
  #408, #444) with their fixtures, ratchet tests, sources, and licenses; guarded
  by `tests/unit/docs_requirements.rs`.
- **Grounded meta-algorithm that reproduces topic handlers on demand (issue #444).**
  `data/meta/procedural-howto-recipe.lino` is a machine-readable recipe naming
  every seed role, handler function, evidence stage, Rust↔JS parity target,
  external-service toggle, and benchmark that make up the procedural how-to
  topic, plus eight ordered steps that generalise to any topic.
  `tests/unit/specification/meta_algorithm.rs` keeps the recipe grounded by
  asserting the live source still matches every entry, and
  `docs/meta-algorithm.md` explains how to run and generalise it — so we learn
  from our own source code how to produce changes on the topic rather than only
  emitting one-off code changes.

### Fixed

- **Procedural elaboration follow-ups rebind to the prior how-to (issue #444).**
  After a "how to X" turn, a bare elaboration follow-up such as "Can you give me
  specific instructions?" now rebinds to the established procedure and restates
  the task in both the Rust solver (`src/solver_handler_how.rs`) and the browser
  worker mirror (`src/web/formal_ai_worker.js`), instead of falling through to
  the unknown opener.

## [0.199.0] - 2026-06-13

### Added
- Symbolic AI reference and best-practice audit (issue #451). `README.md`,
  `VISION.md`, and `ARCHITECTURE.md` now cite the Wikipedia
  [*Symbolic artificial intelligence*](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
  article (plus *Semantic network*, *Physical symbol system*, and *Neuro-symbolic
  AI*), making the project's GOFAI lineage explicit: the associative network of
  links is a semantic network in the classical sense.
- A case study under `docs/case-studies/issue-451/` with deep analysis, collected
  issue/PR data, and cited online research (`raw-data/online-research.md`,
  including 2024–2026 neuro-symbolic surveys).
- `docs/case-studies/issue-451/symbolic-ai-best-practices.md` — a 20-row audit
  mapping every technique family the article names to the associative-stack
  component that realizes it (`solver.rs`, `proof_engine/`, `probability.rs`,
  `substitution.rs`, `rule_synthesis.rs`, `knowledge.rs`, `event_log.rs`), with an
  honest applied/partial/proposed status and named reuse targets for each gap.
- Requirements **R298–R305** in `REQUIREMENTS.md` and the regression test
  `tests/unit/docs_requirements.rs::issue_451_symbolic_ai_reference_documents_are_present_and_traceable`,
  which pins the reference, the audit, and the requirement list so they cannot
  silently regress.

The documentation half of this work is reference and tests only; the
accompanying engine change (the DPLL satisfiability backend that closes the
audit's single proposed gap, R305) is described in its own changelog entry.

### Added

- A deterministic, dependency-free **DPLL satisfiability backend** at
  `src/proof_engine/decision/sat.rs` (issue #451, R305), closing the
  best-practice audit's single proposed gap (§3.2 SAT / constraint solving). The
  solver works over CNF (`CnfFormula`, `Literal`, `SatOutcome`) with unit
  propagation, pure-literal elimination, and chronological backtracking, and is
  byte-reproducible and WebAssembly-safe (lowest-index variable, `false`-before-
  `true` branch order). The Rust crates `splr` / `varisat` were evaluated as
  reuse targets but set aside to keep the engine free of native dependencies;
  they remain the documented upgrade path for CDCL/CSP-scale workloads.

### Changed

- The propositional decision procedure (`src/proof_engine/decision/boolean.rs`)
  now generalizes past the eight-variable truth-table limit: claims with more
  variables are Tseitin-encoded to CNF and handed to the new DPLL backend behind
  the existing "formalize → delegate → trace" seam. An unsatisfiable negation
  yields a tautology proof; a satisfiable one yields a concrete countermodel
  disproof. Claims of eight or fewer variables keep the exhaustive truth-table
  witness unchanged, so every prior proof and test is byte-for-byte unaffected.
- Doubled the propositional-decision test surface: new
  `tests/source/source_tests/proof_engine/decision/{sat,boolean}/tests.rs` unit
  suites plus `tests/unit/proof_request.rs` integration cases exercise the SAT
  path (wide tautology proven via DPLL, wide non-tautology disproven with a
  countermodel, the truth-table boundary, Tseitin-encoding fidelity, and the
  over-width decline), keeping coverage close to 100%.

## [0.198.0] - 2026-06-13

### Fixed
- **Bare "invert the sort" follow-up no longer answers `unknown` (issue #427).**
  After a numeric-list sorting conversation, the bare follow-up
  "Сделай инверсию сортировки." (make the inversion of the sort) fell through to
  `unknown`: the operation vocabulary did not recognize the *invert* phrasing as
  a descending sort, and even when an operation was named the handler had no
  numbers to act on because the follow-up lists none of its own. The
  numeric-list handler now inherits the list from the most recent operation turn
  that carried a concrete list — while the language and code request keep coming
  from the most recent turn that named a language (issue #412) — so a
  number-less invert-sort continues the established coding context and emits the
  descending code plus result.

### Added
- `reverse_sort` operation vocabulary now matches *invert*-style phrasings across
  every supported language: English `invert the sort` / `invert the sorting` /
  `invert sort` and `combo invert+sort`; Russian `combo инверс+сортиров` /
  `combo инверт+сортиров`; Hindi `combo उलट+क्रम`; Chinese `combo 反转+排序` /
  `combo 颠倒+排序`.
- Implemented identically in the Rust solver
  (`src/solver_handlers/numeric_list/mod.rs`) and the browser worker mirror
  (`src/web/formal_ai_worker.js`) so both runtimes inherit the prior list and
  reverse it; covered by `tests/integration/issue_427_invert_sort.rs`, the
  `operation_vocabulary_reverse_sort_matches_invert_phrasings` source test, and
  the `experiments/issue-427-worker-invert-sort-parity.mjs` cross-runtime check.

## [0.197.0] - 2026-06-13

### Added
- Symbolic evidence count `C` tracked separately from accumulated utility `U` in
  `src/probability.rs` via `ProbabilityStore::target_evidence_count`, porting the
  interpretable transition model from Kolonin's arXiv:2605.00940 onto the
  associative stack (issue #449).
- Counted-utility decision policy and under-evidenced gating in
  `ProbabilityRankingConfig`: `counted_utility` (rank by `U·C`),
  `min_transition_utility`, and `min_transition_count`. Defaults preserve the
  prior additive behavior.
- `RankedProbabilityCandidate::evidence_count`, surfacing the evidence count next
  to the evidence weight so each ranked option stays locally interpretable.
- Case study `docs/case-studies/issue-449/` with compiled raw data, online
  research, deep analysis, requirement enumeration, and per-requirement plans.

### Changed
- Documented the evidence-count / counted-utility / transition-threshold
  mechanisms in `ARCHITECTURE.md` section 6.1.

### Added

- Ported the remaining interpretable, non-neural mechanisms from Kolonin's
  "Interpretable Experiential Learning" (arXiv:2605.00940) onto the symbolic
  probability layer (issue #449):
  - `symbolic_cosine_similarity` plus `ProbabilityStore::nearest_similar_evidence`
    implement the paper's `SS` inexact-state fallback — a candidate with no exact
    evidence borrows the nearest stored target's utility, scaled by a
    deterministic bag-of-words cosine, gated by a similarity floor.
  - `ProbabilityStore::reinforce_transition_path` implements the paper's
    episode-wide global feedback — one append-only `markov_transition`
    observation per adjacent state pair, replayable through the event log and
    link-store projection.
  - `ProbabilityDecisionPolicy` groups the `CU`/`TU`/`TC`/`SS` knobs into one
    `Copy` policy, threaded through `SolverConfig::probability_policy` into every
    selection use case via `ProbabilityRankingConfig::with_decision_policy`.
  - `RankedProbabilityCandidate` now exposes a `similarity` field so a
    fallback-driven decision stays locally interpretable.
- Added the `examples/issue_449_interpretable_learning.rs` worked tour of all
  four mechanisms (Bayesian utility, counted utility, thresholds, similarity
  fallback, and episode reinforcement).

### Changed

- Doubled the probabilistic-reasoning specification suite to lock the new
  behaviour and keep coverage close to 100%; existing callers are byte-for-byte
  unaffected because the default policy reproduces the paper's recommended
  baseline (`CU=False`, `TU=0`, `TC=1`, no similarity fallback).

## [0.196.0] - 2026-06-13

### Added
- Added `docs/USER-JOURNEYS.md` (issue #454): a dedicated document that states the pain Formal AI closes, the personas it is for, and the concrete user journeys it supports today (transparent "why did you answer that?", multilingual chat, code generation with honest execution notes, math/units/currency, sourced fact and definition lookup, single-file memory export and cross-surface migration, OpenAI-compatible endpoint, Telegram, edit-the-data reconfiguration, bounded agent tasks, and follow-up edits) plus the journeys it could support next (visual graph, compiled skills, cloud memory sync, search-based solving, WebVM, shared associative packages). Each journey is mapped to the implemented surfaces and the relevant `VISION.md` principles, with a journey-to-surface coverage matrix and a worked end-to-end example.

### Changed
- `VISION.md` now opens with a "Who This Is For And What Pain It Closes" section and a concrete example user journey, linking to `docs/USER-JOURNEYS.md` so the vision makes a concrete promise rather than only describing the machine. `README.md` links the new document from its project-direction paragraph.

## [0.195.0] - 2026-06-13

### Added
- Natural-language calendar event creation that exports a **real, importable
  calendar event** in every supported environment (issue #404). The prompt
  "Забей мне 18 число в 17:00 по грузии на встречу с Леваном" (and its English,
  Hindi, and Chinese equivalents) now resolves to a `calendar_create_event`
  intent instead of `unknown`, and the confirmation proposal carries two
  login-free, portable artifacts:
  - a universal **RFC 5545 `.ics` VEVENT** document (CRLF line endings,
    `DTSTART;TZID=`/`DTEND;TZID=`, escaped `SUMMARY`, stable content-derived
    `UID`) that imports cleanly into Apple Calendar, Outlook, Google Calendar,
    Thunderbird, and any other iCalendar client — the simplest method available
    in the CLI and HTTP environments, where the user can save/import a file; and
  - a **Google Calendar "render" template URL**
    (`calendar.google.com/calendar/render?action=TEMPLATE&text=…&dates=START/END&ctz=…`)
    that pre-fills a new event in the user's browser with no API token or server
    — the simplest method in a browser environment.
- Full multilingual support (en, ru, hi, zh). Surface words — schedule verbs,
  "meeting"/"встреча"/"मीटिंग"/"会议", clock times, and timezone aliases such as
  "по грузии" → `Asia/Tbilisi` — live as self-describing meanings in
  `data/seed/meanings-calendar.lino`; the code knows only roles and English
  slugs. Hindi (verb-final) and Chinese (no word spaces) titles are trimmed of
  trailing/leading schedule-action fragments so the `.ics` SUMMARY keeps only
  the event and its participant.
- Byte-for-byte Rust ↔ WASM parity: the `.ics` builder, Google Calendar URL
  builder, and title tidying are mirrored in the browser worker
  (`src/web/formal_ai_worker.js`), verified to produce identical artifacts
  across all four languages.

### Changed
- Extracted the calendar export logic (the `ScheduledEvent` model, RFC 5545
  `.ics` builder, Google Calendar URL builder, and date/duration helpers) into a
  new `src/solver_handlers/calendar_ics.rs` module, and split the docs-method /
  how-to procedure reasoning-path tests into
  `tests/unit/specification/reasoning_paths_procedures.rs`, keeping every file
  under the repository's 1000-line limit.

The core remains purely symbolic and deterministic: the solver *proposes* the
event and invites confirmation rather than silently mutating a remote calendar.
No existing weekday-relation or "today" calendar behaviour was changed.

Example (ru, Asia/Tbilisi):
  U: Забей мне 18 число в 17:00 по грузии на встречу с Леваном
  A: (calendar_create_event) "Создать событие «Встречу с Леваном» на 18 число
     (2026-06-18). Время: 17:00, часовой пояс: Asia/Tbilisi. … BEGIN:VCALENDAR …
     https://calendar.google.com/calendar/render?action=TEMPLATE… Ответьте «да»…"

### Added
- Relative-date calendar scheduling (issue #435). The prompt
  "Можешь поставить мне созвон в кальндарь на завтра?" — which carries no day
  number and no clock time, only a relative-date word ("на завтра") and an event
  noun ("созвон") — now resolves to a `calendar_create_event` instead of
  `unknown`. The solver recognizes relative-date words as a date anchor, resolves
  "завтра"/"tomorrow"/"कल"/"明天" to **tomorrow** (and "послезавтра"/"day after
  tomorrow"/"परसों"/"后天" to the day after), and titles the draft from the matched
  event noun when no explicit subject is given.
- New `calendar_relative_date` role with the `calendar_tomorrow` and
  `calendar_day_after_tomorrow` meanings in `data/seed/meanings-calendar.lino`,
  grounded in Wikidata and surfaced in en/ru/hi/zh. As with the rest of the
  lexicon-driven design, the code knows only the role and English slugs; adding a
  language never touches code.
- A new `calendar:parsed_relative_offset` evidence link records the resolved
  day offset, alongside the existing `calendar:parsed_*` trace, in both the Rust
  engine and the byte-for-byte browser worker mirror (`src/web/formal_ai_worker.js`).

The core remains purely symbolic and deterministic: the solver *proposes* the
tomorrow event with an importable RFC 5545 `.ics` VEVENT and a login-free Google
Calendar render URL, and invites confirmation rather than silently writing the
calendar. A bare relative-date mention with no schedule verb or event noun is
not hijacked into a create request.

Example (ru):
  U: Можешь поставить мне созвон в кальндарь на завтра?
  A: (calendar_create_event) "Создать событие «Созвон» на 14 число (2026-06-14).
     … BEGIN:VCALENDAR … calendar.google.com/calendar/render… Ответьте «да»…"

## [0.194.0] - 2026-06-13

### Fixed

- CI no longer runs the Rust test suite on changes that touch no code (issue #442). The `test` job in `release.yml` previously ran whenever the `changelog` job was *skipped* — but `changelog` is skipped precisely when there are no code changes, so docs-only / `.gitkeep` / changelog-fragment-only commits triggered the full `cargo test` matrix. The `test` job now gates on the `detect-changes` outputs (`any-code-changed` / `rs-changed` / `toml-changed` / `workflow-changed`), the same way `lint` and `coverage` already do.

## [0.193.0] - 2026-06-13

### Changed

- Generalized the installation-conversion command recognizer
  (`installation_conversion::looks_like_command`) from a fixed tool-prefix
  whitelist to a provenance-aware structural rule: any well-formed command line
  is accepted regardless of which tool it invokes, while prose lines are rejected
  even when they mention a tool (issue #433).
- Replaced the install-step description table (`describe_command`) with verb/object
  intent inference, so unseen but recognizable tools (`bun install`, `pdm install`,
  `just build`, `zig build`) get accurate step descriptions without extending a
  table.
- Mirrored the structural recognizer and verb/object describer into the browser
  worker (`src/web/formal_ai_worker.js`) for cross-runtime parity.

### Added

- Case study `docs/case-studies/issue-433/` with (1) an audit classifying every
  specialized handler recognizer as fixed-enumeration vs compositional, (2) the
  installation-conversion generalization, and (3) a documented reconstruction of
  the `numeric_list` coding handler from the meta-algorithm rule primitives.
- False-positive (prose) and unlisted-tool regression coverage for the
  installation-conversion recognizer across the Rust unit suite, the
  source-mirror private-function suite, and the browser-worker experiment.

## [0.192.0] - 2026-06-13

### Added
- Seeded a `metatheory` concept so prompts like `theory of theory`,
  `metatheory`, `теория теории`, and `元理论` resolve to a verified
  `concept_lookup` answer instead of the unknown fallback (issue #436). The
  record carries en/ru/zh summaries grounded in Wikipedia, keeps the existing
  Link Foundation `Links meta-theory` (`теория связей`) routing intact, and is
  grounded for total reference-closure via a new `metatheory` meaning and
  `proof_concept_metatheory` role.

## [0.191.0] - 2026-06-13

### Added
- Added deterministic installation conversion support for README.md
  install/deploy guides, Bash/sh scripts, and PowerShell scripts. The new
  `installation_conversion` handler extracts ordered install commands into a
  shared IR, renders scripts or README guides from that IR, and is mirrored in
  the browser worker so conversion prompts no longer fall through to `unknown`
  or generic script generation.

- Added issue #423 regression coverage, including README-to-Bash/PowerShell,
  script-to-README, nested fenced README content, PowerShell-to-README,
  meta-algorithm trace assertions, and a 100-project GitHub repository corpus
  captured from the most-starred repository snapshot.

- Added an algorithm-construction trace for installation conversion responses,
  connecting the problem-class -> shared-IR -> renderer -> verification pattern
  to the existing coding catalog, program synthesis, program blueprint,
  numeric-list, and rule-synthesis surfaces.

## [0.190.0] - 2026-06-12

### Changed
- Updated the `meta-language` dependency to 0.40.0 and documented the issue
  #428 upstream research, compatibility audit, and follow-up integration plan.

## [0.189.0] - 2026-06-12

### Fixed
- Apply user-requested text replacements to generated code answers, including follow-up replacement requests that refer to the previous assistant response.
- Accept broader replacement prompt shapes, including input-first phrasing, smart quotes, corner quotes, and punctuation-tolerant multi-word matches.
- Add deterministic remove, append, prepend, trim-whitespace, normalize-whitespace, case-conversion, extraction, counting, punctuation, and line-shape text/code edit operations with multilingual operation vocabulary triggers.
- Cover 61 benchmark-family prompt-answer examples across CoEdIT, EditEval, InstrEditBench, CodeEditorBench, CanItEdit, EDIT-Bench, HumanEvalFix, and SWE-bench style edit tasks.
- Add a manifest-backed issue #408 benchmark profile with 48 researched sources, 30 local variations per source, a per-source 3-check 10% floor, and a 1,440/1,440 pass-count ratchet.
- Document the issue #408 benchmark-source audit and keep the roadmap, vision, requirements, architecture, and case-study benchmark contract in sync.

## [0.188.0] - 2026-06-12

### Fixed
- Delegated `?` and `*` placeholder equations to `link-calculator` 0.18.2, with expanded coverage for symbolic multi-variable and polynomial equation categories.
- Kept the web worker aligned for placeholder, symbolic, and polynomial equation prompts, including Markdown-safe rendering of spaced `*` placeholders.

## [0.187.0] - 2026-06-12

### Fixed
- Recognize short behavior-rule list prompts such as `Покажи правила`, `Show rules`, `नियम दिखाओ`, and `显示规则` instead of falling through to the unknown fallback.

### Fixed
- Recognize calculation requests embedded inside longer user statements, including Russian prompts like `хочу понять сколько будет 2+2`.

### Changed
- Replaced emoji toolbar glyphs with accessible local icons and a persisted toolbar icon-pack setting.

## [0.186.0] - 2026-06-11

### Fixed
- Answer Russian hidden-number interval riddles by formalizing the bounds as a
  linear constraint and showing the proof-engine verification instead of
  returning the unknown fallback.

### Fixed

- Answer Russian “Что делаешь в свободное время?” small talk with localized assistant free-time responses instead of the unknown fallback.

## [0.185.0] - 2026-06-11

### Fixed
- A bare numeric-list follow-up no longer answers `unknown` (issue #412). After
  a turn establishes a coding context — e.g. "…отсортируй их в JavaScript, дай
  мне код и результат" — a follow-up that names no language and does not ask for
  code, such as `Отсортируй 4, 3, 1, 17, 8, 9, 15`, now recovers the target
  language (and the code request) from the conversation and continues the coding
  context: idiomatic code in the established language plus the deterministically
  computed result.

### Added
- Conversational coreference for the numeric-list coding path. A new
  `numeric_list_history_context` inherits the language / code request from a
  prior turn **only** when that turn was itself a genuine numeric-list coding
  request (a recognised operation, a supported program language, and ≥2 numbers),
  so unrelated chatter never leaks a language. A `numeric_list_coreference`
  trace event records what was inherited. Implemented identically in the Rust
  solver (`src/solver_handlers/numeric_list/mod.rs`) and the browser worker
  mirror (`src/web/formal_ai_worker.js`); the 170-cell cross-runtime parity
  matrix stays byte-identical.

### Added
- Coding oracle backed by external knowledge sources (issue #412, R6). The
  solver now treats Rosetta Code, Wikifunctions, the Hello World Collection, and
  Stack Overflow as cached external APIs (even though they expose no machine
  API) and generalises `write_program` beyond the verified catalogue: a request
  for a language the catalogue does not template — Kotlin, Swift, PHP, Bash, Lua,
  Haskell — now returns a reviewed snippet, its deterministic output, and its
  source attribution instead of dead-ending on the unsupported answer.
  Catalogued languages keep their verified "compiled and ran" route untouched;
  the oracle only ever supplies an answer the solver would otherwise lack. New
  module `src/knowledge.rs` (sources + `CodingOracle`) and handler
  `src/solver_handler_oracle.rs`, mirrored byte-for-byte in the browser worker
  (`src/web/formal_ai_worker.js`).
- Bounded-cache policy (issue #412, R8). `cache_capacity` /
  `within_cache_capacity` / `KNOWLEDGE_CACHE_FLOOR` enforce "never cache more
  than 1% of a source, or 512 items when 1% is smaller", clamped to the source
  size, for every per-source / per-topic cache. A ratchet test fails CI if the
  committed snapshot set ever exceeds the cap, so a cache can never silently grow
  into a mirror.

## [0.184.0] - 2026-06-10

### Added
- Issue #395: a concrete "sort these numbers in <language>, give me the code and the result" request now routes to `write_program` instead of `unknown`. Rather than a narrow sorting handler, this ships a universal, data-oriented list coding engine (`src/solver_handlers/numeric_list/`): it reads the operation, the given values, and the target language from meanings, builds a semantic `NumericProgram` syntax tree, renders idiomatic code in the requested programming language (JavaScript, TypeScript, Python, Rust, Go, Ruby, Java, C#, C, C++), and computes and shows the deterministic result in the solver itself.
- Added CST/AST validation for generated programs in native Rust using [link-foundation/meta-language](https://github.com/link-foundation/meta-language) 0.39 (`meta_language::LinkNetwork`) as the sole, mutable CST/AST engine. `src/coding/cst.rs` parses rendered source through the links network and accepts it only when the parse produces real `LinkType::Syntax` links, round-trips the text, and has no errors. meta-language 0.39 ships grammars for every target (JavaScript, Python, Rust, Java, C, C++, C#, TypeScript, Go, Ruby), so all of them validate through the same links network and the direct `tree-sitter` dependency is gone. The TypeScript/Go/Ruby grammar gaps were reported upstream and resolved in meta-language — [#41](https://github.com/link-foundation/meta-language/issues/41), [#42](https://github.com/link-foundation/meta-language/issues/42), [#43](https://github.com/link-foundation/meta-language/issues/43). The remaining missing feature — rendering target-language source from a programmatically constructed syntax network, so generated code would be valid by construction instead of validated after composition — is reported as [meta-language#64](https://github.com/link-foundation/meta-language/issues/64). The trace now logs `synthesis:cst_engine` (`meta_language`) alongside `synthesis:cst_tree`.
- The list engine covers seven operations out of one shared algorithm — `sort`, descending sort (`reverse_sort`), `reverse`, `sum`, `product`, `minimum`, `maximum` — driven by `data/seed/numeric-list-operations.lino`. Transformations now support both numeric lists and quoted string lists; numeric reductions remain gated behind a `code_request` signal to avoid over-matching prose. Localized rendering covers all four supported UI languages (English, Russian, Hindi, Chinese).

- Code generation is now seed data, not code: `data/seed/coding-idioms.lino` declares per-language scaffolds and idioms (code fragments with cases selected by operation and value class, inherited through `extends`), and both runtimes discover the composition at execution time by walking the language's inheritance chain and recursively expanding idiom slots (`src/solver_handlers/numeric_list/codegen.rs` and the matching composer in `src/web/formal_ai_worker.js`). The per-language renderer functions are gone, so covering a new language or coding task is a seed-data change. Composition failures are explicit (`None`/`null`), never silent fallbacks.

### Changed
- Mirrored the list engine in the web runtime (`src/web/formal_ai_worker.js`, `tryNumericList`) so the browser and Rust paths build equivalent syntax trees and produce equivalent code/results, including quoted string list transforms, verified by `experiments/issue-395-js-numeric-list.mjs` and exhaustively by `experiments/issue-395-cross-runtime-codegen-parity.mjs`, which byte-compares all 170 (operation × language × value class) answers between the Rust engine (`examples/numeric_list_matrix.rs`) and the worker.
- Updated the related Python program-synthesis handler in Rust and the web worker to store a `PythonFunctionTree` with semantic statement nodes and render source from that tree. Native Rust now also validates the rendered Python source through meta-language and exposes `synthesis:cst_engine` and `synthesis:cst_tree` evidence before the code fragment.
- Added `examples/numeric_list_execution.rs`, an execution-verification harness that compiles and runs every generated program across the available toolchains and asserts the program's stdout equals the solver's computed result, closing the loop between the "verified by construction" claim and real execution.

## [0.183.0] - 2026-06-10

### Added
- Added meaning-level semantic facets so seed meanings can link notation, annotation, denotation, and connotation to other meanings.
- Added word-form semantic facets plus derived notation/denotation links and lexical meta meanings for word surfaces, lexical forms, lexical senses, and part-of-speech links.
- Added a compact Links-Theory semantic root seed with self-equations, defined connectives, quantity primitives, and one-symbol-one-meaning sense splits.
- Added semantic grounding checks and source cache records so Links-root definitions resolve recursively through checked-in Wikidata and Wiktionary data.
- Added issue #398 case-study documentation and semantic meta-language seed vocabulary.

### Fixed
- Replaced the codepoint byte-dump encoding of seed text (e.g. `answer codepoints 72 105 ...`) with human-readable quoted scalars, so `data/seed/*.lino` stays legible while every runtime parser decodes the same values.

### Changed
- Taught the Rust, web, and e2e LiNo seed parsers (plus the worker's embedded fallback) to decode single-quote, double-quote, and backtick scalars with a non-escaping delimiter.
- Removed the 4,677 synthetic `seed-surface-<hash>` ids from `data/seed/*.lino`: a surface is now the text (and facets) recorded under a language, not an opaque minted id. Added `scripts/clean-seed-readability.rs` to perform the lossless migration and regenerate the browser worker fallback.
- Stripped keyword-restating noise comments (`# language`, `# definition-link`, `# semantic-role`, `# facet`, `# seed lexical surface`, `# source-id`, `# action`) from the seed while keeping comments that carry the human meaning of an opaque id.

### Added
- Added a CI guard that bans codepoint byte-dumps in seed data and a guard that bans inline `#[test]`/`#[cfg(test)]` scaffolding under `src/`.
- Added CI guards that ban reintroducing synthetic `seed-surface-<hash>` ids and keyword-restating noise comments in seed data.

### Fixed
- Collapsed every `facet <kind>` wrapper whose child was an empty-bodied colon redefinition (`word_surface:`, `lexical_sense:`, ...) into native Links Notation `subject predicate` lines (`notation word_surface`, `denotation lexical_sense`). This removes the valueless `concept:` shape the review banned, across the whole `data/seed` tree.

### Changed
- Taught the Rust seed consumer (`parse_semantic_facets`) to read the direct `<kind> <target>` subject-predicate form in addition to the legacy `facet <kind>` wrapper, de-duplicating targets so both forms project identical facets.
- Added `scripts/migrate-empty-facet-fields.rs`, a std-only re-runnable migration that performs the collapse tree-wide and regenerates the embedded browser worker fallback (`src/web/formal_ai_worker.js`).

### Added
- Added a tree-walking CI guard (`seed_lino_files_have_no_empty_redefinition_fields`) that fails when any `data/seed/**/*.lino` line is an empty-bodied colon field with no deeper-indented child, with no hard-coded filename.

### Added
- Added the `data/overrides/` grounding override layer beside `data/cache/` with the same per-id structure. Resolution is `(cache or live API) then overrides`: `formal_ai::seed::resolve` decorates a cached external-source record with an override's facts, and every override records why it exists in a `reason` line.
- Added a tree-walking CI suite (`tests/unit/overrides.rs`) that fails when an override references an id with no checked-in cache record, omits its reason, carries no facts, or is redundant (repeats a value the cache already holds), so the layer self-prunes once upstream catches up.

### Changed
- Recorded the issue #398 PR review data-quality standards in `REQUIREMENTS.md` (R278-R283) under the governance rule "latest requirement overrides any earlier one", mapping each CI check to its requirement.

### Changed
- Made the JSON ↔ Links Notation cache codec (`formal_ai::json_lino`) losslessly round-trip the *entire* Wikidata/Wiktionary snapshot — `forms`, `senses`, `claims`, and every metadata key — instead of the previous lexeme-only projection. `data/cache/**/*.lino` were regenerated as full native snapshots (e.g. `L3412` 6 → 195 lines), empty arrays/objects/nulls are never emitted, and the Wiktionary source JSON is pretty-printed multi-line.
- Replaced the circular round-trip test (which compared the lino to the converter's own lossy output) with one that rebuilds the full original JSON from the lino and asserts key-for-key equality with the raw `.json` (`wikidata_lino_cache_rebuilds_full_json_losslessly`, `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`, and the `verify_cache_roundtrip` example).
- Migrated every meaning header in `data/seed/**/*.lino` from the YAML-style trailing-colon form (`monday:`) to native Links Notation nodes (`monday`), removing all 428 empty colon redefinition fields tree-wide. The transform is parse-equivalent (`parse_colon_definition` already mapped `monday:` to `(name = "monday", id = "")`) and regenerates the embedded browser-worker seed. `seed_lino_files_have_no_empty_redefinition_fields` now enforces the reviewer's exact `^\s*[\w-]+:\s*$` regex.

### Added
- Added `scripts/migrate-empty-redefinition-fields.rs`, a re-runnable whole-tree migration that strips trailing-colon redefinition headers and refreshes the `src/web/formal_ai_worker.js` embed.

### Added
- Added `scripts/ground-meanings.rs`, a re-runnable, self-verifying Wikidata grounding pipeline (issue #398, defect #3). For each curated `(slug, id, expected-label-token)` it fetches `Special:EntityData/<id>.json`, trims it to the cache convention (`type`/`id`/`labels`/`descriptions`/`aliases` in en/ru/hi/zh, wrapped in `{entities:{…}, success:1}`), **verifies** the entity's labels actually contain the expected concept token before grounding — refusing wrong ids such as `Q206` ("Stephen Harper", not "seven") — writes the lossless `.lino` snapshot, and inserts `grounded-in <id>` into the meaning block idempotently. Ids are sharded by kind, so items land under `entity/`, properties under `property/`, and lexemes under `lexeme/`.
- Grounded 114 common-vocabulary meanings to verified Wikidata items, raising grounded-meaning coverage from 18 to 131 `grounded-in` anchors (30.6% of the 428 seed meanings). Coverage now spans calendar weekdays, days, dates and weeks; arithmetic operations and mathematical functions (including `cosine`, `tangent`, `modulo`); cardinal numbers 0–10; currencies and exchange rate; length/mass/time/temperature/data-size units and their physical dimensions; unit conversion; the 11 catalogued programming languages; the four supported natural languages; the concrete-noun translation vocabulary (`apple`, `bread`, `water`, …) plus `translate` and `synonym`; the lexical-meta concepts (`noun`, `part_of_speech`, `noun_phrase`, `grammatical form`, `word sense`); finance concepts (`investment`, `interest`, `compound interest`, `year`); and core quantities and the `physical constant`. Fact relations ground to Wikidata **properties** — `capital` → `P36`, `population` → `P1082`, `continent` → `P30`, `currency` → `P38`, `official_language` → `P37`, `author_of_book` → `P50`, `painter_of_painting` → `P170`, `built_year` → `P571` — rather than to items. Every id's source snapshot is checked in under `data/cache/wikidata/`.
- Added the `grounded_meaning_coverage_does_not_regress` data test: a monotonic ratchet that records the grounded-meaning floor (131) so grounding is append-only and progress toward full grounding can only increase.

### Added
- Added `scripts/ground-lexemes.py`, a re-runnable, self-verifying Wikidata **lexeme** grounding pipeline (issue #398, defect #6 / CI check 6). For each curated `(slug, lexeme-id, expected-lemma, sense-id)` it fetches the full `Special:EntityData/<L-id>.json` (lexemes are cached untrimmed so claims, forms and senses survive), caches it pretty-printed under `data/cache/wikidata/lexeme/`, generates the lossless `.lino` snapshot, **verifies** the entry really is the expected English noun (lemma match, `language` Q1860, `lexicalCategory` Q1084, named sense present) — refusing on any mismatch — and rewrites the meaning's plain `lexeme en` surface into the rich `source-lexeme` notation, sourcing the part of speech (`lexical-category`), every form (`form` + grammatical `feature`) and the grounded `sense` directly from the lexeme rather than hand-authoring them.
- Enriched the grounded concrete-noun vocabulary (`apple`, `water`, `bread`, `potato`, `tomato`) with full lexical detail sourced from Wikidata lexemes (`L3257`, `L3302`, `L3865`, `L3784`, `L7993`): each now records its part of speech and its singular/plural forms with grammatical features (`Q110786` singular, `Q146786` plural), and its English surface references a real lexeme form instead of a hand-typed string.
- Added the `lexical_completeness_does_not_regress` data test: a monotonic ratchet (floor 6) that records how many meanings expose a `source-lexeme` with its part of speech and at least one form + feature, and asserts every referenced lexeme resolves to a checked-in cache file. Lexical grounding is append-only, so coverage toward the defect #6 goal — every grounded word carrying its parts of speech and forms from the source — can only increase.

### Changed
- Replaced every pipe-packed multi-value in `data/seed/*.lino` with the canonical
  reference-list form `keyword ("a" "b c" d)`, so multi-values are real links
  instead of in-string separators (issue #398, defect #4). Covers
  `supported_languages`, `tasks`, `languages`, `inputs`, `outputs`, aliases, and
  every other former `"a|b|c"` field; `code` listings remain the sole field that
  may legitimately contain `|`.
- The LiNo reference-list tokenizer now decodes quoted scalars (which may contain
  spaces) across all four parsers (Rust `seed::parser`, `src/web/seed_loader.js`,
  the e2e `lino-seed-parser.mjs`, and migration tooling).

### Added
- `formal_ai::supported_languages()` accessor that reads the declared languages
  from the `agent-info.lino` reference list, replacing ad-hoc `split('|')` parsing
  scattered across the test suite.
- A comprehensive CI guard (`seed_lino_values_never_pipe_pack_multi_values`) that
  fails on *any* `|` in a seed value except the exempt `code` field, so pipe
  packing can never silently return.

### Added
- Grounded five more conversational/discourse meanings to verified Wikidata
  items (issue #398, defect #3): `greeting_hello` → `Q98815142` (the English
  salutation "hello"), `gratitude_thank_you` → `Q2728730` (gratitude),
  `affirmation_yes` → `Q6452715` (the affirmative particle "yes"), `example`
  → `Q14944328`, and `conjunction_or` → `Q1651704` (logical disjunction). Each
  id was confirmed by the `scripts/ground-meanings.rs` label-token verifier
  before grounding, and its trimmed source snapshot is checked in under
  `data/cache/wikidata/entity/`. The `grounded_meaning_coverage_does_not_regress`
  ratchet floor rises from 131 to 136 (31.8% of the 428 seed meanings).

### Added
- Grounded three core programming-artifact meanings to verified Wikidata items
  (issue #398, defect #3): `program` → `Q40056` (computer program),
  `code` → `Q128751` (source code), and `sort` → `Q2303697` (sorting, the
  action of arranging objects into order). Each id was confirmed by the
  `scripts/ground-meanings.rs` label-token verifier, with its trimmed source
  snapshot checked in under `data/cache/wikidata/entity/`. The
  `grounded_meaning_coverage_does_not_regress` ratchet floor rises from 136 to
  139 (32.5% of the 428 seed meanings).

### Added
- Grounded two more meanings to verified Wikidata items (issue #398, defect #3):
  `politeness` → `Q281287` (politeness, the application of good manners) and
  `calendar_today` → `Q3151690` (today, the current day). Each id was confirmed
  by the `scripts/ground-meanings.rs` label-token verifier, with its trimmed
  source snapshot checked in under `data/cache/wikidata/entity/`. The
  `grounded_meaning_coverage_does_not_regress` ratchet floor rises from 139 to
  141 (32.9% of the 428 seed meanings).

### Added
- Wiktionary grounding pipeline `scripts/ground-wiktionary.py` (issue #398, open
  item #1 of the `92a29b0` review): it **discovers** candidate lemmas from the
  data — every single-word English surface of a `grounded-in` meaning — fetches
  each from the Wiktionary-backed Free Dictionary API (CC BY-SA 3.0, the same
  source and schema as the existing `en/reference.json`), **verifies** the
  response actually describes the requested lemma, and caches it as pretty
  multi-line JSON plus the lossless `.lino` snapshot via the
  `wikidata_json_to_lino` codec. Idempotent and re-runnable.
- 155 verified Wiktionary entries under `data/cache/wikidata`'s sibling
  `data/cache/wiktionary/en/`, raising the cache from a single placeholder entry
  to 156. Each entry round-trips its full JSON through
  `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`.
- `wiktionary_cache_breadth_does_not_regress` ratchet (floor 156) so Wiktionary
  coverage is append-only and can only grow as more grounded surfaces are cached.

### Added
- **Total reference-closure at zero (issue #398, PR #399 review 4668929105).**
  Widened the closure gate from the `defined-by`/facet/role backbone to *every*
  non-keyword, non-quoted value token in `data/seed/**.lino`. `scripts/close-total.py`
  is an idempotent migration that defines each previously-dangling token as a
  first-class meaning — 17 parent category concepts (intent, task,
  prompt_pattern, source_kind, programming_language, …) rooted at `concept`,
  plus 508 member meanings parented under the category their predicate implies.
  `scripts/audit-total-closure.py` now reports **0** unresolved tokens.
- **Open English WordNet 2024 source.** `scripts/ground-wordnet.py` imports OEWN
  2024 offline (one download, no per-word network calls) and caches 312 English
  lemmas as `.json` + lossless `.lino` under `data/cache/wordnet/en/`, recorded
  under CC BY 4.0.
- **Multi-source `data/view/` merge layer.** `scripts/build-views.py` merges the
  WordNet and Wiktionary lexical caches into 536 per-lemma view entities, each
  with a deterministic `M-<sha1[:12]>` id, a `sources` list, and per-sense
  provenance. Senses sharing part-of-speech with gloss Jaccard ≥ 0.5 merge and
  keep both sources; others stay separate. `--check` verifies no drift, id
  determinism, and merge-threshold correctness.
- **`data/seed/sources-registry.lino`** enumerating every ingested source
  (Wikidata, Wiktionary, WordNet, Wikipedia) with its API endpoint, permissive
  license, and cache path.
- **`tests/unit/total_closure.rs` CI gates** that fail immediately if: any seed
  value token is unresolved (naming offenders); the seed collapses below
  hundreds of meanings; the WordNet cache is absent; `sources-registry.lino`
  omits an ingested source, API, or license; `data/view/` is missing, drifted,
  non-deterministic, or has a provenance-less field; or no view entity is
  genuinely multi-source.

### Added

- `meaning_definition_references_resolve_to_defined_meanings` CI gate proving
  the meaning graph is fully reference-closed: every `defined-by` target and
  every semantic-facet reference (notation/annotation/denotation/connotation)
  across all 35 `data/seed/meanings*.lino` files resolves to a meaning defined
  exactly once (issue #398, PR #399 review point #1).
- `data/seed/roles.lino` canonical reserved-role registry declaring all 207
  distinct `role` values exactly once, each classified as `kind meaning` (also
  a defined meaning slug) or `kind predicate` (role-only identifier).
- `scripts/generate-role-registry.py` to regenerate the role registry
  deterministically and idempotently from the meaning seed.
- `every_role_value_is_declared_in_the_registry` and
  `role_registry_is_in_lockstep_with_usage` tests keeping the registry and its
  usage in lockstep (PR #399 review point #2).
- `experiments/closure_audit.py` documenting the meaning-layer closure
  measurement.
