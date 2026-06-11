// Issue #412 — web-runtime parity for the conversational numeric-list
// follow-up. After a first turn establishes a coding context
// ("…отсортируй их в JavaScript, дай мне код и результат"), a bare follow-up
// ("Отсортируй 4, 3, 1, 17, 8, 9, 15") names no language and may not say
// "code". The worker's `tryNumericList(prompt, history)` must recover both
// from the conversation, exactly like the Rust
// numeric_list_history_context / try_numeric_list_with_history fix.
//
// Run: `node experiments/issue-412-js-numeric-list-coreference.mjs`

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

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}

const jsSortContext = [
  {
    role: "user",
    content:
      "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
  },
  {
    role: "assistant",
    content:
      "Вот код на JavaScript:\n\n```javascript\nconst numbers = [3, 5, 6, 7, 8];\nconst sorted = [...numbers].sort((a, b) => a - b);\nconsole.log(sorted.join(\", \"));\n```\n\nРезультат: 3, 5, 6, 7, 8",
  },
];

// 1. The exact reported follow-up: inherits JavaScript + the code request.
{
  const hit = sandbox.tryNumericList("Отсортируй 4, 3, 1, 17, 8, 9, 15", jsSortContext);
  check("bare follow-up routes to write_program", hit && hit.intent === "write_program", hit && hit.intent);
  if (hit) {
    check("bare follow-up inherits javascript fence", hit.content.includes("```javascript"));
    check("bare follow-up keeps given order", hit.content.includes("const numbers = [4, 3, 1, 17, 8, 9, 15];"));
    check("bare follow-up ascending comparator", hit.content.includes(".sort((a, b) => a - b)"));
    check("bare follow-up sorted result", hit.content.includes("Результат: 1, 3, 4, 8, 9, 15, 17"));
    check(
      "bare follow-up records coreference",
      hit.evidence.some((line) => line.includes("numeric_list_coreference inherited_language=javascript")),
      "evidence",
    );
  }
}

// 2. Without context, the same bare prompt must NOT fabricate a language.
check(
  "bare prompt without context → not claimed",
  sandbox.tryNumericList("Отсортируй 4, 3, 1, 17, 8, 9, 15", []) === null,
  "should defer",
);
check(
  "bare prompt with no history arg → not claimed",
  sandbox.tryNumericList("Отсортируй 4, 3, 1, 17, 8, 9, 15") === null,
  "should defer",
);

// 3. A reduction follow-up inherits the code request even without saying "code".
{
  const hit = sandbox.tryNumericList("Теперь просуммируй 2, 4, 6", jsSortContext);
  check("reduction follow-up routes to write_program", hit && hit.intent === "write_program", hit && hit.intent);
  if (hit) {
    check("reduction follow-up inherits javascript", hit.content.includes("```javascript"));
    check("reduction follow-up computes sum", hit.content.includes("Результат: 12"));
  }
}

// 4. English parity.
{
  const history = [
    {
      role: "user",
      content: "I have numbers 3, 5, 6, 7, 8 — sort them in Python, give me the code and the result",
    },
    { role: "assistant", content: "Result: 3, 5, 6, 7, 8" },
  ];
  const hit = sandbox.tryNumericList("Sort 9, 2, 7, 1", history);
  check("english follow-up routes to write_program", hit && hit.intent === "write_program", hit && hit.intent);
  if (hit) {
    check("english follow-up inherits python", hit.content.includes("```python"));
    check("english follow-up sorted result", hit.content.includes("Result: 1, 2, 7, 9"));
  }
}

// 5. Unrelated prior chatter must not leak a language.
{
  const history = [
    { role: "user", content: "What is the capital of France?" },
    { role: "assistant", content: "Paris." },
  ];
  check(
    "unrelated context → not claimed",
    sandbox.tryNumericList("Отсортируй 4, 3, 1, 17, 8, 9, 15", history) === null,
    "should defer",
  );
}

console.log(`\n${fail.length === 0 ? "ALL PASS" : "FAILURES: " + fail.join(", ")}`);
process.exit(fail.length === 0 ? 0 : 1);
