// Issue #408 (R294): deterministic text and code edits keep the Rust solver
// and the browser worker aligned. The benchmark-family matrix lives once, in
// rust/tests/unit/specification/text_manipulation_benchmarks.rs, where the
// Rust solver must answer every case exactly; this suite reads the same cases
// from that file and puts each to the browser worker, which must route it to
// `text_manipulation` and give the same documented answer.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain, REPO_ROOT } from "./support/browser-runtime.mjs";

const MATRIX = "rust/tests/unit/specification/text_manipulation_benchmarks.rs";

/** A Rust string literal's value (the escapes the matrix uses). */
function rustString(literal) {
  return literal.replace(/\\(u\{([0-9a-fA-F]+)\}|.)/gu, (_, escape, code) => {
    if (code) return String.fromCodePoint(Number.parseInt(code, 16));
    return { n: "\n", t: "\t", r: "\r", "0": "\0" }[escape] ?? escape;
  });
}

function matrixCases() {
  const source = readFileSync(path.join(REPO_ROOT, MATRIX), "utf8");
  const field = (body, name) => {
    const raw = new RegExp(`\\b${name}: r(#*)"([\\s\\S]*?)"\\1[,\\n]`, "u").exec(body);
    if (raw) return raw[2];
    const match = new RegExp(`\\b${name}: "((?:[^"\\\\]|\\\\.)*)"`, "u").exec(body);
    return match ? rustString(match[1]) : null;
  };
  return [...source.matchAll(/Case \{([\s\S]*?)\n\s*\},/gu)].map(([, body]) => ({
    source: field(body, "source"),
    family: field(body, "family"),
    prompt: field(body, "prompt"),
    answer: field(body, "answer"),
  }));
}

const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

const cases = matrixCases();

test("R294: the shared matrix is read whole from the Rust specification", () => {
  assert.ok(cases.length >= 60, `only ${cases.length} cases parsed from ${MATRIX}`);
  for (const testCase of cases) assert.ok(testCase.prompt && testCase.answer !== null, JSON.stringify(testCase));
});

for (const { source, family, prompt, answer } of cases) {
  test(`R294: ${source} ${family} answers as the Rust solver does`, async () => {
    await ready;
    const response = plain(await worker.solve(prompt, [], {}, {}, [], {}));
    assert.equal(response.intent, "text_manipulation", `${prompt}\n${JSON.stringify(response)}`);
    assert.equal(String(response.content), answer, prompt);
  });
}
