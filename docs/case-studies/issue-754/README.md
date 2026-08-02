# Issue 754: Cursor CLI through MCP

## Outcome

`formal-ai with cursor` now launches the `cursor-agent` binary in interactive
or headless (`-p`) mode. Because Cursor does not expose a custom model base URL,
the wrapper gives it an isolated `.cursor/mcp.json` whose authenticated HTTP
server points at Formal AI's new `/mcp` route. Explicit `--global` setup merges
the same server into the user's Cursor config and `--undo` restores its backup.

## Requirements and resolution

| Requirement | Resolution | Verification |
| --- | --- | --- |
| Recognize `cursor` | Seed-backed integration with `command "cursor-agent"` | `cursor_is_seeded_as_cursor_agent_with_mcp_configuration` |
| Reach Formal AI without a Cursor base-URL flag | Streamable HTTP `/mcp` endpoint and `formal_ai_chat` tool | MCP endpoint integration test and real Cursor traces |
| Headless mode | Native `-p` invocation mapping | focused wrapper mode test |
| Interactive mode | no mode flag; Cursor opens its native TUI | focused wrapper mode test |
| Avoid persistent one-shot changes | temporary `HOME` with `.cursor/mcp.json` | focused wrapper test and persistent-config invariant test |
| Permanent configuration and rollback | merge-preserving global JSON with `.formal-ai.bak` | global idempotency/undo matrix test |
| Secure local transport | bearer header, server-side auth, and Origin validation | authentication/origin integration test |
| Documentation and release trigger | README, desktop server guide, and minor changelog fragment | source review and CI changelog gate |

## Root cause

The wrapper registry only described CLIs that accept OpenAI-, Anthropic-, or
Gemini-shaped endpoint overrides. Cursor was absent because its CLI model
selection does not provide a supported custom base-URL flag. Treating it as
another OpenAI client would therefore create configuration that Cursor ignores.

Cursor CLI does automatically load MCP configuration and documents `mcp list`
and `mcp list-tools` commands. Its `-p` option is the supported headless switch:

- [Cursor CLI usage and MCP auto-detection](https://docs.cursor.com/en/cli/using)
- [Cursor CLI parameters and MCP commands](https://docs.cursor.com/en/cli/reference/parameters)
- [Cursor CLI installation](https://docs.cursor.com/en/cli/installation)

The correct seam is consequently MCP, not a fabricated model alias. Formal AI
already had a loopback HTTP server and a universal solver, but no MCP transport
or tool projection. The implementation adds that narrow adapter and leaves the
existing protocol adapters unchanged.

## Protocol and security decisions

The endpoint implements the JSON-RPC lifecycle used by Cursor: `initialize`,
`notifications/initialized`, `ping`, `tools/list`, and `tools/call`. It declares
the revision requested by Cursor (currently `2025-11-25`), returns JSON for
requests, and acknowledges initialization in the JSON form required by the
tested Cursor build. A GET receives 405 because this stateless server does not
open an SSE stream. The transport shape follows the
[MCP Streamable HTTP transport](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)
and [base protocol](https://modelcontextprotocol.io/specification/2025-06-18/basic/index).

The 2025-06-18 transport text specifies an empty HTTP 202 response for a
notification. Live testing found that Cursor `2026.07.16-899851b` retries and
marks the MCP connection failed with that response, but becomes ready with an
empty JSON result. The compatibility response is intentional and covered by
both the focused test and the retained real-client trace.

The same transport specification warns that local MCP endpoints need Origin
validation, loopback binding, and authentication. `/mcp` is therefore protected
by the existing bearer-token policy, the generated Cursor config sends the
token as an Authorization header, and browser origins must be same-host or
absent. The wrapper's automatically started server remains bound to loopback.

## Reproduction and evidence

The initial test was written before implementation and failed in all three
expected places. [`red-test.log`](red-test.log) records the missing seed entry,
the 404 response from `/mcp`, and the unsupported-target error. After the fix,
[`green-test.log`](green-test.log) records the seven focused passing tests and
[`matrix-test.log`](matrix-test.log) records the broader wrapper matrix. The
complete post-fix unit run is in [`unit-test.log`](unit-test.log) (1,786 passed,
zero failed), the full integration run is in
[`integration-test.log`](integration-test.log) (145 passed, zero failed), and
[`doc-test.log`](doc-test.log) records the documentation check. Formatting,
Clippy, file-size, and hardcoded-language outputs sit beside them.

[`full-test.log`](full-test.log) is retained only as the earlier serial
diagnostic: it was intentionally stopped after all preceding binaries and the
first part of the unit binary passed because a serial unit run would duplicate
the already isolated unit verification. It is not used as evidence of a
completed repository-wide command. The earlier parallel run's sole timeout was
the heavyweight self-healing fixture under resource contention; its isolated
passing rerun is retained in
[`self-healing-rerun.log`](self-healing-rerun.log).

A real Cursor CLI (`2026.07.16-899851b`) was installed from Cursor's official
installer. Against the locally built authenticated server:

```text
$ cursor-agent mcp list
formal-ai: ready

$ cursor-agent mcp list-tools formal-ai
Tools for formal-ai (1):
- formal_ai_chat (prompt)
```

The outputs are preserved in [`cursor-mcp-list.log`](cursor-mcp-list.log) and
[`cursor-mcp-tools.log`](cursor-mcp-tools.log). The corresponding
[`cursor-server-trace.log`](cursor-server-trace.log) contains Cursor's real
`initialize`, `notifications/initialized`, and `tools/list` POSTs to `/mcp`.

No Cursor account or `CURSOR_API_KEY` is available in the test environment, so
a complete cloud-backed `cursor-agent -p "hi"` stops at Cursor authentication
before MCP initialization. That expected external precondition is captured in
[`cursor-headless.log`](cursor-headless.log); the automated fake-client fixture
still proves the exact wrapper invocation and launch-scoped config, while the
real `mcp` commands prove client/server interoperability.

## Formal AI authorship evidence

The regression test and its integration-module registration were produced by
driving the Agent CLI through the locally built Formal AI server. The raw Agent
event streams, prompts, stderr, and intermediate attempts are retained under
[`agent-cli-red`](agent-cli-red/). This includes the initial red-test session
`ses_0890b9c65ffeFPl7Mlh05JibiQ` and subsequent implementation sessions, making
the tool-driven history inspectable rather than replacing it with a summary.
