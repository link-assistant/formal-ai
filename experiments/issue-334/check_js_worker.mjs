// Issue #334: validate the JS worker's word-problem normalizer mirrors Rust.
// Loads src/web/formal_ai_worker.js in a stubbed VM context (it is a web
// worker, so `self`/`importScripts`/`postMessage` are stubbed) and exercises
// the pure extraction helpers without WASM.
import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(new URL("../../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {
  self: {},
  importScripts() { throw new Error("no seed loader in test"); },
  postMessage() {},
  addEventListener() {},
  console,
  // WASM bridge stubs: force the JS fallbacks so we test pure JS extraction.
  wasmEvaluateArithmetic: () => null,
};
sandbox.self.location = { search: "" };
const context = vm.createContext(sandbox);
try {
  vm.runInContext(source, context, { filename: "formal_ai_worker.js" });
} catch (error) {
  // `init()` at the end touches WASM/network and throws under Node; the
  // function declarations are already bound on the context, so we continue.
}

const { extractArithmeticExpression, normalizeWordProblem, resolveFibonacciReferences, parseOrdinal, fibonacciValue } = context;

function show(label, value) {
  console.log(label.padEnd(60), JSON.stringify(value));
}

show("fibonacciValue(10)", fibonacciValue(10));
show("fibonacciValue(5)", fibonacciValue(5));
show("parseOrdinal('10th')", parseOrdinal("10th"));
show("parseOrdinal('fifth')", parseOrdinal("fifth"));
show("resolve 'the 10th fibonacci number and ...'", resolveFibonacciReferences("the 10th fibonacci number and multiply it by 8% of 500"));

const step2 = "calculate the 10th Fibonacci number and multiply it by 8% of 500. Show me the code and the final result.";
show("extract(step2)", extractArithmeticExpression(step2));
show("normalizeWordProblem('the fifth Fibonacci number multiplied by 10')", normalizeWordProblem("the fifth Fibonacci number multiplied by 10"));
show("extract('What is 3.14 * 2')", extractArithmeticExpression("What is 3.14 * 2"));
show("extract('What is 10 plus 20 times 3?')", extractArithmeticExpression("What is 10 plus 20 times 3?"));
show("extract('What is 8% of $50?')", extractArithmeticExpression("What is 8% of $50?"));

console.log("\n--- evaluation path (WASM stubbed to null, JS fallback) ---");
const { tryArithmetic, evaluatePercentOfExpression } = context;
show("evaluatePercentOfExpression('55 * 8% of 500')", evaluatePercentOfExpression("55 * 8% of 500"));
show("tryArithmetic(step2)", tryArithmetic(step2));
