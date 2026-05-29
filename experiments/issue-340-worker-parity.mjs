// Issue #340: verify the pure-JS demo worker (`src/web/formal_ai_worker.js`)
// answers a composite `write_program` request the verified template catalog
// cannot resolve (HTTP GET -> parse JSON -> mean/median) with a *blueprint*
// instead of dead-ending at `write_program_unsupported`, and that the rendered
// program/plan/honest-execution report mirrors the Rust core
// (`src/coding/blueprint.rs`). Exercises the JS fallback path (no WASM loaded),
// which is what the GitHub Pages worker uses before/without the wasm module.
//
// Run with: node experiments/issue-340-worker-parity.mjs

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

const { tryWriteProgram, selectBlueprint, renderBlueprint } = sandbox;

// 1. The exact issue #340 prompt (English, Rust).
const issuePrompt =
  "Write a Rust program that makes an HTTP GET request, parses the JSON " +
  "response, calculates the mean and median, and outputs the results, with " +
  "error handling and comments.";
const hit = tryWriteProgram(issuePrompt, [], "en");
check(
  "issue prompt is write_program (not unsupported)",
  hit && hit.intent === "write_program",
  hit && hit.intent,
);
check("blueprint embeds a fenced rust block", hit && hit.content.includes("```rust"));
check("blueprint embeds fn main()", hit && hit.content.includes("fn main()"));
check(
  "blueprint uses reqwest::blocking::get",
  hit && hit.content.includes("reqwest::blocking::get"),
);
check(
  "blueprint computes mean and median",
  hit && hit.content.includes("fn mean(") && hit.content.includes("fn median("),
);
check(
  "blueprint lists library prerequisites",
  hit && hit.content.includes("Required libraries:") && hit.content.includes("serde_json"),
);
check(
  "blueprint is honest: not run",
  hit && hit.content.includes("not run"),
);
check(
  "blueprint never claims it ran/compiled",
  hit && !/compiled and ran/i.test(hit.content),
);
check(
  "blueprint response evidence link is present",
  hit && hit.evidence.includes("response:write_program:blueprint:http_json_stats:rust"),
  hit && JSON.stringify(hit.evidence),
);
check(
  "blueprint records the recipe in evidence",
  hit && hit.evidence.includes("program_blueprint:recipe:http_json_stats"),
);
check(
  "blueprint execution status is unavailable",
  hit && hit.evidence.includes("execution_status:rust:unavailable"),
);

// 2. Python and JavaScript variants resolve too.
const py = tryWriteProgram(
  "Write a Python program that makes an HTTP GET request, parses the JSON, and computes the mean and median.",
  [],
  "en",
);
check(
  "python composite request routes to blueprint",
  py && py.intent === "write_program" && py.content.includes("```python") && py.content.includes("import requests"),
  py && py.intent,
);
const js = tryWriteProgram(
  "Write a JavaScript program that fetches JSON over HTTP and reports the mean and median.",
  [],
  "en",
);
check(
  "javascript composite request routes to blueprint",
  js && js.intent === "write_program" && js.content.includes("```javascript") && js.content.includes("await fetch("),
  js && js.intent,
);

// 3. Russian request is answered in Russian.
const ru = tryWriteProgram(
  "Напиши программу на Rust, которая делает HTTP запрос, разбирает JSON и считает среднее и медиану.",
  [],
  "ru",
);
check(
  "russian composite request routes to blueprint in russian",
  ru && ru.intent === "write_program" && ru.content.includes("Статус выполнения") && ru.content.includes("```rust"),
  ru && (ru.intent + " :: " + (ru.content || "").slice(0, 120)),
);

// 4. A partial request (http + json but NO statistics) stays unsupported —
//    the recipe's required capabilities are not all present.
const partial = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request and parses the JSON response.",
  [],
  "en",
);
check(
  "partial composite (no statistics) stays unsupported",
  partial && partial.intent === "write_program_unsupported",
  partial && partial.intent,
);

// 5. An unsupported language for the recipe (Go) stays unsupported.
const go = tryWriteProgram(
  "Write a Go program that makes an HTTP GET request, parses JSON, and computes the mean and median.",
  [],
  "en",
);
check(
  "go composite request stays unsupported (no curated go program)",
  go && go.intent === "write_program_unsupported",
  go && go.intent,
);

// 6. Cross-engine parity: the JS render must byte-match the Rust core. We assert
//    structural anchors here; the Rust test
//    `render_contains_plan_code_libraries_and_honest_execution` asserts the same
//    anchors on the Rust side, and the curated programs are verbatim copies.
const bp = selectBlueprint(
  "http get request parse json calculate mean median statistics",
  "rust",
);
check("selectBlueprint resolves rust http_json_stats", bp && bp.recipe.slug === "http_json_stats");
const rendered = renderBlueprint(bp, "en");
check("render numbers the decomposition plan", rendered.includes("1. Make an HTTP request"));
check("render localizes nothing in en intro", rendered.startsWith("Here is a Rust program"));

// 7. Compositional `comments` axis: a request that asks for comments keeps the
//    documented program; one that does not strips whole-line documentation, so
//    the synthesized program is a projection of the decomposition (not a frozen
//    string). Mirrors the Rust unit tests in `src/coding/blueprint.rs`.
const withComments = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request, parses JSON, computes mean and median, outputs the results, with comments.",
  [],
  "en",
);
check(
  "comments requested keeps the documented program",
  withComments && withComments.content.includes("// 1. Read the target URL"),
);
const noComments = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request, parses JSON, computes mean and median, and outputs the results.",
  [],
  "en",
);
const noCommentsCode = noComments && noComments.content.split("```rust\n")[1].split("\n```")[0];
check(
  "comments omitted strips whole-line documentation",
  noCommentsCode &&
    !noCommentsCode.split("\n").some((line) => line.trimStart().startsWith("//")),
  noCommentsCode,
);
check(
  "comments omitted keeps the core logic",
  noCommentsCode &&
    noCommentsCode.includes("reqwest::blocking::get") &&
    noCommentsCode.includes("fn median(") &&
    !/\n\n\n/.test(noCommentsCode),
);
const pyNoComments = tryWriteProgram(
  "Write a Python program that makes an HTTP GET request, parses JSON, and computes the mean and median.",
  [],
  "en",
);
const pyCode = pyNoComments && pyNoComments.content.split("```python\n")[1].split("\n```")[0];
check(
  "python comments omitted drops docstring and # lines",
  pyCode &&
    !pyCode.includes('"""') &&
    !pyCode.split("\n").some((line) => line.trimStart().startsWith("#")) &&
    pyCode.includes("requests.get"),
  pyCode,
);

if (failures > 0) {
  console.error(`\n${failures} failure(s).`);
  process.exit(1);
}
console.log("\nIssue #340 worker parity checks passed.");
