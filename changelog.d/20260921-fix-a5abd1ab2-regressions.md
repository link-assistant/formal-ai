---
bump: patch
---

### Fixed

- The three regressions batch a5abd1ab2 introduced on the CI Test job. The
  Hindi personal statement of non-understanding ("मुझे समझ नहीं आया") is again
  a clarification: the bare "समझ नहीं" stem no longer sits in the
  prior-turn-reference class, whose other surfaces are re-render idioms ("दूसरे
  शब्दों", "फिर से कहिए") — Russian keeps the same split (impersonal
  "непонятно" re-renders, personal "Я не понимаю" asks for clarification), and
  English never carried the personal form. The embedded rule document's rule
  count pin follows the `clarification_inflected_stem` rule the batch added
  (15 → 16), and the Spanish typed-write pin includes the request-anchors line
  the HonestGap now appends.
