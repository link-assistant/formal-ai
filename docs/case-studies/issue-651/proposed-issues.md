# Proposed Issues For Issue #651

Issue [#651](https://github.com/link-assistant/formal-ai/issues/651) asks for
the most critical missing features that fully fulfill the vision and roadmap,
each with maximum detail on what to do and how to verify it. This file is the
full text of every issue created for that plan. Epic numbering continues the
E1–E34 sequence from
[`docs/case-studies/issue-244/proposed-issues.md`](../issue-244/proposed-issues.md).

Every created issue is registered as a **sub-issue of #651**, and execution
order is encoded as GitHub **blocked-by** relationships:

- [#655](https://github.com/link-assistant/formal-ai/issues/655) E36
  (Hive-Mind end-to-end solve) is blocked by #654 E35 (general agentic
  planning).
- [#657](https://github.com/link-assistant/formal-ai/issues/657) E38
  (self-hosting metric) is blocked by #655 E36 and #656 E37 (promotion
  protocol).
- [#665](https://github.com/link-assistant/formal-ai/issues/665) E46
  (PWA + npm engine package) is blocked by #658 E39 (Rust→WASM worker
  absorption).
- [#667](https://github.com/link-assistant/formal-ai/issues/667) E48
  (interactive debugging view) is blocked by #666 E47 (published VS Code
  extension).

Filed issues (2026-07-12):

| Epic | Issue | Epic | Issue |
|------|-------|------|-------|
| E35 | [#654](https://github.com/link-assistant/formal-ai/issues/654) | E45 | [#664](https://github.com/link-assistant/formal-ai/issues/664) |
| E36 | [#655](https://github.com/link-assistant/formal-ai/issues/655) | E46 | [#665](https://github.com/link-assistant/formal-ai/issues/665) |
| E37 | [#656](https://github.com/link-assistant/formal-ai/issues/656) | E47 | [#666](https://github.com/link-assistant/formal-ai/issues/666) |
| E38 | [#657](https://github.com/link-assistant/formal-ai/issues/657) | E48 | [#667](https://github.com/link-assistant/formal-ai/issues/667) |
| E39 | [#658](https://github.com/link-assistant/formal-ai/issues/658) | E49 | [#668](https://github.com/link-assistant/formal-ai/issues/668) |
| E40 | [#659](https://github.com/link-assistant/formal-ai/issues/659) | E50 | [#669](https://github.com/link-assistant/formal-ai/issues/669) |
| E41 | [#660](https://github.com/link-assistant/formal-ai/issues/660) | E51 | [#670](https://github.com/link-assistant/formal-ai/issues/670) |
| E42 | [#661](https://github.com/link-assistant/formal-ai/issues/661) | E52 | [#671](https://github.com/link-assistant/formal-ai/issues/671) |
| E43 | [#662](https://github.com/link-assistant/formal-ai/issues/662) | E53 | [#672](https://github.com/link-assistant/formal-ai/issues/672) |
| E44 | [#663](https://github.com/link-assistant/formal-ai/issues/663) | E54 | [#673](https://github.com/link-assistant/formal-ai/issues/673) |
| — | — | E55 | [#674](https://github.com/link-assistant/formal-ai/issues/674) |

Tracks: **S** self-coding (the project builds itself), **C** core
completeness (partially-done requirements without an issue), **D**
distribution (reach a wide audience), **A** associative purity.

Sources: [`code-audit.md`](code-audit.md) (repository state vs vision),
[`online-research.md`](online-research.md) (Agent CLI / Hive Mind / prior
art), and [`raw-data/incomplete-work-audit.md`](raw-data/incomplete-work-audit.md)
(the issue-history audit of deferred and ignored work — E52–E54 come
directly from it).

---

## E35 (S) — Generalize the agentic planner beyond pinned task recipes — FOUNDATION

**Problem**

`src/agentic_coding/planner.rs` recognizes tasks through explicit `is_*_task`
recognizers pinned to `*_TASK` constants (formalize, diagram, self-AST,
meaning detail, file read, …). `docs/case-studies/issue-558/pr-601-gap-analysis.md`
(REQUIREMENTS R388) records this "recipe-driven Agent CLI boundary" as the
reason PR #601 is not a complete auto-learning system. As long as the planner
only handles encoded recipes, Formal AI cannot take an *arbitrary* small
repository issue end to end, which blocks the self-coding ladder in
`VISION.md` ("Self-Coding: The Project Builds Itself", rung 2).

**Approach**

1. Introduce a general change-request plan shape in the meta language: a
   `.lino` document with `goal`, ordered `steps`, per-step `capability`
   (`Search`/`Fetch`/`Read`/`Write`/`Run`), expected evidence, and a
   verification command — a data generalization of what each pinned recipe
   already encodes, building on `src/agentic_coding/change_request.rs` and
   `data/meta/*-recipe.lino`.
2. Make the planner *compose* plans from the requirement decomposition the
   universal solver already produces (steps 2–5 of the 11-step loop), instead
   of matching one `is_*_task` recognizer: formalized requirement →
   sub-requirements → capability-tagged steps.
3. Keep determinism: same issue text + same seed ⇒ same plan; the plan itself
   is appended to the event log before execution.
4. Keep the existing recipes as regression fixtures: each pinned recipe must
   be reproducible by the general planner (the recipe becomes a stored plan,
   not a code path).
5. Prove generality per CONTRIBUTING: at least three differently-phrased
   change requests (en/ru at minimum) that no `*_TASK` constant mentions must
   produce executable plans.

**Existing components**

- `src/agentic_coding/{planner,driver,change_request,rebuild_plan}.rs` — the
  capability classification, tool-call loop, and bounded plan shapes.
- `src/solver.rs` decomposition + `src/intent_formalization.rs` — the
  requirement decomposition to reuse.
- `data/meta/recursive-core-recipe.lino`, `data/meta/dreaming-recipe.lino` —
  the algorithm-as-data precedent.

**Acceptance criteria**

- `cargo test agentic_general_planner` — new unit suite: a change request not
  matched by any `is_*_task` recognizer yields a multi-step plan whose steps
  carry capabilities and verification commands; same input twice ⇒ identical
  plan.
- Every existing recipe test in `tests/unit/agentic_coding.rs` and
  `tests/unit/issue_538_agentic.rs` stays green with the general planner in
  front.
- A new fixture `data/meta/general-change-plan.lino` documents the plan shape
  and is pinned by a specification test.
- The `test-agent-cli-e2e` job in `.github/workflows/release.yml` gains one
  general (non-recipe) task and stays green.

---

## E36 (S) — Hive-Mind-dispatched end-to-end issue solve by Formal AI — blocked by E35

**Problem**

The self-coding rung 2 in `VISION.md` requires a real loop: Hive Mind
(`solve <issue-url> --tool agent --model formal-ai`) drives the Agent CLI,
which drives `formal-ai serve --agent-mode`, and the session takes a small,
well-specified repository issue from plan to draft PR. Today every piece
exists separately — Agent CLI ships Formal AI as a built-in provider
(`docs/formal-ai.md` in link-assistant/agent; base URL
`http://127.0.0.1:8080/api/openai/v1`), Hive Mind passes `--model formal-ai`
through unchanged (`src/agent.lib.mjs`, `agentModels[model] || model`) — but
no recorded end-to-end run exists, so the claim is unverified.

**Approach**

1. Author a scripted scenario under `examples/self-coding/`: a scratch
   repository with one small issue (e.g. add a missing plural surface to a
   seed meaning — the issue #538 atomic-edit shape), a `formal-ai serve
   --agent-mode` instance, the Agent CLI, and the Hive Mind `solve` entry.
2. Capture the full session: Hive Mind logs, Agent CLI `--output-format
   stream-json` events, the Formal AI event log, and the resulting branch
   diff, stored under `docs/case-studies/issue-651/self-coding-run/`.
3. Add a CI job (or extend `test-agent-cli-e2e`) that replays the bounded
   inner loop (Agent CLI ↔ Formal AI, no GitHub access) deterministically.
4. Report upstream gaps discovered along the way as issues on
   link-assistant/agent and link-assistant/hive-mind (e.g. adding `formal-ai`
   to `agentModels` in `src/models/index.mjs`).
5. Document the recipe in `CONTRIBUTING.md` as the standard self-coding
   verification path.

**Existing components**

- `formal-ai serve --agent-mode` + `FORMAL_AI_API_BEARER_TOKEN`;
  `GET /health`, `GET /api/openai/v1/models`.
- Agent CLI provider selectors `formal-ai`, `formalai/formal-ai`; env
  `FORMAL_AI_API_KEY`, `FORMAL_AI_BASE_URL`.
- Hive Mind `solve.mjs` / `src/agent.lib.mjs` (validates with
  `printf "hi" | agent --model <m>`).
- `scripts/reproduce-issue-538.sh` and the session JSONs under
  `docs/case-studies/issue-538/` — the bounded replay precedent.

**Acceptance criteria**

- A committed, replayable session under
  `docs/case-studies/issue-651/self-coding-run/` shows issue text in → draft
  branch diff out, with every step present in the Formal AI event log.
- `cargo test self_coding_session_replays` (or an e2e script in CI) replays
  the inner Agent-CLI ↔ Formal-AI exchange byte-for-byte offline.
- The scratch-issue diff passes the scenario's own verification command
  (its `cargo test` / seed parity check).
- `CONTRIBUTING.md` documents the end-to-end command sequence.

---

## E37 (S) — Benchmark-gated promotion protocol for self-improvement proposals

**Problem**

Every self-improvement loop in the codebase is proposal-only by design:
`src/self_improvement.rs` proposes seed rules but never writes `data/seed/`,
`src/meta_self_improvement.rs` defaults to `Off`, `src/self_healing.rs`
produces human-gated `RepairCase`s, and dreaming amendments live only in
memory events. The vision's self-coding rung 3 needs an explicit,
deterministic **promotion protocol**: a proposal that passes its benchmark
ratchets and CI may be promoted into seed data automatically, while draft PRs
and human review remain the outer gate. Without it, learning cannot compound.

This issue is also the missing tracker for R385: REQUIREMENTS.md still says
arbitrary auto-learning is "tracked by issue #558", but #558 was closed by
the deliberately human-gated PR #637, so the capability currently has no
open tracker at all (incomplete-work audit, item 8).

**Approach**

1. Define a `promotion` event protocol in the meta language: proposal link →
   benchmark evidence links (which ratchets ran, at what floor) → promotion
   decision → applied change, all appended to the event log.
2. Implement `formal-ai improve --promote` (dry-run by default, `--apply`
   with confirmation like `memory dream --apply --confirm`): collects open
   proposals, replays their gates (coding-modification suite, industry
   suite, unit specs), and materializes accepted ones as `.lino` seed edits
   on a branch — never a direct push.
3. Rejected proposals persist with the failing evidence (the R425
   `dreaming_candidate_failure` pattern).
4. Wire the branch/PR step through the same Agent-CLI path E36 exercises, so
   a promotion lands as an ordinary reviewed pull request with a changelog
   fragment.

**Existing components**

- `src/self_improvement.rs`, `src/meta_self_improvement.rs`,
  `src/self_healing.rs`, `src/dreaming.rs::MetaAlgorithmAmendment`.
- Ratchets: `data/benchmarks/*.lino` `minimum_pass_count` floors.
- Destructive-action gates: `require_destructive_confirmation`,
  `write_full_memory_backup`.

**Acceptance criteria**

- `cargo test promotion_protocol` — a synthetic proposal that passes its
  gates is materialized as a seed edit in a temp workspace; one that fails a
  ratchet is preserved as a failure record and **not** applied.
- Promotion events round-trip through the bundle export/import.
- `formal-ai improve --promote` (dry run) prints the plan without touching
  files; `--apply` without `--confirm` refuses.
- Documentation: `docs/meta-algorithm.md` gains a promotion section pinned by
  a traceability test.

---

## E38 (S) — Self-hosting metric: measure the share of each release authored by Formal AI — blocked by E36, E37

**Problem**

The vision's self-coding rung 4 says each release should report what share of
its changes was authored by Formal AI itself, and that share should ratchet
upward (the discipline the benchmark suites already apply to solving
ability). No such measurement exists. Prior art (Aider's per-release
self-written percentage; SICA's benchmark-gated self-editing) shows the
metric is both computable and motivating.

**Approach**

1. Define authorship attribution: commits whose recorded session evidence
   (E36 self-coding runs, E37 promotions) links them to a Formal AI session
   count as self-authored; measure in changed lines per release window.
2. Implement `scripts/self-hosting-metric.rs` (rust-script, like the other
   release scripts) that reads the git history between release tags plus the
   committed session ledgers and emits the percentage.
3. Publish the number in release notes via the existing
   `create-github-release.rs` step, and record it as a `.lino` ledger row so
   the trend itself is links data.
4. Start honest: 0% is an acceptable first value; the ratchet is monotonic
   non-decreasing over a trailing window, not a hard floor.

**Existing components**

- `scripts/{get-bump-type,version-and-commit,create-github-release}.rs` —
  the release pipeline to extend.
- `docs/case-studies/issue-538/agent-cli-session*.json` — the session-ledger
  precedent for attribution evidence.

**Acceptance criteria**

- `rust-script scripts/self-hosting-metric.rs --since <tag>` prints a
  deterministic percentage from committed data (covered by a unit test with
  a fixture repo/ledger).
- The release workflow emits the metric into the GitHub release body.
- A `data/meta/self-hosting-ledger.lino` row is appended per release and
  pinned by a specification test.

---

## E39 (C) — Absorb the remaining JS worker logic into the Rust→WASM worker (R380) — FOUNDATION

**Problem**

The "Rust-to-WebAssembly parity with JavaScript reserved for UI/glue" pillar
is only behaviorally satisfied. The `no_std` WASM crate
(`src/web/wasm-worker/src/lib.rs`) covers language detection, arithmetic, and
search cores, while ~26,700 lines of solver logic still live in JavaScript
across `src/web/worker/formal_ai_worker_00..21.js`. REQUIREMENTS R380 marks
this Partial. Every JS-only line is a parity risk (mirror drift), a double
maintenance cost, and a blocker for shipping the engine as one reusable WASM
package (E46).

**Approach**

1. Inventory `src/web/worker/*.js` by capability and map each section to its
   Rust counterpart (most have one — the JS is a mirror, not unique logic).
2. Migrate in slices behind the existing parity fixtures
   (`data/parity/cross-runtime-synthesis.json` and the e2e suites): move one
   capability into the WASM crate, delete the JS mirror, keep the fixture
   green, repeat. Start with the highest-drift areas (synthesis, program
   modifiers, text manipulation).
3. Where `no_std` blocks a dependency, promote the WASM crate to `std` with
   `wasm32-unknown-unknown` (the demo already loads a `.wasm` asset; size
   budget enforced by a CI check).
4. End state: `src/web/worker/*.js` contains only UI/glue (message plumbing,
   seed fetching, IndexedDB) — enforced by a line-count/content lint so the
   mirror cannot silently regrow.

**Existing components**

- `src/web_engine_core.rs`, `src/web/wasm-worker/` — the existing bridge.
- `scripts/sync-seed.sh`, `bun run build:web` — the web build pipeline.
- Parity tests: `shared_cross_runtime_synthesis_fixture_matches_rust_solver`,
  `tests/e2e/tests/issue-327.spec.js`,
  `experiments/issue-361-cross-runtime-parity.mjs`.

**Acceptance criteria**

- `src/web/worker/*.js` drops below an agreed UI-glue budget (target:
  ≤ 3,000 lines total), enforced by a CI script.
- All parity fixtures and Playwright e2e suites pass against the WASM-backed
  worker.
- The GitHub Pages demo works offline-identically (no behavior diff on the
  demo dialog set).
- A `wasm-worker` size check in CI keeps the shipped `.wasm` under an agreed
  budget.

---

## E40 (C) — CI lint burning down hardcoded natural-language strings (R379)

**Problem**

"Data is the interface" requires user-facing natural language to come from
the seeded lexicon, not string literals in `src/`. The design exists
(`docs/design/no-hardcoded-natural-language.md`), REQUIREMENTS R379 is
Partial, and there is no enforcement — new literals can land unnoticed.

**Approach**

1. Write `scripts/check-hardcoded-language.rs` (rust-script, like
   `check-file-size.rs`): scan `src/` for user-facing string literals
   (heuristics: sentences with spaces + terminal punctuation, multi-word
   phrases in `format!`/`push_str`/return positions), excluding trace/event
   kind tokens and code snippets.
2. Seed an explicit allowlist file with today's violations
   (`scripts/hardcoded-language-allowlist.txt`); the check fails on any
   literal not in the allowlist — new debt is blocked immediately.
3. Burn down the allowlist in follow-up slices: migrate each entry into
   grounded meanings in `data/seed/` and delete its allowlist row; the check
   also fails if the allowlist contains entries that no longer occur (keeps
   it honest).
4. Wire into CI next to the other repository-hygiene checks and into
   `CONTRIBUTING.md`'s local-checks list.

**Existing components**

- `docs/design/no-hardcoded-natural-language.md` — the rules.
- `data/seed/multilingual-responses.lino`, `operation-vocabulary.lino`,
  `meanings-lexical-meta.lino` — the migration targets.
- `scripts/check-file-size.rs`, `scripts/check-changelog-fragment.rs` — the
  rust-script CI-check pattern.

**Acceptance criteria**

- `rust-script scripts/check-hardcoded-language.rs` passes on the branch and
  fails when a test fixture introduces `"Sorry, I can't do that."` in `src/`.
- The allowlist is committed, sorted, and each row carries the file path.
- CI job added; `CONTRIBUTING.md` updated.
- At least one real allowlist entry is migrated to seed data in the same PR
  to prove the burn-down loop.

---

## E41 (C) — Bulk semantics importer from external lexical sources (R378)

**Problem**

Issue #538 produced richly-detailed meanings (grammatical number, part of
speech, bidirectional word ⇄ meaning links, Wikidata grounding) for a
curated set (tomato, potato) — by hand plus per-concept Agent-CLI recipes.
REQUIREMENTS R378 tracks the scale step: import such semantics **in bulk**
from external lexical sources so the network grows by data collection, not by
per-word engineering. The same pipeline is the natural home for two older
doc-only follow-ups: R282 (extend Wikidata grounding from the core meaning
set toward every seeded meaning) and the issue-#1-era REQUIREMENTS row about
chunked Wikipedia / Wikidata / Rosetta Code / Wikifunctions corpus import
jobs that was never scheduled.

**Approach**

1. Generalize `scripts/ground-meanings.rs` into `formal-ai import lexemes`:
   given a concept list (or a Wikidata SPARQL/lexeme dump slice), fetch
   lexeme forms per language (en/ru/hi/zh first), emit the enriched-surface
   `.lino` template the tomato block demonstrates, and write cache records
   under `data/cache/wikidata/`.
2. Respect the existing bounded-cache policy (`min(1%, 512)` per source) and
   `FORMAL_AI_LIVE_API` gating: committed snapshots make tests offline.
3. Validate on import: every generated surface must parse, denote its
   meaning, carry `grammatical_number`/`part_of_speech` facets, and pass the
   grounding-closure tests that guard the tomato entry today.
4. Import a first batch (target: 100+ common nouns across 4 languages) and
   wire it into the seed so translation and formalization quality visibly
   improve (measure: formalization coverage on the benchmark prompts).

**Existing components**

- `scripts/ground-meanings.rs`, `src/seed/meanings.rs::WordForm`,
  `data/seed/meanings-translation.lino` — the shape to generate.
- `tests/unit/semantic_grounding.rs` closure tests — the validators.
- `src/knowledge.rs` cached-external-API discipline.

**Acceptance criteria**

- `formal-ai import lexemes --concepts <file> --offline` reproduces the
  committed batch byte-for-byte from cache (deterministic).
- `cargo test bulk_lexeme_import` covers: template emission, facet
  completeness, denotation bidirectionality, cache-record grounding.
- The seed grows by ≥ 100 grounded meanings with all existing grounding and
  translation tests green.
- The importer refuses to write entries that fail validation, recording
  `import_rejected` events instead.

---

## E42 (C) — Probability-weighted statement formalization with contradiction warnings (R384)

**Problem**

REQUIREMENTS R384: the universal meta algorithm should formalize every
message as a probability-weighted statement and warn about contradictory
requirements with proposed resolutions. Today the method registry (#559) and
self-AST slice exist, and probability evidence (`src/probability.rs`) can
bias formalization — but statements do not carry explicit posterior weights
in the trace, and contradictions between user requirements are not detected
or surfaced.

**Approach**

1. Extend the formalization step to append a `statement_weight` link per
   accepted interpretation (posterior from the existing symbolic evidence +
   temperature machinery) so every formalized statement is inspectable as a
   weighted claim.
2. Add a contradiction detector over the requirement store: when a new
   formalized requirement conflicts with a retained one (same subject,
   incompatible predicate values — the `bank_river`/`bank_money` split
   machinery generalized to requirements), append a
   `requirement_contradiction` event.
3. Surface the warning in the reply with both statements, their weights, and
   at least one proposed resolution (split meanings, retract one, scope by
   context), reusing the retraction protocol.
4. Use issue #651's own mixed scope as a fixture (the issue text itself
   contains overlapping asks — the R384 note already suggests this).

**Existing components**

- `src/probability.rs`, `src/translation/selection.rs` — posterior scoring.
- `src/dreaming/learning.rs::LearnedRequirement` — retained requirements to
  check against.
- `src/event_log.rs` retraction protocol; meaning-split precedent in
  `src/seed/meanings.rs`.

**Acceptance criteria**

- `cargo test weighted_formalization` — every formalized statement in a
  multi-interpretation prompt carries a `statement_weight` link summing to 1
  across candidates.
- `cargo test requirement_contradiction` — feeding "always answer in
  Russian" then "never answer in Russian" yields a
  `requirement_contradiction` event and a reply containing both statements
  and a proposed resolution, in the prompt's language.
- Diagnostics stay default-off; weights appear in the trace, not the plain
  reply.

---

## E43 (C) — Budget-driven random and evolutionary search in synthesis (F4)

**Problem**

`GOALS.md` (Universal Solver Goals): "When no reusable part exists, combine
reasoning, random search, and evolutionary search according to the available
compute budget instead of giving up." `docs/USER-JOURNEYS.md` F4 lists this
as a potential future journey. Today synthesis composes decomposed
sub-results deterministically; there is no seeded random or evolutionary
strategy, so problems with no rule-derived path still dead-end.

**Approach**

1. Add a `compute_budget` knob to `SolverConfig` (default small) and a
   search stage inside step 7 (solution synthesis) that activates only when
   reuse and rule reasoning produced no candidate.
2. Random search: sample candidate compositions of known parts, seeded from
   the impulse content hash (determinism preserved per the config contract).
3. Evolutionary search: mutate/crossover the best-scoring candidates against
   the step-6 generated tests as the fitness function; budget counts
   candidate evaluations.
4. Record every generation as `search:` events so "why did you answer that?"
   explains the search path; on budget exhaustion, fall back to the honest
   unknown-reasoning reply with the search evidence attached.
5. Add at least one benchmark case solvable only through search to the
   industry suite and raise the ratchet.

**Existing components**

- `SolverConfig` + impulse-hash seeding (`VISION.md` determinism contract).
- `src/solver_handlers/program_synthesis.rs` test-verification loop — the
  fitness harness.
- `src/solver_unknown_reasoning.rs` — the fallback to keep honest.

**Acceptance criteria**

- `cargo test budget_search` — a fixture problem with no direct rule path is
  solved under a sufficient budget, remains `unknown` (with `search:`
  evidence) under budget 0, and produces identical output across runs.
- Benchmark suite grows by ≥ 1 search-only case; `minimum_pass_count` raised
  accordingly.
- `FORMAL_AI_COMPUTE_BUDGET` env var and CLI flag wire the knob per the
  SolverConfig promotion pattern.

---

## E44 (C) — Retire the `SPECIALIZED_HANDLERS` precedence remnant into data-driven routing

**Problem**

ROADMAP pillar 20 acknowledges that a `SPECIALIZED_HANDLERS` precedence table
still sits behind the formalized intent router — a remnant of the
fixed-catalogue era. Handler precedence is behavior, and behavior belongs in
seed data ("Data Is The Interface"), not in a Rust constant.

**Approach**

1. Express the precedence relation as seed data: a `handler-precedence.lino`
   listing each handler meaning and its ordering/guard conditions, loaded via
   `src/seed.rs` and mirrored by the browser worker seed loader.
2. Replace the Rust constant with a loader over that seed; keep a
   compile-time assertion that every registered handler appears exactly once
   so a seed edit cannot silently drop a handler.
3. Verify no behavioral diff: the full spec suite plus benchmark ratchets
   pass unchanged; add a routing-parity fixture across Rust and the browser
   worker.
4. Document in `ARCHITECTURE.md` §2 (routing) and drop the pillar-20 remnant
   note from `ROADMAP.md`.

**Existing components**

- `data/seed/intent-routing.lino` and `operation-vocabulary.lino` — the
  data-driven routing precedents.
- `src/solver_handlers/mod.rs` registry.

**Acceptance criteria**

- `grep -r "SPECIALIZED_HANDLERS" src/` returns nothing (or only the loader
  symbol name reading seed data).
- `cargo test routing_precedence_from_seed` — reordering two rows in the
  seed fixture changes routing in the test store; the shipped seed keeps
  today's behavior (full suite green).
- Rust/browser routing parity pinned by a shared fixture.

---

## E45 (A) — Associative terminology cleanup: links network, not graph

**Problem**

Issue #651 asks to double-check that the project focuses only on associative
technologies — links networks, Links Notation, the meta language — with no
graphs or tables. The audit (`docs/case-studies/issue-651/code-audit.md` §4)
found the architecture complies, but naming does not: `GET /v1/graph`,
`src/self_source_graph.rs`, `src/agentic_coding/source_graph.rs`, and
"link-graph network view" UI strings all say "graph" for what the docs call a
links network.

**Approach**

1. Add `GET /v1/network` as the canonical endpoint; keep `/v1/graph` as a
   deprecated alias (existing desktop/VS Code/e2e clients keep working) and
   emit a deprecation note in its response metadata.
2. Rename `src/self_source_graph.rs` → `src/self_source_links.rs` and
   `src/agentic_coding/source_graph.rs` → `source_links.rs`, updating doc
   comments to links-network vocabulary.
3. Sweep UI strings (web, desktop, VS Code) to "links network view"; the
   seeded user-facing "graph" concept (issue #161) gains a synonym link
   rather than deletion, since user vocabulary is data.
4. Add a repository-hygiene lint that blocks *new* `graph`-named public API
   routes/modules (allowlisting the deprecated alias, Wikidata "knowledge
   graph" citations, and the codecov badge).

**Existing components**

- `src/server.rs::handle_graph_request`; desktop preload bridge; VS Code
  network view.
- Repository-hygiene checks pattern in `scripts/`.

**Acceptance criteria**

- `curl :8080/v1/network` and `/v1/graph` return identical payloads; the
  alias is covered by an integration test asserting the deprecation marker.
- `cargo test` + Playwright suites green after renames.
- Terminology lint fails on a fixture introducing `/v1/knowledge-graph`.
- `ARCHITECTURE.md`/`README.md` references updated.

---

## E46 (D) — Installable offline PWA and npm package for the WASM engine — blocked by E39

**Problem**

The vision's "Reaching A Wide Audience" section promises an installable,
offline-capable progressive web app and an embeddable npm engine package.
Today the GitHub Pages demo is a plain static page (no manifest, no service
worker, no offline install), and the WASM engine is not published anywhere a
web developer could `npm install`.

**Approach**

1. PWA: add a web app manifest, icons, and a service worker that precaches
   the app shell, `formal_ai_worker.wasm`, and the seed files (the seed is
   the knowledge — offline must include it); cache-bust by content hash via
   the existing Bun build.
2. Verify the offline journey with Playwright: load once online, go offline,
   reload, complete a chat exchange with trace links.
3. npm package: publish the WASM worker + JS bindings + seed loader as
   `@link-assistant/formal-ai-engine` with a typed `solve()`/memory API and
   the bundle export/import; wire packaging into the release pipeline next
   to the crates.io publish, versioned with the crate.
4. Document embedding in `README.md` with a minimal example page.

**Existing components**

- `src/web/` app + `wasm-worker` + `seed_loader.js`; `bun run build:web`.
- `.github/workflows/release.yml` publish steps; `links-notation` npm
  package precedent for the JS side.

**Acceptance criteria**

- Lighthouse (or Playwright equivalent) confirms installability; the
  offline-reload chat e2e passes in CI.
- `npm pack` artifact installs in a fixture project and answers a prompt
  with the same output as the Rust core for the parity dialog set.
- Release pipeline publishes the npm package with provenance on the same
  trigger as the crate.

---

## E47 (D) — Publish the VS Code extension to the Marketplace and Open VSX

**Problem**

The VS Code extension (`vscode/`, v0.154.0) works but is installable only by
manually building a `.vsix` (README discloses this). Editor surfaces are
where coding assistants meet most users; unpublished means unreachable.

**Approach**

1. Prepare listing assets: publisher account, icon, gallery banner,
   `README`/screenshots for the extension page, category and keyword
   metadata, telemetry statement (none), license.
2. Add `vsce publish` (Marketplace) and `ovsx publish` (Open VSX) steps to
   the release pipeline, gated on the extension's own version change and
   using repository secrets; keep the `.vsix` GitHub-release artifact.
3. Reconcile versioning: document the crate/desktop/extension version triple
   in `CONTRIBUTING.md` release section so release notes stop being
   confusing (audit gap §6.9).
4. Post-publish smoke: a CI job installs the published extension by id in a
   clean VS Code (or `code-server`) and runs the existing extension tests.

**Existing components**

- `vscode/` extension with tests; vsix build scripts; release workflow
  secrets mechanism used by crates.io/Docker publishing.

**Acceptance criteria**

- Extension resolvable as `link-assistant.formal-ai` (or chosen id) on both
  registries; install-by-id smoke job green.
- Release pipeline publishes both registries automatically on version bump.
- `README.md` install section switches from manual `.vsix` to Marketplace
  instructions (keeping the manual path documented for offline users).

---

## E48 (D) — Interactive step-by-step debugging view (R383) — blocked by E47

**Problem**

REQUIREMENTS R383 (from issue #538): an interactive, step-by-step debugging
view — chat, data, mermaid diagram, and Rust/JS panes side by side — so a
user can watch the 11-step loop execute link by link. Only exploratory notes
exist under `docs/vscode/`.

**Approach**

1. Build on the published VS Code extension (E47): add a "Formal AI
   Debugger" webview with four panes — conversation, live event-log links
   (via `/v1/memory/since` polling or SSE), the generated mermaid view of
   the active recipe (`src/agentic_coding/diagram.rs` output), and the
   source location of the executing stage (method registry from #559 maps
   stage → `path:symbol`).
2. Step controls: a `SolverConfig` debug knob that pauses the loop between
   steps (server holds the solve; the view advances it), off by default and
   only in explicitly-opted-in sessions.
3. Every pane is a projection of links — no debugger-private state; the same
   data must render in the web demo's diagnostics page as a degraded
   fallback.
4. Record a demo session GIF/screenshots for the docs.

**Existing components**

- `docs/vscode/` notes; VS Code network view; `/v1/memory/since`;
  `src/agentic_coding/diagram.rs`; the #559 method registry.

**Acceptance criteria**

- Extension test drives a solve with the debug knob on, steps through ≥ 3
  stages, and asserts pane contents match the event log.
- The pause knob is off by default and refused on non-loopback servers.
- Docs page with screenshots under `docs/vscode/debugger.md`.

---

## E49 (D) — Shareable associative packages between instances (F6)

**Problem**

`docs/USER-JOURNEYS.md` F6: a user packages datasets, skills, rules, and
handlers with permissions and shares them with another instance —
Deep.Foundation-style packages adapted to doublets. `src/associative_package.rs`
unified the in-repo package/permission model (E18), but there is no
export/import of a *package* as a shareable artifact between two running
instances.

**Approach**

1. Define the package manifest in Links Notation: name, version, declared
   permissions, contained links (meanings, rules, skills, handler
   references), and provenance — a scoped subset of the full
   `formal_ai_bundle`.
2. `formal-ai package export <name>` / `formal-ai package import <file>`:
   import runs the permission review (list declared permissions, require
   explicit confirmation for `agent`-tagged tools), appends
   `package_imported` events, and rejects handler references that are not
   locally available instead of silently degrading.
3. Round-trip across surfaces: CLI-exported package imports in the web demo
   (file picker) and vice versa.
4. Ship one real example package (e.g. a language pack or skill pack) under
   `examples/packages/`.

**Existing components**

- `src/associative_package.rs`, permission gating, bundle
  export/import/migration notices (`memory::export_full_memory`).

**Acceptance criteria**

- `cargo test associative_package_sharing` — export → fresh store → import
  reproduces the package's links and permission gates; importing a package
  demanding an unavailable handler fails loudly.
- Web ↔ CLI round-trip covered by an e2e test.
- Example package documented in `README.md`.

---

## E50 (D) — Cloud memory sync for the single-file bundle (F3)

**Problem**

`VISION.md` (Growable Memory) names "future cloud sync" as a persistence
target beyond disk and IndexedDB; F3 describes the journey — the same
`formal_ai_bundle` follows the user between machines automatically. Today
migration is manual export/import only.

**Approach**

1. Keep it associative and vendor-neutral: sync the append-only event log
   through any user-supplied backend, starting with the simplest ones — a
   user-owned git repository and a WebDAV/S3-compatible endpoint. No
   Formal-AI-hosted service.
2. Append-only makes sync tractable: push = append local events since the
   last synced id; pull = append remote events not present locally; the
   projected prefix never shrinks (the existing concurrency guarantee).
   True conflicts are impossible at the log level; duplicate-suppression by
   content-addressed event id.
3. `formal-ai memory sync --remote <url>` plus a `SolverConfig`/env knob;
   off by default, never syncing without explicit opt-in (privacy is the
   product's promise).
4. Browser side: manual "Sync now" against the same remote via fetch;
   background sync deferred.

**Existing components**

- Content-addressed event ids; `memory::{export,import}_full_memory`;
  `/v1/memory/since` incremental endpoint (the same delta shape sync needs).

**Acceptance criteria**

- `cargo test memory_sync` — two stores syncing through a temp-dir remote
  converge to the same projected state from interleaved appends; re-running
  sync is idempotent.
- Sync refuses to run without the explicit opt-in flag/env.
- e2e: CLI ↔ web round-trip through a local WebDAV fixture.

---

## E51 (D) — Browser multi-language execution experiment via WebVM (F5)

**Problem**

`VISION.md` (Product Shape): browser mode starts with JavaScript evaluation
and can later experiment with WebVM so more languages run locally, while
honestly reporting execution limits (a NON-GOAL forbids claiming host-level
execution). F5 tracks the journey; nothing exists yet.

**Approach**

1. Time-boxed experiment under `experiments/webvm/`: evaluate
   CheerpX/WebVM-style x86-in-browser and language-specific WASM runtimes
   (Pyodide for Python, ruby.wasm) as execution backends for generated code.
2. Wire the winner behind the existing execution-capability reporting: the
   browser worker advertises which languages are runnable; generated-code
   replies keep stating compiled/ran/not-run honestly per environment.
3. Ship Python-in-browser as the first target (the benchmark suite already
   synthesizes Python, so the browser can then *verify* HumanEval/MBPP
   candidates client-side — closing a real parity gap with the Rust bounded
   agent).
4. Record findings (size, startup, determinism) in the experiment README;
   promote to `src/web/` only if the demo stays usable on a mid-range
   laptop.

**Existing components**

- Browser JS evaluation path; execution-metadata reporting; the bounded
  agent verification loop to mirror.

**Acceptance criteria**

- Experiment report committed with measurements and a go/no-go
  recommendation.
- If promoted: an e2e test runs a synthesized Python function with its
  assertions fully in-browser and shows the honest execution note; the
  no-WASM fallback still works.
- Non-goal guard: browser replies never claim host-level execution.

---

## E52 (C) — Multi-CLI agentic end-to-end matrix in CI (codex, opencode, gemini, qwen, claude, grok, aider)

**Problem**

`docs/testing/agentic-cli-tools.md` (from issues #625/#628) prescribes a CI
sequence for verifying real third-party CLI clients against `formal-ai serve
--agent-mode` — but it shipped as prose only: `.github/workflows/release.yml`
contains a single `test-agent-cli-e2e` job for our own Agent CLI and no
codex/opencode/gemini/qwen job at all. The cost is already visible: PR #648
closed #647 with claude "intentionally not run" and grok/aider "inferred from
the shared adapters", and hands-on testing immediately produced issue #650
with four defects. Regressions of the #620/#624/#626/#627 fixes have no
guard.

**Approach**

1. Turn the guide's "CI Shape" section into an actual workflow job matrix:
   one leg per CLI (codex, opencode, gemini, qwen, claude, grok, aider, plus
   our Agent CLI as the reference leg), each running the guide's smoke
   sequence against a local `formal-ai serve --agent-mode` with the recorded
   proxy from PR #631 so no leg needs vendor credentials.
2. Encode known upstream constraints as *expected-behavior assertions*, not
   skips: gemini headless `-p` advertises no functionDeclarations (chat-only
   — the #620 comment that was never filed as an issue), codex/gemini/qwen
   lack a headless approval handshake (#511/PR #512). When an upstream
   release lifts a constraint, the assertion fails loudly and we upgrade.
3. Cover the #650 defect surface explicitly: `/responses` instructions
   handling, empty-message interactive behavior per CLI, conversation
   summarization requests, `--globally` alias — each as a regression case
   that fails before the #650 fix and passes after.
4. Keep the matrix fast: legs run in parallel, pinned CLI versions installed
   from a lockfile, recorded transcripts committed so replays are offline
   and deterministic.

**Existing components**

- `docs/testing/agentic-cli-tools.md` — the prescribed sequence.
- The recording proxy from PR #631; `test-agent-cli-e2e` job as template.
- `docs/case-studies/issue-647/` transcripts as seed recordings.

**Acceptance criteria**

- `release.yml` (or a dedicated workflow) runs the full matrix on every PR
  touching server/protocol code; all legs green on the branch.
- Each of the four #650 defects has a failing-then-passing regression case
  in the matrix.
- Upstream-constraint assertions documented inline with links to the
  upstream issues.
- claude, grok, and aider — the never-actually-run integrations from
  PR #648 — each have at least one recorded, replayable session.

---

## E53 (D) — Land the deferred issue-541 UI follow-ups (F1–F5)

**Problem**

PR #542 (issue #541) wrote five concrete UI follow-ups to
`docs/case-studies/issue-541/proposed-issues.md` — dark-theme snapshot
coverage, migration replay UI, animation-budget override,
reasoning-hierarchy editing, IPC mode-flip tests — and none was ever filed
as a GitHub issue (incomplete-work audit, item 12). They are exactly the
"deferral migrated from GitHub into repo docs" pattern issue #651 exists to
fix, and together they finish the desktop/web UI polish wave.

**Approach**

1. Treat `docs/case-studies/issue-541/proposed-issues.md` F1–F5 as the
   specification; implement in that order, one commit per item:
   F1 dark-theme visual-regression snapshots for every surface;
   F2 a UI affordance to replay memory migration notices;
   F3 a user-visible animation-budget override (respects reduced-motion);
   F4 editing of the reasoning hierarchy from the UI (projection of links —
   edits append events, never mutate);
   F5 IPC mode-flip tests (server ⇄ embedded) for the desktop app.
2. Where an item's design has been superseded by later work (e.g. the #557
   multi-skin request), reconcile with the newer issue instead of
   duplicating, and record the reconciliation in the case study.
3. Add before/after screenshots per item to the PR (visual work).

**Existing components**

- `docs/case-studies/issue-541/proposed-issues.md` — full specs F1–F5.
- Playwright e2e + screenshot harness; desktop IPC test setup.

**Acceptance criteria**

- Each of F1–F5 either lands with its own tests (named in the follow-up
  specs) or is explicitly reconciled against a newer issue in the case
  study — no silent drops.
- Dark-theme snapshots run in CI for web and desktop surfaces.
- Desktop mode-flip covered by an automated test.

---

## E54 (S) — Extend the self-AST census from one module to a workspace census

**Problem**

The meta-language self-representation (`data/meta/self-ast.lino`, R381) is
still pinned to a single module (`src/agentic_coding/planner.rs`). PR #601
itself named "the smallest real next slice: extend the pinned target … to a
directory census", and while PR #637 widened the *source-links* index, the
CST/AST census never grew. Self-coding (E35/E36) needs the system to see its
own code: a planner that can only introspect one file cannot plan edits
across the workspace.

**Approach**

1. Generalize the census recipe to take a target set (directory glob) and
   emit per-module `.lino` census documents plus a workspace index —
   incremental, so one changed module re-censuses one document.
2. Scale honestly: full-fidelity AST for `src/agentic_coding/` first, then
   signature-level census (items, symbols, spans) for the rest of `src/`,
   with a documented fidelity marker per module — avoiding a multi-megabyte
   seed while keeping every module addressable.
3. Wire freshness into CI: a check that fails when a committed census
   diverges from the source it describes (the drift-guard pattern used by
   the method registry from #559).
4. Feed E35: the general planner resolves edit targets through the census
   index instead of hardcoded paths.

**Existing components**

- `data/meta/self-ast.lino` + its census recipe and pinning tests.
- `src/agentic_coding/{self_source,source_graph}.rs` widened by PR #637.
- The #559 method registry drift-guard.

**Acceptance criteria**

- `data/meta/self-ast/` contains a census for every `src/` module with its
  fidelity marker; the workspace index resolves any `path:symbol` the
  method registry knows.
- `cargo test self_ast_census` — census regenerates deterministically;
  drift check fails on a fixture with a stale census.
- A planner test resolves an edit target in a module other than
  `planner.rs` via the census index.

---

## E55 (C) — Compile arbitrary natural-language programs beyond the supported skill subset

**Problem**

`ARCHITECTURE.md` §16 has carried one open question since the E20 batch:
"arbitrary natural-language programming beyond the supported subset" of
`src/skill_compiler.rs`. `docs/USER-JOURNEYS.md` F2 describes the journey — a
user states a multi-step procedure in plain language ("when I paste a URL,
fetch its title, translate it to Russian, and store both") and the system
compiles it into a typed, executable skill. Today the compiler handles typed
trigger/response and a bounded multi-step shape; procedures outside that
shape fall back to formalization without compilation. This is the "five rule
shapes ending in compiled natural-language skills" pillar taken to its
stated conclusion.

**Approach**

1. Reuse the solver's own decomposition (the same move as the general
   planner in E35/#654): formalize the stated procedure into ordered
   sub-requirements, then map each sub-requirement onto the skill
   compiler's existing typed step vocabulary (fetch, transform, translate,
   store, reply, condition).
2. Where a step has no existing vocabulary entry, fail honestly with a
   named gap ("no compiled capability for X") and record a
   `skill_gap` event — never silently drop a step.
3. Grow the step vocabulary as seed data (`data/seed/`), not Rust match
   arms, so new step kinds are data edits (the operation-vocabulary
   precedent from E33).
4. Multilingual from the start: the same procedure stated in en/ru/hi/zh
   compiles to the same skill links (round-trip guard).
5. Compiled skills remain inspectable: "why did you do that?" cites the
   compiled steps and their source sentence spans.

**Existing components**

- `src/skill_compiler.rs` typed/multi-step compiler with native lowering.
- `src/intent_formalization.rs` + solver decomposition (steps 2–5).
- `data/seed/operation-vocabulary.lino` — the multilingual step-vocabulary
  pattern.

**Acceptance criteria**

- `cargo test arbitrary_skill_compilation` — a ≥ 4-step procedure phrased
  freely (not matching any existing compiler template) compiles, executes
  end to end, and re-states its steps on request; the same procedure in
  Russian compiles to the same skill links.
- A procedure containing one uncompilable step yields the honest named-gap
  reply plus a `skill_gap` event, and compiles nothing partially.
- `ARCHITECTURE.md` §16 open question removed; `docs/USER-JOURNEYS.md` F2
  marked with its new status.
