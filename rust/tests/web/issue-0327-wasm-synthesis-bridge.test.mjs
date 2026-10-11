// Issue #327 (R249): WASM stays the bridge for the shared primitives. Browser
// synthesis takes its arithmetic and its stable ids from the Rust engine's
// `engine_evaluate_arithmetic` / `engine_stable_id` exports when the module is
// loaded, and keeps its JS fallback only for a worker with no WASM. The
// exports here are a scripted stand-in with the real calling convention
// (input buffer, output buffer, byte length); the worker code is the real one.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const INPUT = 0;
const OUTPUT = 4096;

/** A module whose exports answer from `answers` and record each call. */
function scriptedEngine(answers) {
  const memory = { buffer: new ArrayBuffer(8192) };
  const calls = [];
  const engine = { memory, calls, input_ptr: () => INPUT, output_ptr: () => OUTPUT, input_capacity: () => OUTPUT };
  for (const [name, answer] of Object.entries(answers)) {
    engine[name] = (length) => {
      const input = new TextDecoder().decode(new Uint8Array(memory.buffer, INPUT, length));
      calls.push([name, input]);
      const bytes = new TextEncoder().encode(answer(input));
      new Uint8Array(memory.buffer, OUTPUT, bytes.length).set(bytes);
      return bytes.length;
    };
  }
  return engine;
}

const worker = createWorkerContext();
const call = (expression) => plain(evaluate(worker, expression));

test("R249: without WASM, synthesis falls back to the JS evaluator and FNV-1a ids", () => {
  evaluate(worker, "wasm = undefined");
  assert.deepEqual(call('evaluateSynthesisArithmetic("2 + 3 * 4")'), { formatted: "14", backend: "js-fallback" });
  assert.match(call('synthesisStableId("sub_result", "0:two plus two")'), /^sub_result:[0-9a-f]{16}$/);
});

test("R249: with WASM loaded, arithmetic and stable ids come from the Rust exports", () => {
  const fallbackId = call('synthesisStableId("sub_result", "0:two plus two")');
  worker.__engine = scriptedEngine({
    engine_evaluate_arithmetic: () => "14",
    engine_stable_id: (input) => `${input.split("\n")[0]}:from-wasm`,
  });
  evaluate(worker, "wasm = globalThis.__engine");
  try {
    assert.deepEqual(call('evaluateSynthesisArithmetic("2 + 3 * 4")'), { formatted: "14", backend: "wasm" });
    assert.equal(call('synthesisStableId("sub_result", "0:two plus two")'), "sub_result:from-wasm");
    assert.notEqual(fallbackId, "sub_result:from-wasm");
    assert.deepEqual(worker.__engine.calls, [
      ["engine_evaluate_arithmetic", "2 + 3 * 4"],
      ["engine_stable_id", "sub_result\n0:two plus two"],
    ]);
  } finally {
    evaluate(worker, "wasm = undefined");
  }
});

test("R249: an engine error falls back to JS instead of answering with it", () => {
  worker.__engine = scriptedEngine({ engine_evaluate_arithmetic: () => "ERR:unsupported" });
  evaluate(worker, "wasm = globalThis.__engine");
  try {
    assert.deepEqual(call('evaluateSynthesisArithmetic("7 - 2")'), { formatted: "5", backend: "js-fallback" });
  } finally {
    evaluate(worker, "wasm = undefined");
  }
});
