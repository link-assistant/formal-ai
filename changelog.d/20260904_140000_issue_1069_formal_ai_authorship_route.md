---
bump: minor
---

### Added
- `scripts/author-change-with-formal-ai.sh` lets Formal AI author a repository change through the real Agent CLI. It drives the same live loop the `experiments/agent_cli_e2e/` harnesses drive — `formal-ai serve` plus `@link-assistant/agent` — and then does the part those harnesses drop: it lands the file the CLI wrote, keeps the run's raw traces as evidence, and commits both with the `Formal-AI-Session`, `Formal-AI-Evidence` and `Formal-AI-Pull-Request` trailers the self-hosting metric reads. It opens no pull request and pushes nothing, so the work rides inside an ordinary pull request instead of needing one of its own (issue #1069).
