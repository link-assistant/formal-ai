---
bump: patch
---

### Fixed

- Wait for Codex's selected-model banner and rendered draft, then send Enter separately after quiet output, preventing loading-screen and paste-burst handling from leaving a test prompt unsubmitted.
- Replay the captured loading/ready terminal sequence with a mocked interactive client during ordinary PR validation.
