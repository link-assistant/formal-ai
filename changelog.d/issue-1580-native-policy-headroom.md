---
bump: patch
---

Extract release policy, data integrity and benchmark evaluation helpers into
ordinary native test child modules, retaining every assertion and the existing
size limits. Check standalone Cargo crate-root helper resolution before CI.
