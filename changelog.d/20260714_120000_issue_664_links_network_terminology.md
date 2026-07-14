---
bump: minor
---

### Added

- Canonical `GET /v1/network` endpoint (and `/api/formal-ai/v1/network`) for the
  links-network projection of the event log, returning the same nodes/edges,
  `?trace=` filtering, `?format=dot` export, and 404-on-unknown-trace behavior.
- `scripts/check-associative-terminology.rs` hygiene lint that blocks *new*
  graph-named public API routes and Rust modules/files, wired into the release
  workflow's Lint job. It allowlists the deprecated `/v1/graph` alias, the
  Wikidata `knowledge_graph` engine, and the codecov coverage badge / external
  graph-API citations.

### Changed

- The associative surface is now consistently described as a *links network*,
  not a "graph": renamed `src/self_source_graph.rs` → `src/self_source_links.rs`
  and `src/agentic_coding/source_graph.rs` → `source_links.rs`, swept the web,
  desktop, and VS Code UI strings to "links network view", and updated
  `ARCHITECTURE.md`, `README.md`, and `REQUIREMENTS.md` terminology (fixing the
  stale `src/graph.rs` reference in R81).

### Deprecated

- `GET /v1/graph` (and `/api/formal-ai/v1/graph`) is now a deprecated alias of
  `/v1/network`. It still returns the identical payload, but the response is
  flagged deprecated (a `deprecation: true` header plus a `link` header pointing
  at the `/v1/network` successor) so existing desktop / VS Code / CLI clients
  keep working while migrating.
