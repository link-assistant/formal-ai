# Why the `agent` leg of the issue-#671 matrix aborts every tool call

The `E2E (agent)` job failed on this branch with the server faithfully rendering
an abort back to the reader:

    planned Final("Contents of `alpha.txt`:\n\n```text\nTool execution aborted\n```")

The abort is not ours. It happens inside the client, and it is reproducible
with no server and no network at all.

## The defect

`ai@6.0.260` added a guard (its changelog: *"Prevent automatic tool execution
when a model call ends with an unsafe finish reason"*) that reads the **V3**
shape of a finish reason:

```js
if (isToolExecutionAllowedFinishReason(chunk.finishReason.unified)) { … }
```

`ai` still accepts V2 models: `asLanguageModelV3` wraps them in a proxy whose
`doStream` pipes the stream through `convertV2StreamToV3`, which turns the V2
plain-string `finishReason` into `{unified, raw}` and the flat V2 `usage` into
the V3 nested shape.

`wrapLanguageModel` defeats that. `doWrap` hard-codes the version on the object
it returns while forwarding `doStream` to the wrapped model untouched:

```js
return {
  specificationVersion: "v3",
  …
  async doStream(params) { … return wrapStream ? … : doStream(); }
};
```

So wrapping a V2 model yields an object that *claims* V3 and still emits V2
chunks. `asLanguageModelV3` sees `"v3"` and returns it unconverted, the `finish`
chunk keeps its plain-string `finishReason`, `chunk.finishReason.unified` is
`undefined`, and the guard drops every tool call made on that step. The V2
compatibility warning is not printed either, because nothing detects the V2
model any more.

`@link-assistant/agent@0.25.0` hits this because it wraps its model to rewrite
prompts (`src/session/prompt.ts:1003`) and floats `"ai": "^6.0.1"` beside
`"@ai-sdk/openai-compatible": "^1.0.32"` — the provider is V2, so every release
of the agent's 0.25 line started aborting tool calls the day `ai@6.0.260` was
published, without any change of its own.

## `wrap-v2-repro.mjs` — the proof, no server needed

A hand-written V2 model emits one tool call and a `finish` with
`finishReason: "tool-calls"`. Run it twice, once unwrapped and once through
`wrapLanguageModel` with a middleware that returns its params unchanged:

    bun wrap-v2-repro.mjs

    AI SDK Warning (repro / v2-model): The feature "specificationVersion" is used in a compatibility mode. …
    v2 model, unwrapped      : tool executions = 1
    v2 model, wrapLanguageModel: tool executions = 0

Same model, same tool, same chunks; the wrap is the only difference.

## `streamtext-against-formal-ai.mjs` — the same thing over the wire

Drives `formal-ai serve --agent-mode` through the exact two package versions the
agent CLI resolves today (`ai@6.0.261`, `@ai-sdk/openai-compatible@1.0.48`):

    formal-ai serve --agent-mode --port 9951 &
    PROBE_WRAP=0 bun streamtext-against-formal-ai.mjs   # tool executions = 1
    PROBE_WRAP=1 bun streamtext-against-formal-ai.mjs   # TypeError: usage.inputTokens.total

Unwrapped, the tool runs and the answer carries the marker. Wrapped, the very
same V2/V3 confusion surfaces one line earlier in `ai`'s own transform — the
flat V2 `usage` reaches `asLanguageModelUsage`, which reads `usage.inputTokens.total`.
Two symptoms, one cause.

## Mutation test on the real CLI

Editing the guard in the installed `ai@6.0.261` to `return true` and re-running
the unmodified `@link-assistant/agent@0.25.0` against our server makes the leg
pass — `ALPHA_MARKER_11111`, no aborts. Instrumenting the call site instead
prints what the guard is actually handed:

    [probe] chunk.finishReason = "tool-calls" typeof string
    [probe] finishReason handed to the guard = undefined

## Our server is not implicated

`curl` against `/v1/chat/completions` shows a spec-correct SSE terminator
(`"finish_reason":"tool_calls"`), and `mapOpenAICompatibleFinishReason` in the
provider maps it to `"tool-calls"` correctly. This branch already pins
`@link-assistant/agent@0.26.0` in `experiments/agentic_cli_matrix/clients.lock`,
which passes.
