---
bump: minor
---

### Added

- The deterministic **agentic-coding loop now drives the Anthropic Messages
  (`/v1/messages`) and OpenAI Responses (`/v1/responses`) surfaces**, not just
  Chat Completions. The maintainer's framing for issue #468 was that the system
  must "call all the tools from any agentic CLI"; `claude` speaks Anthropic
  Messages and `codex` speaks OpenAI Responses, so both now emit native tool
  requests (`tool_use` content blocks / `function_call` output items) and
  *understand* fed-back tool results delivered in each protocol's own idiom (an
  Anthropic `tool_result` block carried on a `user` message, an OpenAI
  `function_call_output` item) so the loop advances rather than restarting.
- `AnthropicMessagesRequest` and `ResponsesRequest` now accept `tools` and
  `tool_choice`, translated into the shared OpenAI tool shape so a single
  deterministic planner backs all three surfaces.
- New public types for the Responses tool mirror: `ResponseFunctionToolCall` and
  the `ResponseOutputItem` enum (`Message` | `FunctionCall`), with
  `ResponseObject::output_messages()` and `ResponseObject::function_calls()`
  accessors.
- A focused test module (`tests/unit/agentic_surfaces.rs`, 9 tests) pins the
  mirror: tool emission in agent mode, tool-result feed-back advancing the loop,
  the final knowledge-base answer once the recipe is exhausted, refusal without
  agent mode, SSE `input_json_delta` streaming for `tool_use`, and symbolic
  fall-through for non-agentic tasks.

### Changed

- The chat, Anthropic, and Responses surfaces now share one `agentic_outcome`
  decision (refuse / plan / fall-through) so the agent-mode gate, per-tool
  permission gate, and planner behave identically everywhere; the symbolic
  fall-through still preserves `evidence_links`.
- `AnthropicMessage.content` is now a list of typed content blocks
  (`AnthropicContentBlock::Text` | `ToolUse`) instead of a single text block, and
  `ResponseObject.output` is now a list of `ResponseOutputItem`s. Wire-format JSON
  is unchanged for the text-only case.
