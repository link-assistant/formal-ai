# Upstream investigation — is any defect in OpenCode (or another project)?

Issue #676 was reported while driving the Formal AI server through
[OpenCode](https://github.com/sst/opencode) `1.17.18` over the OpenAI-compatible HTTP API
(`formal-ai with opencode`). Requirement **R11** of the issue asks: *if the problem is
caused by another repository/project (e.g. OpenCode), report it upstream with a
reproducible example, a workaround, and a suggested code fix.*

This note records the upstream triage so the conclusion is auditable.

## Method

For each reported failure we traced the request end-to-end:

1. What OpenCode sent to the server — a standard `POST /v1/chat/completions` with the
   user's prompt in `messages`, optional `tools`, and `stream` set by the client.
2. What the Formal AI server returned — the intent route, the response body, and (in
   agentic mode) the emitted tool calls / `finish_reason`.
3. How OpenCode rendered that response.

The server-side reproductions live in the unit and integration tests referenced in
[`README.md`](./README.md) §6. They reproduce every reported failure **without OpenCode
in the loop** — by calling the solver / planner / HTTP endpoint directly.

## Finding: every defect is on the Formal AI side

| Reported failure | Where it is decided | OpenCode's role |
|------------------|--------------------|-----------------|
| `execute pwd` → unknown (R1) | `shell_command_for_task()` could only emit `ls` | Faithfully forwarded the prompt and rendered the server's `Final` fallback |
| NL file listing → no list (R2) | `asks_for_directory_listing()` too narrow | Faithful forward/render |
| `Now your name is Ineffa` → unknown (R3) | No `set_assistant_name` intent existed | Faithful forward/render |
| `How are you?` == `Hello` (R4) | how-are-you phrases were bundled into `intent_greeting` | Faithful forward/render |
| `Can you fix it yourself?` → unknown (R5) | self-heal trigger vocabulary too narrow | Faithful forward/render |

In each case OpenCode transmitted the user's text verbatim and displayed exactly what the
server produced. The generality gap — a slightly different phrasing than the seed
anticipated falling through to the *unknown* opener — is entirely within Formal AI's
deterministic router. Reproducing each failure with a direct HTTP/solver call (no client)
yields the identical wrong answer, which rules OpenCode out as the cause.

## Conclusion: no upstream issue is warranted

Because none of the failures originate in OpenCode (or any other third-party project), we
did **not** open an upstream bug report — filing a defect against a project that behaved
correctly would be noise. R11 is satisfied by this negative finding: the responsible code
was located, and it is in this repository, where it is fixed in PR #678.

If a genuine OpenCode-side defect is discovered in future reproduction (for example a
mishandled `reasoning`/`thinking` block or a tool-call schema mismatch), the report would
go to <https://github.com/sst/opencode/issues> with:

- a minimal `curl` reproduction against a stock `formal-ai` server,
- the exact request/response JSON, and
- a suggested patch or workaround.

No such defect was observed for the behaviours in issue #676.
