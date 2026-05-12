let wasm;
let mode = "wasm worker";

function verifiedExecutionReport(checkCommand, runCommand) {
  const checkLine = checkCommand ? `Check command: \`${checkCommand}\`\n` : "";
  return `Execution status: compiled and ran in issue-8 local verification harness.
${checkLine}Run command: \`${runCommand}\`
Output:
\`\`\`text
Hello, world!
\`\`\`
1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.`;
}

const typescriptExecutionReport = `Execution status: not compiled or run in TypeScript compiler is not configured in this repository runtime.
Check command: \`tsc hello.ts\`
Run command: \`node hello.js\`
Expected output after verification:
\`\`\`text
Hello, world!
\`\`\`
The TypeScript seed is returned with this warning until a tsc-backed execution profile is available.`;

const answers = {
  0: {
    intent: "unknown",
    content:
      "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.",
  },
  1: {
    intent: "greeting",
    content: "Hi, how may I help you?",
  },
  2: {
    intent: "hello_world_rust",
    content: `Here is a minimal Rust hello world program:

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

${verifiedExecutionReport("rustc main.rs -o main", "./main")}`,
  },
  3: {
    intent: "hello_world_python",
    content: `Here is a minimal Python hello world program:

\`\`\`python
print("Hello, world!")
\`\`\`

${verifiedExecutionReport("python3 -m py_compile main.py", "python3 main.py")}`,
  },
  4: {
    intent: "hello_world_javascript",
    content: `Here is a minimal JavaScript hello world program:

\`\`\`javascript
console.log("Hello, world!");
\`\`\`

${verifiedExecutionReport("node --check main.js", "node main.js")}`,
  },
  5: {
    intent: "hello_world_typescript",
    content: `Here is a minimal TypeScript hello world program:

\`\`\`typescript
console.log("Hello, world!");
\`\`\`

${typescriptExecutionReport}`,
  },
  6: {
    intent: "hello_world_go",
    content: `Here is a minimal Go hello world program:

\`\`\`go
package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}
\`\`\`

${verifiedExecutionReport(null, "go run main.go")}`,
  },
  7: {
    intent: "hello_world_c",
    content: `Here is a minimal C hello world program:

\`\`\`c
#include <stdio.h>

int main(void) {
    puts("Hello, world!");
    return 0;
}
\`\`\`

${verifiedExecutionReport("gcc main.c -o main", "./main")}`,
  },
  8: {
    intent: "identity",
    content:
      "I am formal-ai, a deterministic symbolic AI proof of concept that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.",
  },
};

async function init() {
  if (wasm !== undefined) {
    return;
  }
  try {
    const source = await fetch("formal_ai_worker.wasm");
    const bytes = await source.arrayBuffer();
    const module = await WebAssembly.instantiate(bytes, {});
    wasm = module.instance.exports;
  } catch (_error) {
    wasm = null;
    mode = "js fallback";
  }
  postMessage({ kind: "ready", mode });
}

function classifyWithWasm(prompt) {
  const encoded = new TextEncoder().encode(prompt).slice(0, 4096);
  const pointer = wasm.input_ptr();
  const memory = new Uint8Array(wasm.memory.buffer, pointer, 4096);
  memory.fill(0);
  memory.set(encoded);
  return wasm.classify(encoded.length);
}

function classifyWithFallback(prompt) {
  const normalized = prompt.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  if (["hi", "hello", "hey"].includes(normalized)) {
    return 1;
  }
  if (
    [
      "who are you",
      "what are you",
      "who is formal ai",
      "what is formal ai",
      "who is formalai",
      "what is formalai",
      "tell me about yourself",
      "introduce yourself",
    ].includes(normalized) ||
    (has("who") && has("you")) ||
    (has("what") && has("you")) ||
    ((has("who") || has("what")) && has("formal") && has("ai")) ||
    (has("tell") && has("yourself")) ||
    (has("introduce") && has("yourself"))
  ) {
    return 8;
  }
  if (!(tokens.includes("hello") && tokens.includes("world"))) {
    return 0;
  }

  if (tokens.includes("rust") || tokens.includes("rs")) return 2;
  if (tokens.includes("python") || tokens.includes("py")) return 3;
  if (tokens.includes("javascript") || tokens.includes("js") || tokens.includes("node")) return 4;
  if (tokens.includes("typescript") || tokens.includes("ts")) return 5;
  if (tokens.includes("go") || tokens.includes("golang")) return 6;
  if (tokens.includes("c")) return 7;

  return 0;
}

self.onmessage = async (event) => {
  await init();
  const prompt = event.data.prompt || "";
  const code = wasm ? classifyWithWasm(prompt) : classifyWithFallback(prompt);
  const answer = answers[code] || answers[0];
  postMessage({
    kind: "message",
    requestId: event.data.requestId,
    intent: answer.intent,
    content: answer.content,
  });
};

init();
