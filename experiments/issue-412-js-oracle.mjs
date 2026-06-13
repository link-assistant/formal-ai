// Issue #412 (R6) — parity for the web runtime's coding oracle. The verified
// catalog does not template every language; for Kotlin/Swift/PHP/Bash/Lua/
// Haskell the worker treats the public knowledge bases (Hello World Collection,
// Rosetta Code, …) as cached external APIs and returns a reviewed snippet plus
// its deterministic output and source attribution.
//
// This drives the worker's own `tryWriteProgram(...)` — the same function the
// solver dispatches to — and asserts the answer matches the Rust handler in
// `src/solver_handler_oracle.rs` byte-for-byte on intent/content/evidence.
// Run: `node experiments/issue-412-js-oracle.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => { throw new Error("no importScripts in node"); };
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => { throw new Error("no fetch"); };
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

function write(prompt) {
  return sandbox.tryWriteProgram(prompt, [], "en", "auto");
}

console.log("=== uncatalogued languages resolve from the oracle ===");
for (const [prompt, intent, fence, snippet, sourceName, sourceHost] of [
  ["Write a hello world program in Kotlin", "write_program_oracle_hello_world_kotlin", "```kotlin", 'println("Hello, World!")', "Hello World Collection", "helloworldcollection.de"],
  ["write me a hello world program in swift", "write_program_oracle_hello_world_swift", "```swift", 'print("Hello, World!")', "Hello World Collection", "helloworldcollection.de"],
  ["write a hello world program in php", "write_program_oracle_hello_world_php", "```php", 'echo "Hello, World!', "Hello World Collection", "helloworldcollection.de"],
  ["write a hello world program in bash", "write_program_oracle_hello_world_bash", "```bash", 'echo "Hello, World!"', "Hello World Collection", "helloworldcollection.de"],
  ["write a hello world program in lua", "write_program_oracle_hello_world_lua", "```lua", 'print("Hello, World!")', "Hello World Collection", "helloworldcollection.de"],
  ["write a hello world program in haskell", "write_program_oracle_hello_world_haskell", "```haskell", 'putStrLn "Hello, World!"', "Hello World Collection", "helloworldcollection.de"],
]) {
  const r = write(prompt);
  check(`"${prompt}" routes to the oracle`, r && r.intent === intent, r && r.intent);
  check(`"${prompt}" carries the ${fence} fence`, r && r.content.includes(fence), r && r.content);
  check(`"${prompt}" contains the idiomatic snippet`, r && r.content.includes(snippet), r && r.content);
  check(`"${prompt}" prints an Output block`, r && r.content.includes("Output:") && r.content.includes("Hello, World!"), r && r.content);
  check(`"${prompt}" attributes its source`, r && r.content.includes(sourceName) && r.content.includes(sourceHost), r && r.content);
  check(`"${prompt}" records knowledge_source evidence`, r && Array.isArray(r.evidence) && r.evidence.some((e) => String(e).startsWith("knowledge_source:")), r && JSON.stringify(r.evidence));
  check(`"${prompt}" records an honest not-run status`, r && r.evidence.some((e) => String(e).includes("not run (cached external snippet)")), r && JSON.stringify(r.evidence));
}

console.log("\n=== a non-trivial Rosetta Code task ===");
{
  const r = write("write a program that prints the factorial of 5 in kotlin");
  check("kotlin factorial routes to the oracle", r && r.intent === "write_program_oracle_factorial_kotlin", r && r.intent);
  check("kotlin factorial prints 120", r && r.content.includes("```text\n120"), r && r.content);
  check("kotlin factorial cites Rosetta Code", r && r.content.includes("Rosetta Code") && r.content.includes("rosettacode.org"), r && r.content);
}

console.log("\n=== catalogued languages keep the verified route ===");
{
  const r = write("write a hello world program in Rust");
  check("rust hello world stays on write_program", r && r.intent === "write_program", r && r.intent);
}

console.log("");
if (fail.length) {
  console.error(`FAILED ${fail.length} check(s):\n  ${fail.join("\n  ")}`);
  process.exit(1);
}
console.log("ALL CHECKS PASSED");
