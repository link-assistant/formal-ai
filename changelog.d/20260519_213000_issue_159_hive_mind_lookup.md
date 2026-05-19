---
bump: patch
---

### Fixed
- Routed "What is Hive Mind?" / "Что такое Hive Mind?" prompts to a dedicated
  Hive Mind answer that prefers `link-assistant/hive-mind` before showing
  other web-search results, preventing the Wikipedia closest-match fallback
  from answering with unrelated pages such as LOIC.

### Added
- Added `scripts/decode-github-issue-url.rs` to decode prefilled GitHub issue
  URLs into readable Markdown for future overlong report-link investigations.
