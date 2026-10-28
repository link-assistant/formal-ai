---
bump: patch
---

- Fetch only the exact source-event recorder for the existing three-minute provenance job, avoiding full-repository checkout timeouts while preserving the original source coordinates and minimal payload.
- Point the Mistral model-card citation to its official model repository.
- Resolve symlinked recorder entry points so successful invocation always writes the minimal provenance artifact.
