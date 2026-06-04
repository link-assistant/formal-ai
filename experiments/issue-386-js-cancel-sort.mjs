// Issue #386 — end-to-end parity for the web runtime mirror.
//
// Mirrors `examples/repro_issue_386.rs`: replay the four-turn Russian
// conversation through `src/web/formal_ai_worker.js` using the worker's OWN
// rendered answers as the assistant history, then assert that the final
// follow-up "Отмени сортировку" ("cancel the sorting") routes to
// `write_program` and yields the *unsorted* path-argument program — the same
// outcome the Rust solver produces. Run: `node experiments/issue-386-js-cancel-sort.mjs`.

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

const FIRST_PROMPT =
  "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT = "Сделай сортировку результатов в обратном порядке";
const CANCEL_SORT_PROMPT = "Отмени сортировку";

// Replay the conversation, threading each turn's rendered answer back in as the
// assistant turn — exactly how the browser demo accumulates program state.
const history = [];
function turn(prompt) {
  const result = sandbox.tryWriteProgram(prompt, history.slice(), "ru");
  history.push({ role: "user", content: prompt });
  history.push({ role: "assistant", content: result ? result.content : "" });
  return result;
}

const first = turn(FIRST_PROMPT);
check("turn 1 routes to write_program", first && first.intent === "write_program", first && first.intent);
check("turn 1 is the base list_files program", first && !first.content.includes("env::args"), "no path argument yet");

const path = turn(PATH_ARGUMENT_PROMPT);
check("turn 3 routes to write_program", path && path.intent === "write_program", path && path.intent);
check("turn 3 adds the path argument", path && path.content.includes("env::args"), "env::args present");
check("turn 3 sorts ascending", path && path.content.includes("names.sort();") && !path.content.includes(".rev()"), "ascending sort");

const reverse = turn(REVERSE_SORT_PROMPT);
check("turn 5 routes to write_program", reverse && reverse.intent === "write_program", reverse && reverse.intent);
check("turn 5 keeps the path argument", reverse && reverse.content.includes("env::args"), "env::args retained");
check(
  "turn 5 sorts descending",
  reverse && (reverse.content.includes(".rev()") || /b\.cmp\(\s*a\s*\)|b\.cmp\(&a\)/.test(reverse.content)),
  "reverse sort present",
);

const cancel = turn(CANCEL_SORT_PROMPT);
console.log("\n=== turn 7: Отмени сортировку ===");
console.log("intent:", cancel && cancel.intent);
check("turn 7 routes to write_program (was: unknown — issue #386)", cancel && cancel.intent === "write_program", cancel && cancel.intent);
check("turn 7 keeps the path argument", cancel && cancel.content.includes("env::args"), "env::args retained");
check(
  "turn 7 removes the reverse sort (ascending again)",
  cancel && cancel.content.includes("names.sort();") && !cancel.content.includes(".rev()"),
  "ascending sort restored",
);
check(
  "turn 7 surfaces the substitution plan as evidence",
  cancel && Array.isArray(cancel.evidence) &&
    cancel.evidence.some((item) => String(item).startsWith("write_program_plan:")),
  cancel && JSON.stringify(cancel.evidence),
);
check(
  "turn 7 resolves to the unsorted path variant (list_files_arg)",
  cancel && cancel.evidence.some((item) => item === "program_parameter:task:list_files_arg"),
  cancel && JSON.stringify(cancel.evidence),
);
check("turn 7 reports no missing parameters", cancel && !cancel.content.includes("missing"), "no missing");

// The cancel modifier is detected as `cancel_reverse_sort`, and lowering the
// accumulated reverse-sorted task with it derives the unsorted task — pure data,
// no bespoke control flow.
const cancelModifiers = sandbox.detectedProgramModifiers(
  sandbox.normalizeProgramPrompt(CANCEL_SORT_PROMPT),
);
check(
  "cancel prompt detects cancel_reverse_sort modifier",
  JSON.stringify(cancelModifiers) === '["cancel_reverse_sort"]',
  JSON.stringify(cancelModifiers),
);
const undone = sandbox.lowerProgramPlan("list_files_arg_reverse_sort", ["cancel_reverse_sort"]);
check(
  "cancel lowers list_files_arg_reverse_sort -> list_files_arg",
  undone.resolvedTask === "list_files_arg",
  undone.resolvedTask,
);
const undoneBase = sandbox.lowerProgramPlan("list_files_reverse_sort", ["cancel_reverse_sort"]);
check(
  "cancel lowers list_files_reverse_sort -> list_files",
  undoneBase.resolvedTask === "list_files",
  undoneBase.resolvedTask,
);

// Issue #386: the program-artifact follow-up gate must reference *meanings*, not
// a hardcoded per-language word list. These mirror the Rust unit tests in
// `src/program_coreference.rs` so the JS worker stays concept-driven and on par.
console.log("\n=== meaning-lexicon gate (mirror of src/program_coreference.rs) ===");
const norm = (p) => sandbox.normalizeProgramPrompt(p);
for (const prompt of [
  "sort the results in reverse order",
  "сделай сортировку результатов в обратном порядке",
  "परिणामों को उल्टे क्रम में क्रमबद्ध करो",
  "对结果倒序排序",
]) {
  check(`additive follow-up recognized: ${prompt}`, sandbox.looksLikeBareProgramArtifactFollowUp(norm(prompt)));
}
for (const prompt of ["cancel the sorting", "undo the sort", "Отмени сортировку", "убери сортировку", "सॉर्ट हटाओ", "取消排序"]) {
  check(`cancel follow-up recognized: ${prompt}`, sandbox.looksLikeBareProgramArtifactFollowUp(norm(prompt)));
}
for (const prompt of ["what is the capital of France", "напиши стихотворение про осень"]) {
  check(`unrelated prompt does not trip the gate: ${prompt}`, !sandbox.looksLikeBareProgramArtifactFollowUp(norm(prompt)));
}
const lexicon = sandbox.meaningLexicon();
const wordsForRole = (role) => lexicon.filter((m) => m.roles.includes(role)).flatMap((m) => m.words);
check("lexicon is non-empty and self-describing", lexicon.length >= 10, `${lexicon.length} meanings`);
check("program_artifact role has surface words", wordsForRole("program_artifact").length > 0);
check("program_modification role has surface words", wordsForRole("program_modification").length > 0);
check("program_kind role has surface words", wordsForRole("program_kind").length > 0);
check("program_request role has surface words", wordsForRole("program_request").length > 0);

// Issue #386: the "write a <program>" gate must also reference *meanings* — a
// program_kind artifact requested by a program_request verb — in every
// supported language. Mirrors write_program_parameters in
// src/intent_formalization.rs (program_kind && program_request).
console.log("\n=== write-program gate (program_kind && program_request) ===");
// The gate fires when a program_kind noun pairs with a program_request verb;
// the intent then refines to a write_program* variant (no longer `unknown`,
// which was the issue #386 bug). Naming no programming language yields the
// `write_program_unsupported` refinement — still a recognised program request.
for (const [lang, prompt] of [
  ["en", "write a program that lists files in the current directory"],
  ["ru", "напиши программу, которая выводит список файлов"],
  ["hi", "वर्तमान निर्देशिका में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो"],
  ["zh", "写一个程序来列出当前目录中的文件"],
]) {
  const r = sandbox.tryWriteProgram(prompt, [], lang);
  check(
    `write-a-program gate recognised, not unknown (${lang})`,
    r && typeof r.intent === "string" && r.intent.startsWith("write_program"),
    r && r.intent,
  );
}

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
