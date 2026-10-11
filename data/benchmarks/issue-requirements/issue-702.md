Parent: #651

## Motivation and evidence

Issue #649 asked for reasoning with **symbolic world models**: a current-state and a target-state context maintained throughout a dialogue, their difference exposed, user⇄agent target synchronization, context merge/split where each context is a links network (explicitly not embeddings), [relative-meta-logic](https://github.com/link-foundation/relative-meta-logic) dependency propagation (any change recalculates all statement probabilities), and prediction of the consequences of an action.

PR #675 closed #649 as an audit-and-design deliverable: its own requirements matrix declares 8 of 14 concepts **partial** and the agent⇄user target-synchronization loop merely **proposed**, despite the issue's "each and every requirement fully addressed" instruction. The design work is done (`docs/case-studies/issue-649/`: requirements R649-01…R649-19, world-model-mapping.md, solution-plans.md); the implementation issue is missing. Issue #686 (PR #689) adds usage-weighted persistence of meta-language expressions — the storage substrate — but not the world-model behaviors.

## Requirements

Implement the per-requirement plans from `docs/case-studies/issue-649/solution-plans.md`:

1. **Current-state context**: seed and maintain a `current_state` context links network from the dialogue log (every formalized statement lands in it with provenance).
2. **Target-state context**: build a `target_state` context from intent formalization ("I want…", imperative requests route into target edits).
3. **Difference**: expose `diff(current, target)` as a links network of missing/extra/conflicting statements, queryable from chat ("what is left to do?").
4. **Synchronization loop**: the agent proposes target-state edits, the user confirms/corrects, and both converge — every sync step is an append-only event (the concept PR #675 left as "proposed").
5. **Merge and split**: contexts merge (union with conflict detection) and split (by topic/subtree) as first-class operations over links networks, reusing `SubstitutionGraph` / context-merge substrate.
6. **Dependent statements via relative-meta-logic**: statements carry dependency links; changing one recalculates the posterior of every dependent statement (wire the existing RML kernel + `probability.rs`), surfacing changed beliefs in the trace.
7. **Action-consequence prediction**: given a candidate action (agent tool call, program edit, or user-described act), simulate its effect as a hypothetical context (`predict(current, action) -> hypothetical_state`), compare with target, and report which needs it satisfies or violates — the symbolic analog of a world-model rollout; deterministic and inspectable.
8. Everything is links: no embeddings, no graph/edge/vertex terminology (aligns with #664), all four languages, all knobs in `SolverConfig`, trace-only until opted in.

## Acceptance criteria

- A scripted multi-turn dialogue produces inspectable `current_state` / `target_state` / `state_diff` links, and "what remains to reach my goal?" answers from the diff.
- Changing one premise statement recalculates dependent statement probabilities and the trace names each recalculated link (RML integration test).
- `predict` on a bounded agent action (e.g. a file write) yields a hypothetical context whose diff against target shrinks; a destructive action is flagged as violating a target need before execution.
- Merge/split round-trip tests over generated contexts; a benchmark slice of state-tracking cases (e.g. bAbI-style deterministic tasks, permissive subset) with a ratchet.

## Dependencies

- Blocked by #686 (persistence substrate, PR #689) and #661 (E42 probability-weighted formalization feeds statement weights).
- Blocks E63-style prediction of next user requests (target-state modeling is its substrate) and strengthens #656-gated self-improvement (consequence prediction before self-change).
- Related closed parents: #649, #559.

## Process

Collect data to `docs/case-studies/issue-{id}` continuing the #649 study (STRIPS/PDDL, situation calculus, JTMS/ATMS, AGM belief revision already surveyed there); single PR per milestone; the issue closes only when all 8 requirements are implemented with tests, or each remainder is recorded as blocked with evidence.

