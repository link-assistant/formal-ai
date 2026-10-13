---
bump: patch
---

Restore only the generated image publication stage to its retained logical source allowance of 90 minutes. The former blanket 30-minute stage cap cancelled Docker Hub mirroring after successful full and slim GHCR checks, leaving the immutable image transfer unsealed. Keep every publication operation, source/digest/runtime/anonymous check and per-operation limit; all other physical stages retain 30-minute caps. Hosted completion still requires a fresh production run. Supplied orchestration receives zero Formal AI authorship credit.
