// Issue #386: verify the JS worker's coding-catalog matchers resolve every
// language and task through the *embedded seed* (the `program_language_<slug>` /
// `program_task_<slug>` meanings) now that the inline `aliases` arrays are gone.
//
// Mirrors the two Rust behavioural guards in src/coding/catalog/mod.rs
// (`every_language_resolves_from_its_seed_surfaces` /
// `every_task_resolves_through_the_seed`) and additionally pins the convergent
// gains the worker picked up by unifying onto the seed (c++, c#, Devanagari
// surfaces it never carried before).
//
// Run with: node experiments/issue-386-worker-catalog-alias-parity.mjs

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

// Only top-level `function` declarations land on the vm global; the `const`
// catalog objects do not, so the slug lists below are spelled out (a test may
// carry hardcoded examples — issue #386). They mirror the Object.keys order of
// WRITE_PROGRAM_LANGUAGES / WRITE_PROGRAM_TASKS in the worker.
const { programLanguageFromPrompt, programTaskFromPrompt, wordsForMeaning } =
  sandbox;

const LANGUAGE_SLUGS = [
  "rust",
  "python",
  "javascript",
  "typescript",
  "go",
  "c",
  "cpp",
  "java",
  "csharp",
  "ruby",
];
const TASK_SLUGS = [
  "hello_world",
  "count_to_three",
  "list_files",
  "list_files_arg",
  "list_files_reverse_sort",
  "list_files_arg_reverse_sort",
  "fizzbuzz",
  "factorial",
  "reverse_string",
  "sum_to_ten",
  "fibonacci",
];

let failures = 0;
function check(label, condition, detail) {
  if (condition) {
    console.log(`ok   ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}${detail ? `\n     ${detail}` : ""}`);
  }
}

// (1) Every language's canonical slug spelling is a unique token, so feeding it
// must resolve back to that language through the seed.
for (const slug of LANGUAGE_SLUGS) {
  check(
    `language ${slug} resolves from its slug surface`,
    programLanguageFromPrompt(slug) === slug,
    `got ${programLanguageFromPrompt(slug)}`,
  );
}

// (2) Every task's first seed surface must resolve to *some* catalog task
// (priority/substring matching means a longer phrase may legitimately resolve to
// a shorter-prefix task, exactly as on the Rust side).
for (const slug of TASK_SLUGS) {
  const first = wordsForMeaning(`program_task_${slug}`)[0];
  check(
    `task ${slug} first surface "${first}" resolves to a task`,
    typeof first === "string" && programTaskFromPrompt(first) !== null,
    `got ${programTaskFromPrompt(first)}`,
  );
}

// (3) Convergent gains: surfaces the worker did NOT carry before unifying onto
// the seed now resolve (worker gained, lost nothing).
const gains = [
  ["c++", "cpp"],
  ["cplusplus", "cpp"],
  ["c#", "csharp"],
  ["dotnet", "csharp"],
  ["रस्ट", "rust"], // Hindi "Rust"
  ["जावा", "java"], // Hindi "Java"
  ["गो", "go"], // Hindi "go"
];
for (const [surface, expected] of gains) {
  check(
    `convergent surface "${surface}" resolves to ${expected}`,
    programLanguageFromPrompt(surface) === expected,
    `got ${programLanguageFromPrompt(surface)}`,
  );
}

// (4) Every original alias the worker carried before the refactor still resolves
// (regression guard against the seed dropping any surface). These are the exact
// tokens that lived on WRITE_PROGRAM_LANGUAGES before issue #386.
const legacy = [
  ["rust", "rust"], ["rs", "rust"], ["раст", "rust"], ["расте", "rust"],
  ["python", "python"], ["py", "python"], ["питон", "python"], ["питоне", "python"],
  ["javascript", "javascript"], ["js", "javascript"], ["node", "javascript"], ["джаваскрипт", "javascript"],
  ["typescript", "typescript"], ["ts", "typescript"], ["тайпскрипт", "typescript"],
  ["go", "go"], ["golang", "go"], ["го", "go"],
  ["c", "c"],
  ["cpp", "cpp"], ["cplusplus", "cpp"],
  ["java", "java"], ["джава", "java"],
  ["csharp", "csharp"], ["cs", "csharp"], ["dotnet", "csharp"],
  ["ruby", "ruby"], ["rb", "ruby"], ["руби", "ruby"],
];
for (const [surface, expected] of legacy) {
  check(
    `legacy surface "${surface}" still resolves to ${expected}`,
    programLanguageFromPrompt(surface) === expected,
    `got ${programLanguageFromPrompt(surface)}`,
  );
}

if (failures > 0) {
  console.error(`\n${failures} catalog-alias parity check(s) FAILED.`);
  process.exit(1);
}
console.log("\nIssue #386 worker catalog-alias parity checks passed.");
