// Prints the intent the browser worker assigns to a spread of prompts, first
// natively and then with the response language forced, followed by the
// response-language follow-up (issue #556) across three intent families. The
// point of the sweep is that neither seam is intent-specific: forcing a
// language must not change which intent answered, and a bare "reply in X"
// follow-up must replay whatever the previous answer was.
//
// The worker boots through `tests/web/support/browser-runtime.mjs`, which runs
// `src/web/formal_ai_worker.js` and lets the entry point pick the modules it
// loads from the generated `worker-modules.js` (issue #991).

import { createWorkerContext, evaluate } from "../tests/web/support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

const prompts = [
  "what are you",
  "who are you",
  "what can you do",
  "define recursion",
  "what is formal-ai",
  "how do you work",
  "2+2",
  "what is a link",
];
for (const prompt of prompts) {
  const answer = await worker.solve(prompt, [], {}, {}, [], {});
  const forced = await worker.solve(prompt, [], {}, {}, [], {
    forcedResponseLanguage: "ru",
  });
  console.log(
    `${JSON.stringify(prompt).padEnd(22)} intent=${String(answer.intent).padEnd(22)} forced_ru_intent=${forced.intent}`,
  );
}

console.log("\n-- follow-up generalization across intent families --");
for (const [question, followup, language] of [
  ["what can you do", "я не понимаю по английски, напиши по русски", "ru"],
  ["what are you", "用中文回答", "zh"],
  ["2+2", "हिंदी में लिखें", "hi"],
]) {
  const first = await worker.solve(question, [], {}, {}, [], {});
  const history = [
    { role: "user", content: question },
    { role: "assistant", content: String(first.content || "") },
  ];
  const second = await worker.solve(followup, history, {}, {}, [], {});
  const evidence = Array.isArray(second.evidence) ? second.evidence : [];
  console.log(`\n[${question}] -> [${followup}]`);
  console.log(`  first=${first.intent}  second=${second.intent}`);
  console.log(
    `  target:${evidence.includes(`response_language_followup:target:${language}`)}` +
      ` language_to:${evidence.includes(`language_to:${language}`)}` +
      ` handler:${evidence.find((item) => item.startsWith("response_language_followup:handler:"))}`,
  );
  console.log(`  content: ${String(second.content || "").slice(0, 90).replace(/\n/g, " ")}`);
}
