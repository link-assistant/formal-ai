---
bump: minor
---

### Added
- Issue #559 (R335): the recursive meta core now describes *itself* as grounded link data. `data/meta/recursive-core-recipe.lino` enumerates the eight ordered steps that turn any message into a solved, link-native knowledge base — formalize the impulse, build the problem frame, decompose recursively into a bounded work-unit tree, account for every need in a ledger, catalogue the resolving methods, resolve each atomic leaf through the single ordered dispatch, record evidence, and project the answer — and pins each step to the live function that implements it. A grounding test (`tests/unit/specification/recursive_core_recipe.rs`) asserts the source still defines every named function, so the core's self-description can never drift from the code that runs. This is the concrete sense in which the meta algorithm can reason about itself: its own algorithm exists as data the engine can read. Tracked by REQUIREMENTS.md R335.
