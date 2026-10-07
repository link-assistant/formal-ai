// Issue #918 (E71, R914-6), browser root: the `memory_program` and
// `memory_program_gap` rows of the worker's solve table answer exactly as the
// native memory surface does. Both runtimes compile a request against
// data/seed/memory-programs.lino and render the outcome through the seeded
// `memory_program_*` responses. Prompts, stores and expected wording are the
// ones rust/tests/unit/issue_918_browser_twins.rs asserts against
// `execute_memory_query_with_options`.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");
const defaultFetch = worker.fetch;

const RENAME_REQUEST = "List every fact I contributed about X and rename X to Y in all of them.";
const RENAME_EN = "Memory program memory_program_41ef0c9602340676 matched 1 event(s), changed 1, and stopped at fixpoint after 2 iteration(s).";
const RENAME_EMPTY_RU = "Программа памяти memory_program_41ef0c9602340676 сопоставила событий: 0, изменила: 0 и остановилась с результатом fixpoint после итераций: 1.";
const DESTRUCTIVE_EN = "Destructive memory actions require explicit human confirmation. No memory event was erased or retracted.";
const GAP_EN = "I could not compile this memory request without dropping a step. program_gap:no_complete_seeded_family";

const factStore = () => [{ id: "fact", kind: "fact", role: "user", content: "X" }];

async function solve(prompt, memoryEvents = []) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], { memoryEvents });
}

// FNV-1a 64 as rust/src/web_engine_core.rs::stable_id computes it.
function stableId(prefix, text) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(text)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return `${prefix}_${hash.toString(16).padStart(16, "0")}`;
}

test("a seeded rename program runs to fixpoint with the seeded wording", async () => {
  const response = await solve(RENAME_REQUEST, factStore());
  assert.equal(response.intent, "memory_program");
  assert.equal(response.content, RENAME_EN);
  assert.equal(response.memoryOperation.updates[0].fields.content, "Y");
});

test("the program id is the native stable id of the canonical program", async () => {
  await ready;
  const compiled = plain(evaluate(worker, `compileMemoryProgramForWorker(${JSON.stringify(RENAME_REQUEST)})`));
  assert.equal(compiled.status, "compiled");
  assert.equal(compiled.program.canonical.split("\n")[0], "family=contributed_fact_rename");
  assert.equal(stableId("memory_program", compiled.program.canonical), "memory_program_41ef0c9602340676");
});

test("the russian request compiles to the same program and answers in russian", async () => {
  const response = await solve("Перечисли все факты, которые я добавил о X, и переименуй X в Y во всех них.");
  assert.equal(response.intent, "memory_program");
  assert.equal(response.content, RENAME_EMPTY_RU);
  assert.equal(response.memoryOperation, undefined);
});

test("a destructive program is refused without confirmation", async () => {
  const response = await solve("Delete every fact I contributed about X.", factStore());
  assert.equal(response.intent, "memory_program_refused");
  assert.equal(response.content, DESTRUCTIVE_EN);
  assert.equal(response.memoryOperation, undefined);
});

test("a memory request no seeded family covers names its gap", async () => {
  const response = await solve("Transpose every fact matrix in memory.");
  assert.equal(response.intent, "memory_program_gap");
  assert.equal(response.content, GAP_EN);
});

// Issue #918 (R914-6): the compound_interest twins read one policy block and
// one set of seeded responses. Two deliberate parity fixes: the native row
// took its rate from the calculator while the worker took a table of its own
// (both now read the seeded default rates), and the worker appended a
// USD -> USD "conversion" whenever a principal was spelled in dollars (a
// conversion now skips its source currency, and the native row gains the
// ruble target the worker already had).
const COMPOUND_REPORT = "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 1000 USD\nr = 0.08 (8% annual)\nn = 12 (monthly)\nt = 5 years\n\nStep 1: periodic rate = r/n = 0.08/12 = 0.006666666666667\nStep 2: number of periods = n*t = 12*5 = 60\nStep 3: A = 1000 * (1 + 0.006666666666667)^60\nFinal amount: 1489.85 USD";
const COMPOUND_EUR = "\n\nConversion: USD -> EUR\n1 USD in EUR = 0.92 EUR\n1489.85 USD * 0.92 = 1370.66 EUR\nRate detail: Exchange rate: 1 USD = 0.92 EUR (source: default (hardcoded))\nLive web freshness is not independently verified here; this uses the exchange-rate source available through the local calculator.";
const COMPOUND_RUB = "\n\nConversion: USD -> RUB\n1 USD in RUB = 89.5 RUB\n1489.85 USD * 89.5 = 133341.57 RUB\nRate detail: Exchange rate: 1 USD = 89.5 RUB (source: default (hardcoded))";
const COMPOUND_PROMPT = "Invest $1000 at 8% annual interest compounded monthly for 5 years";

async function solveWith(prompt, history) {
  await ready;
  return worker.solve(prompt, history, {}, {}, [], {});
}

test("a compound-interest report converts at the seeded euro and ruble rates", async () => {
  for (const [prompt, expected] of [
    [`${COMPOUND_PROMPT} and convert the final amount to EUR using current exchange rates from the web.`, COMPOUND_REPORT + COMPOUND_EUR],
    [`${COMPOUND_PROMPT} and convert the final amount to rubles.`, COMPOUND_REPORT + COMPOUND_RUB],
  ]) {
    const response = await solveWith(prompt, []);
    assert.equal(response.intent, "calculation", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("a principal spelled in dollars is never converted to dollars", async () => {
  const response = await solveWith("invest 1000 dollars at 8,5% interest compounded daily for 2 years", []);
  assert.equal(response.content, "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 1000 USD\nr = 0.085 (8.5% annual)\nn = 365 (daily)\nt = 2 years\n\nStep 1: periodic rate = r/n = 0.085/365 = 0.000232876712329\nStep 2: number of periods = n*t = 365*2 = 730\nStep 3: A = 1000 * (1 + 0.000232876712329)^730\nFinal amount: 1185.28 USD");
});

test("a follow-up converts the earlier final amount", async () => {
  const history = [
    { role: "user", content: COMPOUND_PROMPT },
    { role: "assistant", content: COMPOUND_REPORT },
  ];
  const response = await solveWith("Convert the final amount to rubles", history);
  assert.equal(response.intent, "calculation");
  assert.equal(response.content, `Final amount conversion\nSource amount: 1489.85 USD${COMPOUND_RUB}`);
});

// Issue #918 (R914-6): the playwright_script prelude row renders the seeded
// playwright_* responses and the docs URL of `policy playwright_script` in
// both runtimes. The expected answers are read from the native pins in
// rust/tests/unit/playwright_script.rs, so the two suites hold one text.
function rustRawConstant(name) {
  const source = readFileSync(new URL("../unit/playwright_script.rs", import.meta.url), "utf8");
  const match = new RegExp(`const ${name}: &str = r"([\\s\\S]*?)";`).exec(source);
  assert.ok(match, `${name} in rust/tests/unit/playwright_script.rs`);
  return match[1];
}

test("playwright starters answer with the native pins in every language", async () => {
  const english = rustRawConstant("PLAYWRIGHT_EN_ANSWER");
  const russian = rustRawConstant("PLAYWRIGHT_RU_TYPO_ANSWER");
  for (const [prompt, expected] of [
    ["Can you write a Playwright script?", english],
    ["Можешь написать мне Playright скрипт?", russian],
    ["क्या तुम Playwright script लिख सकते हो?", english],
    ["可以写一个 Playwright script 吗？", english],
  ]) {
    const response = await solveWith(prompt, []);
    assert.equal(response.intent, "playwright_script", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("a low guess probability asks for the script's scope", async () => {
  await ready;
  const response = await worker.solve("Можешь написать мне Playright скрипт?", [], { guessProbability: 0.1 }, {}, [], {});
  assert.equal(response.intent, "playwright_script_clarification");
  assert.equal(response.content, "Я могу написать Playwright-скрипт. Уточните URL страницы, действия и ожидаемую проверку. Если нужен пример по умолчанию, я могу взять стартовый сценарий из документации Playwright.");
});

// Issue #918 (R914-6): the fetch, navigation and frame-policy wording of the
// http_fetch and url_navigate rows is the seeded web_* responses, which the
// native url_navigate row renders too (its exact answer is pinned in
// rust/tests/unit/formal_ai/seed_and_memory.rs). The browser answers depend
// on what the page and the frame-policy service return, so each outcome is
// pinned against a stubbed fetch; every answer is unchanged.
const WEB_CORS_NOTE = "Browser JavaScript also cannot read the page content directly unless the site allows CORS, so the direct external link is the reliable option.";
const webHeaders = (values) => ({ get: (name) => values[name.toLowerCase()] || null });

async function withFetch(fetchImpl, code) {
  await ready;
  worker.fetch = fetchImpl;
  return plain(await evaluate(worker, code));
}

test("a navigation the frame policy blocks keeps the external link", async () => {
  const response = await withFetch(
    async () => ({ ok: true, status: 200, json: async () => ({ headers: { "x-frame-options": "DENY", "content-security-policy": "frame-ancestors 'none'" } }), headers: webHeaders({}) }),
    'tryUrlNavigate("Navigate to github.com")',
  );
  assert.equal(response.intent, "url_navigate");
  assert.equal(response.content, `I suggest opening this in a new tab: [https://github.com](https://github.com).\n\nI checked the page's frame policy, and it does not allow embedding here because the page sends X-Frame-Options: DENY and CSP frame-ancestors 'none'.\n${WEB_CORS_NOTE}`);
});

test("a navigation the frame policy allows shows the page", async () => {
  const response = await withFetch(
    async () => ({ ok: true, status: 200, json: async () => ({ headers: {} }), headers: webHeaders({}) }),
    'tryUrlNavigate("Navigate to example.com")',
  );
  assert.equal(response.content, "I checked the page's frame policy and can show it here.\n\nDirect link: [https://example.com](https://example.com).");
  assert.equal(response.iframeUrl, "https://example.com");
});

test("an unreachable frame-policy service names its reason", async () => {
  const response = await withFetch(async () => ({ ok: false, status: 503, headers: webHeaders({}) }), 'tryUrlNavigate("Go to example.org")');
  assert.equal(response.content, `I suggest opening this in a new tab: [https://example.org](https://example.org).\n\nI could not verify that this page allows embedding here because the frame-policy service returned HTTP 503.\n${WEB_CORS_NOTE}`);
});

test("a fetched text body is shown and a binary body is not", async () => {
  const text = await withFetch(
    async () => ({ ok: true, status: 200, headers: webHeaders({ "content-type": "text/plain" }), text: async () => "hello {url} world", json: async () => ({ headers: {} }) }),
    'tryFetch("fetch example.com")',
  );
  assert.equal(text.intent, "http_fetch");
  assert.equal(text.content, "Fetched `https://example.com` — status **200**.\n\nResponse body:\n```\nhello {url} world\n```");
  const binary = await withFetch(
    async () => ({ ok: true, status: 204, headers: webHeaders({ "content-type": "image/png" }), text: async () => "", json: async () => ({ headers: {} }) }),
    'tryFetch("fetch example.com")',
  );
  assert.equal(binary.content, "Fetched `https://example.com` — status **204**.\n\nContent-Type: `image/png` — binary or empty body, not shown.\n\nYou can view this URL directly: [https://example.com](https://example.com)");
});

test("a failed fetch falls back to the frame policy", async () => {
  const response = await withFetch(
    async (url) => {
      if (!String(url).includes("api") && String(url).includes("example.com")) throw new Error("boom");
      return { ok: true, status: 200, json: async () => ({ headers: {} }), headers: webHeaders({}) };
    },
    'tryFetch("fetch example.com")',
  );
  assert.equal(response.content, "Could not fetch `https://example.com` directly (network error).\n\nI checked the page's frame policy and can show it in the embedded frame below.");
});

// Issue #918 (R914-6): the browser web search renders its result header,
// source labels, no-result and all-disabled sentences and the Wikinews
// fallback description from seeded web_search_* responses instead of
// per-language tables in js/worker/formal_ai_worker_19.js and
// js/worker/formal_ai_worker_18.js; every text is unchanged.
test("web search texts come from the seed in every language", async () => {
  await ready;
  const texts = plain(evaluate(worker, `["en", "ru", "es"].map((language) => { const t = webSearchTexts(language); return [t.header("rust", 10, 60), t.otherSources, t.via, t.readMore, t.noResults("rust", "duckduckgo, wikipedia"), t.allDisabled("duckduckgo")]; })`));
  assert.deepEqual(texts[0], [
    "Search results for `rust` — top 10 after reciprocal rank fusion (k = 60).",
    "Other sources", "via", "Read more",
    "No CORS-enabled web search results were returned for `rust`.\n\nProviders tried: duckduckgo, wikipedia.",
    "All CORS-readable search providers are disabled for this session. Tried: duckduckgo.",
  ]);
  assert.deepEqual(texts[1], [
    "Результаты поиска для `rust` — топ 10 после реципрокного объединения рангов (k = 60).",
    "Другие источники", "через", "Подробнее",
    "Не получены результаты веб-поиска с поддержкой CORS для `rust`.\n\nПопробованы провайдеры: duckduckgo, wikipedia.",
    "Все CORS-совместимые поисковые провайдеры отключены в этой сессии. Пробовали: duckduckgo.",
  ]);
  assert.deepEqual(texts[2], texts[0]);
  assert.equal(plain(evaluate(worker, 'wikinewsFallbackDescription("Mars rover lands", "ru")')), "В Wikinews есть новостная статья «Mars rover lands».");
  assert.equal(plain(evaluate(worker, 'wikinewsFallbackDescription("Mars rover lands", "en")')), 'Wikinews has a news article titled "Mars rover lands".');
});

// Issue #918 (R914-6): the procedural_how_to and how_it_works rows render
// every line from seeded procedural_how_to_* and how_it_works* responses and
// the search-query frames of the `policy procedural_how_to` and
// `policy how_it_works` blocks, filled in one pass in both runtimes. The
// browser plan runs its stages live, so it is pinned with the harness's
// offline fetch; every answer is unchanged.
async function procedure(prompt, language, preferences) {
  await ready;
  worker.fetch = defaultFetch;
  return plain(await evaluate(worker, `tryProceduralHowTo(${JSON.stringify(prompt)}, ${JSON.stringify(language)}, ${JSON.stringify(preferences)})`));
}

test("a procedural plan renders its seeded lines in English and Russian", async () => {
  assert.equal((await procedure("how to make tea", "en", { externalServiceWikihow: false })).content, "Procedural discovery plan for `make tea` (action `make`, object `tea`).\n\nSource path: Wikipedia -> Wikidata -> web search fallback -> recursive fetch check (wikiHow disabled in settings).\n\nwikiHow is disabled in settings.\n\nFallback web search for `how to make tea`:\n\nNo CORS-enabled web search results were returned for `how to make tea`.\n\nProviders tried: DuckDuckGo Instant Answer, Internet Archive (archive.org), Wikipedia REST, Wikidata entities, Wiktionary opensearch, Wikinews opensearch.");
  assert.equal((await procedure("how to install docker", "en", {})).content, "Procedural discovery plan for `install docker` (action `install`, object `docker`).\n\nFor install tasks, the first source gate prefers the product's official documentation or official repository install page before community how-to sources. It starts with `docker install official documentation` and keeps `how to install docker` as fallback.\n\nSource path: Wikipedia -> Wikidata -> official documentation web search -> wikiHow API fallback -> community web search fallback -> recursive fetch check.\n\nOfficial-documentation web search for `docker install official documentation` did not return ranked guidance; preserving `how to install docker` as the general how-to fallback.\n\nwikiHow candidate `Install-Docker` did not return explicit steps (http_404).\n\nFallback web search for `how to install docker`:\n\nNo CORS-enabled web search results were returned for `how to install docker`.\n\nProviders tried: DuckDuckGo Instant Answer, Internet Archive (archive.org), Wikipedia REST, Wikidata entities, Wiktionary opensearch, Wikinews opensearch.");
  const russian = await procedure("как установить docker", "ru", { externalServiceWikihow: false });
  assert.equal(russian.content, "План поиска процедуры для `установить docker` (действие `install`, объект `docker`).\n\nДля задач установки первый source gate ищет официальную документацию продукта или официальную страницу установки в репозитории, а уже потом переходит к общим how-to источникам. Он начинает с `docker install official documentation` и держит `how to установить docker` как fallback.\n\nПуть источников: Wikipedia -> Wikidata -> official documentation web search -> community web search fallback -> recursive fetch check (wikiHow отключен в настройках).\n\nOfficial-documentation web search for `docker install official documentation` did not return ranked guidance; preserving `how to установить docker` as the general how-to fallback.\n\nwikiHow is disabled in settings.\n\nFallback web search for `how to установить docker`:\n\nНе получены результаты веб-поиска с поддержкой CORS для `how to установить docker`.\n\nПопробованы провайдеры: DuckDuckGo Instant Answer, Internet Archive (archive.org), Wikipedia REST, Wikidata entities, Wiktionary opensearch, Wikinews opensearch.");
  assert.equal(russian.query, "how to установить docker");
});

test("a task text with braces is never re-filled", async () => {
  const response = await procedure("how to {task} it {k}", "en", { externalServiceWikihow: false });
  assert.ok(response.content.startsWith("Procedural discovery plan for `task it k` (action `task`, object `it k`)."));
});

test("the mechanism discovery plan answers as the native row does", async () => {
  const response = await solveWith("how does AUR work?", []);
  assert.equal(response.intent, "how_it_works");
  // Deliberate parity fix: the English record now ends as the native
  // answer pinned in rust/tests/unit/specification/reasoning_paths.rs does.
  assert.equal(response.content, "Mechanism discovery plan for `AUR`.\n\nI do not answer this from a memoized fact. The solver treats the prompt as a question about how `AUR` works, checks Wikipedia for a source-backed overview, Wikidata for entity relationships, then web search across duckduckgo, internet-archive, wikipedia, wikidata, wiktionary, wikinews. If no source explains the mechanism, it should ask for a source or a narrower term instead of inventing details.");
});
