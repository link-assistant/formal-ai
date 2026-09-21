---
bump: patch
---

### Fixed

- A response-language follow-up no longer strands a pattern-inference report
  in English (issue #531's #556 generalization, red in the wave-F tail). The
  pattern handler's report language was a hard-coded `en` default, so the
  replay forced Russian onto `detect` while the one handler that does not
  detect kept answering in English. `language::forced_response_language_slug`
  now exposes the forced slug to such handlers, `try_pattern_inference`
  renders in it, and an explicit switch *to* English still records
  `language_to:en` (the variant treats `en` as its no-op default and would
  not log it). Also covers a language the conversation already established
  (issue #724), since both force the same slot.
- The issue #724 binding test asserted the wrong answer family: it expected
  the unknown-reasoning trace where plan 01's pinned behaviour for a
  definition question is the consulted-source record (batch a5abd1ab2 kept
  the record for "what does X mean"). The test now pins the seed-grounded
  Russian record prefix up to the environment-dependent consulted list, so
  the leaf's actual claim — the demonstrated language binds — is what fails
  if the binding regresses.
