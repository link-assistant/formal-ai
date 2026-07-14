# Issue 687 — upstream assessment (R9)

The issue asks: "If issue related to any other repository/project, where we can
report issues on GitHub, please do so."

## The harness: OpenCode

The reported session used **OpenCode 1.17.18** as the agentic CLI, with Formal AI
as the model backend over the OpenAI-compatible HTTP API.

**Assessment: OpenCode is not at fault.**

- OpenCode advertised its tools (`websearch`, `webfetch`, `read`, `write`,
  `bash`) and faithfully rendered whatever the model returned.
- For all four prompts, Formal AI's planner returned no tool call — it produced
  the unknown-reasoning blurb (or a plan description for "Report"). OpenCode
  correctly displayed that text. There was nothing for it to execute because
  Formal AI emitted no tool call.
- This mirrors the predecessor investigation for issue **#676**, which likewise
  concluded the harness was not the cause; the gap was in Formal AI's own
  deterministic reasoning.

Therefore **no upstream issue is filed** — the defect and its fix are entirely
within `link-assistant/formal-ai` (this PR).

## Dogfooding note

Ironically, one of the four failing prompts was a request to *report the issue on
GitHub*. With this PR, that request now produces a real `gh issue create` tool
call against `link-assistant/formal-ai`, so the exact scenario in the screenshot
is now actionable from agentic mode — the assistant can file its own bug reports
when asked in natural language.
