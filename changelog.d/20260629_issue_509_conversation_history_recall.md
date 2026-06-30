---
bump: patch
---

### Added
- The Rust solver can now answer natural-language queries over prior dialog turns and persisted `.lino` memory, such as "When did I mention Rust?" or "Find Rust in another conversation", through the `conversation_recall` intent.
- The local HTTP surfaces (`/v1/chat/completions`, `/v1/responses`, and `/v1/messages`) now scan `FORMAL_AI_MEMORY_PATH` when a natural-language recall query asks about memory outside the current request history.
- The CLI now includes `formal-ai memory query --prompt ...` for direct natural-language recall over a saved `demo_memory` or `formal_ai_bundle` file.
