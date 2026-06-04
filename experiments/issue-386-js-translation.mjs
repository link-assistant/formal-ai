// Issue #386 — cross-language parity for the lexicon-driven translation cluster.
//
// The browser worker's source/target language detection and unquoted-surface
// extraction used to be three hand-written disjunctions over hardcoded
// natural-language strings. They are now projected from the embedded meaning
// lexicon (data/seed/meanings-translation.lino) by semantic *role*, *slot* and
// *script* — the JS mirror of src/translation/language_markers.rs and
// src/translation/prompt.rs.
//
// This harness replays the SAME 93-row frozen battery the Rust `parity` test in
// src/translation/mod.rs pins, feeding the worker functions in isolation
// exactly as that test does — `detect*(prompt.toLowerCase())` and
// `extractUnquotedTranslationSurface(prompt)` (raw) — and asserting byte-equal
// `(source, target, surface)` outputs. Same battery + same outputs in both
// languages proves the worker mirror never diverges from the Rust solver.
//
// Run: `node experiments/issue-386-js-translation.mjs`.

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

// `(prompt, expected source, expected target, expected surface)`; "-" == None.
// Identical, row for row, to the BATTERY in src/translation/mod.rs. The two
// GAP-FILL rows (с хинди -> hi, с китайского -> zh) reflect the Russian
// from-Hindi / from-Chinese source markers the all-four-languages seed invariant
// forced; they are honest improvements, called out in the Rust test too.
const BATTERY = [
  // --- source markers: English ---------------------------------
  ["translate apple from english", "en", "-", "-"],
  ["переведи apple с английского", "en", "-", "-"],
  ["apple अंग्रेजी से", "en", "-", "-"],
  ["apple अंग्रेज़ी से", "en", "-", "-"],
  ["从英语翻译 apple", "en", "-", "-"],
  ["从英文翻译 apple", "en", "-", "-"],
  // --- source markers: Russian ---------------------------------
  ["translate apple from russian", "ru", "-", "-"],
  ["apple с русского", "ru", "-", "-"],
  ["apple रूसी से", "ru", "-", "-"],
  ["从俄语翻译 apple", "ru", "-", "-"],
  // --- source markers: Hindi -----------------------------------
  ["translate apple from hindi", "hi", "-", "-"],
  ["apple हिंदी से", "hi", "-", "-"],
  ["apple हिन्दी से", "hi", "-", "-"],
  ["从印地语翻译 apple", "hi", "-", "-"],
  ["从印地文翻译 apple", "hi", "-", "-"],
  // --- source markers: Chinese ---------------------------------
  ["translate apple from chinese", "zh", "-", "-"],
  ["apple चीनी से", "zh", "-", "-"],
  ["从中文翻译 apple", "zh", "-", "-"],
  ["从汉语翻译 apple", "zh", "-", "-"],
  ["从漢語翻译 apple", "zh", "-", "-"],
  // --- target markers: English ---------------------------------
  ["translate apple to english", "-", "en", "apple"],
  ["переведи apple на английский", "-", "en", "apple"],
  ["apple на английском", "-", "en", "-"],
  ["apple अंग्रेजी में", "-", "en", "-"],
  ["apple अंग्रेज़ी में", "-", "en", "-"],
  ["apple 成英文", "-", "en", "-"],
  ["apple 成英语", "-", "en", "-"],
  ["apple 为英文", "-", "en", "-"],
  ["apple 为英语", "-", "en", "-"],
  ["apple 為英文", "-", "en", "-"],
  ["apple 為英语", "-", "en", "-"],
  ["apple 到英文", "-", "en", "-"],
  ["apple 到英语", "-", "en", "-"],
  // --- target markers: Russian ---------------------------------
  ["translate apple to russian", "-", "ru", "apple"],
  ["apple на русский", "-", "ru", "-"],
  ["apple 成俄语", "-", "ru", "-"],
  ["apple 成俄語", "-", "ru", "-"],
  ["apple 为俄语", "-", "ru", "-"],
  ["apple 为俄語", "-", "ru", "-"],
  ["apple 為俄语", "-", "ru", "-"],
  ["apple 為俄語", "-", "ru", "-"],
  ["apple 到俄语", "-", "ru", "-"],
  ["apple 到俄語", "-", "ru", "-"],
  // --- target markers: Hindi -----------------------------------
  ["translate apple to hindi", "-", "hi", "apple"],
  ["apple на хинди", "-", "hi", "-"],
  ["apple हिंदी में", "-", "hi", "-"],
  ["apple हिन्दी में", "-", "hi", "-"],
  ["apple 成印地语", "-", "hi", "-"],
  ["apple 成印地文", "-", "hi", "-"],
  ["apple 为印地语", "-", "hi", "-"],
  ["apple 为印地文", "-", "hi", "-"],
  ["apple 為印地语", "-", "hi", "-"],
  ["apple 為印地文", "-", "hi", "-"],
  ["apple 到印地语", "-", "hi", "-"],
  ["apple 到印地文", "-", "hi", "-"],
  // --- target markers: Chinese ---------------------------------
  ["translate apple to chinese", "-", "zh", "apple"],
  ["apple на китайский", "-", "zh", "-"],
  ["apple चीनी में", "-", "zh", "-"],
  ["apple 成中文", "-", "zh", "-"],
  ["apple 成汉语", "-", "zh", "-"],
  ["apple 成漢語", "-", "zh", "-"],
  ["apple 为中文", "-", "zh", "-"],
  ["apple 为汉语", "-", "zh", "-"],
  ["apple 为漢語", "-", "zh", "-"],
  ["apple 為中文", "-", "zh", "-"],
  ["apple 為汉语", "-", "zh", "-"],
  ["apple 為漢語", "-", "zh", "-"],
  ["apple 到中文", "-", "zh", "-"],
  ["apple 到汉语", "-", "zh", "-"],
  ["apple 到漢語", "-", "zh", "-"],
  // --- combined source+target ----------------------------------
  ["translate apple from english to russian", "en", "ru", "apple from english"],
  [
    "переведи яблоко с английского на русский",
    "en",
    "ru",
    "яблоко с английского",
  ],
  ["把 apple 从中文 翻译成英文", "zh", "en", "apple 从中文"],
  // --- extraction: English circumfix ---------------------------
  ["translate apple to russian", "-", "ru", "apple"],
  ["Translate Apple to Russian", "-", "ru", "Apple"],
  ["translate apple to russian.", "-", "ru", "apple"],
  ['translate "apple" to russian', "-", "ru", "-"],
  ["translate apple", "-", "-", "-"],
  ["what is apple", "-", "-", "-"],
  ["translate the red apple to russian", "-", "ru", "the red apple"],
  // --- extraction: Russian circumfix ---------------------------
  ["переведи яблоко на английский", "-", "en", "яблоко"],
  ["переведи красное яблоко на английский", "-", "en", "красное яблоко"],
  // --- extraction: Hindi ---------------------------------------
  ["apple का हिंदी में अनुवाद करो", "-", "hi", "apple"],
  ["सेब को अंग्रेजी में अनुवाद करो", "-", "en", "सेब"],
  // `हिंदी मे` (मे, not में) is not a target marker, so target is None, but the
  // object particle `का` still bounds the surface — asymmetry present in the
  // original behaviour and preserved verbatim.
  ["apple का हिंदी मे अनुवाद करो", "-", "-", "apple"],
  // --- extraction: Chinese -------------------------------------
  ["把 apple 翻译成中文", "-", "zh", "apple"],
  ["将苹果翻译成英文", "-", "en", "苹果"],
  ["翻译 apple 成中文", "-", "zh", "apple"],
  ["把 apple 翻译为英文", "-", "en", "apple"],
  ["把 apple 翻译到英文", "-", "en", "apple"],
  // --- GAP-FILL: original code had no Russian "from Hindi" / "from Chinese"
  //     source markers, so these were `-`/`-`/`-`; the seed now supplies them.
  ["apple с хинди", "hi", "-", "-"],
  ["apple с китайского", "zh", "-", "-"],
  // `रूसी में` already carried a Russian *target* marker pre-conversion.
  ["apple रूसी में", "-", "ru", "-"],
];

const opt = (sentinel) => (sentinel === "-" ? null : sentinel);

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

let passed = 0;
for (const [prompt, source, target, surface] of BATTERY) {
  const normalized = prompt.toLowerCase();
  const okSource = check(
    `source(${prompt})`,
    sandbox.detectTranslationSourceLanguage(normalized) ?? null,
    opt(source),
  );
  const okTarget = check(
    `target(${prompt})`,
    sandbox.detectTranslationTargetLanguage(normalized) ?? null,
    opt(target),
  );
  const okSurface = check(
    `surface(${prompt})`,
    sandbox.extractUnquotedTranslationSurface(prompt) ?? null,
    opt(surface),
  );
  if (okSource && okTarget && okSurface) passed += 1;
}

console.log(
  `\n${passed}/${BATTERY.length} translation rows match the frozen Rust battery (${BATTERY.length * 3} assertions).`,
);

// --- command recognition parity (#386) --------------------------------------
// inferTranslationSource and the try_translation gate no longer hardcode the
// command verbs; they read the translation_action role. Mirror the Rust unit
// tests words_for_role_partition_by_language and
// first_role_language_reads_the_command_language, then exercise the end-to-end
// source inference and the gate predicate (reconstructed from the SAME helper
// calls the worker's tryTranslation uses).
const ROLE_TRANSLATION_ACTION = "translation_action";
const headInitial = sandbox.wordsForRoleInLanguages(ROLE_TRANSLATION_ACTION, ["en", "ru"]);
const headFinal = sandbox.wordsForRoleInLanguages(ROLE_TRANSLATION_ACTION, ["hi", "zh"]);
check("partition en/ru has translate", headInitial.includes("translate"), true);
check("partition en/ru has переведи", headInitial.includes("переведи"), true);
check("partition en/ru has опиши", headInitial.includes("опиши"), true);
check("partition en/ru excludes 翻译", headInitial.includes("翻译"), false);
check("partition hi/zh has 翻译", headFinal.includes("翻译"), true);
check("partition hi/zh has अनुवाद", headFinal.includes("अनुवाद"), true);
check("partition hi/zh excludes translate", headFinal.includes("translate"), false);

const PRIORITY = ["ru", "hi", "zh"];
check("first-lang переведи -> ru", sandbox.firstRoleLanguage(ROLE_TRANSLATION_ACTION, "переведи apple", PRIORITY), "ru");
check("first-lang का अनुवाद -> hi", sandbox.firstRoleLanguage(ROLE_TRANSLATION_ACTION, "apple का अनुवाद करो", PRIORITY), "hi");
check("first-lang 翻译 -> zh", sandbox.firstRoleLanguage(ROLE_TRANSLATION_ACTION, "把 apple 翻译成中文", PRIORITY), "zh");
check("first-lang none -> null", sandbox.firstRoleLanguage(ROLE_TRANSLATION_ACTION, "what is apple", PRIORITY), null);

check("infer-source переведи -> ru", sandbox.inferTranslationSource("переведи apple"), "ru");
check("infer-source опиши -> ru", sandbox.inferTranslationSource("опиши apple"), "ru");
check("infer-source plain translate -> en", sandbox.inferTranslationSource("translate apple"), "en");

// The gate predicate, reconstructed from the SAME helper calls the worker uses.
function translationGate(normalized, targetHint) {
  const headInitialCommand = headInitial.some((stem) => normalized.startsWith(stem));
  const headFinalCommand =
    Boolean(targetHint) && headFinal.some((stem) => normalized.includes(stem));
  return headInitialCommand || headFinalCommand;
}
check("gate translate", translationGate("translate apple to russian", "ru"), true);
check("gate переведи", translationGate("переведи apple", null), true);
check("gate перевести (improvement)", translationGate("перевести apple", null), true);
check("gate опиши", translationGate("опиши apple", null), true);
check("gate 翻译 with target", translationGate("把 apple 翻译成中文", "zh"), true);
check("gate 翻译 without target -> false", translationGate("apple 翻译", null), false);
check("gate plain question -> false", translationGate("what is apple", null), false);

console.log(
  "command-recognition parity (inferTranslationSource + gate + helpers) checked.",
);

if (fail.length) {
  console.error(`\n${fail.length} assertion(s) FAILED — worker diverged from the Rust parity baseline.`);
  process.exit(1);
}
console.log("PASS: worker translation cluster is byte-identical to the Rust solver.");
