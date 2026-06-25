---
bump: minor
---

### Added
- Added repository-file formalization and summarization helpers that record file
  metadata, meta-language parser evidence, and recursive Markdown embedded
  grammar summaries.
- Generalized summarization from files to any repository resource, including
  folders: `RepositoryEntry`, `formalize_repository_resource`, and
  `summarize_repository_resource` summarize a directory tree by the recursive
  decompose → summarize → compose meta-algorithm loop, with recursion depth
  bounded by the summarization mode ladder and link-native `repository_directory`
  evidence.
