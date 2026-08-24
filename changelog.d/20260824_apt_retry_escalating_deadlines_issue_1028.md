---
bump: patch
---

CI apt-install retries now allocate an enclosing step budget across progressively longer attempts, giving the final retry enough time to complete a degraded-but-recovering mirror download.
