---
bump: patch
---

### Fixed
- Remove the redundant task clone exposed by the structured-gap regression update.
- Use native Unicode whitespace semantics when collecting source references in
  JavaScript, keeping the generated TypeScript and native parser aligned.
