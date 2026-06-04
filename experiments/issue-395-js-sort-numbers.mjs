// Issue #395 — web-runtime parity for the sort-numbers coding task.
//
// Loads `src/web/formal_ai_worker.js` in a node VM sandbox (the same harness as
// the other parity experiments) and replays the five reproduction prompts from
// `examples/repro_issue_395.rs`, asserting that `trySortNumbers` routes them to
// `write_program` with the same generated code and the same deterministically-
// computed result the Rust solver produces.
//
// Run: `node experiments/issue-395-js-sort-numbers.mjs`

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

const cases = [
  {
    label: "Russian / JavaScript",
    prompt:
      "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
    fence: "javascript",
    result: "3, 5, 6, 7, 8",
    intro: "Вот код на JavaScript",
    resultLabel: "Результат:",
    codeIncludes: "const sorted = [...numbers].sort((a, b) => a - b);",
  },
  {
    label: "English / JavaScript",
    prompt:
      "I have numbers 5, 3, 8, 1, 9 — sort them in JavaScript, give me the code and the result",
    fence: "javascript",
    result: "1, 3, 5, 8, 9",
    intro: "Here is JavaScript code",
    resultLabel: "Result:",
    codeIncludes: "const sorted = [...numbers].sort((a, b) => a - b);",
  },
  {
    label: "English / Python (descending)",
    prompt:
      "Sort the numbers 4, 2, 7, 1 in descending order in Python and show me the code and result",
    fence: "python",
    result: "7, 4, 2, 1",
    intro: "Here is Python code",
    resultLabel: "Result:",
    codeIncludes: "sorted(numbers, reverse=True)",
  },
  {
    label: "Hindi / JavaScript",
    prompt:
      "मेरे पास संख्याएं 3, 5, 6, 7, 8 हैं, उन्हें JavaScript में क्रमबद्ध करो और मुझे कोड और परिणाम दो",
    fence: "javascript",
    result: "3, 5, 6, 7, 8",
    resultLabel: "परिणाम:",
    codeIncludes: "const sorted = [...numbers].sort((a, b) => a - b);",
  },
  {
    label: "Chinese / Python",
    prompt: "我有数字 3, 5, 6, 7, 8，用 Python 排序，给我代码和结果",
    fence: "python",
    result: "3, 5, 6, 7, 8",
    resultLabel: "结果:",
    codeIncludes: "sorted_numbers = sorted(numbers)",
  },
];

for (const c of cases) {
  const hit = sandbox.trySortNumbers(c.prompt);
  check(`${c.label}: routes to write_program`, hit && hit.intent === "write_program", hit && hit.intent);
  if (!hit) continue;
  check(`${c.label}: code fence ${c.fence}`, hit.content.includes("```" + c.fence), "fence");
  check(`${c.label}: contains generated code`, hit.content.includes(c.codeIncludes), c.codeIncludes);
  check(`${c.label}: shows result ${c.result}`, hit.content.includes(`${c.resultLabel} ${c.result}`), `${c.resultLabel} ${c.result}`);
  if (c.intro) {
    check(`${c.label}: localized intro`, hit.content.includes(c.intro), c.intro);
  }
}

// Negative: a plain arithmetic prompt with no language must NOT be claimed.
check(
  "no language → not claimed",
  sandbox.trySortNumbers("sort 3, 1, 2") === null,
  "should defer",
);
// Negative: a single number is not a sort task.
check(
  "single number → not claimed",
  sandbox.trySortNumbers("sort 3 in JavaScript") === null,
  "should defer",
);

console.log(`\n${fail.length === 0 ? "ALL PASS" : "FAILURES: " + fail.join(", ")}`);
process.exit(fail.length === 0 ? 0 : 1);
