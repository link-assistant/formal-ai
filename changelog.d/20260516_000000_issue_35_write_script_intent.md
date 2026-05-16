---
bump: minor
---

### Added
- Generic "write a script in \<language>" requests now route to the matching code block instead of returning `intent: unknown`. Supports English ("write a script in Python"), Russian with inflected language names ("Напиши скрипт на питоне", "расте", "джаваскрипт"), Hindi, and Chinese phrasing (issue #35).
