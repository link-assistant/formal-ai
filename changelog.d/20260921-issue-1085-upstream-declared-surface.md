---
bump: patch
---

### Fixed

- The two upstream prompt shapes of issue #1085's transfer slice. An MBPP
  assertion's argument tuples no longer type the derived parameters: a bare
  `(3, 4, 5, 6)` is ambiguous between a fixed pair and an ordered sequence,
  so `example_parameter_type` leaves the parameter open and the signature
  stays generic by arity, exactly as the assertion-is-not-a-signature rule
  demands (previously the catch-all read the tuples as text and annotated
  the parameters `str`). A signature the prompt did declare is now echoed
  verbatim into the composition draft: the search still unifies types to
  find the composition, but the declared spellings, return annotation, and
  import block travel with the artifact, so an upstream `List[float]` keeps
  `from typing import List` and its own spelling instead of a reconstructed
  `list[float]` that dropped the import and with it executability
  (HumanEval/0).
- The held-out capability-routing corpus recovers its hi non-understanding
  case: removing the bare "समझ नहीं" stem from the prior-turn class left
  "मैं समझ नहीं पा रहा।" unrouted, and the walk fell through to a
  prohibited web search. The continuous forms ("समझ नहीं पा रहा/रही/रहे")
  join the prior-turn surfaces as full phrases — the same class that
  carries en "lost me" and ru "Я потерял нить" — while the exact personal
  statement "मुझे समझ नहीं आया" stays with the clarification handler.
