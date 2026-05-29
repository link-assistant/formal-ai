// Load the worker in a stubbed `self` context and exercise tryWriteProgram for
// the new tasks, printing the full localized answer (issue #330 R9 parity).
const fs = require("fs");
const vm = require("vm");
const path = require("path");

const code = fs.readFileSync(
  path.join(__dirname, "../../src/web/formal_ai_worker.js"),
  "utf8",
);

const sandbox = {
  self: {
    location: { search: "", origin: "http://localhost" },
    postMessage() {},
    set onmessage(_) {},
  },
  console,
  setTimeout,
  clearTimeout,
  TextEncoder,
  TextDecoder,
  URL,
  URLSearchParams,
};
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(code + "\nthis.__tryWriteProgram = tryWriteProgram;", sandbox, {
  filename: "formal_ai_worker.js",
});

const tryWriteProgram = sandbox.__tryWriteProgram;

const cases = [
  ["fizzbuzz in rust", [], "en"],
  ["напиши факториал 5 на python", [], "ru"],
  ["1 से 10 तक का योग को go में लिखें", [], "hi"],
  ["用 java 写 反转字符串", [], "zh"],
];

for (const [prompt, history, lang] of cases) {
  console.log("\n========== prompt:", prompt, "| lang:", lang, "==========");
  const r = tryWriteProgram(prompt, history, lang);
  if (!r) {
    console.log("(no result)");
    continue;
  }
  console.log("intent:", r.intent);
  console.log(r.content);
}

// Follow-up edit (prior code present): should give concise "test it the same way"
console.log("\n========== follow-up edit (prior code) ==========");
const followup = tryWriteProgram(
  "make it count to three in rust",
  [{ role: "assistant", content: "Here is a program:\n```rust\nfn main(){}\n```" }],
  "en",
);
console.log(followup ? followup.content : "(no result)");
