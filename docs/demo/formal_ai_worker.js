let wasm;
let mode = "wasm worker";

const answers = {
  1: "Hi, how may I help you?",
  2: `Here is a minimal Rust hello world program:

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\``,
  0: "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.",
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
  const tokens = normalized.split(/\s+/);
  if (["hi", "hello", "hey"].includes(normalized)) {
    return 1;
  }
  if (tokens.includes("rust") && tokens.includes("hello") && tokens.includes("world")) {
    return 2;
  }
  return 0;
}

self.onmessage = async (event) => {
  await init();
  const prompt = event.data.prompt || "";
  const code = wasm ? classifyWithWasm(prompt) : classifyWithFallback(prompt);
  postMessage({ kind: "message", content: answers[code] || answers[0] });
};

init();
