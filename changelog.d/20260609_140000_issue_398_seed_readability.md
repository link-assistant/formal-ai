---
bump: patch
---

### Fixed
- Replaced the codepoint byte-dump encoding of seed text (e.g. `answer codepoints 72 105 ...`) with human-readable quoted scalars, so `data/seed/*.lino` stays legible while every runtime parser decodes the same values.

### Changed
- Taught the Rust, web, and e2e LiNo seed parsers (plus the worker's embedded fallback) to decode single-quote, double-quote, and backtick scalars with a non-escaping delimiter.

### Added
- Added a CI guard that bans codepoint byte-dumps in seed data and a guard that bans inline `#[test]`/`#[cfg(test)]` scaffolding under `src/`.
