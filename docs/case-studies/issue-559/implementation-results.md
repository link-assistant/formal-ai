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

## What shipped (R330–R337)

The general meta core is now a fixed pipeline every request passes through, ahead
of the existing specialized dispatch. Each stage is a behavior-preserving,
trace-only loop event: it records an explicit Links Notation artifact into the
append-only `EventLog` and changes neither routing nor the produced answer
(constraint R13). The phases were landed and committed independently:

| Row | Phase | Artifact | Module | Loop event(s) | Commit |
|-----|-------|----------|--------|---------------|--------|
| R330 | 1A | Problem frame: the request as an explicit set of needs | `src/meta_frame.rs` (`ProblemFrame`, `Need`, `NeedStatus`) | `problem_frame` | 354a7c72 |
| R332 | 1B | Recursive, bounded work-unit tree: decompose until atomic | `src/meta_frame.rs` (`WorkUnit`, `AtomicityReason`, `decompose_once`) | `work_unit:enter`, `work_unit:exit` | 353728e8 |
| R333 | 2 | Need-satisfaction ledger: one row per need with status | `src/meta_frame.rs` (`NeedLedger`, `LedgerRow`) | `need:status` | 6082c156 |
| R331 | 3 | Method registry: catalogue derived from live dispatch | `src/method_registry.rs` (`MethodRegistry`, `Method`, `MethodSurface`) | `method_registry`, `method_registry:count` | e7ae154a |
| R334 | 4 | Solution evidence: the join `need → leaf → status → method` | `src/solution_evidence.rs` (`SolutionEvidence`, `EvidenceTrail`) | `solution_evidence`, `solution_evidence:accounted_for` | 5ebdb999 |
| R335 | — | Self-describing recursive-core recipe as link data | `data/meta/recursive-core-recipe.lino` | (data; not a loop event) | c619c447 |
| R336 | 4 | Route→method alias bridge: a coarser/finer intent slug still resolves to a catalogued method | `data/meta/route-method-aliases.lino`, `src/route_method_alias.rs` (`RouteMethodAlias`), consumed by `MethodRegistry::method_for_route` | (data + `solution_evidence` `method_via_alias`) | (this PR) |
| R337 | 5 | White-box recursive reasoning: a downward/upward thought per work unit | `src/meta_reasoning.rs` (`WorkUnitReasoning`) | `work_unit_reasoning`, `work_unit_reasoning:steps` | (this PR) |

The wiring lives in `src/meta_core.rs` (`record_meta_core`), which the solver
loop (`src/solver.rs`) invokes as a single cohesive pass before the existing
`search:local` step: it records the problem frame, then `record_work_units`,
`record_need_ledger`, `record_method_registry`, `record_work_unit_reasoning`,
and `record_solution_evidence` in sequence, so the meta core observes the request
but does not steer it yet.
That ordering is what makes the migration safe: the artifacts are produced and
audited first; routing is moved onto them only in later, behavior-changing phases
that remain deferred (see [solution-plan.md](solution-plan.md)).

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
  method_count "55"
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

Resolution stays trace-only: it enriches the audit's `method`/`method_via_alias`
links and changes neither routing nor the answer (R13). The map is kept grounded
by `tests/unit/specification/route_method_alias.rs` — every alias target must be
a real registered method and every alias must be *necessary* (its route slug is
not already a method name) — so it can never drift into stale or redundant
entries.

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

## Self-description as data (R335)

`data/meta/recursive-core-recipe.lino` describes the meta core to itself: a
`meta_recipe` header (`topic "recursive_core"`), nine ordered `meta_step`
records mapping each stage to its seed source file, and the `meta_function`
records naming the entry points. `tests/unit/specification/recursive_core_recipe.rs`
asserts every named function actually exists in its cited source (`fn {name}`),
so the self-description cannot rot. This is the first concrete step toward
"reason about and modify itself": the core's structure is now queryable link data
on the same footing as everything else.

## Verification and traceability

- Grounding tests: `tests/unit/specification/{meta_frame,method_registry,recursive_core_recipe,solution_evidence,route_method_alias,meta_reasoning}.rs`.
- Requirement traceability: `tests/unit/docs_requirements_issue_559.rs` ties each
  REQUIREMENTS.md row (R330–R337) to its source module, named entry points, and
  solver wiring; a row that loses its implementation fails CI.
- Backward compatibility: every change is additive (new modules, new optional
  `LedgerRow` fields, new trace events). The full unit suite passes with the new
  tests added and no prior tests modified to accommodate them.

## How this answers the issue

- **One general recursive meta algorithm:** every request now flows through the
  same frame → recursive decomposition → ledger → registry → evidence pipeline,
  regardless of which specialized handler ultimately answers.
- **Translate to a meta language and work on it:** the meta language is Links
  Notation; each stage emits its artifact as `.lino` records.
- **Detect all needs and ensure each is addressed:** the frame enumerates needs,
  the ledger gives each a status, and the evidence join makes "every need
  addressed" a single auditable fact (`accounted_for` / `fully_resolved`).
- **Reason about and modify itself:** R335 makes the core's own structure
  grounded link data with tests that keep the description faithful, and R337
  attaches white-box recursive reasoning to every step so the box is inspectable
  by users and developers, not just the predicate.
- **Preserve caches, overrides, meanings, and `.lino` files:** untouched; the new
  artifacts are additive trace events and one new data file.
- **Compile data and do deep case-study analysis:** this document plus the
  reproducible [raw-data/meta-core-artifacts.txt](raw-data/meta-core-artifacts.txt).
