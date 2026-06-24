# Issue 559 — Implementation Results and Deep Case-Study Analysis

Status: implemented. This document closes the loop the planning artifacts opened.
The planning documents in this directory ([solution-plan.md](solution-plan.md),
[recursive-core.md](recursive-core.md), [evidence-pipeline.md](evidence-pipeline.md),
[options-comparison.md](options-comparison.md)) describe the intended migration.
This document records what actually shipped in PR #560, grounded in the source
that landed and in real artifacts the running engine emits.

All data here is reproducible offline with no network and no neural inference:

```text
cargo run --example issue_559_meta_core
```

The captured output is checked in at
[raw-data/meta-core-artifacts.txt](raw-data/meta-core-artifacts.txt) and every
quotation below is copied verbatim from that run.

## What shipped (R330–R344)

The general meta core is now a fixed pipeline every request passes through before
method execution. Its recorder stages append explicit Links Notation artifacts
into the append-only `EventLog`, and the same `MethodRegistry` is consumed by the
live registry-backed executor in `src/meta_method_dispatch.rs::try_dispatch`.
The phases were landed and committed independently:

| Row | Phase | Artifact | Module | Loop event(s) | Commit |
|-----|-------|----------|--------|---------------|--------|
| R330 | 1A | Problem frame: the request as an explicit set of needs | `src/meta_frame.rs` (`ProblemFrame`, `Need`, `NeedStatus`) | `problem_frame` | 354a7c72 |
| R332 | 1B | Recursive, bounded work-unit tree: decompose until atomic | `src/meta_frame.rs` (`WorkUnit`, `AtomicityReason`, `decompose_once`) | `work_unit:enter`, `work_unit:exit` | 353728e8 |
| R333 | 2 | Need-satisfaction ledger: one row per need with status | `src/meta_frame.rs` (`NeedLedger`, `LedgerRow`) | `need:status` | 6082c156 |
| R331 | 3 + live swap | Method registry: catalogue derived from live dispatch and used by the registry-backed executor | `src/method_registry.rs` (`MethodRegistry`, `Method`, `MethodSurface`), `src/meta_method_dispatch.rs` (`try_dispatch`) | `method_registry`, `method_registry:count`, selected `method` | e7ae154a + this PR |
| R334 | 4 | Solution evidence: the join `need → leaf → status → method` | `src/solution_evidence.rs` (`SolutionEvidence`, `EvidenceTrail`) | `solution_evidence`, `solution_evidence:accounted_for` | 5ebdb999 |
| R335 | — | Self-describing recursive-core recipe as link data | `data/meta/recursive-core-recipe.lino` | (data; not a loop event) | c619c447 |
| R336 | 4 | Route→method alias bridge: a coarser/finer intent slug still resolves to a catalogued method | `data/meta/route-method-aliases.lino`, `src/route_method_alias.rs` (`RouteMethodAlias`), consumed by `MethodRegistry::method_for_route` | (data + `solution_evidence` `method_via_alias`) | (this PR) |
| R337 | 5 | White-box recursive reasoning: a downward/upward thought per work unit | `src/meta_reasoning.rs` (`WorkUnitReasoning`) | `work_unit_reasoning`, `work_unit_reasoning:steps` | (this PR) |
| R338 | 5 | Upward construction pass + `recursion_mode` knob: the post-order leaf→root composition, the construction half of the recursion | `src/meta_construction.rs` (`RecursionMode`, `UpwardConstruction`, `ConstructionStep`), gated by `SolverConfig::recursion_mode` | `upward_construction`, `upward_construction:steps` | (this PR) |
| R339 | 6 | Method-selection trace + `selection_mode` knob: per atomic leaf, the method the single data-driven registry authority resolves (or `unresolved`), with resolved/unresolved counts | `src/selection.rs` (`SelectionMode` Off/Record, `LeafSelection`, `MethodSelection`), resolving each leaf through `MethodRegistry::method_for_route`, gated by `SolverConfig::selection_mode` | `selection` | (this PR) |
| R340 | 9 | Gated meta self-improvement loop: the algorithm reads its own recipe against the live pipeline and proposes the updated recipe as links | `src/meta_self_improvement.rs` (`SelfImprovementMode`, `PipelineStage`, `MetaRecipeProposal`, `MetaSelfImprovement`), reading `data/meta/recursive-core-recipe.lino` against `src/meta_core.rs` | (proposal data; gated, not a hot-path loop event) | (this PR) |
| R341 | 5 | Cue lexicon: the hardcoded natural-language recognition cues moved out of inline Rust literals into reviewable link data | `data/meta/cue-lexicon.lino` (`cue_set` records), `src/cue_lexicon.rs` (`CueMatch`, `CueSet`), consumed by `src/intent_formalization.rs` (`append_prompt_relevants`, `looks_arithmetic`, `looks_like_text_manipulation`) | (data; recognition cues, not a loop event) | (this PR) |
| R342 | 7 | Skill-accumulation ledger: each satisfied need distilled into a proposed reusable skill and each blocked need into a curriculum item, proposal-only and gated (no skill ever auto-promoted) | `src/skill_ledger.rs` (`SkillMode`, `SkillStatus`, `PromotionGate`, `CandidateSkill`, `CurriculumItem`, `SkillLedger`), distilling `SolutionEvidence`, gated by `SolverConfig::skill_mode` | `skill_ledger`, `skill_ledger:promotable` | (this PR) |
| R343 | 8 | Recipe interpreter: the recipe runs as an executable program, driving the live recorder primitives in the order the data declares and proving the event log is identical, event-for-event, to the hand-written pipeline's | `src/recipe_interpreter.rs` (`RecipeStep`, `RecipeProgram`, `ExecutionTrace`, `from_lino`, `execute`, `recorder_sequence`, `reproduces_pipeline`), executing the recorders bound via the recipe's `records` fields against `src/meta_core.rs` | (executes the same loop events as `record_meta_core`) | (this PR) |
| R344 | — | Total migration: the legacy route mapper and the dispatch-parity audit scaffolding are **removed outright** once the certificate proved the registry was a behavior-preserving replacement; the registry is the sole dispatch authority and the corpus-wide closure invariant is pinned directly against it | removed `src/dispatch_parity.rs` and `specialized_handler_name`; closure invariant verified by `tests/unit/specification/method_registry.rs::the_registry_is_the_sole_authority_that_closes_over_the_route_corpus` over a corpus grounded in `MethodRegistry::from_dispatch`, `route_method_alias::aliases`, and `seed::intent_routing` | (no event; migration complete) | (this PR) |

The recorder wiring lives in `src/meta_core.rs` (`record_meta_core`), which the
solver loop (`src/solver.rs`) invokes as a single cohesive pass before the
existing `search:local` step: it records the problem frame, then
`record_work_units`, `record_need_ledger`, `record_method_registry`,
`record_work_unit_reasoning`, `record_upward_construction`,
`record_solution_evidence`, and `record_selection` in sequence. Method execution
then goes through `src/meta_method_dispatch.rs::try_dispatch`, which orders names
from the same registry and records the selected `method`.
Which recursive directions are reasoned about — the downward decomposition
(R337), the upward construction (R338), or both — is selected by
`SolverConfig::recursion_mode` (default `Down`); whether the registry's
per-leaf method-selection trace is recorded (R339) is selected by
`SolverConfig::selection_mode` (default `Off`, which records nothing). Both
knobs default to leaving optional trace artifacts quiet while the live executor
uses the registry-backed method path.

## Walking the artifacts for three prompt shapes

The example drives three deliberately chosen shapes — a single routed need, a
conjunction of two needs, and an unroutable need — because those are the cases
the ledger and the evidence join must handle honestly.

### 1. A single routed need — `translate apple to Russian`

The frame detects one need and carries the route `translation`:

```text
problem_frame_d3cef1704a2f498d
  record_type "problem_frame"
  need_count "1"
  route "translation"
  need "problem_need_9d4753ec069ff65a"
```

The work-unit tree is a single atomic leaf — there is nothing to decompose, so
the atomicity predicate fires immediately with reason `direct_method`:

```text
work_unit_1653752b66b67893
  depth "0"
  atomic "true"
  atomicity_reason "direct_method"
  route "translation"
```

The ledger records one satisfied row, linked back to the resolving leaf, and the
evidence join confirms the full chain is connected and resolves to a catalogued
method:

```text
problem_need_9d4753ec069ff65a
  record_type "evidence_trail"
  status "satisfied"
  connected "true"
  work_unit "work_unit_1653752b66b67893"
  route "translation"
  method "translation"
```

`accounted_for=true, fully_resolved=true` — every detected need is addressed, and
the audit is one record rather than four projections a reader must reconcile.

### 2. A conjunction — `translate apple to Russian and write a hello world program in Python`

The frame splits the request into two needs (`need_count "2"`), and — this is the
decomposition the issue asks for — the root work unit is **not** atomic
(`atomic "false"`, `atomicity_reason "not_atomic"`) and recurses into two atomic
children:

```text
work_unit_16efab3092bb4d01
  depth "0"
  atomic "false"
  atomicity_reason "not_atomic"
  child "work_unit_ddbb9068def90fae"
  child "work_unit_751ee4a5ba055be3"
```

Each child is then a `direct_method` leaf at `depth "1"`. The ledger shows both
needs satisfied, and the evidence join produces one trail per need
(`trail_count "2"`), both `connected "true"`.

### 3. An unroutable need — `zzqqx unfathomable gibberish token`

This is the case that proves the core never silently drops a need. The ledger row
is `blocked`, and the evidence trail records it with an explicit status instead of
omitting it:

```text
  status "blocked"
```

`accounted_for` stays honest here: the trail is recorded with a non-pending
status, but `fully_resolved` is `false`. The two-tier distinction —
`accounted_for` (every need has a connected, non-pending status) versus
`fully_resolved` (every need is `Satisfied`) — is what lets the audit be both
complete and truthful: a request can be fully accounted for while openly
admitting one part is blocked.

## The method registry, grounded in live code

`MethodRegistry::from_dispatch()` derives the catalogue directly from the live
`SPECIALIZED_HANDLERS` table and `CONTEXTUAL_HANDLER_NAMES` constant in
`src/solver_dispatch.rs`, so it cannot drift from the executable dispatch by
construction. The captured run reports:

```text
  method_count "60"
  prelude_count "5"
  specialized_count "50"
  contextual_count "5"
```

The specification tests (`tests/unit/specification/method_registry.rs`) go one
step further and assert each derived name appears in the dispatch source text
(`("name",` for the specialized table, `"name" =>` for the contextual override
arms), so removing a match arm without updating the table fails a test.

## A real finding, then its fix: the route↔method vocabulary gap (R336)

The evidence pipeline immediately surfaced a genuine, previously-invisible gap.
In the conjunction case, the second need routes to `write_program`, but the
dispatch handler that serves it is named `write_script`. The routing vocabulary
(`FormalizationCandidate.route`, e.g. `write_program`) is coarser and distinct
from the dispatch handler names the registry is built from, so the chain
`need → method` did not close for it: the evidence recorded the route but left
`method` unset (`trail_count "2"`, `resolved_to_method "1"`), since the honest
behavior is to record the route rather than fabricate a link.

This is exactly the kind of latent inconsistency the unified evidence projection
was meant to expose: before R334 these two vocabularies lived in separate
subsystems and nothing forced them to be reconciled.

**Resolution (shipped in this PR, R336):** the gap is now closed by a
route→method alias map expressed as **link data, not code** —
`data/meta/route-method-aliases.lino` (`route_method_alias` records) loaded by
`src/route_method_alias.rs` and consumed by a single resolver,
`MethodRegistry::method_for_route` (direct match first, then alias). The evidence
join now resolves each need's route through it and records `method_via_alias`
provenance, so the program-writing need resolves to its method and the count
closes:

```text
  trail_count "2"
  resolved_to_method "2"
```

```text
problem_need_…
  record_type "evidence_trail"
  status "satisfied"
  connected "true"
  route "write_program"
  method "write_script"
  method_via_alias "true"
```

That same route→method bridge is now used by the live registry dispatcher, so an
aliased route can promote and execute the correct method instead of only enriching
the evidence audit. The map is kept grounded by
`tests/unit/specification/route_method_alias.rs` — every alias target must be a
real registered method and every alias must be *necessary* (its route slug is not
already a method name) — so it can never drift into stale or redundant entries.

## White-box recursive reasoning (R337)

The work-unit tree records *what* the meta core did at each node — the span, its
depth, whether it was atomic, and its route. R337 adds *why*:
`src/meta_reasoning.rs` (`WorkUnitReasoning::for_unit`) walks the tree and
attaches a human-readable thought to every recursive step, mirroring the
recursion in both directions. For the conjunction, the root reasons downward and
each leaf reasons to the method it resolves to (through the same
`method_for_route` bridge the evidence join uses):

```text
work_unit_…                          # the root
  decision "decompose"
  downward_rationale "The span carries more than one need, so it is not solvable by a single method: decompose it into 2 sub-units and reason about each recursively."
  upward_rationale "Once all 2 children are solved, compose their results in source order into this unit's answer; the answer is complete iff every child's is."
work_unit_…                          # the program-writing leaf
  decision "direct_method"
  downward_rationale "The span is directly solvable: its route resolves to the registered method `write_script`. Invoke that method; no further decomposition is needed."
  upward_rationale "Return the method's result directly; there are no children to compose."
  method "write_script"
```

The reasoning is a parallel tree to the work-unit tree (one step per unit),
serialized to Links Notation and emitted as the trace-only `work_unit_reasoning`
/ `work_unit_reasoning:steps` events, so the white box is inspectable by users and
developers — the reasoning, not just the predicate. It is verified by
`tests/unit/specification/meta_reasoning.rs`, which pins the shape, the downward
and upward thoughts, the decision slugs, the method resolution, and the
trace-only contract (building the reasoning mutates neither the unit tree nor the
resolved methods).

## The upward construction pass (R338)

Decomposition is only half of a recursive algorithm. The downward pass (R332/R337)
splits a request into a work-unit tree and explains *why*; the **upward pass**
(`src/meta_construction.rs`) composes the children's results back into each
parent's answer, leaf to root — the construction half of the recursion. It is a
post-order (bottom-up) walk of the same tree: every leaf is a base case
(`kind "leaf_method"`, constructed directly from the method that resolves its
route through the same `method_for_route` bridge the evidence join uses), and
every parent is a recursive case (`kind "compose"`, composing its
already-constructed children in source order), terminating at the root. For the
conjunction *"translate apple to Russian and write a hello world program in
Python"* the pass is three steps — both leaves first, the root last:

```text
upward_construction
  record_type "upward_construction"
  root_id "work_unit_16efab3092bb4d01"
  step_count "3"
…
  order "1"  kind "leaf_method"  method "translation"
  order "2"  kind "leaf_method"  method "write_script"
  order "3"  kind "compose"      input "…ddbb…"  input "…751e…"
```

Which directions the meta core emits is governed by the `RecursionMode` knob
(`Down` | `Up` | `Both`), surfaced as `SolverConfig::recursion_mode` and the
`FORMAL_AI_RECURSION_MODE` env override. The default is `Down`, which reproduces
the pre-knob trace exactly — the upward pass is always an explicit opt-in, so the
default solver behaves identically to before this knob existed (R13). The
structural decomposition events (`work_unit:enter` / `work_unit:exit`) are always
emitted; the knob gates only the directional *reasoning* artifacts, none of which
change routing or the answer. `record_upward_construction` returns `None` when the
mode does not request the upward direction; otherwise it appends the trace-only
`upward_construction` and `upward_construction:steps` events.
`tests/unit/specification/meta_construction.rs` pins the post-order shape (orders
`1..=N`, root last, root `compose`), the leaf-vs-compose semantics, the compose
inputs as children in source order, the serialization, and the gating contract
(the answer and intent are identical across all three modes).

## The single-authority method-selection trace (R339)

The method registry (R331/R336) is the *sole* selection authority: the solver
executes specialized methods through `meta_method_dispatch::try_dispatch`, which
orders registry method names from the request's relevant tags before invoking the
matching handler. There is no second authority — the previous hardcoded
`specialized_handler_name` mapper has been **removed entirely** (see R344). For
every atomic leaf, `src/selection.rs` names the method the registry resolves
through `MethodRegistry::method_for_route` (alias-aware), or marks the leaf
`unresolved` when no method serves it.

Each leaf is therefore in one of two states — resolved (the registry names a
registered method) or unresolved — and the artifact partitions the tree into
`resolved_count` and `unresolved_count`. For the conjunction *"translate apple to
Russian and write a hello world program in Python"* the registry resolves both
leaves, the second through the R336 alias:

```text
selection
  record_type "selection"
  root_id "work_unit_16efab3092bb4d01"
  leaf_count "2"
  resolved_count "2"
  unresolved_count "0"
…
  record_type "leaf_selection"  route "translation"  method "translation"
  record_type "leaf_selection"  route "write_program"  method "write_script"
```

The `write_program` leaf resolves to `write_script` through the route→method
alias (R336): the intent slug the classifier emits is coarser than the handler
that serves it, and the registry bridges the two. The invariant the case study
pins across all three canonical prompts is that every resolved leaf names a
*registered* method, and that resolved and unresolved leaves partition the tree —
the registry never invents a method that does not exist.

Recording is governed by the `SelectionMode` knob (`Off` | `Record`), surfaced as
`SolverConfig::selection_mode` and the `FORMAL_AI_SELECTION_MODE` env override.
The default `Off` records nothing — `record_selection` returns `None` and no
`selection` event is appended — so the optional trace stays quiet by default.
`Record` appends the per-leaf selection. The live dispatch path does not depend on
recording this optional artifact: it always resolves through the same registry.
`tests/unit/specification/selection.rs` pins the resolution of each leaf shape
(routed, aliased, unroutable), that resolved and unresolved counts partition the
leaves, the mode-gated serialization, and that the answer and intent are identical
whether the mode is `Off` or `Record`.

## Self-description as data (R335)

`data/meta/recursive-core-recipe.lino` describes the meta core to itself: a
`meta_recipe` header (`topic "recursive_core"`), twelve ordered `meta_step`
records mapping each stage to its seed source file, and the `meta_function`
records naming the entry points. `tests/unit/specification/recursive_core_recipe.rs`
asserts every named function actually exists in its cited source (`fn {name}`),
so the self-description cannot rot. This is the first concrete step toward
"reason about and modify itself": the core's structure is now queryable link data
on the same footing as everything else.

## The algorithm improving itself (R340)

R335 made the recipe faithful in the *recipe → code* direction: every function it
names must exist. R340 closes the loop in the other direction and turns the
faithfulness check into a genuine self-improvement step. The headline requirement
of the issue is meta-circular — the algorithm should take *itself* (the recipe,
the algorithm encoded as Links Notation) together with what it is required to do
(the stages the live `record_meta_core` pipeline runs), both meta-language
encoded, and produce the *updated* algorithm, again link-encoded.
`src/meta_self_improvement.rs` realises exactly that, in its safest form.

The loop reads the recipe's `meta_function` citations (the algorithm as data) and
parses the `crate::<module>::record_<name>(` calls out of `src/meta_core.rs` (the
algorithm as code), both embedded at compile time so the algorithm can reason
about itself with no runtime filesystem dependency. It then compares the two and
emits a `MetaRecipeProposal` — the updated algorithm in delta form: which
`meta_function` citations to *add* (a pipeline stage the recipe does not yet
describe) and which to *drop* (a citation the pipeline no longer runs). The
proposal serializes back to Links Notation, so the output is the same kind of
meta-language data as the input:

```text
meta_recipe_proposal
  record_type "meta_recipe_proposal"
  mode "propose"
  self_consistent "true"
  change_count "0"
```

It is deliberately gated and proposal-only. The default `SelfImprovementMode::Off`
returns `None` — the loop is dormant — and even in `Propose` mode it never writes
the recipe back: adoption stays a human review step, exactly as the issue #364
learning loop (`src/self_improvement.rs`) stops at *proposing* seed rules (R12,
C3). So R340 changes neither routing nor the answer.

This is not a toy: the loop found a real drift on the way in. The pipeline calls
`record_solution_evidence` (R334), but the recipe did not cite it — the
self-description had silently fallen behind the code. The loop surfaced that as a
proposed addition (`add_record_solution_evidence`, `source_file
"src/solution_evidence.rs"`), and adopting it — adding the `fn_record_solution_evidence`
citation to the recipe — is what makes the checked-in sources self-consistent
today. `tests/unit/specification/meta_self_improvement.rs` pins both halves: the
loop detects synthetic drift (a proposed addition and a proposed removal, each
serialized), and on the live recipe-and-pipeline it is now self-consistent
(`change_count "0"`), which is the regression guard that keeps the recipe honest
in the code → data direction from here on.

## Recognition cues as data, not Rust literals (R341)

The meta core's first move is to turn a message into a problem frame, and part of
that is recognizing which handler family a phrase points at. Historically the cue
lists that drove that recognition were inline Rust string literals inside
`src/intent_formalization.rs`: a literal `["+", "-", "*", "/", "plus", …]` for
arithmetic, `["search", "google", "find"]` for web search, the fourteen
text-manipulation operations, the Russian/English calendar fallback verbs, and so
on. The issue asks to generalize *away* from hardcoded specific intents, and
R97/R103 already moved most of the surface vocabulary into seed data. R341 finishes
that migration for the meta core's own cue lists.

`data/meta/cue-lexicon.lino` now holds every cue as a reviewable link, grouped into
named `cue_set` records that each declare how they are matched:

```text
cue_arithmetic_operators
  record_type "cue_set"
  name "arithmetic_operators"
  handler "arithmetic"
  match "substring"
  cue "+"
  cue "-"
  cue "plus"
  …
```

`src/cue_lexicon.rs` parses that data once (the same `OnceLock` + `parse_lino`
pattern as the route→method aliases, R336) and exposes `matches(set, haystack)` and
`cues(set)`. The three match modes mirror the predicates they replaced exactly:
`token` is whitespace-bounded for Latin/Cyrillic and substring for CJK — the same
`contains_token` logic that keeps "book" from matching inside "books" — while
`substring` is a raw `contains` and `prefix` is `starts_with`. The Rust call sites
keep only the structural glue: `looks_arithmetic` still requires a digit *and* an
operator cue (now read from `cues("arithmetic_operators")`); the calendar promotion
still conjoins a date signal with a schedule verb; `looks_like_text_manipulation`
is now a one-line `cue_lexicon::matches("text_manipulation", …)`.

The migration is behavior-preserving by construction — the data contains exactly
the strings the literals did — and `tests/unit/specification/cue_lexicon.rs` pins
that on both ends: it asserts every cue set the code consults exists with its
expected match mode, that the migrated cue *contents* reproduce the old lists
verbatim, and that representative prompts still surface the same `handler:*`
relevants (so routing is unchanged), including the "free programming books" → not a
calendar event regression. Adding a trigger word for an existing handler family is
now a one-line data edit rather than a Rust change — and when the meta algorithm
later reasons about its own recognition surface (R340), this is the editable cue
vocabulary it reads.

## Accumulating skills and a curriculum (R342)

The previous stages prove, per request, which needs the core resolved and which it
could not. A system meant to "improve itself" must turn that outcome into *learning*
the next request can reuse — the deterministic analog of an agent that grows a skill
library and a curriculum of what it still cannot do. R342 adds that as the twelfth
step, a pure projection of the solution evidence (`src/skill_ledger.rs`):

- every need that was **satisfied** by a catalogued method becomes a proposed
  `CandidateSkill` — a named, reusable capability the solver demonstrably has,
  captured with the span that demonstrated it and the work-unit leaf it came from; and
- every need that was **blocked** (no method resolved it, or its chain never
  connected) becomes a `CurriculumItem` — a recorded gap to close, with a reason,
  never a silently dropped failure.

Crucially, accumulation is **proposal-only and gated**, exactly like the meta
self-improvement loop (R340). A candidate skill is born `proposed`, and its
`PromotionGate` requires both tests *and* a benchmark delta before it may become
`stable`. Neither exists at trace time, so `promotable_count()` is structurally
always zero: nothing is ever auto-promoted without human review (C3). The default
`SkillMode::Off` (env `FORMAL_AI_SKILL_MODE`) records nothing, so the trace and the
answer are exactly what shipped before this stage existed (R13); `accumulate` emits
the ledger as the trace-only `skill_ledger` event plus a `skill_ledger:promotable`
count that is the auditable proof of the no-auto-promotion invariant.

`tests/unit/specification/skill_ledger.rs` pins the gate (off by default, slug
round-trip), that a satisfied need yields exactly one proposed skill while a blocked
need yields a curriculum item, that every evidence trail is accounted for as one or
the other (nothing dropped), the promotion rule (`has_tests && has_benchmark_delta`),
the never-auto-promotes invariant across prompt shapes, and the serialization. The
recipe (`data/meta/recursive-core-recipe.lino`) gains the twelfth `meta_step` and the
`from_evidence` / `record_skill_ledger` `meta_function` records, so the
self-improvement loop stays self-consistent. This is the foundation for the core to
later reason about and extend its own method set from its own accumulated experience.

## Executing the recipe as a program (R343)

R335 made the recursive core's structure grounded link data, and R340 had the
algorithm *read* that recipe against the live pipeline to propose changes. But a
recipe that is only ever read is still just a checked description — the proof that
it *is* the algorithm rested on a name-by-name comparison, not on running it. R343
closes that gap: the recipe is now an **executable program**.

Each trace-recorded step in `data/meta/recursive-core-recipe.lino` gains a `records`
field naming the recorder primitive it drives (`build_problem_frame` →
`record_problem_frame`, … `accumulate_skills` → `record_skill_ledger`); the three
external steps (`formalize_impulse`, `resolve_leaves`, `project_answer`) carry no
binding because they happen outside the trace loop. `src/recipe_interpreter.rs`
parses those steps into an ordered `RecipeProgram` and **runs** it: walking the
steps in declared order, it invokes each bound recorder against the same live
primitives the hand-written pipeline uses, threading the intermediate artifacts
(problem frame → work-unit tree → need ledger → method registry → solution
evidence) exactly as `record_meta_core` does, and honoring the same mode gates
(`recursion_mode` for upward construction and white-box reasoning, `selection_mode`
for the comparison, `skill_mode` for the ledger).

The headline guarantee is **parity, proven by execution**:
`RecipeProgram::reproduces_pipeline` runs the recipe and `record_meta_core` on fresh
event logs for the same input and compares them — and the two logs are identical,
event-for-event, across every one of the 3×3×2 recursion/selection/skill mode
combinations for all three prompt shapes. The recipe's recorder order also equals,
position for position, the live pipeline's actual stage order (cross-checked against
`MetaSelfImprovement::pipeline_stages`). Divergence cannot pass silently: a step
placed before its dependency (e.g. the need ledger before the problem frame) or a
binding to a recorder that does not exist is rejected with an explicit error rather
than a wrong trace. This is the concrete sense in which the algorithm-as-data and
the algorithm-as-code are *the same algorithm*. The interpreter executes the same
recorder events the pipeline already emits; method execution remains in the
registry-backed executor.

`tests/unit/specification/recipe_interpreter.rs` pins all of this: the twelve
contiguously-ordered steps, the nine-recorder subsequence matching the live
pipeline, event-for-event reproduction in default modes and under every mode
combination, which stages run versus are skipped (external and mode-gated), the
dependency-ordering and unknown-binding errors, and the Links Notation
serialization. The recipe also gains the `from_lino` / `execute` `meta_function`
records so the self-improvement loop continues to see a faithful self-description.

## Retiring the legacy dispatch authority outright (R344)

Earlier in this work an interim `DispatchParity` certificate audited the
data-driven registry against the legacy `specialized_handler_name` mapper across
the *entire route vocabulary the system can ever emit* — every registered method
name (a method is its own self-resolving route), every route→method alias (R336),
and every classifier route slug from `seed::intent_routing` — and proved **zero
contradictions**: wherever the legacy authority named a real method, the registry
named the *same* one, and the registry additionally rescued routes the legacy
catch-all could not serve (e.g. `write_program` → `write_script`). That certificate
established the registry was a behavior-preserving replacement.

With that proof in hand, the migration was completed by **removing the legacy
authority and its audit scaffolding entirely** — `src/dispatch_parity.rs` and
`intent_formalization::specialized_handler_name` are gone. Keeping a retired
mapper "as a baseline" would be exactly the obsolete-by-example code the issue
asks us to eliminate; once the registry is proven equivalent, the only correct
state is one authority. The registry (`MethodRegistry::method_for_route`,
alias-aware) is now the sole route→method resolver and the only live dispatch
path (`meta_method_dispatch::try_dispatch`).

The closure invariant the certificate guaranteed survives directly against the
live registry, with no second authority to compare against.
`tests/unit/specification/method_registry.rs::the_registry_is_the_sole_authority_that_closes_over_the_route_corpus`
rebuilds the same corpus from live data (`MethodRegistry::from_dispatch`,
`route_method_alias::aliases`, `seed::intent_routing`) and pins two facts: (1)
**closure safety** — no route the system can emit ever resolves to an
*unregistered* method, the property that made the registry a safe drop-in; and (2)
**coverage** — every method-name route and every alias route resolves (exactly the
routes the legacy authority also resolved, so the registry loses no coverage).
Classifier slugs with no handler may legitimately stay unresolved, exactly as under
the old certificate's four-way partition.

## Verification and traceability

- Grounding tests: `tests/unit/specification/{meta_frame,method_registry,recursive_core_recipe,solution_evidence,route_method_alias,meta_reasoning,meta_construction,selection,meta_self_improvement,cue_lexicon,skill_ledger,recipe_interpreter}.rs`.
- Requirement traceability: `tests/unit/docs_requirements_issue_559.rs` ties each
  REQUIREMENTS.md row (R330–R344) to its source module, named entry points, and
  solver wiring; a row that loses its implementation fails CI.
- Backward compatibility: every change is additive (new modules, new optional
  `LedgerRow` fields, new trace events). The full unit suite passes with the new
  tests added and no prior tests modified to accommodate them.

## How this answers the issue

- **One general recursive meta algorithm:** every request now flows through the
  same frame → recursive decomposition → ledger → registry → evidence pipeline,
  then through the registry-backed method executor, regardless of which
  specialized handler ultimately answers.
- **Translate to a meta language and work on it:** the meta language is Links
  Notation; each stage emits its artifact as `.lino` records.
- **Detect all needs and ensure each is addressed:** the frame enumerates needs,
  the ledger gives each a status, and the evidence join makes "every need
  addressed" a single auditable fact (`accounted_for` / `fully_resolved`).
- **Reason about and modify itself:** R335 makes the core's own structure
  grounded link data with tests that keep the description faithful, R337
  attaches white-box recursive reasoning to every step so the box is inspectable
  by users and developers, not just the predicate, R338 records the upward
  construction pass so both directions of the recursion — decompose and compose —
  are explicit, inspectable link data, R339 records the registry's per-leaf
  method-selection trace, making the dispatch the sole authority performs
  auditable per request, and R340 closes the
  meta-circular loop: the algorithm reads its own recipe
  against the live pipeline and emits the updated recipe as links — gated and
  proposal-only, so it can reason about modifying itself without a human losing the
  final say (it already caught and fixed one real drift).
- **Generalize away from hardcoded specific intents:** R341 lifts the meta core's
  hardcoded natural-language recognition cues out of inline Rust literals into a
  reviewable cue lexicon (`data/meta/cue-lexicon.lino`), so the recognition surface
  is editable data rather than scattered string lists — behavior-preserving, and a
  new trigger word for an existing handler family is now a one-line data edit.
- **Accumulate skills and learn what is missing:** R342 distils each request's
  outcome into a proposed reusable skill per satisfied need and a curriculum item
  per blocked need — proposal-only and gated so no skill is ever auto-promoted
  without tests and a benchmark delta (C3) — the deterministic analog of an agent
  that grows a skill library and a curriculum, and the foundation for the core to
  reason about and extend its own method set.
- **Algorithm as executable data:** R343 turns the self-describing recipe from a
  checked description into a runnable program — `src/recipe_interpreter.rs` executes
  the recipe's steps against the live recorder primitives and proves the resulting
  trace is identical, event-for-event, to the hand-written pipeline's across every
  mode combination, so the algorithm-as-data and the algorithm-as-code are provably
  the same algorithm (the groundwork for driving the pipeline from its own recipe).
- **Replace the direct dispatch loop:** `src/meta_method_dispatch.rs` executes
  solver methods through `MethodRegistry::ordered_method_names_for_relevants` and
  records the selected method; R344 completes the migration by retiring the legacy
  route mapper and its parity scaffolding outright, leaving the registry as the
  sole dispatch authority, with the corpus-wide closure invariant pinned directly
  against the live registry.
- **Preserve caches, overrides, meanings, and `.lino` files:** untouched; the new
  artifacts are additive trace events and new data files.
- **Compile data and do deep case-study analysis:** this document plus the
  reproducible [raw-data/meta-core-artifacts.txt](raw-data/meta-core-artifacts.txt).
