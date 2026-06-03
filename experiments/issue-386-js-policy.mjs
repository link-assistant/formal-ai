// Issue #386 — web-runtime parity for the policy handlers (meanings-policy.lino).
//
// After converting solver_handlers_policy.rs to recognise the «купи слона»
// circular-joke idiom by *meaning* (the circular_joke_phrase role) rather than
// a hardcoded Russian literal, the JS worker must stay on par: tryKupiSlona now
// asks the lexicon, so the canonical Russian phrase AND its buy-an-elephant
// calque in every supported language route to the kupi_slona intent, while
// unrelated prompts still fall through. Run: `node experiments/issue-386-js-policy.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => { throw new Error("no importScripts in node"); };
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => { throw new Error("no fetch"); };
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

const kupi = (prompt) => sandbox.tryKupiSlona(prompt, sandbox.normalizePrompt(prompt));

// The canonical Russian idiom keeps its dedicated intent and Russian reply —
// the original issue contract, now satisfied through the lexicon.
console.log("=== kupi_slona recognition (mirror of src/solver_handlers_policy.rs) ===");
for (const prompt of ["Купи слона", "купи слона", "ну купи слона уже"]) {
  const r = kupi(prompt);
  check(`russian idiom routes to kupi_slona: ${prompt}`, r && r.intent === "kupi_slona", r && r.intent);
}
const ru = kupi("Купи слона");
check("kupi_slona reply is the traditional Russian text", ru && (ru.content.includes("слон") || ru.content.includes("всех")), ru && ru.content.slice(0, 24));
check("kupi_slona reply is tagged Russian", ru && Array.isArray(ru.evidence) && ru.evidence.includes("language:ru"), ru && JSON.stringify(ru.evidence));

// The concept is now lexicalized in every supported language, so the calque is
// recognised too — recognition is by meaning, not by one language's words.
for (const [lang, prompt] of [
  ["en", "please buy an elephant"],
  ["hi", "हाथी खरीदो"],
  ["zh", "买大象"],
]) {
  const r = kupi(prompt);
  check(`calque routes to kupi_slona (${lang}): ${prompt}`, r && r.intent === "kupi_slona", r && r.intent);
}

// Unrelated prompts must still fall through (null) so the dispatcher continues.
for (const prompt of ["what is the capital of France", "напиши программу на Rust", "купи молоко"]) {
  check(`unrelated prompt falls through: ${prompt}`, kupi(prompt) === null);
}

// Data parity: the worker's embedded lexicon carries every policy role with
// surface words in all four languages — even physical_action_trigger and
// vulgar_content_marker, which only the Rust solver reads (the worker screens
// no content policy, so the data is mirrored but not acted on here).
console.log("\n=== embedded lexicon carries the policy meanings ===");
const lexicon = sandbox.meaningLexicon();
const meaningsForRole = (role) => lexicon.filter((m) => m.roles.includes(role));
const langsForRole = (role) => {
  const langs = new Set();
  for (const m of meaningsForRole(role)) for (const lx of m.lexemes) if (lx.words.length) langs.add(lx.language);
  return [...langs].sort();
};
for (const role of ["circular_joke_phrase", "physical_action_trigger", "vulgar_content_marker"]) {
  check(`${role} present with surface words`, meaningsForRole(role).flatMap((m) => m.words).length > 0);
  check(`${role} covers all four languages`, JSON.stringify(langsForRole(role)) === '["en","hi","ru","zh"]', JSON.stringify(langsForRole(role)));
}

// The English profanity migrated verbatim from the original hardcoded list is
// still present, so the Rust refusal contract (it drives is_inappropriate_content)
// reads the same surfaces — confirmed here against the shared mirror.
const vulgarWords = meaningsForRole("vulgar_content_marker").flatMap((m) => m.words);
for (const surface of ["suck my dick", "fuck you", "you suck"]) {
  check(`vulgar_content_marker carries the migrated English surface: ${surface}`, vulgarWords.includes(surface));
}

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
