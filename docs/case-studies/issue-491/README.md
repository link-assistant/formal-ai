# Issue #491 Case Study: The Principle of Least Action for Splits and Solutions

Issue [#491](https://github.com/link-assistant/formal-ai/issues/491).
Case study written on the `qa-reasoning-coding-bulk-fixes` branch.

## What the issue asks

> Once we will have fully real thinking steps we can start to optimize
> for least action … we should try from the start produce for each task
> split into 2 sub tasks. Meaning on the highest level of abstraction
> there is always 1, 2, 4, 8 and so on tasks. So the balanced tree of
> sub tasks always preserved, but the thing we actually optimizing for
> is the total number of smallest sub task … That also may mean
> shortest path or shortest steps to results, or shortest code (that
> still solves the entire range of inputs) when simplifying the code.
> So when multiple solutions are generated, we can use least action or
> shortest path/steps to evaluate/score solutions … benchmarks and
> evaluation can be done using time, computation resources, memory …
> highest possible user satisfaction in the least amount of resources.

## Solution (this pull request)

`rust/src/least_action.rs` — the scoring half of least action, pure and
std-only so both the engine and the WASM build can consume it:

- **Balanced binary planning** (`plan`): the abstraction ladder is
  regular — 1, 2, 4, 8, … tasks per level, exactly reproducing the
  issue's ladder for power-of-two subtask counts and holding the
  concrete count at the final level otherwise. Some subtasks at the
  bottom are already *atomic with known solutions*; others are open.
- **The optimization target**: `smallest_subtasks` (the count) and
  `planned_steps` (the steps of the known solutions — and nothing else:
  open work is listed under `unhandled`, never costed, so a plan never
  invents numbers for work it cannot do yet; that list is the
  auto-improvement surface the issue points at).
- **Plan comparison** (`least_action_plan`): fewer smallest subtasks
  first; among equal counts, fewer planned steps; then fewer unhandled
  paths.
- **Solution scoring** (`rank_by_least_action`, `least_action_solution`):
  candidates that do not solve the *entire range of inputs* are
  excluded before any comparison — a shorter wrong answer is not least
  action, it is no action toward the requirement. Survivors order
  lexicographically on (steps, code units, compute ms, memory kb), so
  saving a millisecond by adding a reasoning step still loses, stable
  on ties.

## Verification

- `rust/tests/unit/issue_491_least_action.rs` (CI): the 1-2-4-8 ladder
  (and 1-2-4-5 between powers of two); smallest-subtask counting with
  two atomic + three open subtasks and exactly-costed planned steps;
  plan comparison across counts; steps-first ranking; the
  millisecond-vs-step rule; the entire-input-range exclusion; stable
  tie-breaks.

## Known components surveyed (integration sites left for main)

The module is deliberately pure so it drops into the existing ranks:

- `rust/src/coding/composition_search.rs` generates candidate
  compositions — its tie-break could call `rank_by_least_action` on
  candidate costs instead of index order (main's lane; composition_search
  is not this fork's file).
- The solver's multi-path search could prune with
  `ActionCost::is_less_action_than` once step counts are attached to
  reasoning paths — which lands after "fully real thinking steps" (#491's
  own precondition).
- `rust/src/least_action.rs` needs `pub mod least_action;` in lib.rs and
  `mod issue_491_least_action;` in tests/unit/mod.rs (main).

No Cargo.toml changes.

## Residuals

- Satisfaction weighting: the issue's "highest user satisfaction in the
  least resources" is realized as resources-first lexicographic order;
  a satisfaction score that gates the ranking (satisfaction as a
  constraint, resources as the objective) wants a quality signal that
  does not exist yet in the engine.
- Splitting itself (choosing *where* a task halves) is not scored here;
  once a splitter generates alternatives, `least_action_plan` judges
  them, but no splitter emits alternatives today.
