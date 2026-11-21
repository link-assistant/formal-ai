import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { promisify } from "node:util";
import { startCli, temporaryHome } from "./support/structured-log-fixture.mjs";
import { startServer } from "../../../js/server/main.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { indexedProcedure, indexedProcedureBody, unrelatedProcedure } from "./support/indexed-procedure-capture.mjs";
import { procedureBooks, procedureBookCaptures } from "./support/procedure-publisher-captures.mjs";

import { recurrenceCases, recurrenceNativeCaptures } from "./support/recurrence-publisher-captures.mjs";

const RUSSIAN_REQUEST = "Как хорошо покакать?";
const CODEX_REQUEST = "Как какать?";
const RUSSIAN_SLUG = encodeURIComponent("хорошо-покакать");
const WIKIHOW_API = "https://ru.wikihow.com/api.php";
const parseUrl = (title) => `${WIKIHOW_API}?action=parse&page=${title}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
const RUSSIAN_API = parseUrl(RUSSIAN_SLUG);
const searchUrl = task => `${WIKIHOW_API}?action=query&list=search&srsearch=${encodeURIComponent(task)}&srlimit=5&format=json&origin=*`;
const sourceResults = (...titles) => JSON.stringify({ query: { search: titles.map(title => ({ title })) } });
const SHORT_SEARCH = searchUrl("какать");
const DISCOVERED_API = parseUrl(encodeURIComponent("хорошо покакать"));
const RUSSIAN_CAPTURE = readFileSync(new URL("../fixtures/issue-1396/wiki-how-ru.json", import.meta.url), "utf8");
const NHS_SOURCE = "https://www.nhs.uk/conditions/constipation/";
const NHS_CAPTURE = readFileSync(new URL("../fixtures/practical-help/toilet.html", import.meta.url), "utf8");
const NATIVE_CLI = process.env.FORMAL_AI_TEST_NATIVE_CLI;
const NATIVE_CASE = { skip: !NATIVE_CLI, timeout: 30_000 };
const NATIVE_MULTI_TURN_CASE = { ...NATIVE_CASE, timeout: 60_000 };
const WEB_SEARCH_TOOLS = [{ type: "web_search" }];

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

test("native offline chat synthesizes recurrences only from supplied publisher captures", NATIVE_MULTI_TURN_CASE, async (t) => {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  const run = await startCli(t, {
    binary: NATIVE_CLI,
    env: { FORMAL_AI_SOURCE_CACHE_DIR: cache, FORMAL_AI_OFFLINE: "1" },
    lifetime: NATIVE_MULTI_TURN_CASE.timeout,
  });
  await run.ready();
  async function ask(prompt) {
    const response = await fetch(`${run.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        model: "formal-ai", stream: false, messages: [{ role: "user", content: prompt }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).choices[0].message.content;
  }

  assert.doesNotMatch(await ask(recurrenceCases[0].prompt), /def factorial\(n\)/u);
  for (const [url, body] of recurrenceNativeCaptures()) storeCapture(cache, url, body);
  for (const { name, prompt } of recurrenceCases) {
    const retrieved = await ask(prompt);
    assert.ok(retrieved.includes(`def ${name}(n)`), `${prompt} \n${retrieved}`);
    assert.match(retrieved, /https:\/\/www\.wikifunctions\.org\/w\/api\.php/u);
  }
});

test("native offline chat discovers procedures from verified Wikibooks captures", NATIVE_CASE, async (t) => {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  for (const [index, book] of procedureBooks.entries()) {
    for (const [url, body] of procedureBookCaptures(book, index === 0)) storeCapture(cache, url, body);
  }
  const run = await startCli(t, {
    binary: NATIVE_CLI,
    env: {
      FORMAL_AI_SOURCE_CACHE_DIR: cache,
      FORMAL_AI_OFFLINE: "1",
      FORMAL_AI_DISABLED_SERVICES: "externalServiceWikihow",
    },
    lifetime: 25_000,
  });
  await run.ready();
  for (const book of procedureBooks) {
    const response = await fetch(`${run.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        model: "formal-ai", stream: false, messages: [{ role: "user", content: "How to " + book.task + "?" }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    const content = (await response.json()).choices[0].message.content;
    const firstStep = content.match(/^1\. (.+)$/mu)?.[1];
    assert.match(firstStep ?? "", book.firstStep);
    assert.ok(content.includes(book.page), content);
    assert.doesNotMatch(content, /unrelated instruction/u);
  }
});

test("native offline chat reads ranked publisher pages before giving procedural steps", NATIVE_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t);
  const query = encodeURIComponent(`how to ${indexedProcedure.task}`);
  const index = `https://api.duckduckgo.com/?q=${query}&format=json&no_html=1&skip_disambig=1`;
  storeCapture(cache, index, JSON.stringify({
    RelatedTopics: [unrelatedProcedure.page, indexedProcedure.page].map(FirstURL => ({
      FirstURL, Text: indexedProcedure.task,
    })),
  }));
  const pages = path.join(cache, "public-search-pages");
  mkdirSync(path.join(pages, "source-cache", "objects"), { recursive: true });
  storeCapture(pages, unrelatedProcedure.page, unrelatedProcedure.body);
  storeCapture(pages, indexedProcedure.page, indexedProcedureBody());
  const response = await fetch(`${url}/v1/chat/completions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    signal: t.signal,
    body: JSON.stringify({
      model: "formal-ai", stream: false,
      messages: [{ role: "user", content: `How to ${indexedProcedure.task}?` }],
    }),
  });
  assert.ok(response.ok, await response.clone().text());
  const content = (await response.json()).choices[0].message.content;
  assert.match(content.match(/^1\. (.+)$/mu)?.[1] ?? "", indexedProcedure.firstStep);
  assert.match(content, indexedProcedure.finalAction);
  assert.ok(content.includes(indexedProcedure.page), content);
  assert.ok(!content.includes(unrelatedProcedure.page), content);
});

async function startNativeProcedureReplay(t, { agentMode = false, lifetime = NATIVE_CASE.timeout } = {}) {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  storeCapture(cache, RUSSIAN_API, RUSSIAN_CAPTURE);
  storeCapture(cache, NHS_SOURCE, NHS_CAPTURE);
  storeCapture(cache, SHORT_SEARCH, sourceResults("Домашние растения", "хорошо покакать"));
  storeCapture(cache, DISCOVERED_API, RUSSIAN_CAPTURE);
  const run = await startCli(t, {
    binary: NATIVE_CLI,
    args: agentMode ? ["--agent-mode"] : [],
    env: { FORMAL_AI_SOURCE_CACHE_DIR: cache, FORMAL_AI_OFFLINE: "1" },
    lifetime,
  });
  await run.ready();
  return { cache, url: run.url, run };
}

async function askChatProcedure(url, messages, { source = RUSSIAN_API, tools } = {}) {
  const response = await fetch(`${url}/v1/chat/completions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      model: "formal-ai", stream: false, messages, tools,
    }),
  });
  assert.ok(response.ok, await response.clone().text());
  const content = (await response.json()).choices[0].message.content;
  assert.match(content, /1\. Не откладывайте/u);
  assert.ok(content.includes(source), content);
  return content;
}

test("native localized procedure and follow-up use matching captures over HTTP", NATIVE_CASE, async (t) => {
  const { url } = await startNativeProcedureReplay(t);
  const request = { role: "user", content: RUSSIAN_REQUEST };
  const first = await askChatProcedure(url, [request]);
  assert.match(first, /## Как хорошо покакать/u);
  await askChatProcedure(url, [request, { role: "assistant", content: first }, { role: "user", content: "Дай точные шаги" }]);
});

test("native chat uses the localized source for the short Russian prompt", NATIVE_CASE, async (t) => {
  const { url } = await startNativeProcedureReplay(t, { agentMode: true });
  const content = await askChatProcedure(url, [{ role: "user", content: CODEX_REQUEST }], {
    source: DISCOVERED_API, tools: WEB_SEARCH_TOOLS,
  });
  assert.match(content, /## Как какать/u);
});

test("native chat recovers a matching article after discarding an unrelated one", NATIVE_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t);
  const unrelated = JSON.parse(RUSSIAN_CAPTURE);
  unrelated.parse.title = "Домашние растения";
  const title = "хорошо покакать";
  const search = searchUrl(title);
  const matching = parseUrl(encodeURIComponent(title));
  storeCapture(cache, RUSSIAN_API, JSON.stringify(unrelated));
  storeCapture(cache, search, sourceResults(title));
  storeCapture(cache, matching, RUSSIAN_CAPTURE);
  const recovered = await askChatProcedure(url, [{ role: "user", content: RUSSIAN_REQUEST }], { source: matching });
  assert.ok(!recovered.includes(RUSSIAN_API), recovered);
});

test("native Responses uses procedural sources with the client web search tool available", NATIVE_MULTI_TURN_CASE, async (t) => {
  const { url, run } = await startNativeProcedureReplay(t, {
    agentMode: true,
    lifetime: NATIVE_MULTI_TURN_CASE.timeout,
  });
  try {
    await assertProceduralAgentRouting(url, t.signal);
  } catch (error) {
    throw new Error(`native procedural replay exited=${run.closed} signal=${run.signal}\n${run.stderr}`, { cause: error });
  }
});

test("native HTTP documentation answers require a matching publisher capture", NATIVE_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t);
  const source = "https://pandas.pydata.org/docs/reference/api/pandas.Series.diff.html";
  async function ask(prompt) {
    const response = await fetch(`${url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        model: "formal-ai", stream: false, messages: [{ role: "user", content: prompt }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).choices[0].message.content;
  }
  const prompt = "How does pandas Series.diff work?";
  const missing = await ask(prompt);
  assert.doesNotMatch(missing, /First discrete difference/u);
  assert.doesNotMatch(missing, /`read_file` capability/u);
  assert.ok(!missing.includes(source), missing);
  storeCapture(cache, source, readFileSync(new URL("../fixtures/documentation-method/diff.html", import.meta.url), "utf8"));
  for (const question of [prompt, "How does pandas.Series.diff work?"]) {
    const retrieved = await ask(question);
    assert.match(retrieved, /First discrete difference of Series elements/u);
    assert.ok(retrieved.includes(source), retrieved);
  }
  const file = await ask("Explain the file Series.diff in this workspace.");
  assert.match(file, /`read_file` capability/u);
});

test("native Agent CLI requires a retrieved capture for a known benchmark issue", NATIVE_CASE, async (t) => {
  const cache = temporaryHome(t);
  const session = path.join(cache, "session.json");
  const issue = "https://github.com/link-assistant/formal-ai/issues/698";
  const endpoint = "https://api.github.com/repos/link-assistant/formal-ai/issues/698";
  const body = "A test capture for the external benchmark harness.";
  const task = [
    "You are an AI issue solver using OpenAI Codex.",
    `Issue to solve: ${issue}`,
    "Continue pull request https://github.com/link-assistant/formal-ai/pull/816.",
    "Read the issue requirements, reproduce them with tests, and verify the change.",
  ].join("\n");
  const env = { ...process.env, HOME: cache, FORMAL_AI_SOURCE_CACHE_DIR: cache, FORMAL_AI_OFFLINE: "1" };
  const args = ["agent", "--task", task, "--session-json", session];
  async function fetchIssue() {
    await promisify(execFile)(NATIVE_CLI, args, { env, signal: t.signal });
    const step = JSON.parse(readFileSync(session, "utf8")).steps.find(step => step.tool === "web_fetch");
    assert.equal(step?.arguments.url, issue);
    return step.result;
  }
  assert.equal(await fetchIssue(), `no cached capture for ${endpoint}`);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  storeCapture(cache, endpoint, JSON.stringify({ body }));
  assert.equal(await fetchIssue(), body);
});

async function assertProceduralAgentRouting(url, signal) {
  async function ask(input) {
    const response = await fetch(`${url}/v1/responses`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal,
      body: JSON.stringify({ model: "formal-ai", input, stream: false, tools: WEB_SEARCH_TOOLS }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).output;
  }
  const exact = await ask(CODEX_REQUEST);
  const guide = exact.flatMap(item => item.content || []).map(part => part.text || "").join("\n");
  assert.match(guide, /## Как какать/u);
  assert.match(guide, /1\. Не откладывайте/u);
  assert.ok(guide.includes(DISCOVERED_API), guide);
  assert.ok(!exact.some(item => item.type === "web_search_call"), JSON.stringify(exact));
  const general = await ask("How to make pancakes?");
  assert.ok(general.some(item => item.content?.some(part => /make pancakes/iu.test(part.text))), JSON.stringify(general));
  assert.ok(!general.some(item => item.type === "web_search_call"), JSON.stringify(general));
  const help = await ask("Хочу какать, что делать?");
  assert.ok(help.some(item => item.content?.some(part => part.text.includes(NHS_SOURCE)
    && part.text.includes("Do not delay if you feel the urge"))), JSON.stringify(help));
  assert.ok(!help.some(item => item.type === "web_search_call"), JSON.stringify(help));
  const search = await ask("Search the web for pancakes");
  assert.ok(search.some(item => item.type === "web_search_call"), JSON.stringify(search));
}

test("JavaScript Responses uses procedural sources with the client web search tool available", {
  timeout: 30_000,
}, async (t) => {
  const dir = temporaryHome(t);
  const networkFetch = async (url) => {
    if (String(url) === NHS_SOURCE) return new Response(NHS_CAPTURE);
    if (String(url) === SHORT_SEARCH) {
      return Response.json(JSON.parse(sourceResults("Домашние растения", "хорошо покакать")));
    }
    if (String(url) === DISCOVERED_API) {
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
  // This fixture makes several sequential requests through an unoptimized debug
  // binary. Use the test's bound for the whole conversation and a shorter bound
  // for each individual request.
  const run = await startCli(t, {
    binary, args: ["--agent-mode"], lifetime: 120_000,
    env: { FORMAL_AI_ALLOW_NETWORK: "0", FORMAL_AI_LIVE_FETCH: "0" },
  });
  await run.ready();
  async function ask(input, advertised = REPORT_TOOLS) {
    const response = await fetch(`${run.url}/v1/responses`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", stream: false, input, tools: advertised }),
      signal: AbortSignal.timeout(18_000),
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
  const options = { skip: server === "native" && !binary, timeout: binary ? 90_000 : 30_000 };
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
