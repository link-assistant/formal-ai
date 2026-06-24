# Issue 559 Case Study: General Meta Algorithm

Status: implemented in PR #560. The shipped result is a live registry-backed meta
method path plus the link-native recursive/audit artifacts R330–R344 describe;
the deep, data-grounded analysis is in [implementation-results.md](implementation-results.md),
reproducible offline via `cargo run --example issue_559_meta_core` with output
captured at [raw-data/meta-core-artifacts.txt](raw-data/meta-core-artifacts.txt).

Update: two rounds of PR feedback on 2026-06-23 shaped this case study. The first (comment 4783154352) requested a deeper plan that integrates Voyager without a neural runtime, makes the solver recursively decompositional and compositional, stays link-native, treats algorithms as data, and audits related upstream dependencies. The second (comment 4783640128) asked to make the plan at least twice as detailed, critically check everything, compare multiple options (implementing them all where feasible), and re-check everything against the vision, requirements, and roadmap. This case study now includes that expanded second planning pass across a spine plus eight companion documents.

## Source Material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/559>
- Prepared pull request: <https://github.com/link-assistant/formal-ai/pull/560>
- PR feedback integrated (round 1): <https://github.com/link-assistant/formal-ai/pull/560#issuecomment-4783154352>
- PR feedback integrated (round 2): <https://github.com/link-assistant/formal-ai/pull/560#issuecomment-4783640128>
- Local inspiration image: [inspiration.png](inspiration.png)
- Raw GitHub and search data: [raw-data/](raw-data/)
- External research notes: [raw-data/online-research.md](raw-data/online-research.md)
- Upstream dependency audit: [upstream-dependency-audit.md](upstream-dependency-audit.md)

The inspiration image sketches a universal problem-solving loop:

1. Start from a problem.
2. Decompose it into tasks.
3. Derive tests and experiments for the tasks.
4. Generate draft candidates.
5. Select and compose candidates into a final solution.

This closely matches the repo vision in `VISION.md`, `GOALS.md`, and `ARCHITECTURE.md`: formalize every request, decompose when needed, synthesize candidates, validate, simplify, and record what happened.

## Planning Findings

The repository already has many pieces of the requested architecture:

- `UniversalSolver` records formalization, decomposition, candidate, validation, simplification, and presentation events.
- `docs/meta-algorithm.md` documents the procedural-how-to and agentic-coding meta algorithms and their grounding tests.
- `data/meta/*.lino`, `data/seed/*.lino`, meanings, cache, overrides, and source cache files already make part of the behavior data-driven.
- `docs/design/no-hardcoded-natural-language.md` states the design constraint that code should move meanings rather than embed natural-language triggers.
- `docs/design/self-improvement-loop.md` already defines a review-gated learning loop for unknown traces.

The main implementation result is that the control plane now has one registry
method path:

- `src/method_registry.rs` derives prelude, specialized, and contextual method
  records from live dispatch constants.
- `src/meta_method_dispatch.rs::try_dispatch` orders and executes method names
  through that registry, including alias-resolved routes like
  `write_program -> write_script`.
- `src/selection.rs` and `src/dispatch_parity.rs` keep the older route mapper as
  an audit baseline, proving the registry never contradicts a valid legacy
  method.
- `data/meta/recursive-core-recipe.lino` is the general problem-frame recipe that
  every request passes through and that the interpreter can execute event for
  event.

The planning documents remain in this directory for traceability; the current
architecture is recorded in [implementation-results.md](implementation-results.md).

The refined plan adds four constraints from the follow-up feedback:

- Represent problems, tasks, dependencies, evidence, methods, objects, files, sequences, and algorithms as links or link networks, not as a separate non-link ontology. This is anchored to `VISION.md:44` ("Doublet links are the primitive storage model for this project"), with the meta-theory article kept as a cited external influence rather than repo doctrine (see [alignment.md](alignment.md) conflict C7).
- Make solving recursive: decompose until a work unit is directly solvable, while also searching upward from existing libraries, standard functions, repo code, and accumulated skills.
- Adapt Voyager's open-ended curriculum, skill library, execution feedback, and critic loop into deterministic, review-gated formal-ai mechanisms without adding a neural runtime dependency.
- Treat organization-owned dependencies as gates: no upstream blocker was found for the next planning-approved phases, but existing relevant issues are listed in [upstream-dependency-audit.md](upstream-dependency-audit.md).

## What The Second Pass Added

- A **critical check** of the first-session artifacts: twelve corrections (CR1–CR12) with `file:line` evidence, including the lexicon path, the location of `IntentFormalization`, the real 50-entry handler table, and the fact that multi-provider web search and Reciprocal Rank Fusion are already built (so only crawl/extract/compare and non-CORS providers are genuinely absent). See [critical-review.md](critical-review.md).
- A **strategic re-check** against VISION/GOALS/NON-GOALS/ROADMAP/REQUIREMENTS/ARCHITECTURE that resolves seven alignment conflicts (C1–C7), introduces a canonical vocabulary mapping (the new names are explicit forms of existing concepts, not new primitives), positions the work against ROADMAP Pillars 7 and 20, and proposes new requirement rows R330–R335. See [alignment.md](alignment.md).
- An **options comparison** giving 2–4 options per major decision with pros/cons/cost/risk and, where feasible, a `SolverConfig`-knob comparison harness so competing directions can be run side by side and judged by tests and benchmarks. See [options-comparison.md](options-comparison.md).
- Deep specifications for the **recursive bidirectional core** ([recursive-core.md](recursive-core.md)) and the **evidence pipeline** ([evidence-pipeline.md](evidence-pipeline.md)), both grounded in concrete source locations.

## Files In This Case Study

- [requirements.md](requirements.md) — issue requirements, acceptance criteria, inferred constraints, the canonical vocabulary note, new obligations R28–R31, and root-requirement mapping.
- [architecture-inventory.md](architecture-inventory.md) — the relevant current architecture, grounded in `file:line`, with an "already built vs absent" view.
- [alignment.md](alignment.md) — strategic alignment, conflicts C1–C7 and their resolutions, canonical vocabulary mapping, proposed root rows R330–R335.
- [critical-review.md](critical-review.md) — corrections CR1–CR12 with evidence; precise built-vs-absent inventory; residual risks.
- [options-comparison.md](options-comparison.md) — options per decision, recommendations, and comparison harnesses.
- [recursive-core.md](recursive-core.md) — downward/upward passes, atomicity predicate, `SolverConfig` knobs, pseudo-code, Voyager mapping.
- [evidence-pipeline.md](evidence-pipeline.md) — the general fresh-data pipeline grounded in the existing search core and fetch seams.
- [implementation-results.md](implementation-results.md) — what actually shipped
  (R330–R344 plus the live registry-backed method dispatcher), a walk through
  real emitted artifacts for three prompt shapes, the method-registry grounding,
  a real route↔method vocabulary finding, and how it answers the issue.
- [solution-plan.md](solution-plan.md) — the spine: planning status, phases 0A–10 with gates, verification matrix, requirement mapping, risk register.
- [upstream-dependency-audit.md](upstream-dependency-audit.md) — related organization dependencies and existing upstream issues.
- [raw-data/online-research.md](raw-data/online-research.md) — external research and library/component checks.
- [raw-data/meta-core-artifacts.txt](raw-data/meta-core-artifacts.txt) — the verbatim Links Notation the running meta core emits for three prompt shapes plus the full method registry, produced by `cargo run --example issue_559_meta_core`.

## Current Review Focus

Review PR #560 as the implemented migration: problem frames, recursive work
units, need ledger, method registry, solution evidence, recipe/interpreter,
selection/parity audits, cue data, skill ledger, and the live registry-backed
method executor are all in this branch.
