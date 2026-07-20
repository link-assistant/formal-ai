---
bump: patch
---

### Added

- `data/meta/links-network-terminology-recipe.lino` — the grounded meta-algorithm
  recipe for issue #664. It records every ordered step, handler function, module
  rename, lint allowlist entry, CI wiring, and pinning test that produced the
  links-network terminology cleanup, together with a `generalization` note that
  turns "this public surface still says graph, make it a links network" into a
  reusable procedure.
- `tests/unit/specification/links_network_terminology_meta_algorithm.rs` — the
  grounding test that loads the recipe and asserts the live source still matches
  it (functions exist, renames landed and old names are gone, allowlist entries
  are present in the lint, the lint is wired into CI, and the pinning tests
  exist). CI now fails if the recipe and the code drift apart, so the cleanup is
  a reproducible artifact of the meta-algorithm rather than a one-off hand edit.
- A "links-network terminology meta-algorithm" section in `docs/meta-algorithm.md`
  documenting the recipe and how to run its grounding test.
