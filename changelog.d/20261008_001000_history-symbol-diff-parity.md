---
bump: patch
---

### Fixed
- The JavaScript repository-history importer now records the same ES `token_count` change and census-histogram ordering as the native importer, through a shared tokenizer module (`js/agentic/crate/es_tokenizer.mjs`). `formal-ai translate --from js --to ts --write` now also renders the agentic `.mts` twins (issue #1180, R1180-11).
