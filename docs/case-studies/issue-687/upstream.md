# Issue 687 — upstream assessment

The original screenshot used OpenCode, while the required reproduction uses the
OpenCode-compatible Link Assistant Agent CLI. An upstream report would be
appropriate only if either client failed to advertise, relay, or execute a valid
Formal AI tool call.

The release-server E2E established the opposite:

- Agent advertised web search, web fetch, and shell capabilities.
- Formal AI emitted separate search/fetch cycles for the first and fourth turns.
- Agent executed the generated `gh issue create` shell command; the PATH-local
  fake recorded its arguments and returned a realistic issue URL.
- Continued sessions preserved enough history for recall and `it` resolution.
- At least nine `/v1/chat/completions` requests completed across four CLI
  invocations.

The defects were in Formal AI: missing seed-backed orchestration, unscoped
progress, and incomplete browser message routes. No reproducible Agent/OpenCode
defect remains, so filing an upstream issue would misattribute the problem.

The Agent project's unrestricted-execution warning is relevant operationally,
but it is documented behavior rather than a bug. The test follows that guidance
by isolating its working directory and preventing a real GitHub mutation.
