---
bump: minor
---

### Added

- Closed the issue #498 auto-learning loop: `trending_learning_report()` re-answers every Google Trends catalog prompt, separates the ones the engine already routes from the *learning frontier* it cannot yet resolve, and hands that frontier to the human-gated issue #558 self-improvement learner. Because trending searches are open-domain questions, the learner honestly adopts nothing; the proposal-only result is rendered at `data/meta/google-trends-learning.lino`.
- Added a `google_trends_learning` Agent CLI recipe (`GOOGLE_TRENDS_LEARNING_TASK`) with a pinned session under `docs/case-studies/issue-498`, plus tests that keep the frontier split, proposal-only run, recipe routing, and documentation traceable byte-for-byte.

### Fixed

- Made the live Agent-CLI ↔ formal-ai E2E harness (`experiments/agent_cli_e2e/run_agent_cli.sh`) resilient to the third-party `@link-assistant/agent` CLI's non-deterministic early exit: the deterministic server plans the same next step every time, but the external CLI occasionally stops after the first tool round without writing the file, so the harness now retries the whole invocation up to `ATTEMPTS` (default 5) times and still enforces every hard assertion on a genuine, complete round-trip.
