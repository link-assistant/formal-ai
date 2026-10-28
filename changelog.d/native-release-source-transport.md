---
bump: patch
---

Reuse the verified same-run source history for native, desktop and extension
release consumers, and advertise detached source history during tag authorization
so Git does not download it again. Preserve all byte, tag and ancestry checks.
Only stage packaging console evidence after packaging actually starts.
