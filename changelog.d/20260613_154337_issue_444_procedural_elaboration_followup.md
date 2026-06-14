---
bump: patch
---

### Fixed

- Issue #444: a bare elaboration follow-up after a "how to …" answer (e.g.
  "Can you give me specific instructions?") no longer dead-ends at the
  unknown-intent opener. It now rebinds to the procedure recovered from the
  prior turn and answers as `procedural_how_to` in the original language.

### Added

- New `procedural_elaboration` seed meaning (en/ru/hi/zh) and
  `try_procedural_how_to_followup` handler, mirrored in the browser worker
  (`tryProceduralHowToFollowup`), keeping Rust ↔ JS parity.
