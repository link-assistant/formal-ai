---
bump: patch
---

### Fixed
- KISS queries in a programming context (e.g., "что такое Kiss в рамках програмирования") now return the KISS software design principle instead of the rock band. Fixed by adding the " в рамках " Russian context delimiter, extending `context_programming` aliases with genitive/locative misspellings, adding an offline `concept_kiss_principle` corpus entry, and adding a context-aware Wikipedia search fallback in the browser worker.
