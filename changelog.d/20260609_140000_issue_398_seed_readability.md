---
bump: patch
---

### Fixed
- Replaced the codepoint byte-dump encoding of seed text (e.g. `answer codepoints 72 105 ...`) with human-readable quoted scalars, so `data/seed/*.lino` stays legible while every runtime parser decodes the same values.

### Changed
- Taught the Rust, web, and e2e LiNo seed parsers (plus the worker's embedded fallback) to decode single-quote, double-quote, and backtick scalars with a non-escaping delimiter.
- Removed the 4,677 synthetic `seed-surface-<hash>` ids from `data/seed/*.lino`: a surface is now the text (and facets) recorded under a language, not an opaque minted id. Added `scripts/clean-seed-readability.rs` to perform the lossless migration and regenerate the browser worker fallback.
- Stripped keyword-restating noise comments (`# language`, `# definition-link`, `# semantic-role`, `# facet`, `# seed lexical surface`, `# source-id`, `# action`) from the seed while keeping comments that carry the human meaning of an opaque id.

### Added
- Added a CI guard that bans codepoint byte-dumps in seed data and a guard that bans inline `#[test]`/`#[cfg(test)]` scaffolding under `src/`.
- Added CI guards that ban reintroducing synthetic `seed-surface-<hash>` ids and keyword-restating noise comments in seed data.
