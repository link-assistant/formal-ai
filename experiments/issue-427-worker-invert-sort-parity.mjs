// Issue #427: verify the pure-JS demo worker (`src/web/formal_ai_worker.js`)
// mirrors the Rust solver: after a numeric-list sort, the bare follow-up
// "Сделай инверсию сортировки." inherits the language and the list from the
// conversation and emits descending code + result instead of `unknown`.
//
// Run with: node experiments/issue-427-worker-invert-sort-parity.mjs

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

let failures = 0;
function check(label, condition, detail) {
  if (condition) {
    console.log(`ok   ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}${detail ? `\n     ${detail}` : ""}`);
  }
}

const { tryNumericList } = sandbox;

// Russian: JavaScript context, last sorted list is 3, 1, 2.
const ruHistory = [
  {
    role: "user",
    content:
      "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
  },
  { role: "assistant", content: "Результат: 3, 5, 6, 7, 8" },
  { role: "user", content: "Отсортируй 3, 1, 2" },
  { role: "assistant", content: "Результат: 1, 2, 3" },
];
const ru = tryNumericList("Сделай инверсию сортировки.", ruHistory);
check("RU invert-sort is not null (not unknown)", Boolean(ru), JSON.stringify(ru));
check(
  "RU invert-sort fenced as ```javascript",
  ru && ru.content.includes("```javascript"),
  ru && ru.content,
);
check(
  "RU invert-sort inherits list [3, 1, 2]",
  ru && ru.content.includes("const numbers = [3, 1, 2];"),
  ru && ru.content,
);
check(
  "RU invert-sort result is descending 3, 2, 1",
  ru && ru.content.includes("Результат: 3, 2, 1"),
  ru && ru.content,
);

// English: Python context, last sorted list is 4, 1, 3.
const enHistory = [
  {
    role: "user",
    content:
      "I have numbers 4, 1, 3 — sort them in Python, give me the code and the result",
  },
  { role: "assistant", content: "Result: 1, 3, 4" },
];
const en = tryNumericList("Invert the sort.", enHistory);
check("EN invert-sort is not null", Boolean(en), JSON.stringify(en));
check(
  "EN invert-sort fenced as ```python",
  en && en.content.includes("```python"),
  en && en.content,
);
check(
  "EN invert-sort result is descending 4, 3, 1",
  en && en.content.includes("Result: 4, 3, 1"),
  en && en.content,
);

// Hindi: Python context, last sorted list is 4, 1, 3.
const hiHistory = [
  {
    role: "user",
    content:
      "मेरे पास संख्याएँ 4, 1, 3 हैं — उन्हें Python में क्रमबद्ध करो, मुझे कोड और परिणाम दो",
  },
  { role: "assistant", content: "परिणाम: 1, 3, 4" },
];
const hi = tryNumericList("क्रम उलट दो।", hiHistory);
check("HI invert-sort is not null", Boolean(hi), JSON.stringify(hi));
check(
  "HI invert-sort fenced as ```python",
  hi && hi.content.includes("```python"),
  hi && hi.content,
);
check(
  "HI invert-sort result is descending 4, 3, 1",
  hi && hi.content.includes("परिणाम: 4, 3, 1"),
  hi && hi.content,
);

// Chinese: Python context, last sorted list is 4, 1, 3.
const zhHistory = [
  { role: "user", content: "我有数字 4, 1, 3 — 用 Python 排序，给我代码和结果" },
  { role: "assistant", content: "结果: 1, 3, 4" },
];
const zh = tryNumericList("反转排序。", zhHistory);
check("ZH invert-sort is not null", Boolean(zh), JSON.stringify(zh));
check(
  "ZH invert-sort fenced as ```python",
  zh && zh.content.includes("```python"),
  zh && zh.content,
);
check(
  "ZH invert-sort result is descending 4, 3, 1",
  zh && zh.content.includes("结果: 4, 3, 1"),
  zh && zh.content,
);

// No context: must not fabricate a program.
const bare = tryNumericList("Сделай инверсию сортировки.", []);
check(
  "bare invert-sort with no context returns null (no fabrication)",
  bare === null,
  JSON.stringify(bare),
);

if (failures) {
  console.error(`\n${failures} check(s) failed`);
  process.exit(1);
}
console.log("\nall checks passed");
