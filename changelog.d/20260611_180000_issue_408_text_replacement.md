---
bump: patch
---

### Fixed
- Apply user-requested text replacements to generated code answers, including follow-up replacement requests that refer to the previous assistant response.
- Accept broader replacement prompt shapes, including input-first phrasing, smart quotes, corner quotes, and punctuation-tolerant multi-word matches.
- Add deterministic remove, append, prepend, trim-whitespace, normalize-whitespace, case-conversion, and line-shape text/code edit operations with multilingual operation vocabulary triggers.
- Cover 50 benchmark-family prompt-answer examples across CoEdIT, EditEval, InstrEditBench, CodeEditorBench, CanItEdit, EDIT-Bench, HumanEvalFix, and SWE-bench style edit tasks.
- Add a manifest-backed issue #408 benchmark profile with 28 researched sources, 10 local variations per source, and a 280/280 pass-count ratchet.
- Document the issue #408 benchmark-source audit and the boundary between the repository-local profile and official full-upstream benchmark scoring.
