---
bump: patch
---

### Fixed

- Pin the third-party agent CLIs the end-to-end job installs. `@openai/codex@0.148.0` shipped overnight and drops the ENTER that answers its first-run trust dialog ([openai/codex#39487](https://github.com/openai/codex/issues/39487)), turning the Codex terminal leg red before any request reached the server under test; a test now holds the pinning rule `experiments/agentic_cli_matrix/clients.lock` already stated, for every CLI the project does not publish itself.
- Commit the case-study evidence `.gitignore` was silently dropping. Git never descends into an excluded directory, so the `!docs/case-studies/**/*.log` re-include could not rescue files under a `logs/` directory: `git add` reported success and committed nothing, and a test asserting the cited probe output exists passed locally while failing in CI. The directories are re-included beside the files.
