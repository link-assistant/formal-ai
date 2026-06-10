#!/usr/bin/env node
// Regenerate the CODING_IDIOMS_LINO embed in src/web/formal_ai_worker.js from
// data/seed/coding-idioms.lino. The worker cannot include_str! the seed file
// the way the Rust solver does, so it carries a byte mirror; run this script
// after editing the seed file and splice the printed block over the existing
// `const CODING_IDIOMS_LINO = [...]` declaration.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const seed = readFileSync(join(root, "data/seed/coding-idioms.lino"), "utf8");
const lines = seed.replace(/\n$/, "").split("\n");

const out = [];
out.push("const CODING_IDIOMS_LINO = [");
for (const line of lines) {
  out.push(`  ${JSON.stringify(line)},`);
}
out.push('].join("\\n");');
process.stdout.write(out.join("\n") + "\n");
