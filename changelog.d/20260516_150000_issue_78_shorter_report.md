---
bump: minor
---

### Changed
- **Report issue** prefilled body is now short enough to fit GitHub's `/issues/new?body=…` URL-length limit (issue #78). The verbose memory-upload instructions (`.zip` walkthroughs per OS, redaction reminders, full-memory explainer) moved out of the prefilled body and into a single repository doc, [`docs/upload-memory.md`](docs/upload-memory.md). The body now references that page with a one-line link instead of repeating the workflow each click (R112, R115, R117).
- Dialog transcripts inside the prefilled body now render as a single fenced code block with `U:` / `A:` line prefixes (issue example: `U: 1+2` / `A: 3`) instead of one Markdown subsection per message. Known-intent turns stay as plain `A: …`; only the `unknown` intent keeps the inline annotation (`A (intent: unknown, reported): …`) where the marker is needed to identify the missing rule, so the encoded `body=` parameter stays comfortably below GitHub's request-line cap (R116).

### Added
- [`docs/upload-memory.md`](docs/upload-memory.md): single canonical guide that explains what *full memory* means, walks through **Export memory**, redaction, and the two upload paths (GitHub Gist with no extension restrictions, or `.zip` for issue attachments), and documents why `.lino` is not yet a native attachment type (R117, R118).
- `docs/case-studies/issue-78/` with raw GitHub data, mirrored issue screenshots, root-cause analysis (the encoded `?body=` query string overflows the 8192-byte URL cap once the transcript reaches ~5 turns), the R115–R119 requirement matrix, and verification notes.
- Playwright e2e coverage in `tests/e2e/tests/multilingual.spec.js` and `tests/e2e/tests/demo.spec.js`: the Report-issue test now asserts the body links to `docs/upload-memory.md` and no longer contains the long per-OS Compress / Send-to instructions; the dialog-shape tests assert the `Legend: \`U\` = user, \`A\` = agent.` block, `U: …` / `A: …` line prefixes, and the absence of the old `### 1. …` / `- **Role**: …` subsections (R119).
- `experiments/issue-78-dialog-format.mjs` smoke script that prints the compact transcript for a hand-crafted set of cases (empty, greeting, unknown prompt, fenced code block, arithmetic dialogue).
