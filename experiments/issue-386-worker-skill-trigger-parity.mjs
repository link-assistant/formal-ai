// Issue #386: functional parity check for the role-query skill-trigger
// recognizer in the hand-written browser worker. Loads src/web/formal_ai_worker.js
// in a vm sandbox and exercises looksLikeRuntimeRuleUpdate (and the downstream
// runtimeRuleFromText extractor) so the lexicon-driven recogniser is checked
// exactly as in production.
//
// The truth table below mirrors the Rust recogniser pair in src/skill_compiler.rs
// (explicit_teaching_form + looks_like_skill_description), which is independently
// locked by tests/unit/specification/natural_language_skill_compilation.rs and
// the inline unsupported_shape_is_rejected test. Both runtimes now read every
// surface from the same embedded meaning lexicon
// (data/seed/meanings-skill-compiler.lino) by semantic role, so a single truth
// table covers both. This harness proves the JS mirror agrees with it — in
// particular that the worker now recognises the surfaces it used to miss
// ("when the user says", "when the user asks", "respond").
//
// Run with: node experiments/issue-386-worker-skill-trigger-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

const sandbox = {
  self: { location: { search: "" } },
  importScripts: () => {
    throw new Error("no importScripts in node harness");
  },
  postMessage: () => {},
  console,
  TextEncoder,
  TextDecoder,
  WebAssembly,
  fetch: () => Promise.reject(new Error("offline")),
  setTimeout,
  clearTimeout,
};
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(source, sandbox, { filename: "formal_ai_worker.js" });

const { looksLikeRuntimeRuleUpdate, runtimeRuleFromText } = sandbox;
for (const [name, fn] of [
  ["looksLikeRuntimeRuleUpdate", looksLikeRuntimeRuleUpdate],
  ["runtimeRuleFromText", runtimeRuleFromText],
]) {
  if (typeof fn !== "function") {
    throw new Error(`${name} not exposed by worker sandbox`);
  }
}

let failures = 0;
function check(label, got, want) {
  if (got === want) {
    console.log(`ok   ${label} -> ${JSON.stringify(got)}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}: got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);
  }
}

// --- TRUE cases: looksLikeRuntimeRuleUpdate must fire. ------------------------
// Explicit teaching form = (trigger lead AND response verb) OR edit directive;
// when-then form = a circumfix frame with backticks on each side.
const recognised = [
  // English explicit teaching — including the three surfaces the worker used to
  // miss before the lexicon conversion: "when the user says"/"asks"/"respond".
  ["en when-i-say/answer", "When I say `checksum status`, answer `checksum cache is valid.`"],
  ["en user-says/respond", "When the user says `ping`, respond `pong`"],
  ["en user-asks/reply", "When the user asks `status`, reply `all good`"],
  ["en if-i-ask/answer", "If I ask `time`, answer `noon`"],
  // English standalone edit directive — no response verb needed.
  ["en add rule", "Add behavior rule: greet politely"],
  ["en update rule", "Please update behavior rule for greetings"],
  // Russian explicit teaching (ответ is an inflectable stem of ответь).
  ["ru kogda-ya-skazhu/otvet", "Когда я скажу `привет`, ответь `здравствуй`"],
  ["ru esli-ya-sproshu/otvet", "Если я спрошу `время`, ответ `полдень`"],
  ["ru add rule", "Добавь правило поведения: будь вежлив"],
  ["ru update rule", "Обнови правило поведения для приветствий"],
  // Hindi explicit teaching.
  ["hi jab-main-kahun/uttar", "जब मैं कहूँ `नमस्ते` तो उत्तर `नमस्कार`"],
  ["hi add rule", "व्यवहार नियम जोड़ो: विनम्र रहो"],
  // Chinese explicit teaching (no spaces around the trigger lead).
  ["zh dang-wo-shuo/huida", "当我说`你好`，回答`您好`"],
  ["zh add rule", "添加行为规则：保持礼貌"],
  // When-then circumfix frames (backticks on each side of the link).
  ["en when/then", "When `status` then `ok`"],
  ["en when/do", "When `status` do `report ok`"],
  ["ru kogda/togda", "Когда `привет` тогда `здравствуй`"],
  ["ru esli/to", "Если `привет` то `здравствуй`"],
  ["hi jab/tab", "जब `नमस्ते` तब `नमस्कार`"],
  ["zh when/then spaced", "当 `状态` 时 `一切正常。`"],
  ["zh when-then-answer compact", "当 `状态`时回答 `一切正常。`"],
];
for (const [label, prompt] of recognised) {
  check(`recognise ${label}`, looksLikeRuntimeRuleUpdate(prompt), true);
}

// The backtick-quoted cases must also yield an extractable runtime rule.
const extractable = [
  ["en when-i-say", "When I say `checksum status`, answer `checksum cache is valid.`", "checksum status", "checksum cache is valid."],
  ["en when/then", "When `status` then `ok`", "status", "ok"],
  ["zh when/then", "当 `状态` 时 `一切正常。`", "状态", "一切正常。"],
];
for (const [label, prompt, trigger, answer] of extractable) {
  const rule = runtimeRuleFromText(prompt);
  check(`extract ${label} trigger`, rule ? rule.trigger : null, trigger);
  check(`extract ${label} answer`, rule ? rule.answer : null, answer);
}

// --- FALSE cases: looksLikeRuntimeRuleUpdate must NOT fire. -------------------
const notRecognised = [
  // Plain prose with no trigger/response/edit/when-then structure.
  ["en note", "This is only a note."],
  ["en question", "what is the capital of France"],
  // A response verb with no trigger lead (and no when-then backticks).
  ["en answer-alone", "Please answer the question"],
  ["en reply-alone", "reply to this email"],
  // A trigger lead with no response verb and no backticks.
  ["en when-i-say-alone", "When I say hello to people"],
  // A when-then frame with NO backticks — structure present, quotes absent.
  ["en when/then no-backticks", "when it rains then it pours"],
  // Chinese trigger lead "当我说" (no spaces) with no response verb and no
  // when-then frame: the head "当 " needs a space after 当, so neither path fires.
  ["zh trigger-alone", "当我说你好"],
];
for (const [label, prompt] of notRecognised) {
  check(`reject ${label}`, looksLikeRuntimeRuleUpdate(prompt), false);
}

if (failures) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nall worker skill-trigger parity checks passed");
