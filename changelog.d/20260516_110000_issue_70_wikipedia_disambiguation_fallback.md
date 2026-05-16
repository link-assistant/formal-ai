---
bump: patch
---

### Fixed
- Issue #70: Prompts like "what is tesla" that match a Wikipedia disambiguation page now fall back to the full-text search endpoint to find the top-ranked article (e.g. "Tesla, Inc.") instead of returning an unknown-intent error
