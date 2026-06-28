// Issue #513: keep the worker's terminal-command trigger vocabulary in
// lockstep with the canonical seed file.
//
// `data/seed/terminal-commands.lino` is the single source of truth for the
// natural-language triggers (terminal/shell phrases, run verbs, Chinese run
// verbs, leading shell tokens) that classify a prompt as a terminal command.
// The Rust solver reads it via `src/seed/terminal_commands.rs`; the JS worker
// reads the synced `src/web/seed/terminal-commands.lino` deployment copy via
// `src/web/seed_loader.js`.
//
// Run after editing the seed:
//   node experiments/issue-513-sync-worker-terminal.mjs
// Verify in CI (non-zero exit on drift):
//   node experiments/issue-513-sync-worker-terminal.mjs --check
//
// It refreshes the web seed copy byte-identically from the canonical seed and
// verifies the worker still hydrates `TERMINAL_COMMANDS_LINO` from loaded seed
// text instead of reintroducing inline natural-language data.

import fs from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = new URL("..", import.meta.url);
const seedPath = new URL("data/seed/terminal-commands.lino", root);
const webSeedPath = new URL("src/web/seed/terminal-commands.lino", root);
const seedLoaderPath = new URL("src/web/seed_loader.js", root);
const workerDirPath = new URL("src/web/worker/", root);
const workerDir = fileURLToPath(workerDirPath);

const checkOnly = process.argv.includes("--check");
const canonicalSeed = fs.readFileSync(seedPath, "utf8");
let failed = false;

function reportFailure(message) {
  failed = true;
  console.error(`[issue-513] ${message}`);
}

function workerSource() {
  const workerFiles = fs
    .readdirSync(workerDirPath, { withFileTypes: true })
    .filter(
      (entry) =>
        entry.isFile() && /^formal_ai_worker_\d+\.js$/.test(entry.name),
    )
    .map((entry) => path.join(workerDir, entry.name))
    .sort();
  return workerFiles
    .map((file) => fs.readFileSync(file, "utf8"))
    .join("\n");
}

if (!fs.existsSync(webSeedPath)) {
  if (checkOnly) {
    reportFailure(
      "src/web/seed/terminal-commands.lino is missing. Run scripts/sync-seed.sh.",
    );
  } else {
    fs.writeFileSync(webSeedPath, canonicalSeed);
    console.log(
      "[issue-513] created src/web/seed/terminal-commands.lino from data/seed/terminal-commands.lino.",
    );
  }
} else {
  const webSeed = fs.readFileSync(webSeedPath, "utf8");
  if (webSeed !== canonicalSeed) {
    if (checkOnly) {
      reportFailure(
        "src/web/seed/terminal-commands.lino is out of sync with data/seed/terminal-commands.lino. Run scripts/sync-seed.sh.",
      );
    } else {
      fs.writeFileSync(webSeedPath, canonicalSeed);
      console.log(
        "[issue-513] refreshed src/web/seed/terminal-commands.lino from data/seed/terminal-commands.lino.",
      );
    }
  }
}

const seedLoader = fs.readFileSync(seedLoaderPath, "utf8");
if (!seedLoader.includes('"seed/terminal-commands.lino"')) {
  reportFailure(
    "src/web/seed_loader.js does not include seed/terminal-commands.lino in DEFAULT_FILES.",
  );
}

const worker = workerSource();
if (
  !/TERMINAL_COMMANDS_LINO\s*=\s*seedRawText\(raw,\s*"terminal-commands\.lino"\s*\)/.test(
    worker,
  )
) {
  reportFailure(
    "worker does not hydrate TERMINAL_COMMANDS_LINO from terminal-commands.lino seed text.",
  );
}
if (/TERMINAL_COMMANDS_LINO\s*=\s*\[\s*[\r\n]/.test(worker)) {
  reportFailure(
    "worker still embeds TERMINAL_COMMANDS_LINO inline data instead of loading seed text.",
  );
}

if (failed) {
  process.exit(1);
}

console.log("[issue-513] worker terminal vocabulary is in sync with seed.");
