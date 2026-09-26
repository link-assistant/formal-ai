---
bump: patch
---

### Fixed
- The wave-T corpus gate (`issue_1138_no_silent_unknown`) failed assertion (a)
  for 18 of 1355 benchmark prompts: every elliptical news ask (`Anything big I
  missed today?`, `Что важного было сегодня?`, …) routes through the capability
  table to `Routed { web_search }`, but the web-search family's query
  extractors derive no query for them, so nothing answered and the
  comprehension-gap terminal emitted the seeded unknown opener. That terminal
  (`answer_with_legacy_fallback`) now consults the table before giving up
  (plan 10 leaf 10-10): a `Routed`, `Lowered` or `HonestGap` placement renders
  the capability-gap answer naming the capability the chat surface lacks —
  the same honesty the accepted `relative_period` + `web_search` arm already
  answers with — and an agent client keeps the fall-through because it
  advertises the tool itself. An `Ask` outcome is the table declining to place
  the prompt, so the unknown-reasoning ladder's own answer stands for it.
  Silent UNKNOWN is now unreachable for every table-placed benchmark prompt.
