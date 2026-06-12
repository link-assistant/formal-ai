---
bump: patch
---

### Fixed
- Apply user-requested text replacements to generated code answers, including follow-up replacement requests that refer to the previous assistant response.
- Accept broader replacement prompt shapes, including input-first phrasing, smart quotes, corner quotes, and punctuation-tolerant multi-word matches.
- Add deterministic remove, append, prepend, trim-whitespace, and normalize-whitespace text/code edit operations with multilingual operation vocabulary triggers.
- Cover 50 benchmark-family prompt-answer examples across CoEdIT, EditEval, InstrEditBench, CodeEditorBench, CanItEdit, EDIT-Bench, HumanEvalFix, and SWE-bench style edit tasks.
- Document the issue #408 benchmark-source audit and scope boundary so the PR does not overclaim full upstream benchmark coverage.
