// Issue #513 — keep the worker's inline terminal-command trigger vocabulary in
// lockstep with the canonical seed file.
//
// `data/seed/terminal-commands.lino` is the single source of truth for the
// natural-language triggers (terminal/shell phrases, run verbs, Chinese run
// verbs, leading shell tokens) that classify a prompt as a terminal command.
// The Rust solver reads it via `src/seed/terminal_commands.rs`; the JS worker
// embeds a byte-identical inline mirror so detection works without an async
// seed fetch (the same convention as the operation vocabulary, issue #386).
//
// Run after editing the seed:
//   node experiments/issue-513-sync-worker-terminal.mjs
// Verify in CI (non-zero exit on drift):
//   node experiments/issue-513-sync-worker-terminal.mjs --check
//
// It regenerates the inline `TERMINAL_COMMANDS_LINO` array byte-identically from
// the seed, using the shared house quote style so the worker body stays stable.

import fs from "node:fs";

import { serializeMeaningLine } from "./issue-386-meaning-files.mjs";

const root = new URL("..", import.meta.url);
const seedPath = new URL("data/seed/terminal-commands.lino", root);
const workerPath = new URL("src/web/formal_ai_worker.js", root);

const checkOnly = process.argv.includes("--check");

const lino = fs.readFileSync(seedPath, "utf8").replace(/\n+$/, "");
const lines = lino.split("\n");
while (lines.length && lines[lines.length - 1] === "") lines.pop();
const arrayBody = lines.map((l) => "  " + serializeMeaningLine(l)).join(",\n");
const newBlock = `const TERMINAL_COMMANDS_LINO = [\n${arrayBody},\n].join("\\n");`;

const blockRe = /const TERMINAL_COMMANDS_LINO = \[\n[\s\S]*?\n\]\.join\("\\n"\);/;
const worker = fs.readFileSync(workerPath, "utf8");
if (!blockRe.test(worker)) {
  throw new Error("anchor not found: TERMINAL_COMMANDS_LINO inline block");
}
const updated = worker.replace(blockRe, newBlock);

if (checkOnly) {
  if (updated !== worker) {
    console.error(
      "[issue-513] src/web/formal_ai_worker.js is out of sync with " +
        "data/seed/terminal-commands.lino.\n" +
        "Run: node experiments/issue-513-sync-worker-terminal.mjs",
    );
    process.exit(1);
  }
  console.log("[issue-513] worker terminal vocabulary is in sync with seed.");
  process.exit(0);
}

if (updated !== worker) {
  fs.writeFileSync(workerPath, updated);
  console.log(
    "[issue-513] regenerated TERMINAL_COMMANDS_LINO in " +
      "src/web/formal_ai_worker.js from data/seed/terminal-commands.lino.",
  );
} else {
  console.log("[issue-513] worker terminal vocabulary already in sync.");
}
