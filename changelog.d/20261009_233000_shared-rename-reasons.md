---
bump: patch
---

State a common rename reason once on its tree and inherit it for each record. Per-record reasons override the shared value, and a new tree resets it. This preserves rename provenance while removing eight duplicate metadata fields. Twelve focused rename-rule checks pass; the metadata repeated-field ratchet returns from 81 to 80.
