---
bump: patch
---

### Fixed

- Accept link-checker timeouts as host-side, the way 429 and 5xx already are.
  `eur-lex.europa.eu` stopped answering on 2026-08-21 — 45 seconds with no
  response — and reddened every open pull request over three
  `LEGAL-COMPLIANCE.md` citations none of them touches. A timeout carries no
  status code, so no accept range could match it. 404, 403 and 410 still fail.
