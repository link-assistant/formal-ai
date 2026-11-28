import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { promisify } from "node:util";
import { startCli, temporaryHome } from "./support/structured-log-fixture.mjs";
import { startServer } from "../../../js/server/main.mjs";
import { decodeTyped } from "../../../js/server/structured-log.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { indexedProcedure, indexedProcedureBody, unrelatedProcedure } from "./support/indexed-procedure-capture.mjs";
import { procedureBooks, procedureBookCaptures } from "./support/procedure-publisher-captures.mjs";
import { everydayMethodReply, wikiHowArticleCapture, wikiHowMethodsCapture } from "./support/wiki-how-method-capture.mjs";

import { recurrenceCases, recurrenceNativeCaptures } from "./support/recurrence-publisher-captures.mjs";

import { currencyCaptures, currencyExamples, localizedUsdToEur, quoteDate } from "./support/currency-publisher-captures.mjs";

const RUSSIAN_REQUEST = "Как хорошо покакать?";
const CODEX_REQUEST = "Как какать?";
const RUSSIAN_SLUG = encodeURIComponent("хорошо-покакать");
const WIKIHOW_API = "https://ru.wikihow.com/api.php";
const parseUrl = (title) => `${WIKIHOW_API}?action=parse&page=${title}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
const RUSSIAN_API = parseUrl(RUSSIAN_SLUG);
const searchUrl = task => `${WIKIHOW_API}?action=query&list=search&srsearch=${encodeURIComponent(task)}&srlimit=5&format=json&origin=*`;
const sourceResults = (...titles) => JSON.stringify({ query: { search: titles.map(title => ({ title })) } });
const SHORT_SEARCH = searchUrl("какать");
const SHORT_PARSE = parseUrl(encodeURIComponent("какать"));
const PUBLISHER_SEARCH = `https://ru.wikihow.com/Special:LSearch?fulltext=Search&search=${encodeURIComponent("какать покакать")}`;
const MISSING_TITLE = JSON.stringify({ error: { code: "missingtitle" } });
const PUBLISHER_RESULTS = `<a class="result_link" href="https://ru.wikihow.com/${RUSSIAN_SLUG}">хорошо покакать</a>`;
const DISCOVERED_API = parseUrl(encodeURIComponent("хорошо покакать"));
// Citations name the human article, not the API capture (issue #1632).
const RUSSIAN_ARTICLE = `https://ru.wikihow.com/${RUSSIAN_SLUG}`;
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

async function startNativeRecurrenceReplay(t) {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  const run = await startCli(t, {
    binary: NATIVE_CLI,
    silent: false,
    env: {
      FORMAL_AI_SOURCE_CACHE_DIR: cache,
      FORMAL_AI_OFFLINE: "1",
      FORMAL_AI_TRACE_REQUESTS: "1",
      FORMAL_AI_LOG_LEVEL: "trace",
      FORMAL_AI_LOG_STDERR: "1",
    },
    lifetime: NATIVE_CASE.timeout,
  });
  t.after(() => {
    const stages = run.stderr.split("\n")
      .filter(line => /field stage \(string (?:publisher_recurrence_|general_synthesis_|synthesis_)/u.test(line));
    t.diagnostic(stages.length ? stages.join("\n") : "No native synthesis stages were logged");
  });
  await run.ready();
  async function ask(prompt) {
    const started = performance.now();
    t.diagnostic(`Native recurrence request started for ${prompt}`);
    const response = await fetch(`${run.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: t.signal,
      body: JSON.stringify({
        model: "formal-ai", stream: false, messages: [{ role: "user", content: prompt }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    t.diagnostic(`Native recurrence response took ${Math.round(performance.now() - started)} ms`);
    return (await response.json()).choices[0].message.content;
  }
  return { cache, ask };
}

test("native offline chat needs a publisher capture to synthesize a recurrence", NATIVE_CASE, async (t) => {
  const { name, prompt } = recurrenceCases[0];
  const { ask } = await startNativeRecurrenceReplay(t);
  const answer = await ask(prompt);
  assert.match(answer, /no verified source statement was captured/iu);
  assert.ok(!answer.includes(`def ${name}(`), answer);
  assert.doesNotMatch(answer, /\?/u);
  const program = await ask("Write a Python program that calculates the factorial");
  assert.match(program, /^Which integer from 0 to 18/u);
  assert.doesNotMatch(program, /```|Execution status/u);
});

for (const { language, name, prompt } of recurrenceCases) {
  test(`native offline chat synthesizes a sourced ${name} recurrence for ${language}`, NATIVE_CASE, async (t) => {
    const { cache, ask } = await startNativeRecurrenceReplay(t);
    for (const [url, body] of recurrenceNativeCaptures()) storeCapture(cache, url, body);
    const retrieved = await ask(prompt);
    assert.ok(retrieved.includes(`def ${name}(n)`), `${prompt} \n${retrieved}`);
    assert.match(retrieved, /https:\/\/www\.wikifunctions\.org\/w\/api\.php/u);
  });
}

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
    assert.ok(content.includes(book.article), content);
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
  assert.doesNotMatch(content, /sha256|max_depth=|page\(s\)/u);
});

async function startNativeProcedureReplay(t, {
  agentMode = false, publisherSearch = false, publicArticle = false, lifetime = NATIVE_CASE.timeout,
} = {}) {
  const cache = temporaryHome(t);
  mkdirSync(path.join(cache, "source-cache", "objects"), { recursive: true });
  storeCapture(cache, RUSSIAN_API, RUSSIAN_CAPTURE);
  storeCapture(cache, NHS_SOURCE, NHS_CAPTURE);
  if (publisherSearch || publicArticle) {
    storeCapture(cache, SHORT_PARSE, MISSING_TITLE);
    storeCapture(cache, PUBLISHER_SEARCH, PUBLISHER_RESULTS);
  } else {
    storeCapture(cache, SHORT_SEARCH, sourceResults("Домашние растения", "хорошо покакать"));
  }
  if (publicArticle) storeCapture(cache, RUSSIAN_ARTICLE, wikiHowArticleCapture(RUSSIAN_ARTICLE));
  else storeCapture(cache, DISCOVERED_API, publisherSearch ? wikiHowMethodsCapture() : RUSSIAN_CAPTURE);
  const run = await startCli(t, {
    binary: NATIVE_CLI,
    args: agentMode ? ["--agent-mode"] : [],
    silent: !publicArticle,
    env: { FORMAL_AI_SOURCE_CACHE_DIR: cache, FORMAL_AI_OFFLINE: "1", FORMAL_AI_LOG_LEVEL: "debug" },
    lifetime,
  });
  await run.ready();
  return { cache, url: run.url, run };
}

async function askChatProcedure(url, messages, { source = RUSSIAN_ARTICLE, tools } = {}) {
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

async function assertGreetingProcedureStream(url, signal) {
  const ask = (messages, stream) => fetch(`${url}/v1/chat/completions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    signal,
    body: JSON.stringify({ model: "formal-ai", messages, stream, tools: WEB_SEARCH_TOOLS }),
  });
  const hello = { role: "user", content: "привет" };
  const first = await ask([hello], false);
  assert.ok(first.ok, await first.clone().text());
  const greeting = (await first.json()).choices[0].message;
  assert.equal(greeting.content, "Здравствуйте! Чем могу помочь?");
  const response = await ask([hello, { role: "assistant", content: greeting.content },
    { role: "user", content: CODEX_REQUEST }], true);
  assert.ok(response.ok, await response.clone().text());
  const frames = (await response.text()).split(/\r?\n/u)
    .filter(line => line.startsWith("data: ")).map(line => line.slice(6));
  assert.equal(frames.pop(), "[DONE]");
  const deltas = frames.flatMap(frame => JSON.parse(frame).choices.map(choice => choice.delta));
  assert.ok(!deltas.some(delta => delta.tool_calls?.length), "the reply should not request a client tool");
  const content = deltas.map(delta => delta.content || "").join("");
  assert.equal(content, everydayMethodReply(RUSSIAN_ARTICLE));
}

test("native streaming chat recovers the Russian guide after a greeting with only publisher search captured", NATIVE_CASE, async (t) => {
  const { url } = await startNativeProcedureReplay(t, { agentMode: true, publisherSearch: true });
  await assertGreetingProcedureStream(url, t.signal);
});

async function assertAnthropicProcedureStream(url, signal) {
  const turn = (role, text) => ({ role, content: [{ type: "text", text }] });
  const hello = turn("user", "Привет");
  const markdownSystem = "Text you output outside of tool use is displayed to the user as Github-flavored markdown in a terminal.";
  async function ask(messages, system = markdownSystem) {
    const response = await fetch(`${url}/api/anthropic/v1/messages?beta=true`, {
      method: "POST",
      headers: { "content-type": "application/json", "anthropic-version": "2023-06-01" },
      signal,
      body: JSON.stringify({
        model: "formal-ai", max_tokens: 8192, stream: true, messages,
        system: [{ type: "text", text: system }],
        thinking: { type: "adaptive", display: "updates" },
        tools: [{ name: "AskUserQuestion", input_schema: { type: "object" } }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    const events = (await response.text()).split(/\r?\n/u).filter(line => line.startsWith("data: "))
      .map(line => JSON.parse(line.slice(6)));
    assert.equal(events.at(-1).type, "message_stop");
    assert.ok(!events.some(event => event.content_block?.type === "tool_use"));
    assert.doesNotMatch(JSON.stringify(events), /sha256|max_depth=|publisher_parse_failed|publisher_article_failed|api\.php/u);
    return events.filter(event => event.delta?.type === "text_delta").map(event => event.delta.text).join("");
  }
  const greeting = await ask([hello]);
  assert.equal(greeting, "Здравствуйте! Чем могу помочь?");
  const messages = [hello, turn("assistant", greeting), turn("user", CODEX_REQUEST)];
  const markdown = everydayMethodReply(RUSSIAN_ARTICLE);
  assert.equal(await ask(messages), markdown);
  assert.equal(await ask(messages, "Reply in Markdown."), markdown);
  assert.equal(await ask(messages, "Output all communication in your response text instead."), markdown);
  assert.equal(await ask(messages, "Reply in plain text."), markdown.replace(/^#{2,3} /gmu, ""));
}

function assertProcedureLogs(log) {
  assert.match(log, /how_to:bounds/);
  assert.match(log, /publisher_parse_failed/);
  assert.match(log, /how_to:step/);
  const digest = createHash("sha256").update(wikiHowArticleCapture(RUSSIAN_ARTICLE)).digest("hex");
  assert.ok(log.includes(digest), "logs preserve the article capture's full SHA-256");
}

test("native Anthropic stream recovers the public wikiHow article without a MediaWiki capture", NATIVE_MULTI_TURN_CASE, async (t) => {
  const { url, run } = await startNativeProcedureReplay(t, {
    agentMode: true, publicArticle: true, lifetime: NATIVE_MULTI_TURN_CASE.timeout,
  });
  await assertAnthropicProcedureStream(url, t.signal);
  const logs = run.records().filter(record => record.component === "procedure.retrieval");
  assertProcedureLogs(JSON.stringify(logs));
});

test("JavaScript Anthropic stream recovers the public wikiHow article when MediaWiki is unavailable", {
  timeout: 30_000,
}, async (t) => {
  const dir = temporaryHome(t);
  const captures = new Map([
    [SHORT_PARSE, MISSING_TITLE],
    [PUBLISHER_SEARCH, PUBLISHER_RESULTS],
    [RUSSIAN_ARTICLE, wikiHowArticleCapture(RUSSIAN_ARTICLE)],
  ]);
  const networkFetch = async (url) => captures.has(String(url))
    ? new Response(captures.get(String(url))) : new Response(null, { status: 503 });
  const { server, url } = await startServer({
    agentMode: true,
    env: {
      FORMAL_AI_CACHE_DIR: dir, FORMAL_AI_MEMORY_PATH: path.join(dir, "memory.lino"),
      FORMAL_AI_DREAMING: "false", FORMAL_AI_RECORD_CHAT: "0", FORMAL_AI_LOG_DIR: dir,
      FORMAL_AI_LOG_LEVEL: "debug", FORMAL_AI_ALLOW_NETWORK: "true", FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: "false",
    },
    worker: new WorkerHost({ navigator: { onLine: true } }, { networkFetch }),
  });
  t.after(() => new Promise(resolve => server.close(resolve)));
  await assertAnthropicProcedureStream(url, t.signal);
  const logs = readdirSync(dir).filter(file => /^serve-.*\.lino$/u.test(file))
    .flatMap(file => readFileSync(path.join(dir, file), "utf8").trim().split("\n").map(decodeTyped))
    .filter(record => record.component === "procedure.retrieval");
  assertProcedureLogs(JSON.stringify(logs));
});

test("native wikiHow extraction uses the method requested by the user", NATIVE_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t);
  const task = "какать быстро";
  const source = parseUrl(encodeURIComponent(task.replaceAll(" ", "-")));
  storeCapture(cache, source, wikiHowMethodsCapture(task));
  const response = await fetch(`${url}/v1/chat/completions`, {
    method: "POST", signal: t.signal, headers: { "content-type": "application/json" },
    body: JSON.stringify({ model: "formal-ai", messages: [{ role: "user", content: `Как ${task}?` }] }),
  });
  assert.ok(response.ok, await response.clone().text());
  const content = (await response.json()).choices[0].message.content;
  assert.match(content, /1\. Сначала выберите[^\n]+\n2\. Уточните/u);
  assert.doesNotMatch(content, /Не откладывайте/u);
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
  // The matching capture is cited by its article URL (issue #1632).
  const recovered = await askChatProcedure(url, [{ role: "user", content: RUSSIAN_REQUEST }]);
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

test("native HTTP calculates dated currency pairs only from publisher captures", NATIVE_MULTI_TURN_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t, { lifetime: NATIVE_MULTI_TURN_CASE.timeout });
  async function ask(prompt) {
    const response = await fetch(`${url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: t.signal,
      body: JSON.stringify({ model: "formal-ai", stream: false, messages: [{ role: "user", content: prompt }] }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).choices[0].message;
  }

  assert.match((await ask(currencyExamples.at(-1)[0])).content, /verified JPY\/CAD exchange rate is unavailable/u);
  for (const [source, body] of currencyCaptures) storeCapture(cache, source, body);
  for (const [prompt, result, pair, rate] of [...currencyExamples, ...localizedUsdToEur]) {
    const answer = await ask(prompt);
    for (const detail of [result, rate, `/rate/${pair}?date=${quoteDate}`]) {
      assert.ok(answer.content.includes(detail), `${prompt}\n${answer.content}`);
    }
    assert.ok(answer.thinking_steps.some(step => step.detail.includes(result)), JSON.stringify(answer));
  }
  const expression = await ask(`Calculate 100 EUR + 100 USD at ${quoteDate}`);
  const amount = Number(expression.content.match(/= ([0-9.]+) EUR/u)?.[1]);
  assert.ok(Math.abs(amount - 190.86) < 0.01, expression.content);
  const identity = await ask("Calculate -1.005 USD in USD");
  assert.match(identity.content, /-1\.01 USD/u);
  const compound = await ask(`Invest $1000 at 8% annual interest compounded monthly for 5 years and convert the final amount to EUR on ${quoteDate}.`);
  assert.match(compound.content, /1489\.85 USD \* 0\.9086 = 1353\.68 EUR/u);
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

test("native HTTP release timelines require publisher captures for registered and new film series", NATIVE_MULTI_TURN_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t, { lifetime: NATIVE_MULTI_TURN_CASE.timeout });
  const fixture = name => readFileSync(new URL(`../fixtures/release-timeline/${name}.json`, import.meta.url), "utf8");
  const seed = readFileSync(new URL("../../../data/seed/release-timelines.lino", import.meta.url), "utf8");
  const query = seed.match(/^  query "([^"]+)"$/mu)[1];
  function storeTimeline(name, subject, language) {
    const endpoint = new URL("https://query-main.wikidata.org/sparql");
    endpoint.searchParams.set("query", query.replaceAll("{subject}", subject).replace("{language}", language));
    endpoint.searchParams.set("format", "json");
    storeCapture(cache, endpoint.href, fixture(name));
  }
  async function ask(prompt, history = []) {
    const response = await fetch(`${url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: t.signal,
      body: JSON.stringify({
        model: "formal-ai", stream: false, messages: [...history, { role: "user", content: prompt }],
      }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).choices[0].message.content;
  }
  const spiderMan = "Перечисли фильмы про Человека-паука в порядке выхода";
  const harryPotter = "List Harry Potter films in release order.";
  assert.doesNotMatch(await ask(spiderMan), /Тоби Магуайр|Человек-паук \(2002\)/u);
  assert.doesNotMatch(await ask(harryPotter), /Philosopher's Stone/u);
  storeTimeline("spider-man.ru", "Q2307877", "ru");
  storeTimeline("harry-potter.en", "Q216930", "en");
  storeCapture(cache,
    "https://www.wikidata.org/w/api.php?action=wbsearchentities&format=json&type=item&search=harry%20potter%20film%20series&language=en&uselang=en&limit=5",
    fixture("harry-potter-search.en"));
  const history = [
    { role: "user", content: "На php не получится написать?" },
    { role: "assistant", content: "Вот минимальная программа Hello World на языке PHP." },
  ];
  const registered = await ask(spiderMan, history);
  assert.match(registered, /Человек-паук \(2002\).*Человек-паук 2 \(2004\)/u);
  assert.match(registered, /Wikidata Query Service/u);
  const discovered = await ask(harryPotter);
  assert.match(discovered, /Philosopher's Stone \(2001\).*Deathly Hallows – Part 2 \(2011\)/u);
  assert.match(discovered, /Wikidata Query Service/u);
});

test("native HTTP reads registered airline baggage policy from the checked section", NATIVE_CASE, async (t) => {
  const { cache, url } = await startNativeProcedureReplay(t);
  const policy = "https://www.airindia.com/in/en/travel-information/baggage-guidelines/checked-baggage-allowance.html";
  const excerpt = "Infants are entitled to one collapsible stroller/carrycot/infant car seat.";
  storeCapture(cache, policy, `<h1>Checked Baggage</h1><h2>Domestic flights</h2><ul><li>${excerpt}</li></ul>`);
  const response = await fetch(`${url}/v1/chat/completions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    signal: t.signal,
    body: JSON.stringify({ model: "formal-ai", stream: false, messages: [{
      role: "user", content: "Does Air India allow a free stroller for an infant?",
    }] }),
  });
  assert.ok(response.ok, await response.clone().text());
  const answer = (await response.json()).choices[0].message.content;
  assert.ok(answer.includes(excerpt) && answer.includes(policy), answer);
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

test("native learning cycle replays only the frontier reports supplied by the caller", NATIVE_CASE, async (t) => {
  const home = temporaryHome(t);
  const env = { ...process.env, HOME: home, FORMAL_AI_SOURCE_CACHE_DIR: home, FORMAL_AI_OFFLINE: "1" };
  const frontier = "language-gap";
  const args = ["learn", "cycle", "--frontier", frontier, "--dry-run"];
  const execute = (...options) => promisify(execFile)(NATIVE_CLI, [...args, ...options], { env, signal: t.signal });
  await assert.rejects(execute(), /requires a learning report via --from/u);

  function writeReport(name, slug, entries) {
    const file = path.join(home, name);
    const lines = ["learning_frontier", `  frontier "${slug}"`];
    for (const [rank, topic] of entries) {
      lines.push(
        "  frontier_prompt",
        `    rank "${rank}"`,
        `    query "${topic}"`,
        '    language "es"',
        '    variation "concept_lookup"',
        `    prompt "¿Qué es ${topic}?"`,
        '    engine_intent "unknown"',
      );
    }
    writeFileSync(file, lines.join("\n") + "\n");
    return file;
  }

  const primary = writeReport("primary.lino", frontier, [[1, "naranja"], [2, "limón"]]);
  const continuation = writeReport("continuation.lino", frontier, [[3, "pomelo"]]);
  const unrelated = writeReport("other.lino", "upstream-benchmarks", []);
  const { stdout } = await execute("--from", primary, continuation);
  assert.match(stdout, /frontier "language-gap"/u);
  assert.match(stdout, /frontier_items "3"/u);
  assert.match(stdout, /held_out_tests "1"/u);
  await assert.rejects(execute("--from", primary, unrelated), /declares a different frontier/u);
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
  assert.ok(guide.includes(RUSSIAN_ARTICLE), guide);
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
    if (String(url) === SHORT_PARSE) return new Response(MISSING_TITLE);
    if (String(url) === SHORT_SEARCH) return new Response(null, { status: 500 });
    if (String(url) === PUBLISHER_SEARCH) return new Response(PUBLISHER_RESULTS);
    if (String(url) === DISCOVERED_API) return new Response(wikiHowMethodsCapture());
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
  await assertGreetingProcedureStream(url, t.signal);
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
