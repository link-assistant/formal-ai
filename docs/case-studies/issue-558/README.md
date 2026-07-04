# Issue 558 Case Study: Auto Learning

Status: **analysis and delivery plan captured in PR #637**. This case study
answers issue #558's request to collect data, inspect issue #538 / PR #601,
search for related approaches, list requirements, and propose a concrete path to
auto-learning.

PR #601 delivered important slices but not a closed self-learning loop. It
proved bounded Agent CLI recipes, reproducible sessions, generated recipe
diagrams, and a one-module self-AST census. Issue #558 needs the next layer: a
human-gated repair loop that starts from failures, maps them to source-to-links
data, produces reviewable patches, proves them with tests, and promotes accepted
lessons.

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
