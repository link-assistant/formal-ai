---
bump: minor
---

### Added

- Closed the issue #498 auto-learning loop: `trending_learning_report()` re-answers every Google Trends catalog prompt, separates the ones the engine already routes from the *learning frontier* it cannot yet resolve, and hands that frontier to the human-gated issue #558 self-improvement learner. Because trending searches are open-domain questions, the learner honestly adopts nothing; the proposal-only result is rendered at `data/meta/google-trends-learning.lino`.
- Added a `google_trends_learning` Agent CLI recipe (`GOOGLE_TRENDS_LEARNING_TASK`) with a pinned session under `docs/case-studies/issue-498`, plus tests that keep the frontier split, proposal-only run, recipe routing, and documentation traceable byte-for-byte.
