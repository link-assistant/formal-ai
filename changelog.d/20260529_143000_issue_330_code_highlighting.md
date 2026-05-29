---
bump: minor
---

### Added
- Syntax highlighting for chat code blocks via a dependency-free, highlight.js-compatible tokenizer (`src/web/syntax-highlight.js`) covering rust, python, javascript/typescript, go, c, cpp, java, csharp, ruby, bash, and json (issue #330).
- A copy button on every rendered code block that copies the raw source to the clipboard with "Copied!" feedback.
- A "Copy as Markdown" button on each chat message that copies the whole message content (Markdown fences preserved).
- Localized strings for the new copy buttons in all four locales (en/ru/zh/hi).
- End-to-end Playwright tests proving highlighting renders and both copy buttons work against a freshly built `src/web`.
- Runnable example (`examples/issue-330-code-highlighting/`) with run/test instructions and a deep case study in `docs/case-studies/issue-330/`.
