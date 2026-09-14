---
bump: patch
---

### Fixed
- Agentic planner: a namespaced MCP research tool now outranks the client's own research alias. Claude Code advertises `WebSearch` beside a wired-up `mcp__issue781__websearch` but grants permission only for the MCP one, so planning its built-in alias ended the four-client research run at "Claude requested permissions to use WebSearch, but you haven't granted it yet" with no search recorded (issue #781, regression from the issue #1133 ordering).
