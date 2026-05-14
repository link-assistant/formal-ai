---
bump: minor
---

### Added
- Added the universal 11-step solver loop (impulse → formalization → context → history → decomposition → TDD → synthesis → combination → verification → simplification → documentation) and wired it into `FormalAiEngine.answer` so every reply walks the same reasoning pipeline regardless of intent.
- Added a deterministic in-process append-only event log (`EventLog`) that records every step of the loop with content-addressed ids and projects to `evidence_links` plus a Links Notation `steps:` trace block.
- Added lightweight Unicode-block based language detection (`Language::{English, Russian, Hindi, Chinese, Unknown}`) so every impulse is tagged with `language:<slug>` evidence.
- Added in-process knowledge graph projection with `GraphNode`/`GraphEdge`/`KnowledgeGraph` and a `/v1/graph` endpoint that returns either Links Notation, JSON, or Graphviz DOT.
- Added `rate_limit` metadata on `/v1/models` responses, SSE streaming for `/v1/chat/completions` when `stream: true` is requested, and a configurable `SolverConfig` (offline mode, autonomy bounds) driven by environment variables.
- Added Telegram trace pointers: every chat reply now ends with a `/trace <id>` footer so users can request the inline event log for the message they just received.

### Changed
- Re-implemented Telegram, OpenAI, and library code paths to consume the universal solver instead of bespoke per-intent helpers, removing the previous hard-coded greeting/identity/hello-world branches in favor of pattern-matched specialized handlers (diagnostic, conversation memory, translation, algorithm synthesis, source-cache refresh, conflict surfacing, policy bounded-autonomy).
- Split `src/solver.rs` into `solver.rs` + `solver_helpers.rs` to keep both modules under the 1000-line hard limit and re-organized the imports so each helper is explicit.
