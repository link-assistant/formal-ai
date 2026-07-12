# Roadmap: Requirement Status Toward The Vision

This file is the single source of truth for how much of `VISION.md` is
actually built. It tracks **general requirements** — not issues — and gives
each one a status. Issues come and go; the requirement stays on this list
until it is fully done. Restructured on 2026-07-12 for issue
[#651](https://github.com/link-assistant/formal-ai/issues/651), whose case
study ([`docs/case-studies/issue-651/`](docs/case-studies/issue-651/README.md))
holds the audits behind every status below.

It complements the existing docs rather than replacing them:

- `VISION.md`, `GOALS.md`, and `NON-GOALS.md` describe what we are building
  and why.
- `REQUIREMENTS.md` is the fine-grained per-issue requirement matrix
  (R-rows).
- `ARCHITECTURE.md` describes how the implemented pipeline is wired.
- `ROADMAP.md` (this file) tracks requirement status and points every
  partially-done or not-done requirement at its open issue.

Status legend (per issue #651's contract):

- **Done** — implemented and covered by active tests.
- **Partially done** — a useful implementation exists, but a named part of
  the requirement remains; the open issue that owns the remainder is linked.
- **Not done** — not implemented beyond documentation or planning; the open
  issue that owns it is linked.

**Invariant:** no requirement may sit at *Partially done* or *Not done*
without an open issue linked in its row. Deferral into prose (a
"remains a follow-up" note in a doc) is not tracking — that anti-pattern is
documented in
[`raw-data/incomplete-work-audit.md`](docs/case-studies/issue-651/raw-data/incomplete-work-audit.md)
and is what the 2026-07-12 restructure eliminated.

## 1. Associative Core

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| One doublet-links store is the single source of truth; `doublets-rs` is the default native physical store | Done | `src/link_store.rs`, `src/links_format.rs`, active `links_network` specs |
| Links Notation is the universal data surface; `data/seed/*.lino` is the canonical knowledge the system boots from | Done | `src/seed.rs`, seed specification tests |
| Links-network invariants and the dynamic type system hold under all operations | Done | active `links_network` specs |
| The meta language represents algorithms as data (recipes, method registry, rebuild plans) | Done | `data/meta/*-recipe.lino`, the #559 method registry, drift-guard tests |
| The system's own source is represented associatively (CST/AST census) across the whole workspace, not one pinned module | Partially done | single-module `data/meta/self-ast.lino`; workspace census: [#673](https://github.com/link-assistant/formal-ai/issues/673) |
| Only associative concepts in the public surface — links networks, never graphs or tables (naming included) | Partially done | architecture complies (audit §4); naming debt (`/v1/graph`, `*_source_graph.rs`, UI strings): [#664](https://github.com/link-assistant/formal-ai/issues/664) |
| Behavior lives in seed data, not code constants (routing, vocabulary, precedence) | Partially done | `data/seed/intent-routing.lino`, `operation-vocabulary.lino`; `SPECIALIZED_HANDLERS` precedence remnant: [#663](https://github.com/link-assistant/formal-ai/issues/663) |
| No hardcoded user-facing natural language in `src/`; all surface text flows from the seeded lexicon | Partially done | design in `docs/design/no-hardcoded-natural-language.md`; enforcement lint + burn-down: [#659](https://github.com/link-assistant/formal-ai/issues/659) |

## 2. Universal Solver And Reasoning

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| The universal 11-step problem-solving loop runs for every prompt in the same shape | Done | `src/solver.rs::UniversalSolver`, active `reasoning_loop` specs |
| Every message is formalized into a Links-Notation intent; routing follows the formalized intent, not a fixed catalogue | Done | `src/intent_formalization.rs`, `intent_formalization` specs (precedence remnant tracked in §1) |
| Reasoning under unknowns instead of a canned fallback; at most one minimal question | Done | `src/solver_unknown_reasoning.rs`, `unknown_reasoning` specs |
| Formal decision procedures beyond a fixed answer table | Done | `src/proof_engine/decision.rs` |
| General synthesis derives answers by composing decomposed sub-results (no seeded answers) | Done | benchmark suite 10/10 with anti-memorization guards |
| Symbolic probabilistic ranking with temperature-based interpretation selection | Done | `src/probability.rs`, `src/translation/selection.rs` |
| Random and evolutionary search under an explicit compute budget when no reusable part exists | Not done | [#662](https://github.com/link-assistant/formal-ai/issues/662) |
| Every formalized statement carries an inspectable probability weight; contradictory requirements are detected and surfaced with proposed resolutions | Not done | [#661](https://github.com/link-assistant/formal-ai/issues/661) |
| Tasks split recursively by the principle of least action (balanced two-part decomposition) | Not done | [#491](https://github.com/link-assistant/formal-ai/issues/491) |
| Pattern inference via associative deduplication (sequence converters, transformations, ontology seeding) | Not done | [#531](https://github.com/link-assistant/formal-ai/issues/531) |
| Per-dialog world models (current + target state) as links networks with context merge/split | Not done | [#649](https://github.com/link-assistant/formal-ai/issues/649) |

## 3. Knowledge, Translation, And Language Equality

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| Formalization grounds to Wikidata P-ids/Q-ids with fallback sources; public knowledge is a provenance-carrying cache | Done | `src/translation/formalization.rs`, `src/knowledge.rs`, `source_cache` specs |
| Translation flows through link-native meanings; round-trip survival is the regression contract (en/ru/hi/zh) | Done | `src/translation/`, issue-#526 `translation_round_trip` matrix |
| Reasoning handlers trigger equally in every supported language via one shared data-driven vocabulary | Done | `data/seed/operation-vocabulary.lino` (E33) |
| Detailed word semantics: grammatical number, part of speech, bidirectional word ⇄ meaning denotation, Wikidata-grounded | Done for the curated set | issue-#538 rows R370–R377, `tests/unit/issue_538.rs` |
| Semantics grow by bulk import from external lexical sources, not per-word engineering (includes grounding breadth R282, source-response importers R271, and the issue-#1 corpus-import jobs) | Not done | [#660](https://github.com/link-assistant/formal-ai/issues/660) |

## 4. Learning And Self-Improvement

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| Append-only event-log memory with retraction, "why" queries, and single-file bundle export/import | Done | `src/event_log.rs`, `transparent_state` specs, `memory::export_full_memory` |
| Proposal-generating loops: dreaming, self-healing, seed-rule and meta-algorithm proposals (human-gated) | Done | `src/self_improvement.rs`, `src/meta_self_improvement.rs`, `src/self_healing.rs`, `src/dreaming*.rs` |
| Five rule shapes ending in compiled natural-language skills (typed + bounded multi-step) | Done | `src/skill_compiler.rs` with native lowering |
| Arbitrary natural-language procedures compile into executable skills beyond the supported subset (`ARCHITECTURE.md` §16 open question, journey F2) | Partially done | [#674](https://github.com/link-assistant/formal-ai/issues/674) |
| Accepted proposals are promoted into seed data through a deterministic, benchmark-gated protocol (auto-learning; carries R385 forward after #558 closed) | Not done | [#656](https://github.com/link-assistant/formal-ai/issues/656) |

## 5. Self-Coding: The Project Builds Itself

The four-rung ladder from `VISION.md` ("Self-Coding: The Project Builds
Itself").

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| Formal AI serves as a real Agent CLI provider (`serve --agent-mode`, OpenAI-compatible tool calls, 13 default tools) | Done | Agent CLI built-in `formal-ai` provider; `test-agent-cli-e2e` CI job |
| Rung 1 — recipe-driven repository edits through the Agent CLI | Done | issues #538/#540 sessions, `src/agentic_coding/` |
| General agentic planning: plans composed from solver decomposition, not pinned `is_*_task` recipes (R388) | Not done | [#654](https://github.com/link-assistant/formal-ai/issues/654) |
| Rung 2 — Hive-Mind-dispatched end-to-end solve of a real repository issue (blocked by general planning) | Not done | [#655](https://github.com/link-assistant/formal-ai/issues/655) |
| Rung 3 — benchmark-gated promotion protocol (shared with §4) | Not done | [#656](https://github.com/link-assistant/formal-ai/issues/656) |
| Rung 4 — per-release self-hosting metric: the share of changes authored by Formal AI, ratcheting upward | Not done | [#657](https://github.com/link-assistant/formal-ai/issues/657) |

## 6. Interfaces, Protocols, And Product Surfaces

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| OpenAI Chat Completions + Responses, Anthropic Messages, and Gemini protocols with auth and tool-call gating | Done, with open defects | `src/protocol.rs`, `src/server.rs`; the four verified #647-era defects: [#650](https://github.com/link-assistant/formal-ai/issues/650) |
| Real third-party CLI clients (codex, opencode, gemini, qwen, claude, grok, aider) verified continuously in CI, not by prose guide | Not done | [#671](https://github.com/link-assistant/formal-ai/issues/671) |
| Visual links network beside chat, with trace links on every surface | Done | `src/web/app.js`, `/v1/graph` (rename tracked in §1), Telegram trace specs |
| Bounded chat autonomy plus explicit isolated agent mode | Done | agent isolation specs, API gating |
| Desktop application (Electron shell, packaged) | Done | `desktop/` |
| Interactive step-by-step debugging view: chat, data, diagram, and source panes over the live event log (R383) | Not done | [#667](https://github.com/link-assistant/formal-ai/issues/667) |
| Desktop/web UI polish wave: dark-theme snapshots, migration replay, animation budget, hierarchy editing, IPC mode-flip tests (issue-541 follow-ups) | Not done | [#672](https://github.com/link-assistant/formal-ai/issues/672); related UI asks: [#557](https://github.com/link-assistant/formal-ai/issues/557), [#447](https://github.com/link-assistant/formal-ai/issues/447) |
| Optional off-by-default small in-browser model as a formalization-match fallback (never at the steering wheel) | Not done | [#483](https://github.com/link-assistant/formal-ai/issues/483) |

## 7. Distribution And Reach

The `VISION.md` "Reaching A Wide Audience" section; reach must never fork
the core.

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| crates.io crate, Docker/GHCR images, GitHub Pages demo, Telegram bot | Done | release pipeline publishes on every version bump |
| The engine compiles to WebAssembly with JavaScript reserved for UI/glue — the full solver, not a subset (R380) | Partially done | `no_std` WASM cores exist; ~26,700 lines of mirrored JS solver logic remain: [#658](https://github.com/link-assistant/formal-ai/issues/658) |
| Installable, offline-capable PWA and an embeddable npm engine package (blocked by WASM absorption) | Not done | [#665](https://github.com/link-assistant/formal-ai/issues/665) |
| VS Code extension published on the Marketplace and Open VSX | Not done | [#666](https://github.com/link-assistant/formal-ai/issues/666) |
| Shareable associative packages: export/import datasets, skills, rules, and handlers with permission review (journey F6) | Not done | [#668](https://github.com/link-assistant/formal-ai/issues/668) |
| Cloud sync of the single-file bundle through user-owned backends, opt-in only (journey F3) | Not done | [#669](https://github.com/link-assistant/formal-ai/issues/669) |
| More languages executable in the browser (WebVM/WASM runtimes) with honest execution reporting (journey F5) | Not done | [#670](https://github.com/link-assistant/formal-ai/issues/670) |

## 8. Quality, Determinism, And Honesty

| Requirement | Status | Evidence / open issue |
| --- | --- | --- |
| Determinism contract: same input + same config ⇒ same output, everywhere | Done | impulse-hash seeding, parity fixtures |
| Benchmark suites with rising ratchet floors (industry 10/10, coding-modification, text-manipulation 1440, procedural-howto) | Done | `data/benchmarks/*.lino`, `minimum_pass_count` gates |
| Honest reporting: compiled/ran/not-run stated per environment; unknown answers say so with evidence | Done | execution-metadata reporting, `unknown_reasoning` specs |
| Release automation: changelog fragments, automated version bumps, multi-artifact publish | Done | `changelog.d/`, `scripts/*.rs` release pipeline |
| Every partially-done or not-done requirement has an open tracking issue (no deferral into prose) | Done as of 2026-07-12 | this restructure; enforced by the invariant above |
| Reported failures get root-caused, not worked around | Open items | [#534](https://github.com/link-assistant/formal-ai/issues/534) (disk usage), [#482](https://github.com/link-assistant/formal-ai/issues/482) (Nemotron sample tests), [#453](https://github.com/link-assistant/formal-ai/issues/453) (moonshot decomposition) |

## Current Planning Batch (Issue #651 — E35-E55)

Issue [#651](https://github.com/link-assistant/formal-ai/issues/651) filed 21
issues on 2026-07-12, all sub-issues of #651 with blocked-by relations
encoding execution order (#655←#654; #657←#655,#656; #665←#658; #667←#666).
Full bodies with acceptance criteria:
[`docs/case-studies/issue-651/proposed-issues.md`](docs/case-studies/issue-651/proposed-issues.md).

| Epic | Issue | Track |
| --- | --- | --- |
| E35 general agentic planning | [#654](https://github.com/link-assistant/formal-ai/issues/654) | self-coding (foundation) |
| E36 Hive-Mind end-to-end solve | [#655](https://github.com/link-assistant/formal-ai/issues/655) | self-coding |
| E37 promotion protocol | [#656](https://github.com/link-assistant/formal-ai/issues/656) | self-coding |
| E38 self-hosting metric | [#657](https://github.com/link-assistant/formal-ai/issues/657) | self-coding |
| E39 WASM worker absorption | [#658](https://github.com/link-assistant/formal-ai/issues/658) | core (foundation) |
| E40 hardcoded-language lint | [#659](https://github.com/link-assistant/formal-ai/issues/659) | core |
| E41 bulk semantics importer | [#660](https://github.com/link-assistant/formal-ai/issues/660) | core |
| E42 weighted formalization + contradictions | [#661](https://github.com/link-assistant/formal-ai/issues/661) | core |
| E43 budget-driven search | [#662](https://github.com/link-assistant/formal-ai/issues/662) | core |
| E44 data-driven handler precedence | [#663](https://github.com/link-assistant/formal-ai/issues/663) | core |
| E45 associative terminology cleanup | [#664](https://github.com/link-assistant/formal-ai/issues/664) | associative purity |
| E46 offline PWA + npm engine | [#665](https://github.com/link-assistant/formal-ai/issues/665) | distribution |
| E47 publish VS Code extension | [#666](https://github.com/link-assistant/formal-ai/issues/666) | distribution |
| E48 interactive debugging view | [#667](https://github.com/link-assistant/formal-ai/issues/667) | distribution |
| E49 shareable packages | [#668](https://github.com/link-assistant/formal-ai/issues/668) | distribution |
| E50 cloud memory sync | [#669](https://github.com/link-assistant/formal-ai/issues/669) | distribution |
| E51 WebVM experiment | [#670](https://github.com/link-assistant/formal-ai/issues/670) | distribution |
| E52 multi-CLI CI matrix | [#671](https://github.com/link-assistant/formal-ai/issues/671) | core |
| E53 issue-541 UI follow-ups | [#672](https://github.com/link-assistant/formal-ai/issues/672) | distribution |
| E54 workspace self-AST census | [#673](https://github.com/link-assistant/formal-ai/issues/673) | self-coding |
| E55 arbitrary NL skill compilation | [#674](https://github.com/link-assistant/formal-ai/issues/674) | core |

## Planning History

Condensed; each batch's full audit lives in its case study.

- **E1–E14, E15–E20, E21–E27, E28–E32, E33–E34** (issue
  [#244](https://github.com/link-assistant/formal-ai/issues/244), 2026-05-26
  to 2026-05-29): storage boundary, universal loop, formalization,
  reasoning-under-unknowns, general synthesis (benchmarks 10/10 with
  ratchet), and cross-language/cross-runtime parity. All closed and merged.
  Details: [`docs/case-studies/issue-244/`](docs/case-studies/issue-244/proposed-issues.md).
- **#349 reverse-sort roadmap** (issues #355–#365, 2026-05-31): multi-turn
  Russian coding dialog fixed via composable program modifiers and rule
  construction; closure evidence in
  [`docs/case-studies/issue-365/README.md`](docs/case-studies/issue-365/README.md).
- **#408 text/code editing** (PR #416): deterministic symbolic edit path;
  1,440/1,440 local profile ratchet.
- **#511 Agent CLI integration wave** (issues #513–#520, 2026-06): provider
  seam, permission UI, server auto-start, container, chat rendering.
- **#526 translation quality** (PR #635): round-trip survival as the
  regression contract; [`docs/case-studies/issue-526/`](docs/case-studies/issue-526/README.md).
- **#538 detailed meanings** (PR #601) and **#540 recipe-driven edits**:
  enriched word semantics for the curated set and the first Agent-CLI
  self-edit sessions; spawned R378–R386, now tracked by the #651 batch
  above.
- **#558/#559 self-representation wave** (PRs #637 and the method
  registry): human-gated proposal loops, method registry with drift guard,
  single-module self-AST; the auto-learning remainder is carried by
  [#656](https://github.com/link-assistant/formal-ai/issues/656).
- **#606–#650 protocol wave** (2026-07): OpenAI Responses, Anthropic
  Messages, and Gemini protocol support with real-CLI verification; the
  verified defect batch is open as
  [#650](https://github.com/link-assistant/formal-ai/issues/650).
- **#651 restructure** (2026-07-12, this file's current shape): roadmap
  converted from issue-batch chronicle to general-requirement status
  tracking; every partial requirement got an issue (E35–E55). Audits:
  [`docs/case-studies/issue-651/`](docs/case-studies/issue-651/README.md).

## Verification Contract

When any roadmap item changes, the PR should update the corresponding rows
in `REQUIREMENTS.md`, the architecture status table, and this file. If the
change closes a follow-up issue, move the requirement's status here (and
remove the issue link) instead of leaving stale future-work wording behind.
A requirement may never sit at *Partially done* or *Not done* without an
open linked issue.
