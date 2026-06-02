// Issue #386 — regenerate the worker's inline MEANINGS_LINO array body from the
// canonical MEANING_FILES, in place. Uses the SAME worker-style concatenation
// and serializer proven byte-faithful by issue-386-meanings-mirror.mjs, so the
// only diff this introduces is the intended semantic change in the .lino files.
//
// Run: node experiments/issue-386-apply-mirror.mjs
// Then verify: node experiments/issue-386-meanings-mirror.mjs  (expects YES/YES)

import fs from "node:fs";

const root = new URL("..", import.meta.url);

// MEANING_FILES order, mirrored from src/seed/embedded.rs.
const MEANING_FILES = [
  "data/seed/meanings.lino",
  "data/seed/meanings-units.lino",
  "data/seed/meanings-calendar.lino",
  "data/seed/meanings-facts.lino",
  "data/seed/meanings-software-project.lino",
  "data/seed/meanings-program-synthesis.lino",
  "data/seed/meanings-intent.lino",
];
if (fs.existsSync(new URL("data/seed/meanings-ontology.lino", root))) {
  MEANING_FILES.push("data/seed/meanings-ontology.lino");
}

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

function serializeLine(line) {
  if (line.includes("'")) return `  "${line.replace(/"/g, '\\"')}",`;
  if (line.includes('"')) return `  '${line}',`;
  return `  "${line}",`;
}

const workerPath = new URL("src/web/formal_ai_worker.js", root);
const workerSrc = fs.readFileSync(workerPath, "utf8");

const startIdx = workerSrc.indexOf("const MEANINGS_LINO = [");
const openBracket = workerSrc.indexOf("[", startIdx);
const closeIdx = workerSrc.indexOf("\n].join(", openBracket);
if (startIdx < 0 || closeIdx < 0) {
  console.error("could not bound MEANINGS_LINO array");
  process.exit(2);
}

// Old body (for reporting) and new body.
const oldBody = workerSrc.slice(openBracket + 2, closeIdx);
const genBody = workerStyleLines().map(serializeLine).join("\n");

// prefix ends with "[\n"; suffix starts with "\n].join(".
const prefix = workerSrc.slice(0, openBracket + 2);
const suffix = workerSrc.slice(closeIdx);
const next = prefix + genBody + suffix;

if (next === workerSrc) {
  console.log("worker MEANINGS_LINO already up to date — no change");
  process.exit(0);
}

fs.writeFileSync(workerPath, next, "utf8");
console.log(`rewrote worker MEANINGS_LINO body`);
console.log(`  old body lines: ${oldBody.split("\n").length}`);
console.log(`  new body lines: ${genBody.split("\n").length}`);
console.log(`  meaning files : ${MEANING_FILES.length}`);
