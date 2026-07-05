# Issue 558 Case Study: Auto Learning

Status: **first closed self-healing slice implemented in PR #637**. This case
study answers issue #558's request to collect data, inspect issue #538 / PR #601,
search for related approaches, list requirements, and propose a concrete path to
auto-learning — and PR #637 now lands the first end-to-end, human-gated slice of
that path in code.

PR #601 delivered important slices but not a closed self-learning loop. It
proved bounded Agent CLI recipes, reproducible sessions, generated recipe
diagrams, and a one-module self-AST census. Issue #558 needs the next layer: a
human-gated repair loop that starts from failures, maps them to source-to-links
data, produces reviewable patches, proves them with tests, and promotes accepted
lessons. PR #637 closes that loop in its safe, proposal-only form (see
[Delivered In PR #637](#delivered-in-pr-637)): one failure the system could not
answer → the source it maps onto with a verified source-to-links round-trip → a
benchmark-gated lesson → a human-review outcome, wired into the agentic interface
as the fifth recipe and exercised by unit and server integration tests.

## Source Material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/558>
- Related issue: <https://github.com/link-assistant/formal-ai/issues/538>
- Related PR: <https://github.com/link-assistant/formal-ai/pull/601>
- Raw evidence: [raw-data/](raw-data/)
- Requirements matrix: [requirements.md](requirements.md)
- PR #601 gap analysis: [pr-601-gap-analysis.md](pr-601-gap-analysis.md)
- Solution plan: [solution-plan.md](solution-plan.md)
- Online research notes:
  [raw-data/online-research.md](raw-data/online-research.md)

## What Went Wrong In PR #601

PR #601 was not a failure: it shipped several valuable slices. The problem is
status and scope. Its final state was broader than the old root requirements
table said, but narrower than the phrase "auto-learning" implies.

The root `REQUIREMENTS.md` drifted by leaving Agent CLI, diagrams, and self-AST
as follow-ups after the issue #538 case study had recorded delivered slices.
That made the project state harder to reason about. At the same time, none of
those slices formed an arbitrary repair loop. The Agent CLI still follows known
recipe families; the self-AST is a census, not a full compiler round-trip; and
there is no accepted learning record that connects a failure, patch, tests,
review, rebuild, and UI reattach.

See [pr-601-gap-analysis.md](pr-601-gap-analysis.md) for the detailed gap list.

## Auto-Learning Gap Inventory

| Gap | Short name | Required next capability |
| --- | --- | --- |
| G1 | Requirement status drift | Root requirements must match delivered and missing capability. |
| G2 | Recipe-driven Agent CLI | A failed prompt must open a general repair loop, not only a known recipe. |
| G3 | Partial self-AST | The whole repository must have source-to-links projections. |
| G4 | No Links-to-source | Link-native code data must regenerate source and pass rebuild tests. |
| G5 | No promotion ledger | Accepted fixes need durable learning records. |
| G6 | No UI reattach | Rebuilt code must be explicitly attached to the affected UI/service surface. |
| G7 | Prose-heavy self-explanation | Answers about Formal AI should cite source, data, tests, and repair records. |

## Proposed Delivery Architecture

The smallest credible architecture is deliberately human-gated:

1. Record a failure as a structured `repair_case`.
2. Link the failure to source, data, tests, requirements, and prior lessons.
3. Generate a candidate source/data change in an isolated workspace.
4. Run formatting, unit tests, integration tests, and targeted UI checks when
   the changed surface is visual or browser-facing.
5. Open a reviewable PR with the repair transcript and evidence.
6. Promote the accepted lesson into seed/meta data only after human approval.
7. Rebuild and reattach the accepted version, then make the new version explain
   which repair case caused the change.

This is dynamic learning because the system can convert failures into approved
new behavior. It is not uncontrolled self-modification: every Links-to-source
change, rebuild, and UI reattach stays observable, testable, reversible, and
reviewed.

## Delivered In PR #637

PR #637 implements the first end-to-end slice of that architecture as a single,
auditable, proposal-only artifact — the closed loop in its safe form — and wires
it into the agentic interface:

- **Closed self-healing loop** — `src/self_healing.rs` composes the four stages
  the issue calls for into one `RepairCase`: a captured failure the system could
  not answer (`UnknownTrace`), the source it maps onto with a verified
  source-to-links round-trip (`SourceRoundTrip`), a benchmark-gated candidate
  lesson (`LearningRun` + `BenchmarkGateReport`), and a terminal `RepairOutcome`
  that never advances past `AwaitingReview`. Adoption stays a human decision:
  `RepairCase::is_human_gated()` is `true` by construction, mirroring the
  existing self-improvement modules.
- **Verified source ↔ links round-trip** — `SourceRoundTrip::for_pinned_target()`
  parses a real module of the reasoning meta algorithm (the deterministic
  planner) through the sole CST/AST engine in the repo and confirms
  `source → links → source` reproduces it byte-for-byte (`faithful = true`). This
  is the concrete, tested realization of issue #558's "translate the source code
  to the meta language and back" (`R558-05`), stopping short of writing source.
- **Fifth agentic recipe** — `src/agentic_coding/self_heal.rs` makes the loop
  reachable through the agentic interface. When an external agent CLI (Codex,
  OpenCode, Gemini, Agent CLI) — or the in-repo driver — asks Formal AI to run
  its self-healing loop, the deterministic planner walks a write → verify → final
  recipe that emits the repair case as Links Notation, exactly like the self-AST
  and diagram recipes emit their self-inspection documents.
- **Committed, generated artifact** — `data/meta/self-healing-case.lino` is the
  worked repair case, *generated* by running the loop (never hand-written) and
  pinned byte-for-byte to what the Agent CLI writes, so it can never drift.
  Regenerate with `cargo run --example dump_self_healing_case`.
- **Tests** — `tests/unit/issue_558_self_healing.rs` locks the loop's outcome,
  the round-trip faithfulness, the Links Notation validity, the planner recipe
  walk, and the driver's end-to-end write; `tests/integration/issue_558_self_healing.rs`
  boots the agent-mode server and proves a self-healing request over the wire
  routes to a `write_file` tool call carrying the generated repair case.

What remains for later slices (still human-gated by design): projecting the
*whole* repository to source-to-links data (`G3`), Links-to-source regeneration
with rebuild tests (`G4`), a durable promotion ledger (`G5`), and the
rebuild/UI-reattach step (`G6`). The `RepairCase` is the anchor those slices
attach to.

## Related Existing Pieces

- Issue #364 provides the earlier self-improvement framing.
- Issue #538 / PR #601 provides the Agent CLI sessions, clean reproduction,
  recipe diagrams, and first self-AST slice.
- Issue #559 provides meta-algorithm and method-registry work that the repair
  loop should reuse instead of duplicating.
- Issue #628 provides current Agent CLI testing-guide context.

## Verification

`tests/unit/docs_requirements_issue_558.rs` checks that the root requirements,
case-study index, gap analysis, requirements matrix, solution plan, online
research notes, and raw evidence files remain present and traceable.
`tests/unit/issue_558_self_healing.rs` and
`tests/integration/issue_558_self_healing.rs` verify the delivered self-healing
loop: the closed `RepairCase`, the verified source-to-links round-trip, the fifth
agentic recipe, the driver's end-to-end write, and the agent-mode server routing
a self-healing request to the repair-case write.
