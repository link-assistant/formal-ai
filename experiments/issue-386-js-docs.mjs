// Issue #386 — web-runtime parity for the documentation-method handler
// recognizers (meanings-docs.lino + the web_medium role on reference_internet).
//
// After converting src/solver_handler_docs.rs to recognise the "how does the
// pandas join method work?" question by *meaning* rather than three hardcoded
// per-language word lists, the JS worker must stay on par. The three rewritten
// recognizers are mirrored here:
//   isExplanationRequest   ↔ is_explanation_request    (explanation_request_lead)
//   isExplicitWebSearchPrompt ↔ is_explicit_web_search (web_search_imperative_lead
//                                                        + web_medium)
//   isPandasDataFrameJoinPrompt ↔ is_pandas_dataframe_join_prompt (code_method_noun)
// plus the routing (tryDocsMethodExplanation) for the four multilingual prompts
// the Rust spec pins in tests/unit/specification/reasoning_paths.rs. Run:
//   node experiments/issue-386-js-docs.mjs

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

const norm = (prompt) => sandbox.normalizePrompt(prompt);
const isExplanation = (prompt) => sandbox.isExplanationRequest(norm(prompt));
const isWebSearch = (prompt) => sandbox.isExplicitWebSearchPrompt(norm(prompt));
const isJoinPrompt = (prompt) => sandbox.isPandasDataFrameJoinPrompt(prompt, norm(prompt));

// === isExplanationRequest mirrors the original 20-way disjunction ===========
// Every lead-in the function used to hardcode, one positive per surface; the
// prefix surfaces lead the prompt, the bare surfaces sit inside it.
console.log("=== explanation-request recognition (mirror of is_explanation_request) ===");
for (const prompt of [
  "how does this work",            // how … (prefix)
  "but how does this work",        // " how " (bare, whole word)
  "explain the join method",       // explain …
  "describe the join method",      // describe …
  "what does join do",             // what does …
  "what is a dataframe",           // what is …
  "tell me about pandas join",     // tell me about …
  "how to use the join method",    // how to use …
  "как работает join",             // как … (prefix)
  "а как работает join",           // " как " (bare)
  "объясни метод join",            // объясни …
  "расскажи про join",             // расскажи …
  "что такое dataframe",           // что такое …
  "pandas join कैसे काम करता है",   // कैसे काम (bare substring)
  "समझाओ join विधि",               // समझाओ… (prefix, no space)
  "क्या है dataframe",              // क्या है … (prefix, no space before slot? has space)
  "join 如何工作",                  // 如何工作 (bare substring)
  "join 怎么工作",                  // 怎么工作 (bare substring)
  "解释 join 方法",                 // 解释… (prefix, no space)
  "join 是什么",                    // 是什么 (bare substring)
]) {
  check(`explanation prompt is recognised: ${prompt}`, isExplanation(prompt) === true);
}
// क्या है … is a PREFIX lead — it must START the prompt; a mid-sentence
// occurrence is not matched (mirrors normalized.starts_with on before_slot).
check("क्या है as a prefix lead at the start is recognised", isExplanation("क्या है dataframe") === true);

console.log("\n=== non-explanation prompts fall through ===");
for (const prompt of [
  "the cat sat on the mat",
  "search the web for pandas",
  "join two tables in sql",        // no explanation lead-in
  "show me the weather",
]) {
  check(`non-explanation prompt falls through: ${prompt}`, isExplanation(prompt) === false);
}

// === isExplicitWebSearchPrompt mirrors is_explicit_web_search ===============
// Lead = a web_search_imperative_lead prefix at the start; medium = a web_medium
// surface anywhere (whole-token, padding-aware). Both halves required.
console.log("\n=== explicit-web-search screen (mirror of is_explicit_web_search) ===");
for (const prompt of [
  "search the web for pandas join",
  "look up pandas join on the internet",
  "research pandas join online",
  "find pandas join on the web",
]) {
  check(`explicit web search is recognised: ${prompt}`, isWebSearch(prompt) === true);
}
console.log("\n=== web-search screen requires BOTH a lead and a medium ===");
for (const prompt of [
  "search for pandas join",            // lead, no medium
  "explain pandas on the web",         // medium, no search lead
  "how does the join method work",     // neither
]) {
  check(`not an explicit web search: ${prompt}`, isWebSearch(prompt) === false);
}

// === isPandasDataFrameJoinPrompt — the four pinned multilingual prompts =====
// These are the prompts tests/unit/specification/reasoning_paths.rs routes to
// docs_method_explanation; the JS recognizer must accept all four.
console.log("\n=== pandas DataFrame.join recognition (mirror of is_pandas_dataframe_join_prompt) ===");
const PINNED = [
  ["en", "how the join method works in pandas"],
  ["ru", "объясни как работает метод join в pandas"],
  ["hi", "समझाओ pandas में join विधि कैसे काम करती है"],
  ["zh", "请解释 pandas 中的 join 方法如何工作 以及它如何使用索引"],
];
for (const [lang, prompt] of PINNED) {
  check(`pinned ${lang} prompt is a join-method prompt: ${prompt}`, isJoinPrompt(prompt) === true);
}
console.log("\n=== join recognition rejects non-matches ===");
for (const prompt of [
  "how the merge method works in pandas",   // method but not join → falls to other branch? join word absent
  "search the web for the pandas join method", // explicit web search short-circuit
  "explain list comprehensions in python",  // no pandas
  "how do i sort a list",                    // no pandas, no join
]) {
  check(`non-join prompt falls through: ${prompt}`, isJoinPrompt(prompt) === false);
}
// The code-resident API identifiers still bridge directly (no method noun):
check("DataFrame.join identifier is recognised", isJoinPrompt("explain pandas DataFrame.join") === true);
check("df.join identifier is recognised", isJoinPrompt("explain pandas df.join") === true);
check("join + dataframe pairing is recognised", isJoinPrompt("how does join work on a pandas dataframe") === true);

// === routing parity: tryDocsMethodExplanation for the four pinned prompts ===
// Mirror of reasoning_paths.rs: each routes to docs_method_explanation with
// DataFrame.join evidence and a language:<x> tag, at confidence 0.92.
console.log("\n=== docs_method_explanation routing (mirror of reasoning_paths.rs) ===");
for (const [lang, prompt] of PINNED) {
  const detected = sandbox.detectLanguage(prompt);
  const answer = sandbox.tryDocsMethodExplanation(prompt, detected);
  check(`${lang} prompt routes to docs_method_explanation`, answer && answer.intent === "docs_method_explanation");
  if (answer) {
    check(`${lang} answer confidence is 0.92`, answer.confidence === 0.92);
    check(
      `${lang} answer cites pandas.DataFrame.join`,
      answer.evidence.some((e) => e.includes("pandas.DataFrame.join")),
    );
    check(
      `${lang} answer carries a language tag`,
      answer.evidence.some((e) => e.startsWith("language:")),
      JSON.stringify(answer.evidence),
    );
    check(
      `${lang} answer body names DataFrame.join`,
      answer.content.includes("DataFrame.join"),
    );
  }
}

// === data parity: the three migrated roles are embedded with full coverage ==
console.log("\n=== embedded lexicon carries the docs meanings ===");
const lexicon = sandbox.meaningLexicon();
const meaningsForRole = (role) => lexicon.filter((m) => m.roles.includes(role));
const wordsForRole = (role) => meaningsForRole(role).flatMap((m) => m.words);
const langsForRole = (role) => {
  const langs = new Set();
  for (const m of meaningsForRole(role)) for (const lx of m.lexemes) if (lx.words.length) langs.add(lx.language);
  return [...langs].sort();
};
const setEq = (a, b) => a.length === b.length && [...a].sort().join("|") === [...b].sort().join("|");

for (const role of ["explanation_request_lead", "code_method_noun"]) {
  check(`${role} present with surface words`, wordsForRole(role).length > 0);
  check(
    `${role} covers all four languages`,
    JSON.stringify(langsForRole(role)) === '["en","hi","ru","zh"]',
    JSON.stringify(langsForRole(role)),
  );
}
// web_medium lives on reference_internet in meanings-web-search.lino; it carries
// the four-language medium surfaces (the same list the Rust handler matches).
check("web_medium present with surface words", wordsForRole("web_medium").length > 0);
check(
  "web_medium covers all four languages",
  JSON.stringify(langsForRole("web_medium")) === '["en","hi","ru","zh"]',
  JSON.stringify(langsForRole("web_medium")),
);

// Byte-faithful migration of the original isExplanationRequest list: each old
// disjunct's surface (prefix .before or bare .text) must be present in the role.
console.log("\n=== explanation_request_lead surfaces match the original recognizer list ===");
const explanationSurfaces = sandbox
  .roleWordForms("explanation_request_lead")
  .map((form) => (form.slot === "prefix" ? form.before : form.text));
const expectedExplanation = [
  "how ", " how ", "explain ", "describe ", "what does ", "what is ",
  "tell me about ", "how to use ",
  "как ", " как ", "объясни ", "расскажи ", "что такое ",
  "कैसे काम", "समझाओ", "क्या है ",
  "如何工作", "怎么工作", "解释", "是什么",
];
check(
  "explanation_request_lead prefix/bare surfaces equal the original 20-way list",
  setEq(explanationSurfaces, expectedExplanation),
  JSON.stringify(explanationSurfaces),
);

// code_method_noun surfaces: the four per-language method words the join branch
// used to hardcode (method / метод / विधि / 方法).
const methodSurfaces = wordsForRole("code_method_noun");
check(
  "code_method_noun surfaces equal the four hardcoded method words",
  setEq(methodSurfaces, ["method", "метод", "विधि", "方法"]),
  JSON.stringify(methodSurfaces),
);

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
