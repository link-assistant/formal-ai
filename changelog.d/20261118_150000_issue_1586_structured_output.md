---
bump: minor
---

### Added

- Support structured JSON answers and machine-readable refusals on the OpenAI Chat Completions and Responses endpoints. Both runtimes validate answers against the requested schema before returning them and record the schema and result in the answer's derivation.
