// Issue #529: verify the browser worker mirror of the Turing-complete memory
// writes — natural-language append + substitution — across the four supported
// languages, matching the Rust spec in tests/unit/specification/memory_query.rs.
import { readdirSync, readFileSync } from "node:fs";
import vm from "node:vm";
import { TextDecoder, TextEncoder } from "node:util";

const root = new URL("..", import.meta.url);
const webRoot = new URL("src/web/", root);

const sandbox = {
  location: { search: "", origin: "http://localhost" },
  postMessage() {},
  console,
  fetch: () => Promise.reject(new Error("no fetch in harness")),
  WebAssembly,
  TextEncoder,
  TextDecoder,
  URL,
  setTimeout,
  clearTimeout,
};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.onmessage = null;
sandbox.importScripts = (...paths) => {
  for (const requestedPath of paths) {
    const cleanPath = String(requestedPath).split("?")[0];
    const source = readFileSync(new URL(cleanPath, webRoot), "utf8");
    vm.runInContext(source, context, { filename: cleanPath });
  }
};

const context = vm.createContext(sandbox);
process.on("unhandledRejection", () => {});
const source = readFileSync(new URL("formal_ai_worker.js", webRoot), "utf8");
vm.runInContext(source, context, { filename: "formal_ai_worker.js" });

const rawSeed = {};
for (const entry of readdirSync(new URL("data/seed/", root), { withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith(".lino")) continue;
  rawSeed[entry.name] = readFileSync(new URL(`data/seed/${entry.name}`, root), "utf8");
}
context.hydrateLinoSeedText(rawSeed);

const { tryMemoryWrite, normalizePrompt } = context;
for (const [name, fn] of [
  ["tryMemoryWrite", tryMemoryWrite],
  ["normalizePrompt", normalizePrompt],
]) {
  if (typeof fn !== "function") throw new Error(`${name} not found in worker context`);
}

let failures = 0;
function check(label, ok, detail) {
  console.log(`[${label}] ${ok ? "PASS" : "FAIL"} -> ${detail}`);
  if (!ok) failures += 1;
}

// --- Append: store the statement and confirm it in the prompt's language. ---
const APPEND_CASES = [
  ["en", "remember that the sky is blue", "the sky is blue", "Recorded memory: the sky is blue"],
  ["ru", "запомни что небо синее", "небо синее", "Запомнил: небо синее"],
  ["hi", "याद रखो कि आकाश नीला है", "आकाश नीला है", "स्मृति में सहेजा गया:"],
  ["zh", "记住天空是蓝色的", "天空是蓝色的", "已记住:"],
];
for (const [language, prompt, statement, fragment] of APPEND_CASES) {
  const result = tryMemoryWrite(prompt, normalizePrompt(prompt), []);
  const ok =
    result &&
    result.intent === "memory_write" &&
    result.memoryOperation?.action === "append" &&
    result.memoryOperation.statement === statement &&
    result.content.includes(fragment);
  check(`append:${language}`, ok, result ? `${result.intent}: ${result.content}` : "null");
}

// --- Substitution: read every stored value, rewrite the matching ones. The
// snapshot holds two occurrences of `old`, so the applied count must be 2. ---
const SUBSTITUTION_CASES = [
  ["en", "replace alpha with beta in memory", "alpha", "beta",
    'Replaced "alpha" with "beta" in memory (2 occurrence(s) updated).'],
  ["ru", "замени альфа на бета в памяти", "альфа", "бета", 'Заменил "альфа" на "бета"'],
  // Hindi is SOV: "X की जगह Y रखो" (put Y in place of X) → old=X, new=Y.
  ["hi", "स्मृति में अल्फा की जगह बीटा रखो", "अल्फा", "बीटा", 'स्मृति में "अल्फा" को "बीटा" से बदला'],
  ["zh", "在记忆中把阿尔法换成贝塔", "阿尔法", "贝塔", '已在记忆中将"阿尔法"替换为"贝塔"'],
  // The directive may also lead in Russian regardless of scope position.
  ["ru-lead", "в памяти замени alpha на beta", "alpha", "beta", 'Заменил "alpha" на "beta"'],
];
for (const [language, prompt, oldValue, newValue, fragment] of SUBSTITUTION_CASES) {
  const memory = [`${oldValue} keeps ${oldValue}`];
  const result = tryMemoryWrite(prompt, normalizePrompt(prompt), memory);
  const ok =
    result &&
    result.intent === "memory_substitution" &&
    result.memoryOperation?.action === "substitute" &&
    result.memoryOperation.oldValue === oldValue &&
    result.memoryOperation.newValue === newValue &&
    result.memoryOperation.applied === 2 &&
    result.content.includes(fragment);
  check(`substitute:${language}`, ok, result ? `${result.intent} applied=${result.memoryOperation?.applied}: ${result.content}` : "null");
}

// --- A bare "replace X with Y" without a memory scope must NOT be hijacked. ---
const bare = tryMemoryWrite("replace alpha with beta", normalizePrompt("replace alpha with beta"), ["alpha"]);
check("substitute:no-scope", bare === null, bare ? `${bare.intent}` : "null (correct)");

if (failures) {
  console.error(`\n${failures} case(s) failed`);
  process.exit(1);
}
console.log("\nAll memory-write cases passed");
