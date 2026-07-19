# Issue 714 requirement matrix

This matrix combines the issue body with the follow-up review on PR #726. “All
clients” means behavior is selected from advertised capability and protocol data,
not from a hard-coded CLI brand.

| Requirement | Evidence in this PR |
| --- | --- |
| Report performs the real side effect | Report intent emits an advertised run capability containing a shell-quoted `gh issue create`; the real Agent CLI E2E captures the actual argv. |
| Web search uses the client's tool | OpenCode ephemeral configuration enables its documented `websearch`; the planner selects Search, then Fetch, capabilities advertised by the client. |
| Do not substitute fetch for search | Search and Fetch are distinct typed capabilities and the research recipe orders discovery before retrieval. |
| No client-name branches | `interface-capabilities.lino` classifies aliases such as `bash`, `shell`, `run_command`, `websearch`, and `webfetch`; planner recipes consume the class. |
| Generalize report language | Report verbs, subjects, repository, templates, and localized action meanings are seed-backed. Exact whole-action classification prevents “write a report” side effects. |
| Preserve conversation context | Report title/body are derived from bounded user/assistant history and POSIX quoted before reaching `gh`. |
| Complete after tool execution | Tool results are resolved by explicit name or call id; the returned GitHub URL becomes the final answer. |
| Cover API protocols | Chat Completions, Responses, Anthropic Messages, and Gemini translate through the shared planner and completed-exchange recorder. |
| Cover Gemini's full loop | `functionCall` ids/arguments translate to assistant tool calls and `functionResponse` translates back to the matching tool result. |
| Focus Formal AI on reasoning and memory | The Agent CLI executes client tools and enforces its permissions. Formal AI plans, consumes results, and records verified executions; it does not duplicate the CLI runtime. |
| Auto-learn from real work | Completed tool name/input/output is a durable `tool_call` memory event; the final task cites it and stable ids merge retries. |
| Same-task real Agent CLI proof | `run_report_e2e.sh` boots the production server, invokes the installed Agent CLI, executes a sandboxed `gh`, checks the returned URL, and checks the durable tool record. |
| Wider ambitious workflow | Issue #687's merged four-turn Agent CLI experiment covers search→fetch→answer, report, recall, and associative learning in one continued session. |
| Debuggability without noisy defaults | `FORMAL_AI_TRACE_REQUESTS=1` exposes exact advertised tools and turns while remaining off by default. |
| No deferral | Missing run capability returns an actionable capability explanation; advertised capabilities are executed and completed in the same agentic loop. |

## Acceptance tests

- A minimal regression proves unnamed tool results are joined to the originating
  call id and retained.
- A durable-memory regression proves the tool event contains real arguments and
  output and is cited by the final task.
- Protocol integration tests prove report routing across different tool aliases
  and prove Gemini continues after `functionResponse`.
- The real Agent CLI experiment proves the same issue task at the process boundary.
