---
bump: patch
---

### Changed
- Translation answers now read like natural conversation: the response body is just the deformalized target surface (still quoted when the user quoted the source) instead of the `meaning: … / surface (…): …` template. The meaning ID, source language, and target language remain in the Links Notation trace via `evidence_links`.

### Added
- Shared offline meaning registry (`src/translation.rs`, mirrored in `src/web/formal_ai_worker.js`) routes Rust and browser-worker translations through a single `formalize → meaning → deformalize` pipeline, covering greetings, gratitude, farewells, identity probes, and yes/no answers in English, Russian, Hindi, and Chinese.
- `match_source_formatting` / `matchSourceFormatting` helpers preserve the source fragment's leading capitalization and terminal punctuation, so `как у тебя дела?` round-trips to lowercase `how are you?` and an unterminated source stays unterminated in the target.

### Fixed
- Translations no longer fall back to `[en] …` placeholders for any phrase covered by the meaning registry, and the lowercase / uppercase source distinction is preserved in the target.
