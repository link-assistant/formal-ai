---
bump: minor
---

### Added

- Induce computer-use plan schemas from the recorded example tasks and
  synthesize verified plans for requests never seen before, ratcheted by twelve
  held-out four-language cases that also run through the real external Agent
  CLI, with the induced schemas committed as drift-tested evidence and the
  observe/induce/bind/synthesize/verify/refuse loop recorded as a grounded
  meta-recipe.

### Fixed

- Stop computer-use plan synthesis from inheriting another example's state:
  resource-scoped arguments (`selector`, `pointer`, `column`, `equals`) now come
  only from the learned resource binding, operation constants are gated by both
  the primitive's advertised schema and the learned operation schema, and a
  request the corpus never evidenced yields an honest refusal instead of a
  plausible wrong plan.
