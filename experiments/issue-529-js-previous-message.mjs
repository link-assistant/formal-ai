// Issue #529: verify the browser worker mirror recalls the previous message
// for "what was written in the previous message?" across supported languages.
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

const { tryHistorical } = context;
if (typeof tryHistorical !== "function") {
  throw new Error("tryHistorical not found in worker context");
}

const history = [
  { role: "user", content: "Прошлое сообщение" },
  { role: "assistant", content: "Я ещё не научился отвечать на это." },
];

const CASES = [
  ["ru", "что было написано в прошлом сообщении?", "ассистент"],
  ["en", "what was written in the previous message?", "assistant"],
  ["hi", "पिछले संदेश में क्या लिखा था?", "सहायक"],
  ["zh", "上一条消息写了什么?", "助手"],
];

let failures = 0;
for (const [language, prompt, expectedRoleLabel] of CASES) {
  const result = tryHistorical(prompt, history);
  const ok =
    result &&
    result.intent === "recall_previous_message" &&
    result.content.includes("Я ещё не научился отвечать на это.") &&
    result.content.includes(expectedRoleLabel);
  console.log(`[${language}] ${ok ? "PASS" : "FAIL"} -> ${result ? result.intent : "null"}: ${result ? result.content : ""}`);
  if (!ok) failures += 1;
}

// last-question recall must still resolve for the English phrasing.
const lastQuestion = tryHistorical("what was my previous question?", history);
const lqOk = lastQuestion && lastQuestion.intent === "recall_last_question";
console.log(`[en-last-question] ${lqOk ? "PASS" : "FAIL"} -> ${lastQuestion ? lastQuestion.intent : "null"}`);
if (!lqOk) failures += 1;

if (failures) {
  console.error(`\n${failures} case(s) failed`);
  process.exit(1);
}
console.log("\nAll previous-message recall cases passed");
