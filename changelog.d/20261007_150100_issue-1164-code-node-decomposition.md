---
bump: minor
---

### Added
- Recomposition builds a program from the aligned decomposed parts (issue #1164): each decomposed example keeps its own body with the output literal slotted, the generalized procedure carries one body per contributing language beside the `output_call` parameter, and the bound literal is escaped for its quote; a language no example contributed is refused instead of filled from a stored template.
- `adopt_decomposed_procedure` promotes a generalized procedure through the existing execution-and-approval gate of `adopt_extracted_procedure`, filling the capture provenance and license the decomposer leaves empty.
- Parts read from a page's code carry that page's URL (`decompose_code_node_from`), and the R language has a part vocabulary.
- The browser worker gets the decomposer as a JavaScript twin (`js/worker/formal_ai_worker_code_examples.js`): decompose, generalize, recompose, step records and Links Notation, with node tests over the captured Hello World pages.

### Changed
- `data/seed/code-example-parts.lino` stores no program templates and no Pascal vocabulary, so Pascal stays held out until a decomposed Free Pascal example supplies it.
