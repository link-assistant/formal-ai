---
bump: minor
---

### Added
- Response-language preference (`last message language` default, `preferred selected language`, or `UI language`) in the web app, with new `settings.responseLanguage` / `settings.preferredLanguage` i18n entries for all four locales.
- `list_files_arg` `write_program` task (list files at a path supplied on argv) with templates for all ten catalog languages.
- Conversation-context recovery for follow-up program modifications: a follow-up such as "make the program accept a path as an argument" now reuses the language and task from the prior turn instead of failing with `missing`/`missing`.
- Case study `docs/case-studies/issue-324/` with timeline, root-cause analysis, solution plans, and a universal dynamic problem-solving roadmap.

### Fixed
- `write_program` answers (intro, unsupported message, and execution report) are now rendered in the detected response language for Russian, Hindi, and Chinese instead of always English. Applied in both the Rust engine and the browser worker so the GitHub Pages demo stays in parity.
