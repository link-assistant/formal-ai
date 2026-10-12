# Issue #453: moonshot task decomposition and source provenance

## Requirement inventory

The request names four observable obligations: recursively divide a large task
into two checkable children, research approaches from the best available
sources, combine equivalent ideas while retaining the first historical source,
and apply that process to hard tasks such as an Atari Breakout agent scoring
860–864, a symbolic conversational system with a benchmark, and a system that
can produce strong intelligence on demand without claiming to possess a will.

The implementation already supplies the structural half. `balanced_split` and
`TaskSplitter` preserve every byte of a compound request in exactly two
children; `SplitRefusal::Underivable` names the blocker when one clause cannot
be grounded into smaller obligations. `combine_approaches` removes only
grounded equivalents and keeps the first source in input history order.
`docs/requirements/issue-0453-moonshot-splitting.md` records requirements
R453-M1 through R453-M4, and `rust/tests/unit/issue_1138_selection_heuristics.rs`
pins ten moonshot-shaped prompts in five languages.

## End-to-end path still needed

1. For each child, resolve a `Need` and consult the source registry through the
   capture boundary. Keep URL, timestamp, digest, license, and source tier.
2. Extract candidate approaches from the captured bytes. Reject a candidate
   whose evidence cannot be traced to an exact capture.
3. Pass the ordered observations to `combine_approaches`; preserve every
   distinct approach and the first historical source for each duplicate.
4. Convert each approach into independently checkable sub-obligations. If no
   supported decomposition can be derived, return `Underivable` with the
   missing capability or source, rather than declaring the task atomic.
5. Recurse only within the configured work and source budgets. Record the two
   children, stopping reason, evidence links, and verification status at each
   rung. A source claim is not a benchmark result.

## Acceptance examples

| Request | Observable check |
| --- | --- |
| Atari Breakout architecture scoring 860–864 | A sourced architecture plan and an independent score reproduction protocol; no score claim until the benchmark actually runs. |
| Symbolic ChatGPT-class chatbot and benchmark | Two child obligations for the system and evaluation, then recursively checkable components with source and benchmark provenance. |
| “Write a strong AI” | A named `Underivable` gap for capabilities the system cannot prove, with any smaller, checkable work retained. |
| Weak intelligence producing strong intelligence on demand | Separate a proposed generator from a verified generated system; never infer will or ability from the prompt alone. |

Current status is **partial**: the binary split, underivable boundary, and
first-source deduplication are implemented; source-backed recursive execution
and live benchmark evidence are still required before the examples can be
called solved. This case study states that boundary so future issue/PR tracking
does not equate a task plan with a demonstrated moonshot result.
