# Issue 559 Solution Plan

This plan preserves the issue's first-session constraint: document and plan
first, then implement behavior changes after maintainer approval. It folds in the
two rounds of PR feedback (2026-06-23): the core solver must be fully recursive
and bidirectional, links must be the native representation, Voyager must be a
design reference without any neural runtime, an upstream dependency audit must
precede implementation, and the plan must be at least twice as detailed with
options compared and re-checked against the governing documents.

This document is the **spine**. The deep detail lives in focused, cross-linked
companion documents so each stays reviewable:

## Reading Guide

| Document | Purpose |
| --- | --- |
| [requirements.md](requirements.md) | R1–R27 + non-goals derived from the issue and feedback |
| [architecture-inventory.md](architecture-inventory.md) | Current architecture, grounded in `file:line`, with "already built vs absent" |
| [alignment.md](alignment.md) | Re-check against VISION/GOALS/NON-GOALS/ROADMAP/REQUIREMENTS/ARCHITECTURE; resolves conflicts C1–C7; canonical vocabulary mapping; proposed R330+ |
| [critical-review.md](critical-review.md) | Every first-session imprecision corrected with evidence (CR1–CR12); precise built-vs-absent inventory |
| [options-comparison.md](options-comparison.md) | 2–4 options per major decision with pros/cons/cost/risk + recommendation + comparison harnesses |
| [recursive-core.md](recursive-core.md) | Downward/upward passes, atomicity predicate, `SolverConfig` knobs, pseudo-code grounded in `solver.rs:411-653`, Voyager mapping |
| [evidence-pipeline.md](evidence-pipeline.md) | expand→search→rerank→crawl→extract→compare→hypothesize, grounded in the real search core and fetch seams |
| [upstream-dependency-audit.md](upstream-dependency-audit.md) | Org-owned dependency readiness and gates |
| [raw-data/online-research.md](raw-data/online-research.md) | External research (ReAct, ToT, GoT, Reflexion, Self-Refine, Voyager, meta-theory, etc.) |

## Planning Status

Current PR scope:

- Planning artifacts only; no runtime behavior changes.
- No migration of current caches, overrides, meanings, or `.lino` assets.
- No direct replacement of current specialized handlers.
- No new upstream issues opened — the audit found no blocker for the next
  behavior-preserving phases ([upstream-dependency-audit.md](upstream-dependency-audit.md)).

Recommended next approval target:

- Approve **Phase 1A** and **Phase 1B** only (both behavior-preserving).
- Phase 1A adds an observable `ProblemFrame` trace while preserving routing.
- Phase 1B adds a recursive `WorkUnit` trace while preserving answers.
- Later phases move selection into data only once comparison evidence proves
  parity.

## Canonical Vocabulary (read this first)

The working names below are **not new primitives**. They make the existing 11-step
loop's implicit state explicit and link-serializable. This is the resolution of
conflict C1 (full table in [alignment.md](alignment.md)):

| Working name | Canonical concept | Current carrier | Requirement |
| --- | --- | --- | --- |
| `ProblemFrame` | Formalized impulse + 11-step envelope (the "meaning record") | `IntentFormalization` (`src/intent_formalization.rs:48`) | R157, R72 |
| `Need` | A detected requirement / sub-requirement | requirement records; R158 "graph of requirements and subtasks" | R158 |
| `WorkUnit` | A recursively-formalized `sub_impulse` | `UniversalSolver::decompose` output | R74 |
| atomicity / depth | Recursion bound | `SolverConfig::max_decomposition_depth` + new `atomicity_policy` knob | R74, NON-GOALS |
| `evidence_policy` | Fresh-vs-cached source policy | `source:`/`policy:offline`/`FORMAL_AI_OFFLINE` | R67 |
| method/skill registry | Data-described selection | extends §9 ladder + `data/seed/intent-routing.lino` | R103, R97 |

Every new control knob is added to `SolverConfig` **first** (`NON-GOALS.md`: "new
knobs are added to the config first"). The 11-step loop *shape* never branches by
domain (`GOALS.md`).

## Roadmap Positioning

Issue 559 closes a **named residual**, it does not re-open settled work
(conflicts C2/C4 in [alignment.md](alignment.md)):

- **Pillar 20** (routing by formalized intent — Built, caveat
  "`SPECIALIZED_HANDLERS` remain as a precedence table behind the formalized
  router"): the method-registry migration removes that residual.
- **Pillar 7** (task-agnostic meta-builder, "algorithm that builds algorithms",
  R7, tracked in `docs/case-studies/issue-412`): issue 559 advances it.
- **Pillars 2 / 19 / 26** (universal loop / reasoning under unknowns / general
  synthesis — Built): issue 559 makes their internal state explicit and reuses
  them; it does not change the loop shape.

Per the **Verification Contract** (`ROADMAP.md:266`), any phase that changes a
roadmap item's status must update REQUIREMENTS.md, the ARCHITECTURE status table,
and ROADMAP together.

## Target Architecture

A recursive, bidirectional, link-native problem solver. Rust structs, functions,
and standard-library calls remain the implementation, but the **control plane**
becomes data: representable as links, serialized through `.lino`, and eventually
round-tripped through `meta-language`.

Every prompt flows through one shape (the existing 11-step loop with explicit
state):

1. Capture an impulse.
2. Formalize it into a `ProblemFrame` (explicit meaning record).
3. Detect all needs; assign each an evidence policy.
4. Recurse: split into `WorkUnit`s downward while searching the registry/cache to
   construct upward (both directions — [recursive-core.md](recursive-core.md)).
5. Resolve atomic units via existing leaf solvers (search, handlers, synthesis).
6. Compose results upward; record a need-satisfaction ledger.
7. Project the answer, marking every need satisfied/deferred/blocked/rejected.
8. Emit enough link-trace data to ground recipes and (later, gated) propose
   improvements.

### Link-Native Contract (re-anchored)

All planning and execution records are doublet links — because **`VISION.md:44`
states "Doublet links are the primitive storage model for this project"** and
`VISION.md:5` states "The associative network is the AI." (This re-anchors the
first-session draft, which leaned on external meta-theory phrasing absent from
the canonical docs — conflict C7.)

- A user message links to source text and context.
- A need links a requested outcome, constraints, evidence, and status.
- A method links preconditions, input shape, implementation hook, output shape,
  and validation policy.
- A work unit links a parent need, child units, selected method, observations,
  validation results, and composed output.
- Code can be linked as data via `meta-language` snapshots/source spans, then
  linked back to executable hooks (R24; later phase).

The meta-theory article remains a cited *external influence*
([raw-data/online-research.md](raw-data/online-research.md)), not repo doctrine.

### `ProblemFrame` (the explicit meaning record)

Built and traced before specialized behavior runs; routing stays unchanged until
comparison tests exist. Conceptual fields:

- `frame_id`, `impulse`, `mode`, `language` (+ source spans);
- `needs` (questions, commands, constraints, preferences, safety, source/freshness
  requirements, follow-up references);
- `evidence_policy` (memory / cache / search / crawl / tool / test / clarification
  / offline-only);
- `root_work_unit`, `candidate_methods`, `selected_methods`, `validation_plan`,
  `observations`, `composition`, `presentation`.

### Need Records

Every prompt decomposes into need records, not a single intent label — this is
what prevents answering one part of a multi-part prompt and silently dropping the
rest (R7, R8, R158). Fields: `need_id`, `source_span`, `kind`,
`satisfaction_criteria`, `evidence_requirements`, `dependencies`, `status`
(pending/satisfied/deferred/blocked/rejected/superseded), `status_reason`.

### Recursive `WorkUnit`

The recursive execution unit, stored as link data (Rust struct at runtime).
Fields: `work_unit_id`, `parent_work_unit_id`, `linked_need_ids`, `input_links`,
`output_links`, `constraints`, `evidence_policy`, `atomicity_decision`,
`candidate_methods`, `selected_method`, `child_work_units`, `observations`,
`validation_results`, `composition_result`, `status`. The atomicity predicate and
the downward/upward passes are specified in [recursive-core.md](recursive-core.md).

### Method And Skill Records

Current handlers become method records first; skill records accumulate later.
Fields: `method_id`, `aliases`, `meaning_links`, `preconditions`,
`negative_preconditions`, `input_schema`, `output_schema`,
`required_capabilities`, `evidence_policy`, `validation_policy`,
`implementation_hook`, `fallback_hooks`, `compatibility_precedence_group`,
`old_dispatch_handler`, `source_files`, `related_tests`, `benchmark_fixtures`,
`last_successful_examples`, `failure_examples`, `promotion_status`
(seed/experimental/stable/deprecated/retired). The registry starts as data that
mirrors existing behavior; it becomes the control plane only after old/new
selection agree (Decision 2 / 5 in [options-comparison.md](options-comparison.md)).

## Core Algorithm, Evidence, Options (summaries)

To avoid duplication, the deep specifications live in companion docs:

- **Recursive core** — downward decomposition + upward construction meeting at an
  atomicity predicate; pseudo-code grounded in `solver.rs:411-653`; leaf solvers
  reuse today's search/handlers/synthesis; SolverConfig knobs
  (`atomicity_policy`, `recursion_mode`, `selection_mode`); Voyager mapping
  without neural runtime. See [recursive-core.md](recursive-core.md).
- **Evidence pipeline** — reuses the built 33-provider search and RRF
  (`src/web_search_core.rs`); adds crawl/extract/compare/hypothesize and non-CORS
  providers via the desktop fetch seam; offline-deterministic with cached
  fixtures. See [evidence-pipeline.md](evidence-pipeline.md).
- **Options** — frame representation, method selection, recursion direction,
  evidence execution, migration strategy, registry storage, algorithm-as-data —
  each with options, recommendation, and a comparison harness behind a
  `SolverConfig` knob. See [options-comparison.md](options-comparison.md).

## Self-Modification Boundary

The only sanctioned self-modification mechanism is
`docs/design/self-improvement-loop.md` (issue #364): proposal-only, gated by
verification + benchmarks + human review, never auto-appending to `data/seed/`.
Issue 559 adds **no** new autonomy (conflict C3). "Reason about itself" is
delivered as *inspectability* (the algorithm is readable, queryable, diffable
data), not unsupervised self-editing. This is Phase 9, and it cannot ship without
the gate.

## Phased Implementation

Each phase is independently committable and reviewable (R17). Phases that add data
or change routing must pass the full gate matrix (see Verification Matrix). Phases
that touch comparison knobs default them to the compatible value, so merging
changes nothing observable until a knob is flipped — and each flip is gated by a
comparison harness.

### Phase 0A — First Planning Artifact

Status: complete (earlier commit). Deliverables: requirements, architecture
inventory, research, initial phased plan. Exit: maintainers can review direction
without runtime risk.

### Phase 0B — Feedback Integration, Critical Check, Upstream Audit

Status: this revision. Deliverables: the companion docs above; canonical
vocabulary mapping; C1–C7 resolutions; CR1–CR12 corrections; options with
comparison harnesses; recursive core and evidence pipeline grounded in source;
upstream audit; expanded requirements. Checks: documentation diff review,
changelog fragment valid, no code behavior changes. Exit: next behavior-preserving
phases are specific enough to approve/reject. Pause if: a maintainer rejects the
recursive-work-unit direction or asks to open a specific upstream issue first.

### Phase 1A — Add `ProblemFrame` Without Behavior Changes

Goal: make the explicit meaning record observable before changing routing.
Tasks: add the Rust `ProblemFrame` (Option 1A — extend `IntentFormalization`);
populate from current formalization data; link to impulse/context/mode; extract
initial needs without routing on them; record evidence-policy guesses without
enforcing; emit as a solver event / `.lino` trace behind a debug flag if noisy.
Tests: frame-construction unit tests; fixtures for single-question, multi-need,
follow-up, constraint-heavy prompts; structured assertions for needs and policy;
existing answer tests unchanged; **gate matrix** (closure, no-hardcoded-NL,
traceability for R330, parity for any shared logic). Exit: every path can emit a
frame; answers and selection unchanged; source-span data sufficient for later
need tracking. Pause if: formalization can't produce source spans without a
larger parser change, or worker parity can't represent the frame without new
serialization.

### Phase 1B — Add Recursive `WorkUnit` Trace Without Behavior Changes

Goal: prove the recursive shape while old dispatch still controls answers.
Tasks: add `WorkUnit` records linked to needs; deterministic atomicity decisions;
for direct handlers, one root + one leaf wrapping the selected handler; for
multi-need prompts, sibling child units even though old dispatch answers one
route; record the old handler as the leaf hook; validation status as trace-only.
Tests: parent/child link tests; atomicity tests for arithmetic, translation,
sorting, lookup, source-cache, agentic prompts; multi-need prompts create
multiple linked children; **widen
`specialized_handlers_still_publish_loop_events` beyond arithmetic**; existing
answer tests unchanged; gate matrix. Exit: recursive traces for representative
classes; leaves map to a single handler/crate/stdlib call; no visible behavior
change. Pause if: trace size exceeds current serialization (then
`links-notation#197` streaming becomes relevant).

### Phase 2 — Need-Satisfaction Ledger

Goal: make dropped requirements visible before changing execution. Tasks: promote
need statuses to a ledger on the frame; validate every need ends
satisfied/deferred/blocked/rejected/superseded; final-answer coverage check for
multi-part prompts; diagnostics when old dispatch answers only one need. Tests:
multi-need fixtures; negative tests for intentionally unsatisfied constraints;
regressions for issue families that historically lost follow-up context; answer
text unchanged unless diagnostics enabled; gate matrix. Exit: the solver reports
which needs the current path covered; no direct dispatch replaced yet.

### Phase 3 — Inventory Current Handlers As Methods

Goal: make every current specialized handler visible as data. Tasks: create the
method registry under `data/seed/` or `data/meta/` (Option 6A); one entry per the
**50** `SPECIALIZED_HANDLERS` members; entries for the five contextual-override
handlers and method-like seed meanings; preserve current precedence as
compatibility metadata; implementation hooks pointing to current Rust/JS; link
related tests and benchmark fixtures. Tests: registry covers all 50 handlers;
every hook resolves; current dispatch order reconstructs from metadata; a guard
test fails when the Rust table and registry diverge; gate matrix (closure is
strict — author entries closed). Exit: the registry is a complete mirror of
current dispatch; no runtime path depends on registry selection yet.

### Phase 4 — Registry Selection In Comparison Mode

Goal: run old dispatch and registry selection side by side (Option 5B). Tasks:
add `SolverConfig.selection_mode ∈ {legacy, registry, compare}` (config first);
in `compare`, compute both, keep legacy as source of truth, record divergences as
events without changing answers. Tests: old and registry selection agree across
benchmark and prompt-variation fixtures; precedence corner cases covered; unknown
prompts still produce reasoning traces; Rust and worker produce the same
candidate ordering; gate matrix. Exit: divergence is zero (or every divergence is
a tracked, explained issue) across all suites before any default flip.

### Phase 5 — Move Cue Recognition Out Of Rust

Goal: reduce hardcoded natural-language triggers (R97). Tasks: move cues from
`append_prompt_relevants` (`src/intent_formalization.rs:718-800`) and
`looks_like_text_manipulation` (`:832-851`) into seed meanings or method
preconditions; replace Rust string lists with data lookup + structural
predicates; keep only true parser concerns in code; migration notes per cue
family. Tests: no-hardcoded-NL guard updated for migrated cues; prompt-variation
tests per moved family; backward-compat tests; multilingual fixtures where
coverage exists; gate matrix. Exit: migrated cues are reviewable as data;
behavior matches old dispatch in `compare` mode.

### Phase 6 — Fresh Evidence And Search Generalization

Goal: make online research a reusable evidence policy, not a one-off
([evidence-pipeline.md](evidence-pipeline.md)). Tasks: implement query expansion;
reuse the built search + RRF; add crawl/extract/compare/hypothesize; wire non-CORS
providers through the desktop fetch seam (Option 4A+4B); deterministic cache
fixtures; connect evidence artifacts to needs and work units; extend
`evidence_policy` knob (config first). Tests: cached fixtures for current-date,
source-sensitive, citation prompts; rerank tests with stale vs authoritative
sources; contradiction tests; offline-mode tests that block fresh-data needs;
live-search smoke test outside deterministic CI; evidence-quality comparison
benchmark (pipeline off/on, CORS vs CORS+desktop); gate matrix incl. wired
parity. Exit: any method can request fresh evidence through the frame; answers
validate against citation/freshness requirements.

### Phase 7 — Skill Accumulation

Goal: accumulate reusable methods from successful traces (R21). Tasks: store
successful work-unit leaves as candidate skills; promote `experiments/` →
`examples/` when a real reusable use case is demonstrated; record failures and
blocked needs as curriculum items (the deterministic Voyager-curriculum analog);
promotion rules require tests + benchmark deltas; deprecation/retirement statuses.
Tests: a successful trace proposes a skill record; a failed trace creates a
curriculum item without changing behavior; proposed skills cannot become stable
without tests; bad proposals rejected by benchmarks/validation; gate matrix.
Exit: skill accumulation is reviewable and reversible; no unreviewed
self-modification.

### Phase 8 — Link-Native Algorithm-As-Data Integration

Goal: represent algorithms as data that round-trips with source (R24, Option 7B).
Tasks: use `meta-language` source spans/snapshots for method definitions where
practical; `.lino` representations for preconditions, validation policies, simple
compositions; code↔data round-trip fixtures for small methods; keep complex Rust
as hooks until translation is proven. Tests: lossless parse/reconstruct fixtures;
round-trip tests for simple validation/composition records; structural
query/replace tests for registry edits; compatibility tests proving hooks still
execute after round-trip; gate matrix. Exit: simple algorithm records reviewable
as links and reconstructable; hand-written code remains the execution source for
complex methods. Pause if: required `meta-language` APIs are absent in the pinned
version, or npm/browser packaging is required before Rust-side integration (see
`meta-language#165`).

### Phase 9 — Gated Self-Improvement

Goal: let the algorithm propose improvements without silently changing itself
(R12, C3). Tasks: reuse `docs/design/self-improvement-loop.md` and
`src/self_improvement.rs`; convert unknown traces into proposed method/skill/rule
patches; require tests + benchmark deltas before acceptance; human review is the
final gate. Tests: unknown trace produces a proposed entry; proposal not applied
without verification; rejected proposals remain inspectable; accepted proposals
include tests + changelog; gate matrix. Exit: the system can reason about
improving itself without mutating production behavior automatically.

### Phase 10 — Retire Direct Specialized Dispatch

Goal: make the registry the control plane after parity is proven. Tasks: flip
`selection_mode` to `registry` per method family one at a time; keep Rust handlers
as implementation hooks; remove direct table precedence only after registry
parity is proven; update docs/architecture diagrams and the ROADMAP/REQUIREMENTS
per the Verification Contract. Tests: full local CI; worker parity; benchmarks
and prompt variations; changelog/doc checks; old/new comparison evidence for every
retired family; gate matrix. Exit: the registry controls selection; specialized
Rust handlers remain available as hooks; no previously supported prompt family is
removed without explicit approval.

## Concrete First Implementation PR After Approval

The first code PR is deliberately small (Option 1A):

1. Add `ProblemFrame` and `Need` structures (extending `IntentFormalization`).
2. Populate from existing formalization inputs.
3. Add trace-only `.lino` serialization.
4. Add frame-construction unit tests.
5. Add multi-need prompt fixtures.
6. Confirm existing answer tests unchanged.
7. Add a changelog fragment and the R330 traceability test.

The second code PR adds `WorkUnit` traces (Phase 1B). Only after those should
registry selection start (Phase 3+).

## Verification Matrix

Standing local checks before any behavior switch:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs` (`.rs` ≤ 1000, `.lino` ≤ 1500)
- `cargo test`
- existing benchmark / prompt-variation commands
- worker parity tests where changed code crosses the runtime boundary

The six **hard gates** every data/routing phase must pass (resolves C6):

1. **Total reference closure** — `python3 scripts/audit-total-closure.py` →
   `unresolved_distinct: 0` (`tests/unit/total_closure.rs`).
2. **No hardcoded natural language** — the four gates in
   `docs/design/no-hardcoded-natural-language.md`, incl. worker-mirror `--check`.
3. **Requirement traceability** — a new `issue_559_..._are_traceable()` in
   `tests/unit/docs_requirements.rs` asserting each new `| R<n> ` row.
4. **Loop-event compatibility** —
   `specialized_handlers_still_publish_loop_events`
   (`tests/unit/specification/reasoning_loop.rs:44`) passes and is widened.
5. **Recipe grounding** — the general `data/meta/*-recipe.lino` recipe asserted
   against live source (parameterize the existing grounding harness — resolves
   C5).
6. **Cross-runtime parity** — Rust↔JS worker parity for shared logic, with a
   wired check (addresses the weak-flank risk in
   [critical-review.md](critical-review.md)).

Per phase: focused unit tests first; fixtures for the class being migrated;
old/new comparison before routing changes; deterministic cache fixtures for
fresh-data behavior; docs + changelog in the same PR; behavior changes narrow
enough to revert as one commit.

## Upstream Dependency Gates

The audit ([upstream-dependency-audit.md](upstream-dependency-audit.md)) found no
blocker for Phase 1A/1B. Existing issues that may matter later:

- `link-foundation/links-notation#197` (streaming parser) — if trace export grows
  too large.
- `link-foundation/meta-language#168` (shared-dialog source-description schema) —
  when traces must interoperate with shared-dialog source descriptions.
- `link-foundation/meta-language#165` (npm publication) — only if browser-side
  registry tooling needs the npm package before another integration path exists.
- `linksplatform/doublets-rs#22` and related build issues — only if a later phase
  depends on optional features that reproduce them.

Open a new upstream issue only when a phase hits a concrete missing feature that
cannot be worked around locally without distorting the architecture.

## Requirement Mapping

- R5–R8: `ProblemFrame`, need records, evidence policy, need-satisfaction ledger.
- R9: recursive `WorkUnit` records + Phase 1B + agentic todo planning (R314).
- R10, R22: the evidence pipeline.
- R11: method registry + skill records + the recursive algorithm.
- R12: gated self-improvement (Phase 9) and skill promotion rules.
- R13, R14, R16: compatibility/`compare` mode, parity tests, staged retirement.
- R15: keeping `.lino`, cache, meanings, overrides, source cache first-class.
- R17: phase-sized PRs and commits.
- R18: Voyager mapping without neural dependency
  ([recursive-core.md](recursive-core.md)).
- R19, R20: downward decomposition + upward construction
  ([recursive-core.md](recursive-core.md)).
- R21: method/skill records wrapping Rust/crate/stdlib calls.
- R23: link-native records re-anchored to `VISION.md:44`.
- R24: Phase 8 algorithm-as-data integration.
- R25, R26: the upstream audit + future upstream-issue gate.
- R27: detailed phase deliverables, tests, exit/pause criteria, comparison
  harnesses, and the companion deep-dive docs.
- New rows R330–R335 proposed in [alignment.md](alignment.md).

## Risk Register

1. **Handler precedence regressions.** Mitigation: reconstruct precedence from
   registry metadata; run `compare` mode to zero divergence before any flip.
2. **Hardcoded language logic moves but is not more general.** Mitigation: store
   meanings/preconditions as data; test multilingual and variation prompts.
3. **Recursive units too heavy for simple chat.** Mitigation: direct prompts
   produce one root + one atomic leaf with minimal trace; `atomicity_policy`
   default reproduces today.
4. **Fresh-data policy creates flaky tests.** Mitigation: separate live web from
   deterministic cache fixtures; offline mode blocks fresh-data needs.
5. **Rust/JS worker drift (the weak flank).** Mitigation: a wired parity check for
   every migrated family; most `experiments/*-parity.mjs` are not in CI today.
6. **Algorithm-as-data outruns upstream APIs.** Mitigation: keep Rust hooks until
   round-trips are proven; honor Phase 8 pause criteria.
7. **Self-improvement becomes unsafe.** Mitigation: proposal-only by default with
   tests, benchmarks, and human review (C3).
8. **Total-closure / file-size gates fail on new data.** Mitigation: author
   `.lino` closed; shard large traces; respect the 1000/1500-line caps.

## PR Review Notes To Surface

The PR description / review comment should state:

- This PR is still planning-only (draft).
- The plan is now spread across a spine + eight companion docs, roughly doubling
  the prior detail, with options compared and comparison harnesses specified.
- The core is recursive and bidirectional through `WorkUnit` records; the loop
  shape is unchanged.
- New names map onto canonical VISION/REQUIREMENTS terms (C1); link-native
  statements are re-anchored to `VISION.md:44` (C7).
- Issue 559 closes the ROADMAP Pillar 20 residual and advances Pillar 7 (C2/C4).
- Self-modification stays within the existing proposal-only gate (C3).
- The verification matrix includes total-closure, no-hardcoded-NL, traceability,
  loop-event compatibility, recipe grounding, and parity (C6).
- The upstream audit found no blockers for the next behavior-preserving phases;
  no new upstream issue was created.
