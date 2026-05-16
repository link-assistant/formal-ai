---
bump: minor
---

### Added
- Clarification intent that handles "не понял" and similar phrases across Russian, English, Hindi, and Chinese
- Multilingual clarification responses in `data/seed/multilingual-responses.lino`
- `try_clarification` handler in `solver_handlers.rs` with language-aware response selection
- Tests pinning the clarification intent for Russian ("не понял") and English ("I don't understand", "I didn't understand")

### Fixed
- Issue #29: "не понял" (Russian: "I didn't understand") now returns a helpful explanation of what formal-ai can do instead of the generic unknown-intent fallback
