import { readFileSync } from "node:fs";
import vm from "node:vm";
const src = readFileSync("src/web/formal_ai_worker.js", "utf8");
const sandbox = { self:{location:{search:""}}, importScripts:()=>{throw new Error("no")}, postMessage:()=>{}, console, TextEncoder, TextDecoder, WebAssembly, fetch:()=>Promise.reject(new Error("offline")), setTimeout, clearTimeout };
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename:"formal_ai_worker.js" });
const f = sandbox.evaluatePercentOfExpression;
const cases = [
  // [input, expected]
  ["8% of $50", "4 USD"],            // symbol path (Playwright-tested via wrapper)
  ["8% of 500", "40"],               // no currency
  ["20% of 500 usd", "100 USD"],     // English ISO word path (preserved)
  ["20% of 500 dollars", "100 USD"], // English plural (preserved)
  ["20% of 500 dollar", "100 USD"],  // English singular (preserved)
  ["10% of 200 euros", "20 EUR"],    // euro plural (preserved)
  ["10% of 200 eur", "20 EUR"],      // euro ISO (preserved)
  ["50% of 80 rubles", "40 RUB"],    // ruble plural (preserved)
  ["50% of 80 rub", "40 RUB"],       // ruble ISO (preserved)
  ["10% of 200 рублей", "20 RUB"],   // NEW Cyrillic surface (seed-derived)
  ["10% of 200 美元", "20 USD"],      // NEW CJK surface (seed-derived)
  ["10% of 200 rubbish", null],      // must NOT match a non-currency word
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
