---
bump: patch
---

### Fixed
- Russian phonetic transliterations "хелло" and "ворлд" are now recognized as valid hello/world tokens, and Russian language names "питоне" (Python), "расте" (Rust), and "джаваскрипт" (JavaScript) are now matched as language aliases. Previously, prompts like "Напиши хелло ворлд на питоне" fell through to `intent: unknown` (issue #53).
