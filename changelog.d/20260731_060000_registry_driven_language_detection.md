---
bump: minor
---

### Changed
- Derive language detection entirely from `data/seed/language-detection.lino`
  instead of a hardcoded Rust enum, so registering a language (script, Unicode
  range, markers, fallback flag) is a data-only edit shared by the Rust core,
  the WASM worker, and the JS worker.
- Apply the ledger's `explicit_gap` fallback policy through
  `seed::localized_response`: an intent with no text for a registered language
  now surfaces the explicit "unsupported language" record instead of silently
  answering in English.
