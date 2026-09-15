// Browser-worker parity probe for the four conversational regressions in #710.

import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";
import { TextDecoder, TextEncoder } from "node:util";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const webDir = path.join(root, "src", "web");
const sandbox = {
  console,
  location: { search: "" },
  postMessage() {},
  setTimeout,
  clearTimeout,
  TextDecoder,
  TextEncoder,
  URL,
  URLSearchParams,
  WebAssembly: { instantiate: async () => { throw new Error("WASM disabled in parity probe"); } },
};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.fetch = async (url) => {
  const relative = String(url).split("?")[0];
  const file = relative.startsWith("seed/")
    ? path.join(root, "data", relative)
    : path.join(webDir, relative);
  if (!fs.existsSync(file)) {
    return { ok: false, status: 404, async text() { return ""; } };
  }
  return { ok: true, status: 200, async text() { return fs.readFileSync(file, "utf8"); } };
};
vm.createContext(sandbox);

for (const relative of [
  "seed_loader.js",
  ...Array.from({ length: 22 }, (_, index) =>
    `worker/formal_ai_worker_${String(index).padStart(2, "0")}.js`),
]) {
  const file = path.join(webDir, relative);
  vm.runInContext(fs.readFileSync(file, "utf8"), sandbox, { filename: file });
}
await vm.runInContext("loadSeed()", sandbox);

const solve = (prompt, history = []) => sandbox.solve(
  prompt,
  history,
  { greetingVariations: false, temperature: 0 },
  {},
  [],
);

const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};

const compound = await solve("Who are you? What can you do? What is 2 + 2?");
assert(compound.intent === "compound_response", `compound: ${compound.intent}`);
assert(compound.content.trimEnd().endsWith("4"), `compound order: ${compound.content}`);
assert(compound.evidence.filter((item) => item.startsWith("sub_impulse:")).length === 3,
  `compound evidence: ${JSON.stringify(compound.evidence)}`);

const mixedGreeting = await solve("Привет! Кто ты?");
assert(mixedGreeting.intent === "identity",
  `mixed greeting and identity must keep whole-prompt routing: ${mixedGreeting.intent}`);

for (const [assignment, question, name] of [
  ["Now your name is Ada.", "What is your name?", "Ada"],
  ["Теперь тебя зовут Инеффа.", "Как тебя зовут?", "Инеффа"],
  ["अब तुम्हारा नाम इनेफ़ा है।", "तुम्हारा नाम क्या है?", "इनेफ़ा"],
  ["现在你叫伊内法。", "你叫什么名字？", "伊内法"],
]) {
  const acknowledgement = await solve(assignment);
  assert(acknowledgement.intent === "set_assistant_name", `${assignment}: ${acknowledgement.intent}`);
  const history = [
    { role: "user", content: assignment },
    { role: "assistant", content: acknowledgement.content },
  ];
  const directRecall = sandbox.recallAssistantName(history);
  assert(directRecall === name, `${assignment}: direct history recall ${directRecall}`);
  const recall = await solve(question, history);
  assert(recall.intent === "assistant_name" && recall.content.includes(name),
    `${question}: ${recall.intent} ${recall.content}`);
}

for (const prompt of ["Reverse it.", "Измени это.", "इसे बदलो।", "修改它。"]) {
  const result = await solve(prompt);
  assert(result.intent === "ambiguous_modification_clarification",
    `${prompt}: ${result.intent}`);
  assert((result.content.match(/[?？]/gu) || []).length === 1, `${prompt}: ${result.content}`);
}

for (const prompts of [
  ["What do you do in your free time?", "How do you spend your free time?", "What do you do when you are not working?"],
  ["Что делаешь в свободное время?", "Чем занимаешься в свободное время?", "Что делаешь когда свободен?"],
  ["खाली समय में क्या करते हो?", "आप खाली समय में क्या करते हैं?", "फुर्सत में क्या करते हो?"],
  ["你空闲时间做什么?", "你有空的时候做什么?", "你业余时间做什么?"],
]) {
  const answers = [];
  for (const prompt of prompts) {
    const first = await solve(prompt);
    const replay = await solve(prompt);
    assert(first.intent === "assistant_free_time", `${prompt}: ${first.intent}`);
    assert(first.content === replay.content, `${prompt}: unstable response`);
    answers.push(first.content);
  }
  assert(new Set(answers).size >= 2, `canned responses: ${JSON.stringify(answers)}`);
}

console.log("issue #710 browser-worker parity: all conversational cases passed");
