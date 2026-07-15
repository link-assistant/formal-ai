---
bump: patch
---

### Fixed
- Route typed generated-source artifacts and ordered compiler/run commands through the write and shell tools advertised by an agentic CLI instead of scraping rendered answer labels or describing execution performed in a server-private fixture. Follow-up output edits now update the source before it is written, failures stop the command sequence, and Chat Completions, Responses, Anthropic Messages, and Gemini use the same routing behavior.
- Prevent HTTP API requests from executing agent actions in Formal AI's embedded temporary workspace; the client harness remains the auditable execution boundary.
- Persist issue #716 observations and evidence-linked architectural amendments in the associative auto-learning substrate, and produce a human-review-gated client-execution report through Formal AI and the real Agent CLI.

### Tests
- Add issue #716 presentation-independence, all-catalog-language, API-surface, auto-learning, and real Agent CLI E2E coverage that verifies `main.rs` is written and the harness receives both Rust compile and execution commands.
