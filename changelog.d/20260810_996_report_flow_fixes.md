---
bump: patch
---

### Fixed

- Harness and server log exports from the agentic `Report` flow are written
  into a surviving temporary directory and print their final path, instead of
  dropping `formal-ai-*.lino` session dumps into the caller's working
  directory — a repository checkout root stays clean (#945).
- Report-target answers that use machine values now select every target:
  `formal_ai` was silently dropped because prompt normalization turned the
  underscore into a space before matching (#996).
