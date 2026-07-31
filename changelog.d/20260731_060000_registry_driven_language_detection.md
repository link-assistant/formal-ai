---
bump: minor
---

### Changed
- Derive language detection entirely from `data/seed/language-detection.lino`
  instead of a hardcoded Rust enum, so registering a language (script, Unicode
  range, markers, fallback flag) is a data-only edit shared by the Rust core,
  the WASM worker, and the JS worker.
- Move the unknown-intent opener pools out of Rust and JavaScript constants
  into `data/seed/unknown-openers.lino`, and derive the browser worker's
  no-WASM fallbacks — script/marker detection and the known-response-language
  check — from the hydrated registry, so a new language needs no worker edit.
- Apply the ledger's `explicit_gap` fallback policy through
  `seed::localized_response`: an intent with no text for a registered language
  now surfaces the explicit "unsupported language" record instead of silently
  answering in English.
