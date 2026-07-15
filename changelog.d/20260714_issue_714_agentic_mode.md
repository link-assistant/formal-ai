---
bump: patch
---

### Fixed
- Route agentic “Report issue” actions through an advertised shell tool to `gh issue create`, and enable OpenCode's documented Exa-backed `websearch` tool in ephemeral Formal AI sessions.
- Preserve client-executed tool inputs and outputs as durable memory evidence after the final API turn, including unnamed OpenAI tool results and Anthropic/Responses translations, so the associative and dreaming loops can learn from work performed by an Agent CLI.
- Parse Gemini `functionCall`/`functionResponse` history, retain call ids, and continue the shared multi-turn planner after a Gemini client executes a tool.
