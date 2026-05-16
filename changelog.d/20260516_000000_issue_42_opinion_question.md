---
bump: minor
---

### Added
- Opinion question intent (`opinion_question`) that handles prompts like "Do you think space is continuous or discrete?" with a deterministic explanation instead of the generic unknown-intent error
- `try_opinion_question` handler in `solver_handlers.rs` detecting opinion/belief phrasings across multiple patterns
- Tests pinning the opinion question intent for the exact prompt from issue #42 and five related phrasings

### Fixed
- Issue #42: Opinion-style questions such as "Do you think space is continuous or discrete?" now return a helpful deterministic explanation instead of the confusing "I do not have a learned symbolic rule" fallback
