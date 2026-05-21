---
bump: patch
---

### Changed
- Translation answers now read like natural conversation: the response body is just the deformalized target surface (still quoted when the user quoted the source) instead of the `meaning: … / surface (…): …` template. The meaning ID, source language, and target language remain in the Links Notation trace via `evidence_links`.

### Added
- Generalized `formalize → meaning → deformalize` pipeline under `src/translation/` (`http`, `cache`, `wiktionary`, `wikidata`, `meaning`, `pipeline`, `formatting`) routes Rust translations through real Wiktionary translation tables and Wikidata lexeme/sense joins, so any surface pair resolves through public data rather than a hand-written list.
- `CachedHttpClient` persists raw API responses under `data/translation-cache/` (FNV-1a-keyed `.body` + `.url` files); tests run deterministically offline against the committed cache, and contributors can refresh it with `FORMAL_AI_LIVE_API=1` via `examples/refresh_translation_cache.rs`.
- `match_source_formatting` / `matchSourceFormatting` helpers preserve the source fragment's leading capitalization and terminal punctuation, so `как у тебя дела?` round-trips to lowercase `how are you?` and an unterminated source stays unterminated in the target.

### Fixed
- Translations no longer rely on a hardcoded set of meanings: any surface routed through `TranslationPipeline::translate` can now resolve via Wiktionary + Wikidata, and the lowercase / uppercase source distinction is preserved in the target.
