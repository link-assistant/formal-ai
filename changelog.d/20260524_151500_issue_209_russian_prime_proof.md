---
bump: patch
---

### Fixed
- Russian proof requests such as `привет. докажи что простых бесконечно` now
  resolve to the formal Euclid infinitude-of-primes proof instead of the
  generic proof-plan fallback.
- English, Russian, Hindi, and Chinese prime-infinitude prompts now share a
  coverage-checked proof test matrix so localized phrasing cannot regress to a
  generic plan or capability response.
