---
bump: patch
---

### Fixed
- Route generated source, compiler/check commands, and run commands through the write and shell tools advertised by an agentic CLI instead of describing execution performed in a server-private fixture. Follow-up output edits now update the source before it is written, failures stop the command sequence, and Chat Completions, Responses, and Gemini use the same routing behavior.
- Prevent HTTP API requests from executing agent actions in Formal AI's embedded temporary workspace; the client harness remains the auditable execution boundary.

### Tests
- Add issue #716 unit and API-surface regressions plus a real Agent CLI E2E run that verifies `main.rs` is written and the harness receives both Rust compile and execution commands.
