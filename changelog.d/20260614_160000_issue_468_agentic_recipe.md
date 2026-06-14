---
bump: patch
---

### Added

- A **grounded meta-algorithm recipe for the agentic-coding loop**
  (`data/meta/agentic-coding-recipe.lino`), following the issue #444 pattern. It
  names every part the deterministic loop is made of — the plan constants
  (`SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH`), the four advertised tools
  and their capabilities and permissions, the `search → fetch → write → run →
  final` state-machine stages, the fourteen handler functions, the nine protocol
  primitives the product realises, the `MAX_TURNS` cap, and the CLI/example/test
  exposure surfaces — plus the eight ordered steps that generalise to a new task.
- A **grounding test** (`tests/unit/specification/agentic_meta_algorithm.rs`)
  that loads the recipe and asserts the live source still matches every entry, so
  the recipe can never silently drift from the code (CI fails if it does).

### Changed

- `docs/meta-algorithm.md` now documents the agentic-coding meta-algorithm as a
  second grounded recipe alongside the procedural how-to one, including the state
  machine, the eight ordered steps, and the grounded-record table.
