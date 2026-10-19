---
bump: patch
---

### Fixed

- Continue missing-command recovery before the generic tool-result fallback when the current request has an exact bound failure receipt and an explicit caller workspace grant. Preserve existing recipe ownership and refuse stale, unrelated or altered recovery transactions.
