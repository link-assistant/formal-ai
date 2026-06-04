// Issue #386 — parity guard for the web-navigation meaning vocabulary.
//
// Commit A adds data/seed/meanings-web-navigation.lino (the web_resource,
// http_fetch and url_navigate meanings, each surface marking its URL slot with
// the … (U+2026) marker) and regenerates the worker's inline MEANINGS_LINO.
// Commit B rewrites isHttpFetchPrompt / isUrlNavigatePrompt to ask the lexicon
// for those slot-marked forms by meaning instead of carrying inline per-language
// HTTP_FETCH_PREFIXES / HTTP_FETCH_MARKERS / URL_NAVIGATE_PREFIXES /
// URL_NAVIGATE_MARKERS arrays (and the isFetchPrompt / includesAny helpers).
//
// This harness proves four things against the live worker:
//   (1) roleWordForms() for both web roles reproduces the SAME surface set as the
//       canonical seed file, with the expected per-slot bucket counts — so both
//       engines (the Rust loader and this JS mirror read the SAME .lino) consume
//       an identical vocabulary;
//   (2) the data-driven isHttpFetchPrompt / isUrlNavigatePrompt — wired through
//       the real URL gate + fetch-before-navigate dispatch precedence — return
//       byte-identical routing to the PRE-conversion hardcoded logic
//       (reconstructed inline here from the worker's still-exported helpers)
//       across an English + Russian prompt battery — the behaviour-preservation
//       proof for the two languages the old arrays covered;
//   (3) the Hindi + Chinese surface forms the conversion ADDED route their URL
//       prompts to the right intent (and the pre-conversion logic returned
//       nothing for them) — the additive-coverage proof, with fetch vs navigate
//       verbs staying disjoint;
//   (4) the concrete issue-#386 / tests/unit/formal_ai.rs reasoning-path
//       expectations still hold (the canonical Russian/English cases).
// Run: `node experiments/issue-386-js-web-navigation.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const root = new URL("..", import.meta.url);
const src = fs.readFileSync(new URL("src/web/formal_ai_worker.js", root), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => {
  throw new Error("no importScripts in node");
};
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => {
  throw new Error("no fetch");
};
sandbox.location = { search: "", origin: "http://localhost" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}
function eq(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

// --- (1a) canonical surface set parsed straight from the seed file -----------
const navLino = fs.readFileSync(
  new URL("data/seed/meanings-web-navigation.lino", root),
  "utf8",
);
const seedWords = new Map(); // role -> [words] in declaration order
{
  let role = "";
  let words = [];
  const flush = () => {
    if (role) seedWords.set(role, words);
    words = [];
    role = "";
  };
  for (const raw of navLino.split("\n")) {
    const line = raw.trimEnd();
    if (/^  meaning "(.+)"$/.test(line)) {
      flush();
      continue;
    }
    const r = line.match(/^    role "(.+)"$/);
    if (r) {
      role = r[1];
      continue;
    }
    const w = line.match(/^      word "(.+)"$/);
    if (w) words.push(w[1]);
  }
  flush();
}

const HTTP_FETCH = "http_fetch";
const URL_NAVIGATE = "url_navigate";

for (const role of [HTTP_FETCH, URL_NAVIGATE]) {
  const want = seedWords.get(role) || [];
  const got = sandbox.roleWordForms(role).map((f) => f.text);
  check(
    `roleWordForms("${role}") reproduces the seed surface set (declaration order)`,
    eq(want, got),
    `seed=${want.length} worker=${got.length}`,
  );
}

// --- (1b) per-slot bucket counts -------------------------------------------
// http_fetch: 6 en + 10 ru = 16 prefix forms; 9 en + 12 ru + 2 hi + 2 zh = 25
// bare markers. url_navigate: 24 en + 21 ru = 45 prefix forms; 10 en + 10 ru +
// 3 hi + 4 zh = 27 bare markers. The web roles never use suffix/circumfix slots
// (a URL always trails its verb), unlike the how-cluster.
const fetchForms = sandbox.roleWordForms(HTTP_FETCH);
const navForms = sandbox.roleWordForms(URL_NAVIGATE);
const bucket = (forms, slot) => forms.filter((f) => f.slot === slot);
check(
  "http_fetch bucket counts (prefix/bare/suffix/circumfix)",
  bucket(fetchForms, "prefix").length === 16 &&
    bucket(fetchForms, "bare").length === 25 &&
    bucket(fetchForms, "suffix").length === 0 &&
    bucket(fetchForms, "circumfix").length === 0,
  `prefix=${bucket(fetchForms, "prefix").length} bare=${bucket(fetchForms, "bare").length} suffix=${bucket(fetchForms, "suffix").length} circumfix=${bucket(fetchForms, "circumfix").length}`,
);
check(
  "url_navigate bucket counts (prefix/bare/suffix/circumfix)",
  bucket(navForms, "prefix").length === 45 &&
    bucket(navForms, "bare").length === 27 &&
    bucket(navForms, "suffix").length === 0 &&
    bucket(navForms, "circumfix").length === 0,
  `prefix=${bucket(navForms, "prefix").length} bare=${bucket(navForms, "bare").length} suffix=${bucket(navForms, "suffix").length} circumfix=${bucket(navForms, "circumfix").length}`,
);
// No web word form carries an `action` override (the URL is the object; the
// intent is fixed by the meaning) — guard that none crept in.
check(
  "no web word form declares an action override",
  fetchForms.every((f) => !f.action) && navForms.every((f) => !f.action),
);

// --- (2) reconstruct the PRE-conversion hardcoded logic ----------------------
// Verbatim copies of the arrays the worker carried before Commit B (extracted
// from git HEAD:src/web/formal_ai_worker.js), driving the worker's still-present
// URL helpers (firstUrlCandidate, normalizePrompt). If the data-driven
// predicates agree with these on every English/Russian probe, behaviour is
// preserved for the two languages the arrays covered.
const OLD_HTTP_FETCH_PREFIXES = [
  "fetch ",
  "fetch url ",
  "fetch the url ",
  "http fetch ",
  "request ",
  "make request to ",
  "send request to ",
  "сделай запрос ",
  "сделай http запрос ",
  "выполни запрос ",
  "выполни http запрос ",
  "запроси ",
  "получи ",
  "http запрос к ",
  "http запрос на ",
];
const OLD_HTTP_FETCH_MARKERS = [
  "make a request to",
  "make an http request to",
  "send a request to",
  "send an http request to",
  "http request to",
  "http get to",
  "fetch the url",
  "fetch this url",
  "fetch the page",
  "сделай запрос к",
  "сделай запрос на",
  "сделай http запрос к",
  "сделай http запрос на",
  "выполни запрос к",
  "выполни запрос на",
  "выполни http запрос к",
  "выполни http запрос на",
  "запрос к",
  "запрос на",
  "http запрос к",
  "http запрос на",
];
const OLD_URL_NAVIGATE_PREFIXES = [
  "navigate to ",
  "navigate ",
  "go to ",
  "goto ",
  "visit ",
  "browse to ",
  "browse ",
  "show ",
  "show me ",
  "display ",
  "load ",
  "open ",
  "open url ",
  "open the url ",
  "open site ",
  "open website ",
  "open page ",
  "open the page ",
  "open the website ",
  "take me to ",
  "preview ",
  "view ",
  "see ",
  "get ",
  "перейди ",
  "перейди на ",
  "переходи на ",
  "переходи ",
  "перейдите на ",
  "открой ",
  "открой сайт ",
  "открой страницу ",
  "открой ссылку ",
  "открой урл ",
  "покажи ",
  "покажи сайт ",
  "покажи страницу ",
  "покажи мне ",
  "загрузи ",
  "загрузи страницу ",
  "посети ",
  "зайди на ",
  "зайди ",
  "просмотри ",
  "отобрази ",
];
const OLD_URL_NAVIGATE_MARKERS = [
  "navigate to",
  "go to",
  "goto",
  "browse to",
  "take me to",
  "open the page",
  "open the site",
  "open the website",
  "open the url",
  "open url",
  "перейди на",
  "переходи на",
  "перейдите на",
  "открой сайт",
  "открой страницу",
  "открой ссылку",
  "открой урл",
  "покажи сайт",
  "покажи страницу",
  "зайди на",
];
function oldIsFetchPrompt(normalized) {
  return normalized.startsWith("fetch ") && normalized.length > 6;
}
function startsWithAny(haystack, prefixes) {
  return prefixes.some((prefix) => haystack.startsWith(prefix));
}
function includesAny(haystack, markers) {
  return markers.some((marker) => haystack.includes(marker));
}
function oldIsHttpFetchPrompt(prompt, normalized) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  if (oldIsFetchPrompt(normalized)) return true;
  if (
    startsWithAny(normalized, OLD_HTTP_FETCH_PREFIXES) ||
    startsWithAny(raw, OLD_HTTP_FETCH_PREFIXES)
  ) {
    return true;
  }
  return (
    includesAny(normalized, OLD_HTTP_FETCH_MARKERS) ||
    includesAny(raw, OLD_HTTP_FETCH_MARKERS)
  );
}
function oldIsUrlNavigatePrompt(prompt, normalized, rawCandidate) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  if (raw.startsWith(String(rawCandidate || "").toLowerCase())) return true;
  if (
    startsWithAny(normalized, OLD_URL_NAVIGATE_PREFIXES) ||
    startsWithAny(raw, OLD_URL_NAVIGATE_PREFIXES)
  ) {
    return true;
  }
  return (
    includesAny(normalized, OLD_URL_NAVIGATE_MARKERS) ||
    includesAny(raw, OLD_URL_NAVIGATE_MARKERS)
  );
}

// End-to-end routing with the production URL gate + fetch-before-navigate
// dispatch precedence (extractHttpFetchUrl is tried before extractUrlNavigateUrl
// in both engines). Returns "fetch:<url>" / "navigate:<url>" / "none".
function routeWith(isFetch, isNavigate, prompt) {
  const normalized = sandbox.normalizePrompt(prompt);
  const cand = sandbox.firstUrlCandidate(prompt);
  if (!cand) return "none";
  if (isFetch(prompt, normalized)) return `fetch:${cand.url}`;
  if (isNavigate(prompt, normalized, cand.raw)) return `navigate:${cand.url}`;
  return "none";
}
const routeNew = (p) =>
  routeWith(sandbox.isHttpFetchPrompt, sandbox.isUrlNavigatePrompt, p);
const routeOld = (p) => routeWith(oldIsHttpFetchPrompt, oldIsUrlNavigatePrompt, p);

// --- (2) the English + Russian behaviour-preservation battery ---------------
const EN_RU_PROBES = [
  // English http_fetch — prefixes
  "fetch https://example.com",
  "fetch url example.com",
  "http fetch example.com",
  "request example.com",
  "make request to example.com",
  "send request to example.com",
  // English http_fetch — bare markers
  "make a request to example.com",
  "make an http request to example.com",
  "send a request to example.com",
  "send an http request to example.com",
  "please send an http request to example.com now",
  "http request to example.com",
  "http get to example.com",
  "fetch the url example.com",
  "fetch this url example.com",
  "fetch the page example.com",
  // Russian http_fetch — prefixes + markers + new infinitives
  "сделай запрос к google.com",
  "сделай http запрос на example.com",
  "выполни запрос к example.com",
  "выполни http запрос к example.com",
  "запроси google.com",
  "получи example.com",
  "http запрос к example.com",
  "http запрос на example.com",
  "сделать запрос к example.com",
  "выполнить запрос к example.com",
  // English url_navigate — prefixes
  "navigate to example.com",
  "navigate example.com",
  "go to example.com",
  "goto example.com",
  "visit example.com",
  "browse to example.com",
  "browse example.com",
  "show example.com",
  "show me example.com",
  "display example.com",
  "load example.com",
  "open example.com",
  "open url example.com",
  "open the url example.com",
  "open site example.com",
  "open website example.com",
  "open page example.com",
  "open the page example.com",
  "open the website example.com",
  "take me to example.com",
  "preview example.com",
  "view example.com",
  "see example.com",
  "get example.com",
  // Russian url_navigate — prefixes
  "перейди на github.com",
  "перейди github.com",
  "переходи на github.com",
  "переходи github.com",
  "перейдите на github.com",
  "открой github.com",
  "открой сайт github.com",
  "открой страницу github.com",
  "открой ссылку github.com",
  "открой урл github.com",
  "покажи github.com",
  "покажи сайт github.com",
  "покажи страницу github.com",
  "покажи мне github.com",
  "загрузи github.com",
  "загрузи страницу github.com",
  "посети github.com",
  "зайди на github.com",
  "зайди github.com",
  "просмотри github.com",
  "отобрази github.com",
  // bare URL early-return (navigation)
  "https://example.com",
  "example.com",
  "www.example.com/path",
  // URL-bearing negatives that must stay "none" on BOTH engines
  "the site example.com is great",
  "i was reading about example.com yesterday",
  "tell me a story about example.com",
  "fetching example.com slowly",
  // no-URL negatives
  "what is the capital of france",
  "just some chatter with no link at all",
  "open the door please",
  "",
  "   ",
];
let enRuMismatch = 0;
for (const p of EN_RU_PROBES) {
  const want = routeOld(p);
  const got = routeNew(p);
  if (want !== got) {
    enRuMismatch += 1;
    check(`web routing parity «${p}»`, false, `old=${want} new=${got}`);
  }
}
check(
  `isHttpFetchPrompt/isUrlNavigatePrompt match pre-conversion logic on ${EN_RU_PROBES.length} EN/RU probes`,
  enRuMismatch === 0,
);

// --- (3) Hindi + Chinese additive-coverage proof ----------------------------
// The conversion ADDED bare markers for Hindi and Chinese. With the verb FIRST
// and the URL trailing (so the bare-URL early-return does not fire), the
// pre-conversion logic recognised nothing — the new lexicon routes each to the
// right intent. fetch verbs (获取 / 发送请求 / अनुरोध भेजें / अनुरोध करें) and navigate
// verbs (打开 / 访问 / 前往 / 查看 / पर जाएं / खोलें / देखें) stay disjoint.
const ADDED_CASES = [
  ["获取 https://example.com", "fetch:https://example.com"],
  ["发送请求 https://example.com", "fetch:https://example.com"],
  ["अनुरोध भेजें https://example.com", "fetch:https://example.com"],
  ["अनुरोध करें https://example.com", "fetch:https://example.com"],
  ["打开 https://example.com", "navigate:https://example.com"],
  ["访问 https://example.com", "navigate:https://example.com"],
  ["前往 https://example.com", "navigate:https://example.com"],
  ["查看 https://example.com", "navigate:https://example.com"],
  ["पर जाएं https://example.com", "navigate:https://example.com"],
  ["खोलें https://example.com", "navigate:https://example.com"],
  ["देखें https://example.com", "navigate:https://example.com"],
];
for (const [prompt, expected] of ADDED_CASES) {
  const got = routeNew(prompt);
  const old = routeOld(prompt);
  check(
    `added hi/zh routing «${prompt}» -> ${expected}`,
    got === expected,
    `new=${got}`,
  );
  check(
    `pre-conversion logic returned nothing for «${prompt}» (genuinely additive)`,
    old === "none",
    `old=${old}`,
  );
}

// --- (4) concrete issue-#386 / tests/unit/formal_ai.rs expectations ----------
// Mirror the canonical fetch/navigate cases the Rust unit suite pins, proving
// the worker routes them identically to the Rust solver (both now read the same
// data/seed/meanings-web-navigation.lino surface set).
const FETCH_CASES = [
  ["Сделай запрос к google.com", "https://google.com"],
  ["сделай запрос к https://example.com/path", "https://example.com/path"],
  ["Выполни запрос к google.com", "https://google.com"],
  ["запроси google.com", "https://google.com"],
];
for (const [prompt, url] of FETCH_CASES) {
  check(`fetch «${prompt}» -> ${url}`, routeNew(prompt) === `fetch:${url}`, routeNew(prompt));
}
const NAVIGATE_CASES = [
  ["Перейди на github.com", "https://github.com"],
  ["Перейдите на github.com", "https://github.com"],
  ["Переходи на github.com", "https://github.com"],
  ["Открой github.com", "https://github.com"],
  ["Открой сайт github.com", "https://github.com"],
  ["Открой страницу github.com", "https://github.com"],
  ["Открой ссылку github.com", "https://github.com"],
  ["Покажи github.com", "https://github.com"],
  ["Покажи сайт github.com", "https://github.com"],
  ["Загрузи github.com", "https://github.com"],
  ["Посети github.com", "https://github.com"],
  ["Зайди на github.com", "https://github.com"],
];
for (const [prompt, url] of NAVIGATE_CASES) {
  check(
    `navigate «${prompt}» -> ${url}`,
    routeNew(prompt) === `navigate:${url}`,
    routeNew(prompt),
  );
}

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
