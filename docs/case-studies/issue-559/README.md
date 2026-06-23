# Issue 559 Case Study: General Meta Algorithm

Status: first-session planning artifact only. Issue 559 explicitly asks for a detailed planning pass before implementation, with execution to follow after maintainer approval or requested changes.

Update: PR feedback on 2026-06-23 requested a deeper plan that integrates Voyager, makes the solver recursively decompositional and compositional, stays link-native with meta-theory, and audits related upstream dependencies. This case study now includes that expanded second planning pass.

## Source Material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/559>
- Prepared pull request: <https://github.com/link-assistant/formal-ai/pull/560>
- PR feedback integrated: <https://github.com/link-assistant/formal-ai/pull/560#issuecomment-4783154352>
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

The main gap is that the control plane is still split across specialized recognizers and handler ordering:

- `src/solver_dispatch.rs` contains an ordered table of specialized handlers.
- `src/intent_formalization.rs` still contains prompt cue recognizers for routing.
- `UniversalSolver::handle_specialized_pattern` still uses the specialized table as the executable method-selection path.
- Existing meta recipes are specific examples, not one general problem-frame recipe that every request passes through.

The recommended plan is therefore not a rewrite-first change. It is an incremental migration that introduces a general problem frame, recursive work units, and a data-described method/skill registry, proves parity with the current specialized handlers, then moves routing and preconditions out of hardcoded code in phases.

The refined plan adds four constraints from the follow-up feedback:

- Represent problems, tasks, dependencies, evidence, methods, objects, files, sequences, and algorithms as links or link networks, not as a separate non-link ontology.
- Make solving recursive: decompose until a work unit is directly solvable, while also searching upward from existing libraries, standard functions, repo code, and accumulated skills.
- Adapt Voyager's open-ended curriculum, skill library, execution feedback, and critic loop into deterministic, review-gated formal-ai mechanisms without adding a neural runtime dependency.
- Treat organization-owned dependencies as gates: no upstream blocker was found for the next planning-approved phases, but existing relevant issues are listed in [upstream-dependency-audit.md](upstream-dependency-audit.md).

## Files In This Case Study

- [requirements.md](requirements.md) captures the issue requirements, acceptance criteria, and inferred constraints.
- [architecture-inventory.md](architecture-inventory.md) documents the relevant current architecture before planning changes.
- [solution-plan.md](solution-plan.md) proposes implementation phases, tests, risks, and alternatives.
- [upstream-dependency-audit.md](upstream-dependency-audit.md) checks related organization dependencies and existing upstream issues.
- [raw-data/online-research.md](raw-data/online-research.md) records external research and library/component checks.

## Recommended Approval Decision

Approve Phases 1A and 1B from [solution-plan.md](solution-plan.md): add the problem-frame model, recursive work-unit trace, and need-satisfaction tests while preserving current output behavior. That creates a measurable base before moving handler metadata, hardcoded cue recognition, and reusable skills into `.lino` data.
