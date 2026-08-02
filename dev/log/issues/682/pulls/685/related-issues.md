# Related issues

Issue #682 was reported alongside two companion defects from the same 300-run live matrix
(5 CLIs × 6 tools × 10 phrasings). They are distinct root causes but share the agentic-CLI
context. Full bodies are captured in `raw/issue-680.txt` and `raw/issue-681.txt`.

## #680 — Tool calls are phrasing-gated, not intent-based (OPEN, `bug`)

URL: https://github.com/link-assistant/formal-ai/issues/680

> Tool-call emission is gated on a handful of hard-coded phrasings rather than on
> natural-language intent. Only a few exact wordings cause the planner to emit an
> OpenAI/Responses/Gemini `tool_call`; almost every other natural phrasing of the *same*
> request returns prose instead. Two whole capabilities — **web search** and **web fetch** —
> never emit a tool call for *any* phrasing.

Affects every targeted CLI (codex, opencode, gemini, qwen, `@link-assistant/agent`); single
root cause in the shared planner. Umbrella issue for the tool-routing defects.

## #681 — File-creation request emits a `read` tool_call instead of `write` (OPEN, `bug`)

URL: https://github.com/link-assistant/formal-ai/issues/681

> A natural-language file-creation request causes the server to emit a `read` tool_call on the
> target file (which does not exist yet) instead of a `write` tool_call — even when the client
> advertises both `read` and `write` tools.

Distinct from #680 ("no tool_call emitted"): here a tool_call *is* emitted but is the **wrong
tool**. In the 300-run matrix only 1 of 50 write runs actually created the file.

## Relationship to #682

- #682 is a **parsing/wire-protocol** defect: a well-formed OpenAI request is rejected at
  deserialization (`content: null`). It is independent of the planner logic behind #680/#681.
- #680 and #681 are **planner/routing** defects: the request parses fine but the wrong (or no)
  tool_call is produced.
- All three surfaced in the same live CLI matrix; #682 is the one that specifically kills the
  `qwen` loop because qwen is the only CLI that emits explicit `content: null`.
