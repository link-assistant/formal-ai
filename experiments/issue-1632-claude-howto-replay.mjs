// Replay the reported Claude Code turn ("Как какать?") through the Anthropic
// Messages endpoint with Claude Code's tool list, against the living publisher.
// Tool calls the server plans are answered the way Claude Code would: WebFetch
// is fetched live, anything else gets an empty result. Run with
// `node experiments/issue-1632-claude-howto-replay.mjs [prompt]`.
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { startServer } from "../js/server/main.mjs";
import { createReadOnlyFetch } from "../js/server/read-access.mjs";
import { WorkerHost } from "../js/server/worker-host.mjs";

const directory = await mkdtemp(path.join(tmpdir(), "formal-ai-claude-howto-"));
const env = { ...process.env, FORMAL_AI_CACHE_DIR: directory, FORMAL_AI_MEMORY_PATH: path.join(directory, "memory.lino"),
  FORMAL_AI_DREAMING: "false", FORMAL_AI_RECORD_CHAT: "0", FORMAL_AI_OFFLINE: "0" };
const tool = (name) => ({ name, description: name, input_schema: { type: "object", properties: {} } });
const tools = process.env.NO_TOOLS ? [] : ["Task", "Bash", "Glob", "Grep", "Read", "Edit", "Write", "WebFetch", "WebSearch", "AskUserQuestion"].map(tool);
let running;
try {
  running = await startServer({ agentMode: true, env, worker: new WorkerHost({ navigator: { onLine: true } }, { networkFetch: createReadOnlyFetch({ env }) }) });
  const messages = [{ role: "user", content: process.argv[2] ?? "Как какать?" }];
  for (let turn = 0; turn < 8; turn += 1) {
    const response = await fetch(`${running.url}/v1/messages`, { method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", max_tokens: 4096, ...(tools.length ? { tools } : {}), messages }) });
    const body = await response.json();
    if (!body.content) { console.log(response.status, JSON.stringify(body)); break; }
    messages.push({ role: "assistant", content: body.content });
    const uses = body.content.filter(block => block.type === "tool_use");
    if (!uses.length) { console.log(body.content.map(block => block.text ?? "").join("")); break; }
    const results = [];
    for (const use of uses) {
      console.error(`tool_use ${use.name} ${JSON.stringify(use.input).slice(0, 300)}`);
      let content = "";
      if (use.name === "WebFetch" && use.input.url) content = await (await fetch(use.input.url)).text().catch(String);
      results.push({ type: "tool_result", tool_use_id: use.id, content: content.slice(0, 100000) });
    }
    messages.push({ role: "user", content: results });
  }
} finally {
  if (running) await new Promise(resolve => running.server.close(resolve));
  await rm(directory, { recursive: true, force: true });
}
