---
bump: minor
---

### Added
- Recognize a language-agnostic "learn from this data source" directive so the
  reported issue #499 prompt is routed to a new `learn_from_source` intent instead
  of `intent: unknown`. Recognition is data-driven from a seed-declared
  learnable-source registry (`data/seed/learning-sources.lino`) shared by the chat
  handler and the Agent CLI planner, and the same teaching directive drives
  Formal AI's own Agent CLI learning recipe end-to-end (pinned session plus a live
  external-CLI E2E step in CI).
