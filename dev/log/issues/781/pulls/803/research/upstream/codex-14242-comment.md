I can confirm the namespace-routing half of this issue with Codex CLI `0.144.6`, a local tool-only MCP server, and a custom Responses API provider.

## Minimal reproduction

1. Configure a local MCP server that advertises two read-only tools, `websearch` and `webfetch`, and no resources.
2. Configure Codex to use a custom Responses provider.
3. The provider receives the MCP tools as a Responses namespace similar to:

```json
{
  "type": "namespace",
  "name": "mcp__issue781",
  "tools": [
    {"type": "function", "name": "websearch", "parameters": {"type": "object"}},
    {"type": "function", "name": "webfetch", "parameters": {"type": "object"}}
  ]
}
```

4. If the provider replies with the apparently qualified but flat call below, Codex rejects it:

```json
{
  "type": "function_call",
  "call_id": "call_1",
  "name": "mcp__issue781__websearch",
  "arguments": "{\"query\":\"Acer A325-45 charger\"}"
}
```

Observed result: `unsupported call: mcp__issue781__websearch`.

5. Returning the same logical call as a `(namespace, name)` pair works:

```json
{
  "type": "function_call",
  "call_id": "call_1",
  "namespace": "mcp__issue781",
  "name": "websearch",
  "arguments": "{\"query\":\"Acer A325-45 charger\"}"
}
```

With that envelope, Codex dispatched one search, three sequential fetches, and received the final cited synthesis.

## Workaround

Our provider now recursively expands namespace children into qualified names only for internal capability selection, retains the original namespace definition, and rehydrates a selected child to separate `namespace` and `name` fields in the Responses output. We also mark the MCP tools `readOnlyHint: true`; without the annotation, a non-interactive read-only Codex run cancelled the otherwise valid calls as approval-requiring operations.

The complete red/green transcripts and regression are being preserved in [link-assistant/formal-ai#803](https://github.com/link-assistant/formal-ai/pull/803).

## Suggested fixes

- Document that Responses namespace calls must round-trip as the exact `(namespace, child name)` pair; a flattened fully-qualified `name` is not equivalent.
- Validate malformed/flattened namespace calls with a targeted diagnostic that includes the expected envelope instead of the generic `unsupported call` message.
- For custom/OSS providers that do not natively understand Responses namespace tools, expose an explicit provider capability to request flattened function definitions and normalize the response before routing.
- Keep tool discovery independent of resource discovery so a valid tool-only MCP server remains callable.

This matches the router analysis already posted above and adds a current, deterministic end-to-end reproduction against a custom Responses provider.
