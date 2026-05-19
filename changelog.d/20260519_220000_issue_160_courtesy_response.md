---
bump: patch
---

### Fixed
- **Issue #160 — polite follow-up returned unknown intent.** Added a `courtesy_response` intent for phrases such as "I am fine, thank you", "thanks", "спасибо", "धन्यवाद", and "谢谢", with localized responses across the Rust solver and browser worker so small-talk acknowledgements stay in normal chat flow instead of showing the missing-rule fallback.
- Added browser courtesy-response variation that composes the acknowledgement and optional next-action question separately, with a settings slider for how often the assistant should ask/propose the next step.
