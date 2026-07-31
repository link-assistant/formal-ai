---
bump: minor
---

### Added
- Record the language learning frontier: `data/language-additions/<code>.lino`
  can now carry a prompt corpus, and `src/language_frontier.rs` runs the live
  engine over it to record only the prompts that still fail, keeping a language
  without a corpus as an explicit `frontier_gap`.
- Register `--frontier` as an open registry in `formal-ai learn cycle`, so the
  issue-#701 learning cycle replays the new `language-gap` frontier with no new
  learning logic. Over the Spanish corpus it derives, validates on held-out
  prompts, and proposes the `qué es …` and `cuéntame sobre …` request frames.
- Pin the adoption evidence in `data/meta/language-adoption-ledger.lino`: 7 of 7
  recorded Spanish prompts leave the unknown path and recover their term after
  the proposals are adopted as seed data.

### Changed
- Move the unknown-intent opener pools into `data/seed/unknown-openers.lino`,
  shared by the Rust core, the WASM worker, and the JS worker.
- Derive language display names, concept slugs, and per-language script checks
  from the seed ledger instead of Rust `match` arms.
