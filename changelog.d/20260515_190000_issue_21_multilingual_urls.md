---
bump: patch
---

### Fixed
- Wikipedia and concept-lookup source URLs now render in human-readable IRI form (e.g. `https://ru.wikipedia.org/wiki/Изумруд`) across every formal-ai surface — web demo, CLI, HTTP server, Telegram bot, and library — while the underlying link target stays the canonical percent-encoded URI so the link still resolves (issue #21).

### Added
- `formal_ai::humanize_url` — public helper that decodes percent-encoded UTF-8 sequences in a URL while preserving reserved URI delimiters (`; / ? : @ & = + $ , #`), with `String::from_utf8` fallback for malformed input. Mirrored by `humanizeUrl` in `src/web/formal_ai_worker.js`. Covered by Cyrillic, Devanagari, CJK, Japanese, Arabic, query-string-preservation, malformed-escape, lowercase-hex, and invalid-UTF-8 unit tests.
- Playwright e2e regression test that stubs the Wikipedia REST endpoint with the exact percent-encoded URL pattern from the bug report and asserts both the readable display text and the canonical `href` are present on the rendered assistant message.
- `docs/case-studies/issue-21/` — full root-cause analysis (URI vs IRI), reproducible curl example, library survey, and upstream considerations.
