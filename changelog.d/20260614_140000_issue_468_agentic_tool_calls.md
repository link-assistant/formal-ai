---
bump: minor
---

### Added

- A **deterministic agentic planner** (`src/agentic_coding/planner.rs`) — the
  server's "brain" for issue #468's *"solve such tasks in agentic mode"*
  framing. It is a pure function of the conversation so far and the tool names an
  agentic CLI advertised, driving a small state-machine recipe
  (`web_search → web_fetch → write_file → run_command → final`) that formalizes
  «Сказка о рыбаке и рыбке» into a Links Notation knowledge base. Steps whose
  tool the CLI did not advertise are skipped, and tool *errors* are observed (an
  errored fetch is ignored and the formalizer falls back to the canonical
  synopsis), so the loop always completes with a stable, all-nine-primitive
  document. No sampling, no hidden state — the same history always yields the
  same plan, keeping neural inference a NON-GOAL.

### Changed

- The OpenAI-compatible chat endpoint (`create_chat_completion_with_solver`) now
  **emits `tool_calls`** with `finish_reason: "tool_calls"` when agent mode is on
  and a formalization task is in flight, closing the core gap that the server
  could never *request* a tool — it previously hard-coded `finish_reason: "stop"`
  on every turn. A `tool`-role result feeds back into the planner on the next
  request until the recipe is exhausted, at which point the server answers with
  the knowledge base inline (`finish_reason: "stop"`). Unrecognised requests
  still fall through to the ordinary symbolic solver, so non-agentic behaviour is
  byte-for-byte unchanged. `ChatMessage` gained OpenAI `tool_calls` /
  `tool_call_id` / `name` fields and `ToolCall` / `FunctionCall` types so tool
  requests and results round-trip through the wire format (issue #468).
