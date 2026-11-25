---
bump: minor
---

### Added

- Claude Code runs in `--permission-mode auto` by default, and the server answers Claude Code's auto-mode security-monitor requests at once from the seed policy in `data/seed/auto-mode-classifier.lino`, so Bash calls are no longer refused with "temporarily unavailable (timed out)".
- Claude Code's `AskUserQuestion` receives `multiSelect: true` for the multi-choice report-destination question.

### Fixed

- "How to X?" questions always probe wikiHow, the priority how-to source, even after a cached transient failure.
- How-to citations name the human article page (for example `ru.wikihow.com/хорошо-покакать`) instead of the MediaWiki `api.php` capture URL.
- The web-search-unavailable reply no longer lists unused sources.
- Russian "Отправь отчёт об ошибке" and similar phrasings are recognised as a request to report an issue.
