// Issue #386 — cross-language parity for the lexicon-driven feature-capability
// cluster.
//
// The browser worker's feature-capability recognizers used to be a hand-written
// `featureAliases` map plus a 60-line `WEB_SEARCH_CAPABILITY_PHRASES` array of
// hardcoded natural-language strings. They are now projected from the embedded
// meaning lexicon (data/seed/meanings-feature-capability.lino) by semantic
// *role* — the JS mirror of src/solver_handlers/feature_capability.rs:
//   * detectFeatureCapability      <- detect_feature_capability      (ROLE_FEATURE_CAPABILITY_ALIAS)
//   * isFeatureCapabilityQuestion  <- is_feature_capability_question (ROLE_FEATURE_CAPABILITY_QUESTION)
//   * isFeatureActionRequest       <- is_feature_action_request      (ROLE_FEATURE_ACTION_ARITHMETIC / _PLANNING)
//
// This harness replays the SAME battery the Rust integration test
// tests/unit/specification/capabilities.rs pins — every
// FEATURE_CAPABILITY_LANGUAGE_CASES row, the web-search question rows, and the
// action-routing cases — feeding the worker functions in isolation exactly as
// the Rust solver does: language = detectLanguageSlug(prompt) (byte-identical to
// src/language.rs::detect), normalized = prompt.toLowerCase() (the solver's
// `let normalized = prompt.to_lowercase()` at src/solver.rs), then asserting the
// detected feature slug, the question predicate, and the action-routing
// predicate match the Rust behavior. Same battery + same outputs in both
// languages proves the worker mirror never diverges from the Rust solver.
//
// Run: `node experiments/issue-386-js-feature-capability.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const src = fs.readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

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
function check(name, actual, expected) {
  const ok = actual === expected;
  if (!ok) {
    fail.push(name);
    console.log(
      `FAIL: ${name} :: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
  return ok;
}

// `(feature, language, prompt)`; row for row identical to
// FEATURE_CAPABILITY_LANGUAGE_CASES in tests/unit/specification/capabilities.rs.
// The `language` column documents the language the Rust test author intended;
// the harness ALSO asserts detectLanguageSlug(prompt) resolves to it, proving
// the worker's script detector agrees with src/language.rs::detect.
const FEATURE_CASES = [
  ["web_search", "en", "Can you search the internet?"],
  ["web_search", "ru", "Ты можешь искать в интернете?"],
  ["web_search", "hi", "क्या तुम इंटरनेट पर खोज सकते हो?"],
  ["web_search", "zh", "你能上网搜索吗？"],
  ["diagnostics", "en", "Do you support diagnostics?"],
  ["diagnostics", "ru", "У тебя есть диагностика?"],
  ["diagnostics", "hi", "क्या diagnostics उपलब्ध है?"],
  ["diagnostics", "zh", "诊断可用吗？"],
  ["agent_mode", "en", "Do you support agent mode?"],
  ["agent_mode", "ru", "У тебя есть agent mode?"],
  ["agent_mode", "hi", "क्या agent mode उपलब्ध है?"],
  ["agent_mode", "zh", "支持代理吗？"],
  ["definition_fusion", "en", "Do you support definition fusion?"],
  ["definition_fusion", "ru", "У тебя есть слияние определений?"],
  ["definition_fusion", "hi", "क्या परिभाषा विलय उपलब्ध है?"],
  ["definition_fusion", "zh", "支持合并定义吗？"],
  ["configuration", "en", "Can you configure settings?"],
  ["configuration", "ru", "Ты можешь менять настройки?"],
  ["configuration", "hi", "क्या सेटिंग उपलब्ध है?"],
  ["configuration", "zh", "可以配置设置吗？"],
  ["memory_actions", "en", "Can you export memory?"],
  ["memory_actions", "ru", "У тебя есть экспорт памяти?"],
  ["memory_actions", "hi", "क्या स्मृति निर्यात उपलब्ध है?"],
  ["memory_actions", "zh", "可以导出记忆吗？"],
  ["greeting", "en", "Can you respond to hello?"],
  ["greeting", "ru", "Ты умеешь здороваться?"],
  ["greeting", "hi", "क्या आप नमस्ते का जवाब दे सकते हैं?"],
  ["greeting", "zh", "你能打招呼吗？"],
  ["write_program", "en", "Do you support hello world code generation?"],
  ["write_program", "ru", "Ты можешь написать hello world программу?"],
  ["write_program", "hi", "क्या प्रोग्राम उपलब्ध है?"],
  ["write_program", "zh", "支持代码生成吗？"],
  ["concept_lookup", "en", "Do you support concept lookup?"],
  ["concept_lookup", "ru", "У тебя есть поиск понятий?"],
  ["concept_lookup", "hi", "क्या अवधारणा उपलब्ध है?"],
  ["concept_lookup", "zh", "支持概念查找吗？"],
  ["arithmetic", "en", "Can you do arithmetic?"],
  ["arithmetic", "ru", "Ты умеешь считать?"],
  ["arithmetic", "hi", "क्या अंकगणित उपलब्ध है?"],
  ["arithmetic", "zh", "支持算术吗？"],
  ["translation", "en", "Can you translate text?"],
  ["translation", "ru", "Ты умеешь переводить?"],
  ["translation", "hi", "क्या आप अनुवाद कर सकते हैं?"],
  ["translation", "zh", "你能翻译吗？"],
  ["memory", "en", "Can you remember conversation context?"],
  ["memory", "ru", "Ты можешь помнить контекст?"],
  ["memory", "hi", "क्या स्मृति उपलब्ध है?"],
  ["memory", "zh", "你有会话记忆吗？"],
  ["demo_mode", "en", "Do you support demo mode?"],
  ["demo_mode", "ru", "У тебя есть демо-режим?"],
  ["demo_mode", "hi", "क्या डेमो उपलब्ध है?"],
  ["demo_mode", "zh", "支持演示模式吗？"],
  ["http_url", "en", "Do you support open url?"],
  ["http_url", "ru", "У тебя есть URL-навигация?"],
  ["http_url", "hi", "क्या लिंक खोलना उपलब्ध है?"],
  ["http_url", "zh", "支持 URL 导航吗？"],
  ["javascript_execution", "en", "Can you execute JavaScript?"],
  ["javascript_execution", "ru", "Ты можешь выполнять JavaScript?"],
  ["javascript_execution", "hi", "क्या js उपलब्ध है?"],
  ["javascript_execution", "zh", "支持脚本执行吗？"],
  ["planning", "en", "Do you support project plan?"],
  ["planning", "ru", "Ты можешь планировать проект?"],
  ["planning", "hi", "क्या परियोजना योजना उपलब्ध है?"],
  ["planning", "zh", "支持项目计划吗？"],
];

// The web-search question rows from capabilities.rs (ENGLISH/RUSSIAN/HINDI/
// CHINESE_WEB_SEARCH_CAPABILITY): every one must detect the web_search feature
// AND register as a capability question in its own language.
const WEB_SEARCH_QUESTIONS = [
  ["en", "Can you search the internet?"],
  ["en", "Can you search the web?"],
  ["en", "Can you search online?"],
  ["en", "Do you have internet search?"],
  ["en", "Are you connected to search engines?"],
  ["ru", "Ты можешь искать в интернете?"],
  ["ru", "Можешь искать в интернете?"],
  ["ru", "Ты умеешь искать в интернете?"],
  ["ru", "У тебя есть веб-поиск?"],
  ["ru", "Ты подключен к поисковикам?"],
  ["hi", "क्या तुम इंटरनेट पर खोज सकते हो?"],
  ["hi", "क्या आप इंटरनेट पर खोज सकते हैं?"],
  ["hi", "क्या तुम ऑनलाइन खोज सकते हो?"],
  ["hi", "क्या तुम्हारे पास इंटरनेट खोज है?"],
  ["hi", "क्या आप सर्च इंजन से जुड़े हैं?"],
  ["zh", "你能上网搜索吗？"],
  ["zh", "你可以搜索互联网吗？"],
  ["zh", "你能搜索网络吗？"],
  ["zh", "你有联网搜索吗？"],
  ["zh", "你能用搜索引擎吗？"],
];

// `(prompt, expected feature slug, expected isFeatureActionRequest)`. Mirrors
// action_requests_keep_routing_to_primary_handlers: the two action prompts must
// be gated OUT of the capability handler (action=true) so calculation/summarize
// can answer, while the plain capability questions for the same features stay
// IN (action=false). All English, so language detection is "en". Each action
// prompt carries an arithmetic/planning ALIAS word ("calculate", "summarize",
// "brainstorm", "roleplay") so the prompt detects a feature AND opens with an
// action frame — the only way the gate is reachable (see ROLE_SEPARATION below).
const ACTION_CASES = [
  ["Can you calculate 2 + 2?", "arithmetic", true],
  ["Can you do arithmetic?", "arithmetic", false],
  ["Can you summarize Rust?", "planning", true],
  ["Can you brainstorm 5 ideas?", "planning", true],
  ["Can you roleplay a teacher?", "planning", true],
  ["Do you support project plan?", "planning", false],
];

// Alias vs. action role separation, byte-faithful to origin/main: the arithmetic
// ALIAS set is "arithmetic|calculate|math|2 + 2" — it never contained "compute".
// "Can you compute 7 * 6?" therefore detects NO capability, so it falls straight
// through to the calculation handler rather than being gated by the capability
// handler. The "can you compute" action frame is only reachable when the prompt
// ALSO carries an alias word, which is why the gate sees null here and the whole
// capability path is skipped. Both the old Rust, the new Rust, and the worker
// agree. `(prompt, expected detect slug, expected gate result against detect)`.
const ROLE_SEPARATION = [["Can you compute 7 * 6?", null, false]];

// English "is/are … enabled/available" availability frame — a grammatical
// pattern kept in code (isEnglishAvailabilityQuestion, unchanged by #386) that
// isFeatureCapabilityQuestion ORs in for English. Each must register as a
// question AND resolve its feature via the lexicon. `false` rows confirm the
// frame does not over-fire.
const AVAILABILITY_FRAME = [
  ["is agent mode enabled?", "agent_mode", true],
  ["are diagnostics available?", "diagnostics", true],
  ["is web search enabled?", "web_search", true],
  ["what is apple", null, false],
];

let passed = 0;

for (const [feature, language, prompt] of FEATURE_CASES) {
  const detected = sandbox.detectLanguageSlug(prompt);
  const normalized = prompt.toLowerCase();
  const okLang = check(`lang(${prompt})`, detected, language);
  const cap = sandbox.detectFeatureCapability(normalized, detected);
  const okFeature = check(
    `feature(${prompt})`,
    (cap && cap.slug) ?? null,
    feature,
  );
  const okQuestion = check(
    `question(${prompt})`,
    sandbox.isFeatureCapabilityQuestion(normalized, detected),
    true,
  );
  if (okLang && okFeature && okQuestion) passed += 1;
}

for (const [language, prompt] of WEB_SEARCH_QUESTIONS) {
  const detected = sandbox.detectLanguageSlug(prompt);
  const normalized = prompt.toLowerCase();
  const okLang = check(`ws-lang(${prompt})`, detected, language);
  const cap = sandbox.detectFeatureCapability(normalized, detected);
  const okFeature = check(
    `ws-feature(${prompt})`,
    (cap && cap.slug) ?? null,
    "web_search",
  );
  const okQuestion = check(
    `ws-question(${prompt})`,
    sandbox.isFeatureCapabilityQuestion(normalized, detected),
    true,
  );
  if (okLang && okFeature && okQuestion) passed += 1;
}

for (const [prompt, feature, action] of ACTION_CASES) {
  const detected = sandbox.detectLanguageSlug(prompt);
  const normalized = prompt.toLowerCase();
  const cap = sandbox.detectFeatureCapability(normalized, detected);
  const okFeature = check(
    `act-feature(${prompt})`,
    (cap && cap.slug) ?? null,
    feature,
  );
  const okAction = check(
    `act-gate(${prompt})`,
    sandbox.isFeatureActionRequest(normalized, cap),
    action,
  );
  if (okFeature && okAction) passed += 1;
}

for (const [prompt, slug, gate] of ROLE_SEPARATION) {
  const detected = sandbox.detectLanguageSlug(prompt);
  const normalized = prompt.toLowerCase();
  const cap = sandbox.detectFeatureCapability(normalized, detected);
  const okFeature = check(`sep-feature(${prompt})`, (cap && cap.slug) ?? null, slug);
  const okGate = check(
    `sep-gate(${prompt})`,
    sandbox.isFeatureActionRequest(normalized, cap),
    gate,
  );
  if (okFeature && okGate) passed += 1;
}

for (const [prompt, feature, question] of AVAILABILITY_FRAME) {
  const detected = sandbox.detectLanguageSlug(prompt);
  const normalized = prompt.toLowerCase();
  const cap = sandbox.detectFeatureCapability(normalized, detected);
  const okFeature = check(
    `avail-feature(${prompt})`,
    (cap && cap.slug) ?? null,
    feature,
  );
  const okQuestion = check(
    `avail-question(${prompt})`,
    sandbox.isFeatureCapabilityQuestion(normalized, detected),
    question,
  );
  if (okFeature && okQuestion) passed += 1;
}

const total =
  FEATURE_CASES.length +
  WEB_SEARCH_QUESTIONS.length +
  ACTION_CASES.length +
  ROLE_SEPARATION.length +
  AVAILABILITY_FRAME.length;

console.log(
  `\n${passed}/${total} feature-capability rows match the Rust battery in tests/unit/specification/capabilities.rs.`,
);

if (fail.length) {
  console.error(
    `\n${fail.length} assertion(s) FAILED — worker diverged from the Rust feature-capability baseline.`,
  );
  process.exit(1);
}
console.log(
  "PASS: worker feature-capability cluster is lexicon-driven and matches the Rust solver.",
);
