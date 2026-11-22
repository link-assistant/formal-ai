---
bump: patch
---

### Fixed

- Allow OpenAI-compatible embeddings to use 1 to 4096 dimensions and return a `no_concepts` error with the input index when the lexicon finds no match.
- Recognize more multilingual occupation titles and preserve the B2B qualifier in job-title embeddings.
