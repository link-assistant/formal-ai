#!/usr/bin/env node
// Issue #550: prove an h()->JSX conversion of the front-end is behaviour-
// preserving. tsconfig pins bun's JSX transform to the classic runtime
// (jsxFactory: h), so JSX compiles back to the same h() calls. We compile both
// the BEFORE and AFTER files with `bun build … --packages external` (app code
// only) and compare. To ignore cosmetic reformatting that @babel/generator
// introduces in the rewritten subtrees (e.g. `[ x ]` -> `[x]`) — which bun's
// non-minified output otherwise preserves — we re-parse each compiled module
// and re-print it through @babel/generator with `compact: true` and comments
// stripped. That canonicalises all insignificant whitespace and comments while
// preserving identifier names (no minification), so a byte-identical result
// means the two modules are semantically the same program.
//
// Usage: node experiments/verify-jsx-equivalence.mjs <before> <after>
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";

const require = createRequire(import.meta.url);
const { parse } = require("@babel/parser");
const _generate = require("@babel/generator");
const generate = _generate.default || _generate;

function canonical(file) {
  const compiled = execFileSync(
    "bun",
    ["build", file, "--target", "browser", "--format", "esm", "--packages", "external"],
    { encoding: "utf8", maxBuffer: 128 * 1024 * 1024 },
  );
  const ast = parse(compiled, { sourceType: "module", plugins: ["jsx"] });
  return generate(ast, { compact: true, comments: false, jsescOption: { minimal: true } }).code;
}

const [before, after] = process.argv.slice(2);
if (!before || !after) {
  console.error("usage: verify-jsx-equivalence.mjs <before> <after>");
  process.exit(2);
}
const a = canonical(before);
const b = canonical(after);
if (a === b) {
  console.log("EQUIVALENT: canonical compiled output is byte-identical ✓");
  process.exit(0);
}
// Localise the first difference.
let i = 0;
const m = Math.min(a.length, b.length);
while (i < m && a[i] === b[i]) i += 1;
console.error("DIFFERS at char", i, "(len before", a.length, "after", b.length, ")");
console.error("BEFORE:", JSON.stringify(a.slice(Math.max(0, i - 100), i + 150)));
console.error("AFTER :", JSON.stringify(b.slice(Math.max(0, i - 100), i + 150)));
process.exit(1);
