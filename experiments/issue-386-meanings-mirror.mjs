// Issue #386 — verify the worker's inline MEANINGS_LINO is a faithful mirror of
// the canonical MEANING_FILES, and provide a provably-correct regenerator.
//
// The Rust lexicon does `MEANING_FILES.join("\n")` (src/seed/meanings.rs). Both
// parsers ignore blank lines, so the worker carries the same records inline but
// WITHOUT the inter-file blank lines that join() introduces (and without the
// trailing-newline empty line). Each record line is serialized by the shared
// serializeMeaningLine (issue-386-meaning-files.mjs) — the SAME function the
// worker sync uses to write the array — so check (2) below is a true tautology:
// it passes iff the worker body is exactly what the sync would emit.
//
// This script proves, against the CURRENT tree, that:
//   (1) the worker's line array equals the worker-style concatenation of the
//       canonical files, and
//   (2) the serializer reproduces the worker's array body byte-for-byte.
// Once both hold, `--emit` can regenerate the body after editing the canonical
// files, guaranteeing the only diff is the intended semantic change.
//
// Modes:
//   node experiments/issue-386-meanings-mirror.mjs          # verify (exit 1 on drift)
//   node experiments/issue-386-meanings-mirror.mjs --emit    # print regenerated body lines

import fs from "node:fs";

import { MEANING_FILES, serializeMeaningLine } from "./issue-386-meaning-files.mjs";

const root = new URL("..", import.meta.url);

// Worker-style concatenation: each file's content lines, trailing empty dropped.
function workerStyleLines() {
  const out = [];
  for (const rel of MEANING_FILES) {
    const text = fs.readFileSync(new URL(rel, root), "utf8");
    const lines = text.split("\n");
    if (lines.length && lines[lines.length - 1] === "") lines.pop();
    out.push(...lines);
  }
  return out;
}

// Wrap the authoritative serializer (shared with the worker sync) as one inline
// array entry: two-space indent + the JS string literal + trailing comma.
function serializeLine(line) {
  return `  ${serializeMeaningLine(line)},`;
}

// --- extract the worker's MEANINGS_LINO array body ---------------------------
const workerSrc = fs.readFileSync(new URL("src/web/formal_ai_worker.js", root), "utf8");
const startIdx = workerSrc.indexOf("const MEANINGS_LINO = [");
const openBracket = workerSrc.indexOf("[", startIdx);
const closeIdx = workerSrc.indexOf("\n].join(", openBracket);
if (startIdx < 0 || closeIdx < 0) {
  console.error("could not bound MEANINGS_LINO array");
  process.exit(2);
}
const arrayText = workerSrc.slice(openBracket, closeIdx + 2); // "[ ... ]"
const rawBody = workerSrc.slice(openBracket + 2, closeIdx); // between "[\n" and "\n]"
// eslint-disable-next-line no-new-func
const workerArr = Function(`return ${arrayText}`)();

const wantLines = workerStyleLines();

if (process.argv.includes("--emit")) {
  process.stdout.write(wantLines.map(serializeLine).join("\n"));
  process.stdout.write("\n");
  process.exit(0);
}

// (1) parsed-line equality
let lineEq = workerArr.length === wantLines.length;
let firstDiff = -1;
for (let i = 0; i < Math.max(workerArr.length, wantLines.length); i++) {
  if (workerArr[i] !== wantLines[i]) { lineEq = false; firstDiff = i; break; }
}
console.log(`meaning files: ${MEANING_FILES.length}`);
console.log(`worker lines: ${workerArr.length}, canonical lines: ${wantLines.length}`);
console.log(`(1) PARSED-LINE EQUALITY: ${lineEq ? "YES" : "NO"}`);
if (!lineEq && firstDiff >= 0) {
  console.log(`  first diff @${firstDiff}: worker=${JSON.stringify(workerArr[firstDiff])} canonical=${JSON.stringify(wantLines[firstDiff])}`);
}

// (2) serializer fidelity: regenerate the body and compare to the raw source
const genBody = wantLines.map(serializeLine).join("\n");
const serEq = genBody === rawBody;
console.log(`(2) SERIALIZER REPRODUCES WORKER BODY: ${serEq ? "YES" : "NO"}`);
if (!serEq) {
  const a = rawBody.split("\n");
  const b = genBody.split("\n");
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    if (a[i] !== b[i]) {
      console.log(`  first body diff @${i}:`);
      console.log(`    worker: ${JSON.stringify(a[i])}`);
      console.log(`    gen   : ${JSON.stringify(b[i])}`);
      break;
    }
  }
}

process.exit(lineEq && serEq ? 0 : 1);
