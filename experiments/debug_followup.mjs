// Traces the response-language follow-up (issue #556) one layer at a time:
// what the normalizer produces, what the two detectors say about it, what a
// forced replay of an earlier prompt renders, and finally the whole two-turn
// exchange the e2e test drives. Printing the intermediate values is the point —
// when the end-to-end case fails, this says which layer lost the language.
//
// The worker boots through `tests/web/support/browser-runtime.mjs`, which runs
// `src/web/formal_ai_worker.js` and lets the entry point pick the modules it
// loads from the generated `worker-modules.js` (issue #991).

import { createWorkerContext, evaluate } from "../tests/web/support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

for (const prompt of [
  "Я не понимаю по-английски, ответь по-русски",
  "用中文",
  "मुझे समझ नहीं आता, हिंदी में लिखें",
]) {
  const normalized = worker.normalizePrompt(prompt);
  console.log(`prompt=${prompt}`);
  console.log(`  normalized=${normalized}`);
  console.log(`  detectResponseLanguage=${worker.detectResponseLanguage(normalized)}`);
  console.log(`  detectComprehensionFailure=${worker.detectComprehensionFailure(normalized)}`);
}

console.log("\n-- replay of previous prompts (forced) --");
for (const [prompt, language] of [
  ["What is the deep-theory repository?", "ru"],
  ["What is a formal system?", "zh"],
  ["What is deep-theory?", "hi"],
]) {
  const answer = await worker.solve(prompt, [], {}, {}, [], {
    forcedResponseLanguage: language,
  });
  console.log(
    `  "${prompt}" -> intent=${answer.intent}  lang content: ${String(answer.content || "").slice(0, 60).replace(/\n/g, " ")}`,
  );
}

console.log("\n-- e2e-style project lookup then RU followup --");
const question = "ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?";
const first = await worker.solve(question, [], {}, {}, [], {});
console.log(`first intent=${first.intent}`);
const history = [
  { role: "user", content: question },
  { role: "assistant", content: String(first.content || "") },
];
const second = await worker.solve(
  "я не понимаю по английски, напиши по русски",
  history,
  {},
  {},
  [],
  {},
);
console.log(`second intent=${second.intent}`);
const evidence = Array.isArray(second.evidence) ? second.evidence : [];
console.log(`  target: ${evidence.includes("response_language_followup:target:ru")}`);
console.log(`  language_to: ${evidence.includes("language_to:ru")}`);
console.log(
  `  handler: ${evidence.find((item) => item.startsWith("response_language_followup:handler:"))}`,
);
console.log(`  content[0:120]: ${String(second.content || "").slice(0, 120).replace(/\n/g, " ")}`);
