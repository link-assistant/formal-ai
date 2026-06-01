// Issue #361: verify the browser worker mirror handles the issue #349
// reverse-sort follow-up the same way as the Rust core. This harness exercises
// the pure-JS fallback path directly, including the synthesized unknown-path
// trace that the UI diagnostics panel renders.
//
// Run with: node experiments/issue-361-cross-runtime-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const FIRST_PROMPT =
  "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT = "Сделай сортировку результатов в обратном порядке";

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

function answerReversesSort(answer) {
  const compact = String(answer || "").toLowerCase().split(/\s+/).join("");
  return (
    compact.includes("names.sort_by(") ||
    compact.includes(".sort().reverse()") ||
    compact.includes("reverse=true") ||
    compact.includes("sort.reverse")
  );
}

const { tryWriteProgram, solve } = sandbox;

const first = tryWriteProgram(FIRST_PROMPT, [], "ru");
check(
  "turn 1 routes to write_program",
  first && first.intent === "write_program",
  first && first.intent,
);
check(
  "turn 1 resolves rust/list_files",
  first &&
    first.evidence.includes("program_parameter:language:rust") &&
    first.evidence.includes("program_parameter:task:list_files"),
  first && JSON.stringify(first.evidence),
);

const pathHistory = [
  { role: "user", content: FIRST_PROMPT },
  {
    role: "assistant",
    content: first.content,
    intent: first.intent,
    evidence: first.evidence,
  },
];
const pathArgument = tryWriteProgram(PATH_ARGUMENT_PROMPT, pathHistory, "ru");
check(
  "turn 3 path-argument follow-up routes to write_program",
  pathArgument && pathArgument.intent === "write_program",
  pathArgument && pathArgument.intent,
);
check(
  "turn 3 resolves list_files_arg",
  pathArgument && pathArgument.evidence.includes("program_parameter:task:list_files_arg"),
  pathArgument && JSON.stringify(pathArgument.evidence),
);

const fullHistory = [
  ...pathHistory,
  { role: "user", content: PATH_ARGUMENT_PROMPT },
  {
    role: "assistant",
    content: pathArgument.content,
    intent: pathArgument.intent,
    evidence: pathArgument.evidence,
  },
];
const reverseSort = tryWriteProgram(REVERSE_SORT_PROMPT, fullHistory, "ru");
check(
  "turn 5 reverse-sort follow-up is not unknown",
  reverseSort && reverseSort.intent === "write_program",
  reverseSort && reverseSort.intent,
);
check(
  "turn 5 resolves rust/list_files_arg_reverse_sort",
  reverseSort &&
    reverseSort.evidence.includes("program_parameter:language:rust") &&
    reverseSort.evidence.includes("program_parameter:task:list_files_arg_reverse_sort"),
  reverseSort && JSON.stringify(reverseSort.evidence),
);
check(
  "turn 5 answer sorts file names in reverse order",
  answerReversesSort(reverseSort && reverseSort.content),
  reverseSort && reverseSort.content,
);
check(
  "turn 5 exposes coreference evidence",
  reverseSort &&
    reverseSort.evidence.some((item) =>
      item.startsWith("write_program_coreference_rewrite:"),
    ),
  reverseSort && JSON.stringify(reverseSort.evidence),
);

const reverseTrace = (reverseSort && reverseSort.trace) || [];
for (const expected of [
  "selected_rule:selected_rule initial unknown reason no_seed_route next try_rule_synthesis",
  "write_program_coreference_rewrite:referent=active_program_artifact task=list_files_arg language=rust",
  "rule_synthesis_operation_vocabulary:reverse_sort",
  "rule_synthesis_request:rule_synthesis_request",
  "rule_synthesis_candidate:rule_synthesis_candidate",
  "rule_verification:rule_verification",
  "write_program_plan:program_plan",
]) {
  check(
    `turn 5 trace includes ${expected.split(":")[0]}`,
    reverseTrace.some((line) => String(line).includes(expected)),
    reverseTrace.join("\n"),
  );
}

const solved = await solve(REVERSE_SORT_PROMPT, fullHistory, {
  diagnosticsMode: true,
  greetingVariations: false,
});
check("solve() turn 5 intent is write_program", solved.intent === "write_program", solved.intent);
check(
  "solve() turn 5 keeps reverse-sort output",
  answerReversesSort(solved.content),
  solved.content,
);
for (const expectedStep of [
  "route_attempt",
  "coreference_binding",
  "modifier_detection",
  "rule_construction",
  "rule_verification",
  "program_plan",
  "deformalize",
]) {
  check(
    `solve() diagnostics include ${expectedStep}`,
    solved.steps.some((step) => step && step.step === expectedStep),
    JSON.stringify(solved.steps),
  );
}

const standalone = tryWriteProgram("Sort the results in reverse order", [], "en");
check(
  "standalone reverse-sort prompt does not invent a write_program target",
  standalone === null,
  standalone && JSON.stringify(standalone),
);

if (failures > 0) {
  console.error(`\n${failures} failure(s).`);
  process.exit(1);
}

console.log("\nIssue #361 cross-runtime parity checks passed.");
