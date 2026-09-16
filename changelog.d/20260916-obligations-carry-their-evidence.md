---
bump: minor
---

### Added

- A runtime obligation ledger. An obligation reaches `satisfied` only while
  carrying an execution record — the command, its exit status or an explicit
  none, and a SHA-256 of the observed bytes — and that variant has exactly one
  field, so a satisfied obligation nobody observed cannot be constructed at all.
- A fourteenth step in the recursive meta core, `verify_obligations`, described
  in `data/meta/recursive-core-recipe.lino` and executable from it. Executing
  the recipe still reproduces the native trace event for event under every mode
  combination.
- `data/meta/obligation-evidence-contract.lino`, which states how a clause's
  shape becomes the observation that would settle it, so a new expectation shape
  is a data edit rather than a new branch.

### Changed

- A need row reaches `satisfied` in exactly one place: the join that reads a
  discharged obligation. Route selection no longer implies satisfaction
  anywhere.
- An observation discharges only the obligation whose expectation names its
  path, command or check. An unrelated successful result now clears nothing.
- A clause no composer can read an artifact out of is kept as a node with its
  byte span and split, instead of being dropped.
