# Issue 744: advertised tool-schema projection

## Timeline and evidence

- `formal-ai` v0.297.1 routed qwen fetch/search/shell requests to tools, but qwen rejected the calls before execution.
- The original report captured missing `prompt` for `web_fetch` and missing `pattern` for `grep_search`.
- The first issue comment added qwen's required `is_background` shell field and showed that the validation message was subsequently presented as command output.
- The second comment generalized the contract: emitted arguments must honor the schema advertised in the current request, independent of CLI or natural language.
- The immutable GitHub API captures used for this investigation are in [`raw-data/`](raw-data/).

## Requirements reconstructed

1. `web_fetch` must emit qwen's required `url` and `prompt` fields.
2. Code search must map the planner's search subject to qwen's declared `pattern` field, never an undeclared `query` field.
3. Shell calls must include every declared required field, notably `is_background: false`, and use the declared command key (`command` or `cmd`).
4. File reads must map path aliases to the declared key; qwen's `absolute_path` must be absolute.
5. Strict schemas with `additionalProperties: false` must receive no shotgun aliases.
6. The same projection must apply on Chat Completions, Responses, Anthropic, and Gemini protocol paths and must not depend on the request language.
7. Tools with permissive object schemas must retain the legacy planner arguments.
8. Request tracing must expose planned versus emitted arguments when `FORMAL_AI_TRACE_REQUESTS=1`, while remaining off by default.
9. Client-injected `<system-reminder>` metadata must not participate in intent routing or become a synthesized tool instruction.

## Root cause

Planner functions intentionally carried semantic aliases such as `path`, `filePath`, and `file_path`, but the protocol boundary assumed the client would discard aliases it did not understand. Strict clients do the opposite: they validate the complete argument object and reject extra or missing keys.

An older boundary adapter handled only one special case (`command` to `cmd`) and only for the OpenAI Responses API. Chat Completions—the qwen path in the report—did not call it. The implementation therefore had neither a general schema projection nor a place to synthesize required fields.

Live qwen replay also exposed a second request-boundary fault: qwen serializes startup and deferred-tool metadata inside a `user` content part. Formal AI flattened that metadata together with the trailing request, so unrelated reminder phrases could win intent routing. The protocol now removes those marked reminder blocks before planning, history recording, and instruction synthesis.

## Prior art and design choice

JSON Schema already supplies the needed contract: `properties` defines the allowed output vocabulary, `required` defines completeness, and `type`/`enum`/`default` constrain synthesized values. The fix treats the request's schema as an executable interface description rather than maintaining a per-CLI branch table.

The planner remains capability-oriented and protocol-neutral. At the final protocol boundary, the projector:

1. locates the exact advertised tool schema;
2. maps semantic alias groups (path, command, search, and edit fields);
3. derives instruction fields from the user's request;
4. resolves `absolute_path` values;
5. fills required booleans, collections, numbers, enums, defaults, and strings with schema-valid values; and
6. emits only declared properties.

This applies to every tool call that Formal AI emits, including tools added later, without matching a CLI name. Empty object schemas remain permissive for backwards compatibility. This is a server protocol concern; the browser worker does not expose the OpenAI/Anthropic/Gemini tool-call server surfaces and therefore has no duplicate argument-emission path to mirror.

## Reproduction and verification

The regression was first run against the unmodified v0.297.1 commit in a detached worktree. It failed with:

```text
web_fetch omitted required prompt: {"format":"text","url":"https://example.com"}
```

`tests/integration/issue_744_qwen_tool_schema.rs` then verifies:

- qwen `web_fetch {url,prompt}` with no undeclared `format`;
- qwen `grep_search {pattern}` with no undeclared `query`;
- qwen shell `{command,is_background}`;
- qwen `read_file {absolute_path}`;
- qwen multipart startup reminders followed by the real fetch request;
- English, Russian, Hindi, and Chinese fetch requests through the same schema path;
- Codex's Responses-style `exec_command {cmd}`; and
- Anthropic's strict `Read {file_path}` with no extra path aliases.

The real Agent CLI evidence directory contains the server trace, CLI transcript, deterministic replay, and byte-for-byte generated file from a live OpenAI-compatible round trip. The qwen transcript records the reported non-interactive probes against the release binary; its trace also captures which capabilities that qwen version advertised eagerly versus deferred behind `tool_search`.
