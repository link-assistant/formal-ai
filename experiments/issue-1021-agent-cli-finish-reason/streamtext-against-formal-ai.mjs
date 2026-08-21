// The same call as probe.mjs, with the one difference the agent CLI makes:
// the model goes through `wrapLanguageModel` before `streamText` sees it.
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";
import { streamText, tool, stepCountIs, wrapLanguageModel } from "ai";
import { z } from "zod";

const base = process.env.PROBE_BASE ?? "http://127.0.0.1:9951/api/openai/v1";
const provider = createOpenAICompatible({ name: "formalai", baseURL: base, apiKey: "unused" });
const WRAP = process.env.PROBE_WRAP === "1";
const inner = provider("formal-ai");
const model = WRAP
  ? wrapLanguageModel({ model: inner, middleware: [{ async transformParams(args) { return args.params; } }] })
  : inner;

let executed = 0;
const result = streamText({
  model,
  stopWhen: stepCountIs(3),
  tools: {
    read: tool({
      description: "read a file",
      inputSchema: z.object({ filePath: z.string() }),
      execute: async () => { executed += 1; return "ALPHA_MARKER_11111"; },
    }),
  },
  prompt: "read the file alpha.txt and print its contents",
});
for await (const part of result.fullStream) {
  if (part.type === "finish") console.log("finish finishReason =", JSON.stringify(part.finishReason));
}
console.log("wrapped         =", WRAP);
console.log("tool executions =", executed);
console.log("final text      =", JSON.stringify(await result.text));
