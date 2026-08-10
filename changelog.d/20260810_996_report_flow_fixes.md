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
- Final answers that inline machine text — the general-change plan event and
  the formalized knowledge base — wrap it in a fenced `lino` code block, so
  the text survives GitHub-comment markdown rendering instead of collapsing
  into flowing prose (#996, hive-mind #2146).
