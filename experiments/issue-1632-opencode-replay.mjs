// Replay the reported OpenCode conversation against the living publisher.
// Run with `node experiments/issue-1632-opencode-replay.mjs` and OpenCode on PATH.
// Use `--http` to replay the same streamed conversation without the OpenCode client.
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { startServer } from "../js/server/main.mjs";
import { createReadOnlyFetch } from "../js/server/read-access.mjs";
import { WorkerHost } from "../js/server/worker-host.mjs";

const exec = promisify(execFile);
const directory = await mkdtemp(path.join(tmpdir(), "formal-ai-opencode-procedure-"));
const env = {
  ...process.env,
  FORMAL_AI_CACHE_DIR: directory,
  FORMAL_AI_MEMORY_PATH: path.join(directory, "memory.lino"),
  FORMAL_AI_DREAMING: "false",
  FORMAL_AI_RECORD_CHAT: "0",
  FORMAL_AI_OFFLINE: "0",
  FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: "false",
};
const networkFetch = createReadOnlyFetch({ env });
const publisherRequests = [];
const worker = new WorkerHost({ navigator: { onLine: true } }, {
  async networkFetch(url, options) {
    const response = await networkFetch(url, options);
    if (new URL(url).hostname === "ru.wikihow.com") publisherRequests.push([response.status, String(url)]);
    return response;
  },
});

let running;
try {
  running = await startServer({ agentMode: true, env, worker });
  const clientEnv = {
    ...env,
    XDG_CONFIG_HOME: path.join(directory, "config"),
    XDG_DATA_HOME: path.join(directory, "data"),
    XDG_CACHE_HOME: path.join(directory, "cache"),
    OPENCODE_DISABLE_AUTOUPDATE: "true",
    OPENCODE_DISABLE_DEFAULT_PLUGINS: "true",
    OPENCODE_DISABLE_MODELS_FETCH: "true",
    OPENCODE_CONFIG_CONTENT: JSON.stringify({
      enabled_providers: ["local"],
      provider: { local: {
        name: "Formal AI", npm: "@ai-sdk/openai-compatible",
        options: { baseURL: `${running.url}/v1`, apiKey: "local" },
        models: { "formal-ai": { name: "Formal AI" } },
      } },
      model: "local/formal-ai",
    }),
  };
  async function askOpenCode(prompt, session) {
    const command = exec("opencode", ["run", "--pure", "--format", "json", "--dir", directory,
      ...(session ? ["--session", session] : []), prompt], { cwd: directory, env: clientEnv });
    command.child.stdin.end();
    const { stdout } = await command;
    const events = stdout.trim().split("\n").map(line => JSON.parse(line));
    const parts = events.filter(event => event.type === "text");
    assert.ok(parts.length, stdout);
    return { session: parts[0].sessionID, text: parts.map(event => event.part.text).join("") };
  }
  const messages = [];
  async function askHttp(prompt) {
    messages.push({ role: "user", content: prompt });
    const response = await fetch(`${running.url}/v1/chat/completions`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", stream: true, messages }),
    });
    assert.ok(response.ok, await response.clone().text());
    const frames = (await response.text()).split(/\r?\n/u)
      .filter(line => line.startsWith("data: ")).map(line => line.slice(6));
    assert.equal(frames.pop(), "[DONE]");
    const text = frames.flatMap(frame => JSON.parse(frame).choices).map(choice => choice.delta.content || "").join("");
    messages.push({ role: "assistant", content: text });
    return { text };
  }
  const http = process.argv.includes("--http");
  const ask = http ? askHttp : askOpenCode;
  const first = await ask("привет");
  if (!http) assert.ok(first.session);
  assert.equal(first.text, "Здравствуйте! Чем могу помочь?");
  const next = await ask("Как какать?", first.session);
  assert.match(next.text, /1\. Прислушивайтесь к своему организму/u);
  assert.match(next.text, /\n2\. Сделайте так/u);
  assert.match(next.text, /https:\/\/ru\.wikihow\.com\//u);
  assert.doesNotMatch(next.text, /План поиска процедуры|reciprocal rank fusion|sha256|max_depth=/u);
  console.log(JSON.stringify({ client: http ? "http" : "opencode", greeting: first.text, verifiedNumberedInstructions: true,
    numberedSteps: next.text.split("\n").filter(line => /^\d+\. /u.test(line)).length,
    verifiedEverydayMethod: true, publisherRequests }, null, 2));
} finally {
  if (running) await new Promise(resolve => running.server.close(resolve));
  await rm(directory, { recursive: true, force: true });
}
