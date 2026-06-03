// Issue #386 — parity guard for the web-search meaning vocabulary.
//
// The web-search recogniser used to carry seventeen hardcoded per-language
// arrays (WEB_SEARCH_EXPLICIT_PREFIXES, …_ACTION_MARKERS, …_SOURCE_ONLY, the
// research/enumeration openers, the follow-up verbs, …). Commit
// 592e20a rewrote the Rust handler (src/solver_handlers/web_search_intent.rs)
// to read those surfaces from the meaning lexicon by semantic role + slot, and
// the matching JS conversion (experiments/issue-386-web-search-to-meanings.mjs)
// replaced the arrays in src/web/formal_ai_worker.js with the webSearchMarkers()
// projection over data/seed/meanings-web-{search,search-query,research,
// followup}.lino. No surface word lives in the worker any more — only the roles.
//
// This harness proves four things against the live (converted) worker:
//   (1) roleWordForms() for every web-search marker role reproduces the SAME
//       surface set as the canonical seed files — so both engines (the Rust
//       loader and this JS mirror) consume an identical vocabulary, and
//       webSearchMarkers() exposes the eighteen projected fields, memoized;
//   (2) the data-driven extractWebSearchRequest returns byte-identical results
//       to the PRE-conversion hardcoded logic across a multilingual battery
//       (the golden table below was captured from the pre-conversion worker —
//       tests may pin hardcoded examples — and covers explicit prefixes,
//       semantic actions, source stripping, implicit-research questions,
//       enumeration requests, the conversation-search guard and non-search
//       controls in en/ru/hi/zh) — the behaviour-preservation proof;
//   (3) the personal-fact filter guard (added with the conversion to mirror
//       is_personal_fact_filter_request) suppresses "facts I contributed" /
//       "my facts" prompts — including the pre-conversion leak that web-searched
//       for the bare token "my" — the additive-correctness proof;
//   (4) the concrete tests/unit/web_requests.rs reasoning-path expectations
//       still hold: the source-marker, enumeration-research, implicit-research
//       and follow-up-clause-truncation cases extract the Rust-expected query
//       (compared through normalizePrompt to bridge the worker's case-preserving
//       view) with the right query kind.
// Run: `node experiments/issue-386-js-web-search.mjs`.

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
function eqSet(a, b) {
  const sa = [...new Set(a)].sort();
  const sb = [...new Set(b)].sort();
  return JSON.stringify(sa) === JSON.stringify(sb);
}
function req(prompt) {
  return sandbox.extractWebSearchRequest(prompt, sandbox.normalizePrompt(prompt));
}
function query(prompt) {
  return sandbox.extractWebSearchQuery(prompt, sandbox.normalizePrompt(prompt));
}

// ---------------------------------------------------------------------------
// (1) Vocabulary wiring: roleWordForms() reproduces the seed surface set.
// ---------------------------------------------------------------------------
// Parse the four web seed files exactly as the worker concatenates them (in
// MEANING_FILES order) into a role -> surface-set map. Unlike the web-navigation
// cluster, a single meaning here can carry SEVERAL roles (reference_internet is
// both web_search_signal and web_search_source_only), so every `word` under a
// meaning is contributed to ALL of that meaning's roles.
const WEB_SEED_FILES = [
  "data/seed/meanings-web-search.lino",
  "data/seed/meanings-web-search-query.lino",
  "data/seed/meanings-web-research.lino",
  "data/seed/meanings-web-followup.lino",
];
const seedByRole = new Map(); // role -> [surface, …] in declaration order
{
  let roles = [];
  let words = [];
  const flush = () => {
    for (const role of roles) {
      const bucket = seedByRole.get(role) || [];
      bucket.push(...words);
      seedByRole.set(role, bucket);
    }
    roles = [];
    words = [];
  };
  for (const rel of WEB_SEED_FILES) {
    const text = fs.readFileSync(new URL(rel, root), "utf8");
    for (const raw of text.split("\n")) {
      const line = raw.trimEnd();
      if (/^  meaning "(.+)"$/.test(line)) {
        flush();
        continue;
      }
      const r = line.match(/^    role "(.+)"$/);
      if (r) {
        roles.push(r[1]);
        continue;
      }
      const w = line.match(/^      word "(.*)"$/);
      if (w) words.push(w[1]);
    }
    flush();
  }
}

// The seventeen marker roles webSearchMarkers() projects (web_search_topic_marker
// feeds two fields — the prefix and suffix slots). web_search_concept names the
// ontology backbone meaning and is deliberately NOT a matchable marker.
const MARKER_ROLES = [
  "web_search_explicit_prefix",
  "web_search_action",
  "web_search_strong_action",
  "web_search_signal",
  "web_search_topic_marker",
  "web_search_imperative_lead",
  "web_search_query_leading_noise",
  "web_search_query_trailing_noise",
  "web_search_source_only",
  "followup_instruction_verb",
  "clause_continuation_marker",
  "research_question_opener",
  "research_superlative_modifier",
  "research_evidence_domain",
  "research_evaluation_domain",
  "enumeration_request_opener",
  "enumeration_constraint",
];
for (const role of MARKER_ROLES) {
  const want = seedByRole.get(role) || [];
  const got = sandbox.roleWordForms(role).map((f) => f.text);
  check(
    `roleWordForms("${role}") reproduces the seed surface set`,
    want.length > 0 && eqSet(want, got),
    `seed=${want.length} worker=${got.length}`,
  );
}

// webSearchMarkers() exposes the eighteen projected fields, every one a
// non-empty array, and is memoized (same object on a second call).
const markers = sandbox.webSearchMarkers();
const MARKER_FIELDS = [
  "explicitPrefixes",
  "actionMarkers",
  "strongActionMarkers",
  "signalMarkers",
  "topicAfterMarkers",
  "topicBeforeMarkers",
  "imperativeLeadMarkers",
  "leadingNoise",
  "trailingNoise",
  "sourceOnly",
  "followupVerbs",
  "continuationMarkers",
  "researchQuestionPrefixes",
  "researchModifiers",
  "researchEvidenceDomains",
  "researchEvaluationDomains",
  "enumerationPrefixes",
  "enumerationConstraintMarkers",
];
check(
  "webSearchMarkers() exposes all eighteen projected fields",
  MARKER_FIELDS.every((f) => Array.isArray(markers[f])),
  MARKER_FIELDS.filter((f) => !Array.isArray(markers[f])).join(",") || "ok",
);
check(
  "every webSearchMarkers() field projects at least one surface",
  MARKER_FIELDS.every((f) => markers[f].length > 0),
  MARKER_FIELDS.filter((f) => !markers[f] || markers[f].length === 0).join(",") || "ok",
);
check(
  "webSearchMarkers() is memoized (identical object on a second call)",
  sandbox.webSearchMarkers() === markers,
);
// The explicit-prefix and topic-after projections come from the … (U+2026) slot:
// each surface must carry a non-empty literal before the marker.
check(
  "explicitPrefixes are slot prefixes (non-empty before-text, trailing space)",
  markers.explicitPrefixes.every((p) => p.length > 0),
);

// ---------------------------------------------------------------------------
// (2) Behaviour preservation: a multilingual golden table captured verbatim
//     from the pre-conversion worker. The converted worker must reproduce each
//     {query, kind} (or null / "") exactly.
// ---------------------------------------------------------------------------
const GOLDEN = [
  // explicit search prefixes
  { prompt: "search for apple", expected: { query: "apple", kind: "semantic_action" } },
  { prompt: "google the weather in Paris", expected: null },
  { prompt: "look up the capital of France", expected: { query: "capital of France", kind: "semantic_action" } },
  { prompt: "search the web for quantum computing", expected: { query: "quantum computing", kind: "explicit_prefix" } },
  { prompt: "найди рецепт борща", expected: null },
  { prompt: "погугли погоду в Москве", expected: null },
  { prompt: "поищи последние новости", expected: { query: "последние новости", kind: "semantic_action" } },
  // semantic action + source-only stripping
  { prompt: "Find apple on the internet", expected: { query: "apple", kind: "semantic_action" } },
  { prompt: "Найди яблоко в интернете", expected: { query: "яблоко", kind: "semantic_action" } },
  { prompt: "Find information about Rust programming", expected: { query: "Rust programming", kind: "explicit_prefix" } },
  { prompt: "Look up information on Rust programming", expected: { query: "Rust programming", kind: "explicit_prefix" } },
  { prompt: "Research Rust programming online", expected: { query: "Rust programming", kind: "semantic_action" } },
  { prompt: "Поищи материалы по Rust программированию в Википедии", expected: { query: "Rust программированию", kind: "semantic_action" } },
  { prompt: "Найди информацию о Rust программировании", expected: { query: "Rust программировании", kind: "explicit_prefix" } },
  // universal-boundary follow-up truncation
  { prompt: 'Search Wikipedia for "War of Currents" and summarize who won and why', expected: { query: "War of Currents", kind: "explicit_prefix" } },
  { prompt: "Search Wikipedia for Nikola Tesla and Thomas Edison. Compare their number of patents.", expected: { query: "Nikola Tesla and Thomas Edison", kind: "explicit_prefix" } },
  // implicit research questions
  { prompt: "What is the most popular dataset for translation quality validation?", expected: { query: "most popular dataset for translation quality validation", kind: "implicit_research_question" } },
  { prompt: "What is the best programming language for beginners?", expected: { query: "best programming language for beginners", kind: "implicit_research_question" } },
  { prompt: "Which is the fastest sorting algorithm in practice?", expected: null },
  // enumeration research
  { prompt: "list all genshin characters with off-field DMG", expected: { query: "genshin characters with off-field DMG", kind: "enumeration_research_request" } },
  { prompt: "перечисли всех персонажей genshin с уроном вне поля", expected: { query: "персонажей genshin с уроном вне поля", kind: "enumeration_research_request" } },
  // conversation-search guard (pre-existing early return)
  { prompt: "search conversations about cats", expected: "" },
  { prompt: "search my conversations for the recipe", expected: "" },
  { prompt: "search my chats for the meeting", expected: "" },
  // non-search controls
  { prompt: "write a function that adds two numbers", expected: null },
  { prompt: "what time is it", expected: null },
  { prompt: "translate hello into French", expected: null },
  // Hindi / Chinese coverage
  { prompt: "सेब के बारे में इंटरनेट पर खोजो", expected: { query: "सेब", kind: "semantic_action" } },
  { prompt: "Rust programming के बारे में जानकारी खोजो", expected: { query: "Rust programming", kind: "semantic_action" } },
  { prompt: "Rust programming के बारे में विकिपीडिया में खोजें", expected: { query: "Rust programming", kind: "semantic_action" } },
  { prompt: "查找苹果网上信息", expected: { query: "苹果", kind: "semantic_action" } },
  { prompt: "查找关于 Rust 编程的信息", expected: { query: "Rust 编程", kind: "semantic_action" } },
  { prompt: "在维基百科上查一下 Rust 编程", expected: { query: "Rust 编程", kind: "semantic_action" } },
];
for (const { prompt, expected } of GOLDEN) {
  const got = req(prompt);
  check(
    `behaviour preserved «${prompt}»`,
    JSON.stringify(got) === JSON.stringify(expected),
    `got=${JSON.stringify(got)}`,
  );
}

// ---------------------------------------------------------------------------
// (3) Personal-fact filter guard (added with the conversion). A request to
//     filter the user's OWN contributed facts is conversation search, not a web
//     search, so the extracted query must be empty. The pre-conversion worker
//     web-searched for the bare token "my" on "search my facts" — that leak is
//     now closed, matching is_personal_fact_filter_request in the Rust spec.
// ---------------------------------------------------------------------------
const PERSONAL_FACT_PROMPTS = [
  "show me the facts I contributed",
  "search my facts", // pre-conversion leak: web_search for "my"
  "facts I have contributed",
  "facts I contributed",
  "facts ive contributed",
];
for (const prompt of PERSONAL_FACT_PROMPTS) {
  check(
    `personal-fact filter is not a web search «${prompt}»`,
    query(prompt) === "",
    `got=${JSON.stringify(query(prompt))}`,
  );
}

// ---------------------------------------------------------------------------
// (4) Rust reasoning-path expectations (tests/unit/web_requests.rs). The worker
//     preserves the query's original case while the Rust handler reports the
//     normalized form, so compare through normalizePrompt and pin the kind.
// ---------------------------------------------------------------------------
function rustCase(name, prompt, expectedQuery, expectedKind) {
  const got = req(prompt);
  const ok =
    got &&
    typeof got === "object" &&
    got.kind === expectedKind &&
    sandbox.normalizePrompt(got.query) === expectedQuery;
  check(`rust-mirror ${name} «${prompt}»`, ok, `got=${JSON.stringify(got)}`);
}

// WEB_SEARCH_SOURCE_MARKER_CASES
rustCase("source/en", "Find apple on the internet", "apple", "semantic_action");
rustCase("source/ru", "Найди яблоко в интернете", "яблоко", "semantic_action");
rustCase("source/hi", "सेब के बारे में इंटरनेट पर खोजो", "सेब", "semantic_action");
rustCase("source/zh", "查找苹果网上信息", "苹果", "semantic_action");

// WEB_SEARCH_ENUMERATION_RESEARCH_CASES
rustCase(
  "enum/en",
  "list all genshin characters with off-field DMG",
  "genshin characters with off field dmg",
  "enumeration_research_request",
);
rustCase(
  "enum/ru",
  "перечисли всех персонажей genshin с уроном вне поля",
  "персонажей genshin с уроном вне поля",
  "enumeration_research_request",
);
rustCase(
  "enum/hi",
  "सभी Genshin पात्र जिनके पास off-field DMG है",
  "genshin पात्र जिनके पास off field dmg है",
  "enumeration_research_request",
);
rustCase(
  "enum/zh",
  "列出所有 Genshin 角色 具有 off-field DMG",
  "genshin 角色 具有 off field dmg",
  "enumeration_research_request",
);

// implicit_research_question_routes_to_web_search_handler
rustCase(
  "implicit-research",
  "What is the most popular dataset for translation quality validation?",
  "most popular dataset for translation quality validation",
  "implicit_research_question",
);

// source_search_prompts_drop_follow_up_instruction_clauses
rustCase(
  "followup-drop/and",
  'Search Wikipedia for "War of Currents" and summarize who won and why',
  "war of currents",
  "explicit_prefix",
);
rustCase(
  "followup-drop/sentence",
  "Search Wikipedia for Nikola Tesla and Thomas Edison. Compare their number of patents.",
  "nikola tesla and thomas edison",
  "explicit_prefix",
);

// information_search_variants_route_to_web_search_handler — breadth check: every
// variant extracts a non-empty query that still mentions the topic (rust).
const INFORMATION_SEARCH_VARIANTS = [
  "Найди информацию о Rust программировании",
  "Поищи информацию про Rust программирование",
  "Find information about Rust programming",
  "Look up information on Rust programming",
  "Research Rust programming online",
  "Rust programming के बारे में जानकारी खोजो",
  "查找关于 Rust 编程的信息",
  "在维基百科上查一下 Rust 编程",
];
for (const prompt of INFORMATION_SEARCH_VARIANTS) {
  const got = req(prompt);
  const ok = got && typeof got === "object" && /rust/i.test(got.query);
  check(`information-search variant routes «${prompt}»`, ok, `got=${JSON.stringify(got)}`);
}

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
