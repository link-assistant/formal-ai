---
bump: patch
---

### Fixed

- Wait for Codex's selected-model banner before submitting interactive prompts, preventing the loading-screen footer from consuming a test prompt before the client can submit it.
- Replay the captured loading/ready terminal sequence with a mocked interactive client during ordinary PR validation.
