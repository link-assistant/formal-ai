// Issue #386 — parity guard for the behavior-rules-list meaning vocabulary.
//
// Commit adds data/seed/meanings-behavior-rules.lino (the rule_listing_subject /
// rule_listing_request / rule_listing_scope / rule_listing_phrase roles) and
// rewrites isBehaviorRulesList / isSupportedLanguageBehaviorRulesListQuery in the
// worker to ask the lexicon for those role words *by meaning* instead of carrying
// inline per-language `.contains()` word lists and four per-language functions.
//
// This harness proves three things against the live worker:
//   (1) the per-role surface set the worker derives from the lexicon reproduces
//       the ORIGINAL hardcoded vocabulary exactly (no word gained or lost);
//   (2) the data-driven isBehaviorRulesList returns byte-identical results to the
//       PRE-conversion hardcoded logic (reconstructed inline here from the Rust
//       HEAD source) across a generated multilingual prompt battery — the
//       behaviour-preservation proof, including per-language AND (an English verb
//       must not satisfy a Russian-scoped query) and the bare-phrase shortcuts;
//   (3) the concrete issue-#386 routing expectations still hold (the pinned
//       list prompts stay list requests; the detail prompt does not).
// Run: `node experiments/issue-386-js-behavior-rules.mjs`.

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

// --- ORIGINAL vocabulary, copied verbatim from the pre-conversion Rust HEAD ----
// (src/solver_handlers/behavior_rules.rs @ HEAD: the four per-language functions
// and the bare-phrase `.contains()` list in is_behavior_rules_list). The new
// seed file must reproduce exactly these surface forms, partitioned by role.
const ORIGINAL = {
  en: {
    subject: ["rules", "rule list", "rules list"],
    request: ["list", "show", "what", "which"],
    scope: ["behavior", "your", "own", "current", "existing"],
  },
  ru: {
    subject: ["правил", "правила"],
    request: ["список", "перечисли", "покажи", "какие"],
    scope: [
      "поведения",
      "своих",
      "свои",
      "твоих",
      "твои",
      "собственные",
      "список правил",
    ],
  },
  hi: {
    subject: ["नियम", "नियमों"],
    request: ["सूची", "सूचीबद्ध", "दिखाओ", "दिखाएं", "बताओ", "गिनाओ", "कौन"],
    scope: ["व्यवहार", "अपने", "तुम्हारे", "आपके", "नियमों की सूची"],
  },
  zh: {
    subject: ["规则", "規則"],
    request: ["列出", "显示", "顯示", "展示", "哪些", "什么"],
    scope: ["行为", "行為", "你的", "您的", "自己", "规则列表", "規則列表"],
  },
};
// The bare phrases that name the rule-set outright (role rule_listing_phrase).
const ORIGINAL_PHRASES = [
  "list behavior rules",
  "list all behavior rules",
  "show behavior rules",
  "show all behavior rules",
  "what behavior rules",
  "existing behavior rules",
  "список правил поведения",
  "покажи правила поведения",
  "какие правила поведения",
  "व्यवहार के नियम",
  "व्यवहार नियम सूचीबद्ध करें",
  "行为规则",
  "列出行为规则",
];

// --- (1) the worker derives exactly the ORIGINAL vocabulary from the lexicon ---
// Module-level `const` role names are lexically scoped inside the worker and do
// NOT leak onto the sandbox global (only function declarations do), so reference
// the role slugs by their literal value — the same strings the worker `const`s
// bind and that data/seed/meanings-behavior-rules.lino declares.
const ROLE = {
  subject: "rule_listing_subject",
  request: "rule_listing_request",
  scope: "rule_listing_scope",
  phrase: "rule_listing_phrase",
};
for (const [dim, role] of [
  ["subject", ROLE.subject],
  ["request", ROLE.request],
  ["scope", ROLE.scope],
]) {
  const wanted = [];
  for (const lang of ["en", "ru", "hi", "zh"]) wanted.push(...ORIGINAL[lang][dim]);
  const got = sandbox.calendarWordsForRole(role);
  check(`role ${role} union reproduces original ${dim} vocabulary`, eqSet(got, wanted),
    eqSet(got, wanted) ? "" : `got=${JSON.stringify([...new Set(got)].sort())}`);
  // per-language partition must also match (no cross-language leakage)
  for (const lang of ["en", "ru", "hi", "zh"]) {
    const perLang = sandbox.wordsForRoleInLanguages(role, [lang]);
    check(`role ${role} @ ${lang} reproduces original`, eqSet(perLang, ORIGINAL[lang][dim]),
      eqSet(perLang, ORIGINAL[lang][dim]) ? "" : `got=${JSON.stringify([...new Set(perLang)].sort())}`);
  }
}
check(`role ${ROLE.phrase} union reproduces original phrase set`,
  eqSet(sandbox.calendarWordsForRole(ROLE.phrase), ORIGINAL_PHRASES),
  eqSet(sandbox.calendarWordsForRole(ROLE.phrase), ORIGINAL_PHRASES)
    ? "" : `got=${JSON.stringify([...new Set(sandbox.calendarWordsForRole(ROLE.phrase))].sort())}`);

// --- (2) reconstruct the ORIGINAL recognizer and prove byte-identical routing --
// matchesBehaviorRulesListSeedPattern is UNCHANGED by the conversion, so reuse
// the worker's own (live) implementation for that clause; only the hardcoded
// vocabulary clauses are reconstructed here from HEAD.
function originalPerLanguage(normalized) {
  const has = (w) => normalized.includes(w);
  const en =
    (ORIGINAL.en.subject.some(has)) &&
    (ORIGINAL.en.request.some(has)) &&
    (ORIGINAL.en.scope.some(has));
  const ru =
    (ORIGINAL.ru.subject.some(has)) &&
    (ORIGINAL.ru.request.some(has)) &&
    (ORIGINAL.ru.scope.some(has));
  const hi =
    (ORIGINAL.hi.subject.some(has)) &&
    (ORIGINAL.hi.request.some(has)) &&
    (ORIGINAL.hi.scope.some(has));
  const zh =
    (ORIGINAL.zh.subject.some(has)) &&
    (ORIGINAL.zh.request.some(has)) &&
    (ORIGINAL.zh.scope.some(has));
  return en || ru || hi || zh;
}
function originalIsBehaviorRulesList(normalized) {
  return (
    sandbox.matchesBehaviorRulesListSeedPattern(normalized) ||
    ORIGINAL_PHRASES.some((p) => normalized.includes(p)) ||
    originalPerLanguage(normalized)
  );
}

// Generate a thorough battery: for every language, sweep each dimension's words
// (one word present at a time, the other two dimensions fixed to their first
// word and also to empty), plus all bare phrases, plus cross-language mixes,
// plus the pinned prompts and a set of negatives.
const battery = [];
for (const lang of ["en", "ru", "hi", "zh"]) {
  const v = ORIGINAL[lang];
  // full positive: subject0 + request0 + scope0
  battery.push(`${v.subject[0]} ${v.request[0]} ${v.scope[0]}`);
  // sweep each dimension's every word with the other two at index 0
  for (const s of v.subject) battery.push(`${s} ${v.request[0]} ${v.scope[0]}`);
  for (const r of v.request) battery.push(`${v.subject[0]} ${r} ${v.scope[0]}`);
  for (const c of v.scope) battery.push(`${v.subject[0]} ${v.request[0]} ${c}`);
  // missing-dimension cases (each pair, expect false unless a phrase matches)
  battery.push(`${v.subject[0]} ${v.request[0]}`);
  battery.push(`${v.subject[0]} ${v.scope[0]}`);
  battery.push(`${v.request[0]} ${v.scope[0]}`);
  battery.push(`${v.subject[0]}`);
  battery.push(`${v.request[0]}`);
  battery.push(`${v.scope[0]}`);
}
// cross-language mixes (per-language AND must reject these unless a phrase hits)
battery.push("rules покажи 行为"); // en subject + ru request + zh scope
battery.push("नियम show поведения"); // hi subject + en request + ru scope
battery.push("规则 list आपके"); // zh subject + en request + hi scope
battery.push("правил which behavior"); // ru subject + en request + en scope
// every bare phrase, raw and embedded in a sentence
for (const p of ORIGINAL_PHRASES) {
  battery.push(p);
  battery.push(`please ${p} now`);
}
// pinned issue prompts + negatives (raw; normalized below)
const pinned = [
  "Show list of your rules",
  "Покажи список своих правил",
  "Перечисли свои правила",
  "अपने नियमों की सूची दिखाओ",
  "显示你的规则列表",
  "List behavior rules",
  "behavior rules",
  "Покажи правило unknown", // detail, not a list
  "What is the weather today?",
  "Кто ты?",
  "Translate hello to french",
  "",
];
for (const p of pinned) battery.push(p);

let mismatches = 0;
for (const raw of battery) {
  const normalized = sandbox.normalizePrompt(raw);
  const want = originalIsBehaviorRulesList(normalized);
  const got = sandbox.isBehaviorRulesList(normalized);
  if (want !== got) {
    mismatches += 1;
    check(`parity «${raw}»`, false, `old=${want} new=${got} normalized=«${normalized}»`);
  }
}
check(
  `isBehaviorRulesList matches pre-conversion logic on ${battery.length} prompts`,
  mismatches === 0,
);

// --- (3) concrete issue-#386 routing expectations ----------------------------
const mustList = [
  "Show list of your rules",
  "Покажи список своих правил",
  "Перечисли свои правила",
  "अपने नियमों की सूची दिखाओ",
  "显示你的规则列表",
  "List behavior rules",
  // Bare-phrase shortcuts that name the rule-set with no separate verb — only
  // languages whose original code carried a verb-less phrase qualify (Chinese
  // "行为规则", Hindi "व्यवहार के नियम"); bare English "behavior rules" still
  // required a request verb pre-conversion, so it stays out of this list.
  "行为规则",
  "व्यवहार के नियम",
];
// Bare English "behavior rules" was NOT a list request pre-conversion (the
// English path requires a request verb): parity must keep it negative.
check(
  "«behavior rules» stays NOT a list request (English needs a verb)",
  !sandbox.isBehaviorRulesList(sandbox.normalizePrompt("behavior rules")),
);
for (const p of mustList) {
  check(`«${p}» -> behavior rules list`, sandbox.isBehaviorRulesList(sandbox.normalizePrompt(p)));
}
const mustNotList = [
  "Покажи правило unknown",
  "What is the weather today?",
  "Кто ты?",
];
for (const p of mustNotList) {
  check(`«${p}» -> NOT a list request`, !sandbox.isBehaviorRulesList(sandbox.normalizePrompt(p)));
}

console.log(
  fail.length === 0
    ? `\nALL PASS (${battery.length}-prompt battery)`
    : `\n${fail.length} FAILED`,
);
process.exit(fail.length === 0 ? 0 : 1);
