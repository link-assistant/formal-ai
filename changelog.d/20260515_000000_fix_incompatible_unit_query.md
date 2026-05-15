---
bump: minor
---

### Added
- `try_incompatible_units` handler: queries that mix dimensionally incompatible units
  (e.g. meters vs kilobytes) now return `intent:unit_incompatibility` with a clear
  symbolic explanation instead of falling through to `intent:unknown` (fixes #43).
- Five new `reasoning_paths` tests covering the Russian prompt from the bug report
  (`"Сколько метров в килобайте?"`), the English equivalent, evidence-link emission,
  and regression guards for greetings and arithmetic.
