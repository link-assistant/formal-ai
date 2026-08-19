---
bump: patch
---

### Fixed

- Pin the third-party agent CLIs the end-to-end job installs. `@openai/codex@0.148.0` shipped overnight and drops the ENTER that answers its first-run trust dialog ([openai/codex#39487](https://github.com/openai/codex/issues/39487)), turning the Codex terminal leg red before any request reached the server under test; a test now holds the pinning rule `experiments/agentic_cli_matrix/clients.lock` already stated, for every CLI the project does not publish itself.
