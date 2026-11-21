import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { startCli, temporaryHome } from "./support/structured-log-fixture.mjs";
import { startServer } from "../../../js/server/main.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

const RUSSIAN_REQUEST = "Как хорошо покакать?";
const CODEX_REQUEST = "Как какать?";
const RUSSIAN_SLUG = encodeURIComponent("хорошо-покакать");
const WIKIHOW_API = "https://ru.wikihow.com/api.php";
const parseUrl = (title) => `${WIKIHOW_API}?action=parse&page=${title}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
const RUSSIAN_API = parseUrl(RUSSIAN_SLUG);
const RUSSIAN_CAPTURE = readFileSync(new URL("../fixtures/issue-1396/wiki-how-ru.json", import.meta.url), "utf8");
const NATIVE_CLI = process.env.FORMAL_AI_TEST_NATIVE_CLI;

function storeCapture(cache, source, content) {
  const digest = createHash("sha256").update(content).digest("hex");
  // Source capture filenames use the URL's FNV-1a key. The content has its own SHA-256.
  let key = 0xcbf29ce484222325n;
  for (const byte of Buffer.from(source)) {
    key = BigInt.asUintN(64, (key ^ BigInt(byte)) * 0x100000001b3n);
  }
  writeFileSync(path.join(cache, "source-cache", "objects", `${digest}.body`), content);
  writeFileSync(path.join(cache, "source-cache", `${key.toString(16).padStart(16, "0")}.meta`),
    `source-capture-v2\nurl=${source}\nfetched_at=${Math.floor(Date.now() / 1000)}\nsha256=${digest}\n`);
}

async function startNativeProcedureReplay(t) {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  storeCapture(cache, RUSSIAN_API, RUSSIAN_CAPTURE);
  const run = await startCli(t, {
    binary: NATIVE_CLI,
    args: ["--agent-mode"],
    env: { FORMAL_AI_SOURCE_CACHE_DIR: cache, FORMAL_AI_LIVE_FETCH: "0" },
    lifetime: 25_000,
  });
  await run.ready();
  return { cache, url: run.url };
}

test("native localized procedure and follow-up use matching captures over HTTP", {
  skip: !NATIVE_CLI,
  timeout: 30_000,
}, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t);
  async function ask(messages, source = RUSSIAN_API) {
    const response = await fetch(`${url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        model: "formal-ai", stream: false, messages, tools: [{ type: "web_search" }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    const content = (await response.json()).choices[0].message.content;
    assert.match(content, /1\. Не откладывайте/u);
    assert.ok(content.includes(source), content);
    return content;
  }
  const request = { role: "user", content: RUSSIAN_REQUEST };
  const first = await ask([request]);
  assert.match(first, /## Как хорошо покакать/u);
  await ask([request, { role: "assistant", content: first }, { role: "user", content: "Дай точные шаги" }]);
  await ask([{ role: "user", content: CODEX_REQUEST }]);

  const unrelated = JSON.parse(RUSSIAN_CAPTURE);
  unrelated.parse.title = "Домашние растения";
  const title = "хорошо покакать";
  const search = `${WIKIHOW_API}?action=query&list=search&srsearch=${encodeURIComponent(title)}&srlimit=5&format=json&origin=*`;
  const matching = parseUrl(encodeURIComponent(title));
  storeCapture(cache, RUSSIAN_API, JSON.stringify(unrelated));
  storeCapture(cache, search, JSON.stringify({ query: { search: [{ title }] } }));
  storeCapture(cache, matching, RUSSIAN_CAPTURE);
  const recovered = await ask([request], matching);
  assert.ok(!recovered.includes(RUSSIAN_API), recovered);
});

test("native Responses uses procedural sources with the client web search tool available", {
  skip: !NATIVE_CLI,
  timeout: 30_000,
}, async (t) => {
  const { url } = await startNativeProcedureReplay(t);
  await assertProceduralAgentRouting(url);
});

async function assertProceduralAgentRouting(url) {
  async function ask(input) {
    const response = await fetch(`${url}/v1/responses`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", input, stream: false, tools: [{ type: "web_search" }] }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).output;
  }
  const exact = await ask(CODEX_REQUEST);
  assert.ok(exact.some(item => item.content?.some(part => /Не откладывайте/u.test(part.text))), JSON.stringify(exact));
  assert.ok(!exact.some(item => item.type === "web_search_call"), JSON.stringify(exact));
  const general = await ask("How to make pancakes?");
  assert.ok(general.some(item => item.content?.some(part => /make pancakes/iu.test(part.text))), JSON.stringify(general));
  assert.ok(!general.some(item => item.type === "web_search_call"), JSON.stringify(general));
  const help = await ask("Хочу какать, что делать?");
  assert.ok(help.some(item => item.content?.some(part => /nhs\.uk/u.test(part.text))), JSON.stringify(help));
  assert.ok(!help.some(item => item.type === "web_search_call"), JSON.stringify(help));
  const search = await ask("Search the web for pancakes");
  assert.ok(search.some(item => item.type === "web_search_call"), JSON.stringify(search));
}

test("JavaScript Responses uses procedural sources with the client web search tool available", {
  timeout: 30_000,
}, async (t) => {
  const dir = temporaryHome(t);
  const networkFetch = async (url) => {
    const request = new URL(url);
    if (request.hostname === "ru.wikihow.com" && request.searchParams.get("page") === "хорошо-покакать") {
      return Response.json(JSON.parse(RUSSIAN_CAPTURE));
    }
    return new Response(null, { status: 404 });
  };
  const { server, url } = await startServer({
    agentMode: true,
    env: {
      FORMAL_AI_CACHE_DIR: dir, FORMAL_AI_MEMORY_PATH: path.join(dir, "memory.lino"),
      FORMAL_AI_DREAMING: "false", FORMAL_AI_RECORD_CHAT: "0",
      FORMAL_AI_ALLOW_NETWORK: "true", FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: "false",
    },
    worker: new WorkerHost({ navigator: { onLine: true } }, { networkFetch }),
  });
  t.after(() => new Promise(resolve => server.close(resolve)));
  await assertProceduralAgentRouting(url);
});

const REPORT_TOOLS = [
  {
    type: "function", name: "request_user_input",
    description: "Request user input for one to three short questions and wait for the response. This tool is only available in Plan mode.",
    parameters: { type: "object" },
  },
  {
    type: "function", name: "exec_command",
    parameters: { type: "object", properties: { cmd: { type: "string" } }, required: ["cmd"] },
  },
];

async function startReportSession(t, binary) {
  const run = await startCli(t, { binary, args: ["--agent-mode"], lifetime: 25_000 });
  await run.ready();
  async function ask(input, advertised = REPORT_TOOLS) {
    const response = await fetch(`${run.url}/v1/responses`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", stream: false, input, tools: advertised }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).output;
  }
  async function question(input, expected) {
    const output = await ask(input);
    assert.ok(!output.some(item => item.type === "function_call"), JSON.stringify(output));
    const content = output.find(item => item.type === "message")?.content
      .filter(item => item.type === "output_text").map(item => item.text).join("\n");
    assert.match(content, expected);
    return { role: "assistant", content };
  }
  return { ask, question };
}

const user = content => ({ role: "user", content });

for (const [server, binary] of [["JavaScript", undefined], ["native", NATIVE_CLI]]) {
  const options = { skip: server === "native" && !binary, timeout: 30_000 };
  test(`${server} Codex report confirms destination and context without a Plan-only tool`, options, async (t) => {
    const { ask, question } = await startReportSession(t, binary);
    const report = user("зарепорти баг");
    const destination = await question([report], /3\. Задача GitHub/u);
    await question([report, destination, user("Не создавай задачу GitHub")], /3\. Задача GitHub/u);
    const targetHistory = [report, destination, user("3")];
    const context = await question(targetHistory, /1\. Оба журнала/u);
    await question([...targetHistory, context, user("1, 2")], /1\. Оба журнала/u);
    const output = await ask([...targetHistory, context, user("1")]);
    const call = output.find(item => item.type === "function_call");
    assert.equal(call?.name, "exec_command", JSON.stringify(output));
    assert.match(JSON.parse(call.arguments).cmd, /^formal-ai report create-issue .* --source both /u);
    const result = await ask([...targetHistory, context, user("1"), call, {
      type: "function_call_output", call_id: call.call_id,
      output: "https://github.com/link-assistant/formal-ai/issues/42",
    }]);
    assert.ok(result.some(item => item.content?.some(part => part.text?.includes("/issues/42"))), JSON.stringify(result));
  });

  test(`${server} report shows localized destination and context menus`, options, async (t) => {
    const { question } = await startReportSession(t, binary);
    const reportLocales = [
      { language: "en", prompt: "report", destination: /3\. GitHub issue/u, scope: /1\. Both logs/u },
      { language: "hi", prompt: "रिपोर्ट", destination: /3\. GitHub समस्या/u, scope: /1\. दोनों लॉग/u },
      { language: "zh", prompt: "报告问题", destination: /3\. GitHub 问题/u, scope: /1\. 两个日志/u },
      { language: "es", prompt: "reporta un fallo, por favor", destination: /3\. Incidencia de GitHub/u, scope: /1\. Ambos registros/u },
    ];
    for (const { language, prompt, destination, scope } of reportLocales) {
      await t.test(`shows both report menus in ${language}`, async () => {
        const request = user(prompt);
        const menu = await question([request], destination);
        await question([request, menu, user("3")], scope);
      });
    }
  });

  test(`${server} report honors Plan mode and compatible prompt tools`, options, async (t) => {
    const { ask, question } = await startReportSession(t, binary);
    const plan = {
      role: "developer",
      content: "<collaboration_mode># Plan Mode (Conversational)\n\nPlan the work.</collaboration_mode>",
    };
    const planOutput = await ask([plan, user("report")]);
    assert.ok(planOutput.some(item => item.name === "request_user_input"), JSON.stringify(planOutput));
    const defaultMode = { role: "developer", content: "<collaboration_mode></collaboration_mode>" };
    await question([plan, defaultMode, user("report")], /3\. GitHub issue/u);
    await question([user(plan.content), user("report")], /3\. GitHub issue/u);
    const compatibleTools = [{ ...REPORT_TOOLS[0], description: "Ask the user to choose options." }, REPORT_TOOLS[1]];
    const structured = await ask([user("report")], compatibleTools);
    assert.ok(structured.some(item => item.name === "request_user_input"), JSON.stringify(structured));
  });
}
