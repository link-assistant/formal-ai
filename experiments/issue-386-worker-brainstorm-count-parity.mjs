// Issue #386: functional parity check for the seed-driven brainstorm-count
// recogniser in the hand-written browser worker. Loads src/web/formal_ai_worker.js
// in a vm sandbox and exercises requestedBrainstormCount / cardinalValue /
// findMeaning against the same prompts pinned in
// tests/unit/specification/prompt_variations.rs (BRAINSTORMING_PROMPTS) and the
// english/russian word cases. Confirms the worker reads the count from the
// `ten` cardinal's numeral surface instead of a hardcoded tenHints table.
//
// Run with: node experiments/issue-386-worker-brainstorm-count-parity.mjs

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

const { requestedBrainstormCount, cardinalValue, findMeaning } = sandbox;
for (const [name, fn] of [
  ["requestedBrainstormCount", requestedBrainstormCount],
  ["cardinalValue", cardinalValue],
  ["findMeaning", findMeaning],
]) {
  if (typeof fn !== "function") {
    throw new Error(`${name} not exposed by worker sandbox`);
  }
}

let failures = 0;
function check(label, got, want) {
  if (got === want) {
    console.log(`ok   ${label} -> ${JSON.stringify(got)}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label} -> got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);
  }
}

// The `ten` cardinal must resolve and expose its numeral surface so the count
// is read from data, not a literal.
const ten = findMeaning("ten");
check("findMeaning('ten') resolves", Boolean(ten), true);
check("cardinalValue(ten) reads numeral surface", cardinalValue(ten), 10);
check("cardinalValue('cardinal_number' genus has no numeral)", cardinalValue(findMeaning("cardinal_number")), null);

// Prompts pinned in prompt_variations.rs::BRAINSTORMING_PROMPTS, plus the
// english/russian spelled-word cases. normalized in the worker is lowercased.
const cases = [
  ["give me five ideas for an open-source side project.", 5],
  ["brainstorm ten names for a code review tool.", 10],
  ["suggest five open-source utilities for developers.", 5],
  ["brainstorm 5 small tools for link notation.", 5],
  ["give me 5 ideas for a local-first ai helper.", 5],
  ["brainstorm ten names for a symbolic assistant.", 10],
  // multilingual cardinal "ten" evidence
  ["придумай десять идей для проекта", 10],
  ["दस नाम सुझाओ", 10],
  ["给我十个想法", 10],
  // explicit numeral 10 mid-sentence
  ["give me 10 ideas for a tool", 10],
  // substring guard: "often" must not match the cardinal "ten"
  ["brainstorm ideas i often use", 5],
];
for (const [prompt, want] of cases) {
  check(`requestedBrainstormCount(${JSON.stringify(prompt)})`, requestedBrainstormCount(prompt), want);
}

if (failures > 0) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nALL BRAINSTORM-COUNT PARITY CHECKS PASSED");
