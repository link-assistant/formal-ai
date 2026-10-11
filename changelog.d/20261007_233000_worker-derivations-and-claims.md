---
bump: minor
---

### Added
- The browser worker records a white-box derivation for every answer. It also answers `explain <answer id>` from the memory event log, matching the server byte for byte (issue #1184, R1184-9).
- Code debugging, explanation and review now claim a prompt only when it carries code. Summarization and rewriting claim one only when it carries the text to transform. Both runtimes enforce this (issue #1175, R1175-3).
- Statistics, word problems, arithmetic, compound interest, number constraints, unit conversion and calendar reasoning now claim a prompt only when it states the operand their own reader parses (issue #1175, R1175-3).
