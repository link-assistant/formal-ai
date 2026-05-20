---
bump: minor
---

### Added
- Added a `summarization` module (`src/summarization.rs`) implementing a
  deterministic formalize → summarize → deformalize pipeline with configurable
  `SummarizationMode` (`Topic` 1–5 words, `Short` ~20%, `Standard` ~50%, `Full`
  100%, `Expand` ~200%), explicit statement caps, NSM-style semantic-prime
  expansion, compound-word shortening, and a boilerplate filter that drops
  install / example sentences from compressed answers.
- Added a curated project registry (`data/seed/projects.lino`,
  `src/seed/projects.rs`) covering Link Assistant, Link Foundation, and
  LinksPlatform projects with weighted statements, English/Russian localized
  variants, repository URLs, topic labels, and aliases.
- Added a `project_lookup` handler that runs after `concept_lookup` and answers
  "What is <project>?" prompts using the curated registry plus the summarization
  pipeline, logging `summarization:mode`, `summarization:language`, repository
  evidence, and the web-search providers consulted alongside the local answer.
- Added `scripts/decode-github-issue-url.rs` to decode prefilled GitHub issue
  URLs into readable Markdown for future overlong report-link investigations.

### Fixed
- Routed "What is Hive Mind?" / "Что такое Hive Mind?" prompts through
  `project_lookup` as a promoted registry match for `link-assistant/hive-mind`
  before showing other web-search results, preventing the Wikipedia
  closest-match fallback from answering with unrelated pages such as LOIC.
  Hive Mind now shares the same generic project path as other repository
  records.
