# Issue 559 Strategic Alignment

This document re-checks the issue 559 plan against the repository's governing
documents and resolves the conflicts found during the critical review. It exists
because the PR feedback asked to "re-check everything according to our vision,
requirements, roadmap" and to prefer "as general and universal solutions and
decisions as possible."

Every alignment claim cites a specific governing-document location. Conflicts are
labelled C1–C7 and each has an explicit resolution that the rest of the case
study follows.

## Governing Documents And What They Bind

| Document | Binding statement for issue 559 | Location |
| --- | --- | --- |
| `VISION.md` | "Doublet links are the primitive storage model for this project." | line 44 |
| `VISION.md` | "The associative network is the AI." | line 5 |
| `VISION.md` | 11-step Universal Problem-Solving Algorithm (Impulse → Presentation). | lines 84–100 |
| `VISION.md` | Configurable Solver Knobs (depth, temperature, guess probability, offline, probability policy). | lines 102–116 |
| `VISION.md` | "Every input message is first translated into Links Notation." | lines 118–124 |
| `VISION.md` | "Data Is The Interface" — intent routing is a data rule book, not code. | lines 190–194 |
| `GOALS.md` | "The shape of the loop should not branch by domain." | lines 35–43 |
| `NON-GOALS.md` | "Bypassing `SolverConfig` for hard-coded behavior is not acceptable; new knobs are added to the config first." | lines 15–23 |
| `NON-GOALS.md` | No neural runtime; no memoization-as-reasoning; no hidden autonomy; no unbounded loops. | lines 15–23 |
| `ROADMAP.md` | Vision Pillars table (status per pillar). | lines 105–141 |
| `ROADMAP.md` | Pillar 20 caveat: "`SPECIALIZED_HANDLERS` remain as a precedence table behind the formalized router." | line ~127 |
| `ROADMAP.md` | Pillar 7: "task-agnostic meta-builder ('algorithm that builds algorithms', R7)", tracked in `docs/case-studies/issue-412`. | line ~112 |
| `ROADMAP.md` | Verification Contract: changing a roadmap item updates REQUIREMENTS.md + architecture status table + ROADMAP. | line 266 |
| `REQUIREMENTS.md` | R72 — 11-step loop runs for every request without domain branching. | row at line 110 |
| `REQUIREMENTS.md` | R74 — recursively-formalized sub-impulses bounded by `max_decomposition_depth`. | row at line 112 |
| `REQUIREMENTS.md` | R157 — formalize to a Links Notation meaning record. | row at line 326 |
| `REQUIREMENTS.md` | R158 — model the task as a graph of requirements and subtasks. | row at line 327 |
| `ARCHITECTURE.md` | §2 — specialized handlers are plugged in, not branched on by domain. | lines 160–165 |
| `ARCHITECTURE.md` | §9 — five rule kinds (data → Rust → JS → compiled-as-data → NL skill). | line 643 |
| `ARCHITECTURE.md` | §12 — the answer field is a projection. | lines 840–841 |
| `docs/meta-algorithm.md` | Grounded-recipe discipline: each recipe asserted against live source by a CI test. | whole document |
| `docs/design/no-hardcoded-natural-language.md` | Natural language is data, never a string literal in the engine; four CI gates. | whole document |
| `docs/design/self-improvement-loop.md` | The only sanctioned self-modification path: proposal-only, gated by verification + benchmark + human review. | whole document |

## The Canonical Vocabulary Mapping (resolves C1)

**C1 — vocabulary divergence.** The first-session plan introduced new type names
(`ProblemFrame`, `Need`, `WorkUnit`) without connecting them to the canonical
terms the governing documents already use (impulse, requirement, sub-impulse,
candidate, validation, the 11-step loop, `SolverConfig`). It also never mentioned
`SolverConfig`, `sub_impulse`, `requirement`, or `max_decomposition_depth`. Read
literally, the plan looked like a parallel ontology — which both C1 and
`NON-GOALS.md` ("new knobs are added to the config first") forbid.

**Resolution.** The issue-559 types are **not new primitives**. They are names
for making the *existing* 11-step loop's implicit state explicit and
link-serializable. The plan adopts this mapping everywhere and treats the
canonical column as authoritative:

| Issue-559 working name | Canonical concept it makes explicit | Current carrier in code | Requirement |
| --- | --- | --- | --- |
| `ProblemFrame` | Formalized impulse + the 11-step loop envelope; the "meaning record" | `IntentFormalization` (`src/intent_formalization.rs:48`) | R157, R72 |
| `Need` | A detected requirement / sub-requirement | requirement records inside formalization; R158 "graph of requirements and subtasks" | R158 |
| `WorkUnit` | A recursively-formalized `sub_impulse` | `UniversalSolver::decompose` output | R74 |
| atomicity / depth control | Recursion bound | `SolverConfig::max_decomposition_depth` (+ a new `atomicity_policy` knob, added to config first) | R74, NON-GOALS |
| `evidence_policy` | Fresh-vs-cached source policy | `source:` provenance, `policy:offline`, `FORMAL_AI_OFFLINE`, `ProbabilityDecisionPolicy` | R67 |
| method / skill registry | Data-described selection of methods | new; extends the §9 five-rule ladder + `data/seed/intent-routing.lino` | R103, R97 |
| candidate / selected | Candidate methods + selection | candidate events in the loop | R72 |
| validation | Validation step + TDD | validation events | R72 |
| composition | Combination step | combination events | R72 |
| presentation | Simplification + answer projection | answer projection (`ARCHITECTURE.md` §12) | R72 |

Consequence for the plan: wherever a deep-dive doc says `ProblemFrame`/`Need`/
`WorkUnit`, it means "the explicit, link-serializable form of impulse /
requirement / sub-impulse." The migration extends `IntentFormalization` and the
decomposition output rather than introducing a competing structure, and every
depth/atomicity control is a `SolverConfig` knob added to config first.

## Roadmap Positioning (resolves C2 and C4)

**C2 — missing roadmap linkage.** The first-session plan did not cite the
roadmap pillar it delivers. **Resolution:** issue 559 closes the residual of
**Pillar 20** ("routing by formalized intent, not a fixed catalogue" — marked
Built, caveat: "`SPECIALIZED_HANDLERS` remain as a precedence table behind the
formalized router") and advances **Pillar 7** ("task-agnostic meta-builder —
'algorithm that builds algorithms', R7", tracked in `docs/case-studies/issue-412`).
It also operationalizes **Pillar 2** (universal loop, Built) and **Pillar 26**
(general synthesis, Built) by making their internal state explicit, and **Pillar
19** (reasoning under unknowns, Built) via the evidence policy.

**C4 — re-litigating "Built" work.** The roadmap marks Pillars 2/20/26 as Built.
The plan must not read as a rewrite of finished work. **Resolution:** every phase
that touches Built territory quotes the pillar's caveat so a reviewer sees issue
559 closes a *named residual*, not re-opens settled design. Concretely:

- Pillar 20 is Built *except* the `SPECIALIZED_HANDLERS` precedence table — the
  exact thing the method registry migration replaces.
- Pillar 2 is Built as a loop *shape*; issue 559 does not change the shape (that
  would violate `GOALS.md` "the shape of the loop should not branch by domain"),
  only makes per-step state explicit and link-serializable.
- Pillar 26 (general synthesis) is Built; issue 559 reuses it as one method among
  many in the registry, not as a competing path.

Per the **Verification Contract** (`ROADMAP.md:266`), if any phase changes a
roadmap item's status, that phase's PR step must update REQUIREMENTS.md, the
ARCHITECTURE status table, and ROADMAP together. The plan's phases include this
as an explicit checklist item where applicable.

## Self-Modification Boundary (resolves C3)

**C3 — self-modification scope.** The issue's long-term aim ("reason about and
modify itself") can be read as unbounded autonomy, which `NON-GOALS.md` forbids
("no hidden autonomy"). **Resolution, stated plainly:** the only sanctioned
self-modification mechanism is `docs/design/self-improvement-loop.md` (issue
#364): proposals only, never silently applied, gated by verification +
benchmarks + human review, and it never auto-appends to `data/seed/`. Issue 559
adds no new autonomy. "Algorithm reasons about itself" is delivered as
*inspectability* (the algorithm is data that can be read, queried, and diffed),
not as *unsupervised self-editing*. Any change to the general meta algorithm
flows through the existing proposal gate and a human PR review. This is the
content of the self-improvement surface, and that surface cannot ship without
the gate.

## Grounded-Recipe Discipline (resolves C5)

**C5 — missing grounding discipline.** The first-session plan described a general
recipe but did not say it must be grounded the way the two existing recipes are.
**Resolution:** the general meta algorithm ships as a third
`data/meta/*-recipe.lino` recipe, grounded by a CI test in the exact style of
`tests/unit/specification/meta_algorithm.rs` (#444) and `agentic_meta_algorithm.rs`
(#468). The preferred implementation parameterizes a single grounding harness
over **all** `data/meta/*-recipe.lino` files so adding a recipe automatically
adds grounding, and drift between recipe and live source fails CI. This keeps
the "algorithms as data" target (R24) honest: the data description and the code
cannot silently diverge.

## Verification Gates (resolves C6)

**C6 — missing CI gates in the plan.** The first-session verification matrix
omitted the repo's hard gates. **Resolution:** the verification matrix
([solution-plan.md](solution-plan.md) and
[critical-review.md](critical-review.md)) must include, for every phase that
adds data or changes routing:

1. **Total reference closure** — `python3 scripts/audit-total-closure.py` must
   report `unresolved_distinct: 0` (`tests/unit/total_closure.rs`).
2. **No hardcoded natural language** — the four gates in
   `docs/design/no-hardcoded-natural-language.md`, including the worker-mirror
   `--check`.
3. **Requirement traceability** — `tests/unit/docs_requirements.rs` must contain
   a new `issue_559_..._are_traceable()` asserting each new `| R<n> ` row exists.
4. **Loop-event compatibility** — `specialized_handlers_still_publish_loop_events`
   (`tests/unit/specification/reasoning_loop.rs:44`) must keep passing and be
   widened beyond arithmetic.
5. **Recipe grounding** — the general recipe's grounding test must pass.
6. **Cross-runtime parity** — Rust↔JS worker parity for any shared logic.

## Link-Native Re-Anchoring (resolves C7)

**C7 — external doctrine cited as repo doctrine.** The first-session plan leaned
on the meta-theory "point-like / relation-like are both links" phrasing, which
appears **zero times** in the canonical repo docs and is therefore an external
reference, not repo doctrine. **Resolution:** R23's link-native requirement is
re-anchored to the canonical source — `VISION.md:44` "Doublet links are the
primitive storage model for this project" and `VISION.md:5` "The associative
network is the AI." The meta-theory article remains a *cited external influence*
in [raw-data/online-research.md](raw-data/online-research.md), but every binding
link-native statement in the plan points at VISION, not at meta-theory. Frames,
needs, work units, methods, evidence, dependencies, and sequences are modelled as
doublet links because VISION says the doublet is the primitive, not because an
external article frames structures as links.

## Mapping To Existing Root Requirements

Issue 559 should be expressed as much as possible in terms of requirements the
repo already tracks, adding new rows only for genuinely new obligations.

| Root requirement | Issue-559 relevance |
| --- | --- |
| R72 (`:110`) — 11-step loop, no domain branch | The general meta algorithm *is* this loop with explicit state; do not add a domain branch. |
| R74 (`:112`) — recursive sub-impulses bounded by depth | The recursive work-unit model generalizes `decompose`; bound stays a `SolverConfig` knob. |
| R97 (`:156`) — externalize hardcoded surfaces | Move `append_prompt_relevants` / `looks_like_text_manipulation` cue lists into seed data. |
| R103 (`:175`) — intent routing from `.lino` | The method registry is the data-described generalization of intent routing. |
| R157 (`:326`) — formalize to a meaning record | `ProblemFrame` is the explicit meaning record. |
| R158 (`:327`) — graph of requirements and subtasks | The need ledger + recursive work units operationalize this. |
| R67 (`:105`) — search external + cache provenance | The evidence policy reuses `source:`/`fetched_at`/`sha256`/`cache_hit`/`policy:offline`. |
| R264 (`:634`) — white-box self-improvement | Reuse this exact mechanism; no new autonomy. |
| R311 (`:780`) — everything is a link (`format_lino_record`) | All new frames/units/registry entries serialize as links. |
| R314 (`:783`) — agentic CLI loop (`plan_chat_step`) | Big-task todo planning extends the existing agentic loop. |
| R129 (`:249`) — 5–10 variations per test case | New routing tests follow the prompt-variation harness. |

## New Requirements Added (R330+)

Per `ROADMAP.md:266` and the traceability pattern in
`tests/unit/docs_requirements.rs`, PR #560 adds rows R330-R344 to
REQUIREMENTS.md, each pinned by an `issue_559_..._is_traceable()` test.

1. **R330** — Every prompt produces an explicit, link-serializable problem frame
   (the meaning record made first-class), emitted as a loop event.
2. **R331** — Method selection is data-described: a `.lino` method/skill registry
   records prelude, specialized, and contextual method surfaces, and the live
   solver executes the selected registry ordering through
   `meta_method_dispatch::try_dispatch`.
3. **R332** — Recursive work units split a non-atomic frame until each unit is
   directly solvable by a method, library call, repo function, or reviewed skill,
   bounded by a `SolverConfig` atomicity/depth knob.
4. **R333** — A need-satisfaction ledger marks every detected need
   satisfied/deferred/blocked/rejected in the final answer projection.
5. **R334** — A general evidence pipeline (expand → search → rerank → crawl →
   extract → compare → hypothesize) runs when the evidence policy requires fresh
   data, reusing existing provenance fields.
6. **R335-R344** — The shipped recursive-core recipe, alias bridge, reasoning
   passes, method-selection trace, solution evidence, self-improvement gate,
   skill ledger, and recipe interpreter complete the migration to the
   registry-backed path as the sole dispatch authority; R344 retires the legacy
   route mapper and its parity scaffolding outright while preserving behavior.

These map back to the product requirements R5–R12 and the feedback requirements
R18–R27 in [requirements.md](requirements.md). The number of new rows is kept
small on purpose: most of issue 559 is realized through existing requirements,
honoring the "prefer general and universal" instruction by reusing the existing
contract surface rather than inventing a parallel one.

## Alignment Summary

| Conflict | Status | Where resolved |
| --- | --- | --- |
| C1 vocabulary divergence | Resolved via canonical mapping | this doc + every deep-dive |
| C2 missing roadmap linkage | Resolved (Pillars 7/20, supporting 2/19/26) | this doc + solution-plan |
| C3 self-modification scope | Resolved (proposal-only gate) | this doc + recursive-core |
| C4 re-litigating Built work | Resolved (quote caveats, Verification Contract) | this doc + solution-plan |
| C5 missing grounding discipline | Resolved (third grounded recipe) | this doc + solution-plan |
| C6 missing CI gates | Resolved (six-gate matrix) | this doc + critical-review |
| C7 external doctrine as repo doctrine | Resolved (re-anchor R23 to VISION) | this doc + requirements |
