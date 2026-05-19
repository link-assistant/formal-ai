// Smoke test: WASM bridge for web_search core (R194).
//
// Boots the same byte-buffer protocol formal_ai_worker.js uses and verifies:
//   1. web_search_rrf_k, web_search_concurrency_per_category, web_search_provider_limit
//   2. web_search_request_evidence — multi-line prefix with language line
//   3. web_search_fuse — parses tab-delimited rows and returns ranked output
//   4. parseFusedOutput on the JS side decodes the Rust format correctly
//
// Run with: node experiments/issue-133-wasm-bridge.mjs

import { readFile } from "node:fs/promises";

const bytes = await readFile(new URL("../src/web/formal_ai_worker.wasm", import.meta.url));
const module = await WebAssembly.instantiate(bytes, {});
const wasm = module.instance.exports;

const enc = new TextEncoder();
const dec = new TextDecoder();

function writeInput(text) {
  const data = enc.encode(text);
  const view = new Uint8Array(wasm.memory.buffer, wasm.input_ptr(), data.length);
  view.set(data);
  return data.length;
}

function readOutput(length) {
  if (length === 0) return "";
  const view = new Uint8Array(wasm.memory.buffer, wasm.output_ptr(), length);
  return dec.decode(view);
}

function parseFusedOutput(text) {
  return text
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const fields = line.split("\t");
      const providerSpecs = (fields[4] || "")
        .split("+")
        .filter((part) => part.length > 0)
        .map((part) => {
          const hash = part.lastIndexOf("#");
          return { id: part.slice(0, hash), rank: Number.parseInt(part.slice(hash + 1), 10) || 0 };
        });
      return {
        url: fields[0] || "",
        title: fields[1] || "",
        excerpt: fields[2] || "",
        score: Number.parseFloat(fields[3] || "0") || 0,
        providers: providerSpecs,
      };
    });
}

let failures = 0;
function check(label, actual, expected) {
  const ok = JSON.stringify(actual) === JSON.stringify(expected);
  if (!ok) {
    failures += 1;
    console.error(`FAIL ${label}\n  expected: ${JSON.stringify(expected)}\n  actual:   ${JSON.stringify(actual)}`);
  } else {
    console.log(`ok   ${label}`);
  }
}

// 1. Constants
check("rrf k", wasm.web_search_rrf_k(), 60);
check("concurrency", wasm.web_search_concurrency_per_category(), 5);
check("provider limit", wasm.web_search_provider_limit(), 10);
check("registry len", wasm.web_search_registry_len(), 26);

// 2. Plan
const planLen = wasm.web_search_plan();
const plan = readOutput(planLen).split("\n");
check("plan: duckduckgo first", plan[0], "duckduckgo");
check("plan: 3 cors-readable entries", plan.length, 3);

// 3. Request evidence (with language)
const evLen = wasm.web_search_request_evidence(writeInput("formal-ai\nen"));
const evLines = readOutput(evLen).split("\n");
check("evidence: first line", evLines[0], "web_search:request:formal-ai");
check("evidence: language line", evLines[1], "web_search:language:en");
check("evidence: last line", evLines[evLines.length - 1], "web_search:combined:rrf:k=60");

// 4. Request evidence (no language)
const noLangLen = wasm.web_search_request_evidence(writeInput("foo\n"));
const noLangLines = readOutput(noLangLen).split("\n");
check("evidence (no lang): first line", noLangLines[0], "web_search:request:foo");
check("evidence (no lang): no language line", noLangLines[1].startsWith("web_search:language:"), false);

// 5. Fuse — DuckDuckGo and Wikipedia both rank URL #1 at position 1.
const rows = [
  "duckduckgo\t1\thttps://a.test\tAlpha\t",
  "duckduckgo\t2\thttps://b.test\tBravo\t",
  "wikipedia\t1\thttps://a.test\tAlpha-Wiki\tFrom wiki",
  "wikipedia\t2\thttps://c.test\tCharlie\t",
];
const fuseLen = wasm.web_search_fuse(writeInput(rows.join("\n")));
const fused = parseFusedOutput(readOutput(fuseLen));
check("fuse: top result url", fused[0].url, "https://a.test");
check("fuse: top result merged providers", fused[0].providers.map((p) => p.id), ["duckduckgo", "wikipedia"]);
check("fuse: result count", fused.length, 3);

// score(a) = 1/(60+1) + 1/(60+1) = 2/61 ≈ 0.032787
const expectedScore = 2 / 61;
const delta = Math.abs(fused[0].score - expectedScore);
check("fuse: score precision (delta < 1e-5)", delta < 1e-5, true);

if (failures > 0) {
  console.error(`\n${failures} failure(s).`);
  process.exit(1);
}
console.log("\nAll WASM bridge checks passed.");
