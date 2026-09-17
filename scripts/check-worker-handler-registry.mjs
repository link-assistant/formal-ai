#!/usr/bin/env node

import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const seedPath = path.join(root, "data/seed/browser-handler-precedence.lino");
const workerDir = path.join(root, "src/web/worker");
const dispatchPath = path.join(workerDir, "formal_ai_worker_dispatch.js");

function fail(message) {
  throw new Error(`worker handler registry: ${message}`);
}

function parseRegistry(source) {
  const records = [];
  let current = null;
  for (const line of source.split(/\r?\n/)) {
    const handler = /^  handler\s+(\S+)\s*$/.exec(line);
    if (handler) {
      current = { name: handler[1], contextBinding: "" };
      records.push(current);
      continue;
    }
    const binding = /^    context_binding_(\S+)\s*$/.exec(line);
    if (binding && current) current.contextBinding = binding[1];
  }
  return records;
}

const seed = readFileSync(seedPath, "utf8");
const records = parseRegistry(seed);
if (records.length === 0) fail(`${path.relative(root, seedPath)} is empty`);
const names = records.map((record) => record.name);
if (new Set(names).size !== names.length) fail("seed handler names are not unique");

const workerFiles = readdirSync(workerDir)
  .filter((name) => name.endsWith(".js"))
  .map((name) => path.join(workerDir, name));
const sources = workerFiles.map((file) => [file, readFileSync(file, "utf8")]);
const dispatchOwners = sources.filter(([, source]) => {
  return source.includes("function synchronousHandlerCandidates");
});
if (dispatchOwners.length !== 1 || dispatchOwners[0][0] !== dispatchPath) {
  fail("exactly one worker module must own synchronousHandlerCandidates");
}

for (const [file, source] of sources) {
  if (/const\s+syncHandlers\s*=\s*\[/.test(source) || /name:\s*["']try[A-Z]/.test(source)) {
    fail(`${path.relative(root, file)} contains an independent handler inventory`);
  }
}

const allWorkerSource = sources.map(([, source]) => source).join("\n");
for (const record of records) {
  if (record.contextBinding) continue;
  const escaped = record.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  if (!new RegExp(`function\\s+${escaped}\\s*\\(`).test(allWorkerSource)) {
    fail(`seed handler ${record.name} has no worker function`);
  }
}

const loader = readFileSync(path.join(root, "src/web/seed_loader.js"), "utf8");
if (!loader.includes("extractBrowserHandlerPrecedence")) {
  fail("src/web/seed_loader.js does not export the seed projection");
}
const generatedInventory = readFileSync(path.join(root, "src/web/seed-files.js"), "utf8");
if (!generatedInventory.includes('"seed/browser-handler-precedence.lino"')) {
  fail("src/web/seed-files.js is stale; regenerate the seed registry");
}

const debtChecker = readFileSync(path.join(root, "scripts/check-debt-ratchet.rs"), "utf8");
if (!debtChecker.includes('join("src/web/worker")') ||
    !debtChecker.includes('contains("function synchronousHandlerCandidates")')) {
  fail("the debt scanner must discover the real registry across the worker directory");
}

console.log(`worker handler registry: ${records.length} seed-derived bindings; no JS inventory`);
