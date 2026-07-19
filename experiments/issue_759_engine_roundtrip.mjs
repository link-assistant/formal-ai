import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createAgentProvider } = require("../desktop/lib/agent-provider.cjs");
const { createEngineManager } = require("../desktop/lib/engine-manager.cjs");

const apiBase = process.env.ISSUE_759_API_BASE || "http://127.0.0.1:8796";
const engineManager = createEngineManager({
  commandExists: (command) => command === "agent",
});
const proxyLog = [];
const passthrough = createAgentProvider({
  type: "commander",
  workingDirectory: process.cwd(),
  onEvent: (event) => proxyLog.push({ engine: "agent", event }),
});

const agentResult = await passthrough.run({
  commanderTool: engineManager.status().activeEngine,
  mode: "chat",
  readOnly: true,
  prompt: "Reply with exactly: agent desktop roundtrip",
  apiBase,
  agentProvider: { apiBase, model: "formal-ai" },
  sessionKey: "issue-759-live-fixture",
});
proxyLog.push({
  engine: "agent",
  status: agentResult.status,
  answer: agentResult.answer && agentResult.answer.content,
});

engineManager.setActiveEngine("out-of-box");
const nativeResponse = await fetch(`${apiBase}/v1/chat/completions`, {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({
    model: "formal-ai",
    messages: [{ role: "user", content: "Reply with exactly: native desktop roundtrip" }],
    stream: false,
  }),
});
const nativePayload = await nativeResponse.json();
proxyLog.push({
  engine: "out-of-box",
  status: nativeResponse.ok ? "ok" : "error",
  answer: nativePayload.choices?.[0]?.message?.content || "",
});

await passthrough.stop();
console.log(JSON.stringify({
  detected: engineManager.status().engines.map((engine) => engine.id),
  activeEngine: engineManager.status().activeEngine,
  proxyLog,
}, null, 2));
