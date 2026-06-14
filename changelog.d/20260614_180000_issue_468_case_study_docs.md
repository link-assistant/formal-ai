---
bump: patch
---

### Changed
- Rewrote the issue #468 case study (`docs/case-studies/issue-468/README.md`) and
  `REQUIREMENTS.md` rows **R306–R319** to describe the shipped `src/agentic_coding/`
  agentic-coding loop — the deterministic planner (server brain), the in-repo driver
  and offline corpus (client), the two permission gates, and the
  nine-primitives-as-links formalizer — replacing the earlier text that described a
  removed typed-struct draft.

### Added
- `docs/desktop/server-api.md` §4e documents the multi-surface agentic tool-calling
  loop: how each agentic CLI (`codex` via Responses, `opencode` + `agent` via Chat
  Completions, `claude` via Anthropic Messages) drives `formal-ai serve`, how the
  server emits the next tool call and consumes the fed-back result, and the
  `agent_mode` + `pkg_agentic_coding` gating — with external CLIs as front-ends,
  never embedded in the engine.
- A traceability test (`issue_468_agentic_coding_case_study_is_traceable` in
  `tests/unit/docs_requirements.rs`) pins `REQUIREMENTS.md` rows R306–R319, the case
  study `README.md`, and `formal-protocol-mapping.md` to the live implementation.
