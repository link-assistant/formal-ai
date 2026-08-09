---
bump: minor
---

### Added
- Unknown online requests now enter evidence-producing web research instead of
  stopping at an `unknown` answer, including imperative prompts without question
  punctuation. A data-defined research-learning cycle keeps disposable source
  captures separate from versioned knowledge, promotes only immutable-baseline
  passing candidates, restores earlier stable versions, and supports user-led,
  full-trust, and per-command recovery with a configurable one-hour default
  continuation boundary. ([#873](https://github.com/link-assistant/formal-ai/issues/873))

### Fixed
- The self-AST workspace aggregate is now rendered on demand instead of tracked,
  preventing unrelated source branches from repeatedly conflicting in the same
  generated `index.lino` while retaining per-module drift checks.
- Repository summarization now bounds optional concrete-syntax parsing for
  oversized traces, preventing seeded validation from spending hours on a
  single generated evidence file while still summarizing its full structure.
