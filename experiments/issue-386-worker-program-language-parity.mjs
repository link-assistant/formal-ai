// Issue #386: verify the worker's programLanguageFromPrompt still behaves
// identically after sourcing the "in"/"на" preposition and "language"/"языке"
// noun surfaces from the seed lexicon (roles implementation_language_preposition
// / implementation_language_noun) instead of hardcoded literals. The fallback
// path only fires for languages NOT in the WRITE_PROGRAM_LANGUAGES catalog.
import { readFileSync } from "node:fs";
import vm from "node:vm";
const src = readFileSync("src/web/formal_ai_worker.js", "utf8");
const sandbox = { self:{location:{search:""}}, importScripts:()=>{throw new Error("no")}, postMessage:()=>{}, console, TextEncoder, TextDecoder, WebAssembly, fetch:()=>Promise.reject(new Error("offline")), setTimeout, clearTimeout };
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename:"formal_ai_worker.js" });
const f = sandbox.programLanguageFromPrompt;
const cases = [
  // [normalized input (already lowercased by the engine), expected]
  ["hello world in elvish", "elvish"],            // English "in" + unknown name
  ["напиши программу на эльфийском", "эльфийском"], // Russian "на" + unknown name
  ["program in language brainfuck", "brainfuck"],  // English "in" + "language" skip
  ["программу на языке brainfuck", "brainfuck"],   // Russian "на" + "языке" skip
  ["write me hello world in rust", "rust"],        // known via catalog alias scan
  ["хелло ворлд на питоне", "python"],             // known Russian alias via catalog
  ["just some unrelated text", null],              // no marker -> null
  ["in language", null],                           // marker+noun with no trailing name
];
let fail = 0;
for (const [input, want] of cases) {
  const got = f(input);
  const ok = String(got) === String(want);
  if (!ok) fail++;
  console.log(`${ok?"ok  ":"FAIL"} ${JSON.stringify(input)} -> ${JSON.stringify(got)} (want ${JSON.stringify(want)})`);
}
console.log(fail ? `\n${fail} FAILED` : "\nALL PASS");
process.exit(fail?1:0);
