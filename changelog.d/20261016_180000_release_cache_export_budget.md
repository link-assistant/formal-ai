---
bump: patch
---

Move optional Docker build cache export after immutable pull-back, version, executable hash, privileged runtime, and anonymous registry verification. Give cache-only export a hard 90-second deadline so a slow cache service cannot consume the release verification budget. Required publication failures still refuse release receipts.
