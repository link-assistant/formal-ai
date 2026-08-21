// Minimal, network-free reproduction:
// `wrapLanguageModel` stamps `specificationVersion: "v3"` on its result while
// forwarding `doStream` to the wrapped model unchanged. Wrap a *V2* model and
// the returned object lies: `asLanguageModelV3` sees "v3" and skips
// `convertV2StreamToV3`, so the `finish` chunk keeps the V2 plain-string
// `finishReason`. Since ai@6.0.260 the tool-execution guard reads
// `chunk.finishReason.unified` -- `undefined` on a string -- so every tool call
// made on the final step is silently dropped.
import { streamText, tool, stepCountIs, wrapLanguageModel } from "ai";
import { z } from "zod";

function chunks() {
  return [
    { type: "stream-start", warnings: [] },
    { type: "response-metadata", id: "id-1", modelId: "v2-model", timestamp: new Date(0) },
    { type: "tool-input-start", id: "call-1", toolName: "read" },
    { type: "tool-input-delta", id: "call-1", delta: '{"filePath":"alpha.txt"}' },
    { type: "tool-input-end", id: "call-1" },
    { type: "tool-call", toolCallId: "call-1", toolName: "read", input: '{"filePath":"alpha.txt"}' },
    // A LanguageModelV2 `finish` part: `finishReason` is a plain string and
    // `usage` is flat numbers. That is the whole V2 surface under test.
    { type: "finish", finishReason: "tool-calls", usage: { inputTokens: 10, outputTokens: 5, totalTokens: 15 } },
  ];
}

const v2Model = {
  specificationVersion: "v2",
  provider: "repro",
  modelId: "v2-model",
  supportedUrls: {},
  async doGenerate() { throw new Error("unused"); },
  async doStream() {
    return {
      stream: new ReadableStream({
        start(controller) {
          for (const chunk of chunks()) controller.enqueue(chunk);
          controller.close();
        },
      }),
    };
  },
};

async function run(label, model) {
  let executed = 0;
  const result = streamText({
    model,
    stopWhen: stepCountIs(1),
    tools: {
      read: tool({
        description: "read a file",
        inputSchema: z.object({ filePath: z.string() }),
        execute: async () => { executed += 1; return "TOOL_RAN"; },
      }),
    },
    prompt: "read alpha.txt",
  });
  for await (const _ of result.fullStream) { /* drain */ }
  console.log(`${label}: tool executions = ${executed}`);
}

await run("v2 model, unwrapped      ", v2Model);
await run("v2 model, wrapLanguageModel", wrapLanguageModel({
  model: v2Model,
  middleware: [{ async transformParams({ params }) { return params; } }],
}));
