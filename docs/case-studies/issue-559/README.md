# Issue 559 Case Study: General Meta Algorithm

Status: first-session planning artifact only. Issue 559 explicitly asks for a detailed planning pass before implementation, with execution to follow after maintainer approval or requested changes.

## Source Material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/559>
- Prepared pull request: <https://github.com/link-assistant/formal-ai/pull/560>
- Local inspiration image: [inspiration.png](inspiration.png)
- Raw GitHub and search data: [raw-data/](raw-data/)
- External research notes: [raw-data/online-research.md](raw-data/online-research.md)

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

The recommended plan is therefore not a rewrite-first change. It is an incremental migration that introduces a general problem frame and data-described method registry, proves parity with the current specialized handlers, then moves routing and preconditions out of hardcoded code in phases.

## Files In This Case Study

- [requirements.md](requirements.md) captures the issue requirements, acceptance criteria, and inferred constraints.
- [architecture-inventory.md](architecture-inventory.md) documents the relevant current architecture before planning changes.
- [solution-plan.md](solution-plan.md) proposes implementation phases, tests, risks, and alternatives.
- [raw-data/online-research.md](raw-data/online-research.md) records external research and library/component checks.

## Recommended Approval Decision

Approve Phase 1 from [solution-plan.md](solution-plan.md): add the problem-frame model and tests while preserving current output behavior. That creates a measurable base before moving handler metadata and hardcoded cue recognition into `.lino` data.
