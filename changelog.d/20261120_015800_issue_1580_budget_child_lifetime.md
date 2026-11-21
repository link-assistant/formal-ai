---
bump: patch
---

### Fixed

- Keep execution budgets active after a command exits while its children still run. Preserve successful and failed command statuses once the process group finishes; expire lingering children, including TERM-resistant workers, and fail explicitly when group inspection is unavailable.
