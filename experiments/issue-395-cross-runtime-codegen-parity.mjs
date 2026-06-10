// Issue #395 — exhaustive cross-runtime parity for the seed-driven coding
// composer.
//
// `examples/numeric_list_matrix.rs` dumps the Rust engine's full answer for
// every (operation, language, value class) cell as JSON lines. This harness
// replays the identical prompts through the browser worker mirror
// (`src/web/formal_ai_worker.js`, loaded in a node VM sandbox) and requires
// the worker's answer to be byte-identical to the Rust answer — both
// runtimes compose code from the same `data/seed/coding-idioms.lino`
// knowledge base, so any divergence is a mirror bug.
//
// Run: node experiments/issue-395-cross-runtime-codegen-parity.mjs
// (spawns `cargo run --example numeric_list_matrix` unless a dump path is
// passed as the first argument)

import fs from "node:fs";
import vm from "node:vm";
import { spawnSync } from "node:child_process";
import { TextEncoder, TextDecoder } from "node:util";

const root = new URL("..", import.meta.url);

function loadMatrix() {
  if (process.argv[2]) {
    return fs.readFileSync(process.argv[2], "utf8");
  }
  const run = spawnSync(
    "cargo",
    ["run", "--quiet", "--example", "numeric_list_matrix"],
    { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (run.status !== 0) {
    console.error(run.stderr);
    throw new Error(`cargo example failed with status ${run.status}`);
  }
  return run.stdout;
}

const records = loadMatrix()
  .split("\n")
  .filter((line) => line.trim().length > 0)
  .map((line) => JSON.parse(line));

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

function firstDifference(a, b) {
  const max = Math.max(a.length, b.length);
  for (let i = 0; i < max; i += 1) {
    if (a[i] !== b[i]) {
      return `byte ${i}: rust ${JSON.stringify(a.slice(i, i + 40))} vs js ${JSON.stringify(b.slice(i, i + 40))}`;
    }
  }
  return "identical";
}

let pass = 0;
const failures = [];
for (const record of records) {
  const label = `${record.canonical}/${record.slug}/${record.class}`;
  const hit = sandbox.tryNumericList(record.prompt);
  if (!hit || hit.intent !== "write_program") {
    failures.push(`${label}: worker did not claim the prompt (${hit && hit.intent})`);
    continue;
  }
  if (hit.content === record.answer) {
    pass += 1;
  } else {
    failures.push(`${label}: ${firstDifference(record.answer, hit.content)}`);
  }
}

console.log(`${pass}/${records.length} matrix cells byte-identical across runtimes`);
if (failures.length > 0) {
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log("ALL PASS");
