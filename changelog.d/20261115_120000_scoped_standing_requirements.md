---
bump: patch
---

### Fixed
- A system prompt of one agent session is no longer stored as a learned standing requirement and replayed in another repository's session (issue #1559). A prompt that configures a session is recorded with role `instruction` and never learned. A learned rule is one sentence of at most 240 characters (`rule_max_chars`) that names no single task. Its topic is the statement's subject, never a pronoun or function word in en/ru/hi/zh. Each amendment carries the scope it was learned in (user, or repository and session for agent sessions) and applies only there, unless it was stated in several scopes and generalized after replay verifies it. Agentic final answers list applied amendments in their trace (`thinking_steps`, `reasoning`, `evidence_links`) instead of their text. Agent sessions sharing one memory store, as on a Hive Mind sidecar, now see only their own requirements unless `FORMAL_AI_SHARE_STANDING_REQUIREMENTS=1` is set.
