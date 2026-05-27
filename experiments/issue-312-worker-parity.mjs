// Issue #312: verify the pure-JS demo worker (`src/web/formal_ai_worker.js`)
// routes the Russian "list files" prompt to `write_program`, produces the Rust
// `read_dir` snippet, and stays honest about not running the filesystem code in
// the browser sandbox. This exercises the JS fallback path (no WASM loaded),
// which is exactly what the GitHub Pages worker uses before/without the wasm
// module.
//
// Run with: node experiments/issue-312-worker-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

// Minimal Web Worker-ish globals. `importScripts` throws so the seed loader is
// skipped (its try/catch swallows the error) and the hard-coded fallbacks load.
const sandbox = {
  self: { location: { search: "" } },
  importScripts: () => {
    throw new Error("no importScripts in node harness");
  },
  // The worker calls `init()` at load time and posts a ready message; swallow it.
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

const { tryWriteProgram, writeProgramParameters } = sandbox;

// 1. The exact issue prompt (Russian).
const issuePrompt =
  "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const params = writeProgramParameters(issuePrompt);
check(
  "Russian prompt parses to rust/list_files",
  params && params.language === "rust" && params.task === "list_files",
  JSON.stringify(params),
);
const hit = tryWriteProgram(issuePrompt);
check("Russian prompt is write_program (not unknown)", hit && hit.intent === "write_program", hit && hit.intent);
check("Rust snippet uses read_dir", hit && hit.content.includes("read_dir"));
check("Rust snippet is fenced as ```rust", hit && hit.content.includes("```rust"));
check(
  "list_files rust reported as not run (no fs in sandbox)",
  hit && hit.content.includes("Execution status: not run"),
);
check(
  "documented sample output is shown",
  hit && hit.content.includes("Cargo.toml\nREADME.md\nmain.rs"),
);
check(
  "execution_status evidence marks rust unavailable",
  hit && hit.evidence.includes("execution_status:rust:unavailable"),
);

// 2. English list_files in python.
const py = tryWriteProgram("Write a program in Python that lists files in the current directory");
check("English python list_files routes", py && py.intent === "write_program" && py.content.includes("listdir"));

// 2b. Hindi list_files in rust (Devanagari uses spaces; token/phrase matching).
const hiParams = writeProgramParameters("Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो");
check(
  "Hindi prompt parses to rust/list_files",
  hiParams && hiParams.language === "rust" && hiParams.task === "list_files",
  JSON.stringify(hiParams),
);
const hi = tryWriteProgram("Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो");
check(
  "Hindi rust list_files is write_program with read_dir",
  hi && hi.intent === "write_program" && hi.content.includes("read_dir"),
  hi && hi.intent,
);

// 2c. Chinese list_files in rust (CJK has no spaces; substring matching).
const zhParams = writeProgramParameters("用 Rust 编写一个列出当前目录中文件的程序");
check(
  "Chinese prompt parses to rust/list_files",
  zhParams && zhParams.language === "rust" && zhParams.task === "list_files",
  JSON.stringify(zhParams),
);
const zh = tryWriteProgram("用 Rust 编写一个列出当前目录中文件的程序");
check(
  "Chinese rust list_files is write_program with read_dir",
  zh && zh.intent === "write_program" && zh.content.includes("read_dir"),
  zh && zh.intent,
);

// 3. list_files JavaScript stays honest (needs Node fs, cannot run in sandbox).
const js = tryWriteProgram("Write a JavaScript program that lists files");
check(
  "JS list_files not run in sandbox",
  js && js.intent === "write_program" && js.content.includes("Execution status: not run"),
  js && js.content,
);
check(
  "JS list_files evidence is unavailable",
  js && js.evidence.includes("execution_status:javascript:unavailable"),
);

// 4. Regression: self-contained JS hello world still runs in the sandbox.
const jsHello = tryWriteProgram("Write a JavaScript program that prints hello world");
check(
  "JS hello world still runs in sandbox",
  jsHello && jsHello.content.includes("Execution status: ran") && jsHello.evidence.includes("execution_status:javascript:ran"),
  jsHello && jsHello.content,
);

// 5. Russian unsupported task still reports a helpful unsupported message.
const unsupported = tryWriteProgram(
  "Напиши программу на Python, которая вычисляет факториал числа",
);
check(
  "Russian unknown task is unsupported (not null/unknown)",
  unsupported && unsupported.intent === "write_program_unsupported",
  unsupported && unsupported.intent,
);

// 6. A plain language word is not hijacked as a program request.
const plain = writeProgramParameters("rust");
check("bare language word is not a program request", plain === null, JSON.stringify(plain));

if (failures > 0) {
  console.error(`\n${failures} failure(s).`);
  process.exit(1);
}
console.log("\nIssue #312 worker parity checks passed.");
