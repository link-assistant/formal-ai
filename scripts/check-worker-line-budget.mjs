#!/usr/bin/env node
// JavaScript twin of `scripts/check-worker-line-budget.rs` (PR #1188,
// SCRIPTS-B).
//
// Enforce the UI-glue line budget for the split JavaScript worker: each
// `js/worker/*.js` module stays under the ceiling its own shard in
// `data/meta/worker-line-budget/` records (issues #658 and #991). Same output,
// exit codes and `::error` annotations as the Rust original, and `--write`
// writes the same shards; CI runs both and diffs them
// (`data/meta/ci-gates/check-worker-line-budget-js-twin.lino`). Read the Rust
// script's doc comments for why each rule exists.
//
// Usage:
//   node scripts/check-worker-line-budget.mjs           # enforce
//   node scripts/check-worker-line-budget.mjs --write   # re-baseline

import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import {
  RustError, compareStrings, extension, isFile, lineCount, lines, parseUnsigned, readDirEntries,
  readToString, sortedKeys, splitOnce, trim, trimStart,
} from './lib/checks-rust-compat.mjs';

const TARGET_TOTAL_LINES = 3000;
const WORKER_DIR = 'js/worker';
const BUDGET_DIR = 'data/meta/worker-line-budget';

/** The subject of a module, read from its own leading `//` comment paragraph. */
export function leadingSummary(source) {
  let paragraph = '';
  for (const line of lines(source)) {
    const start = trimStart(line);
    if (!start.startsWith('//')) break;
    const comment = trim(start.slice(2));
    if (comment === '') break;
    if (paragraph !== '') paragraph += ' ';
    paragraph += comment;
  }
  if (paragraph.startsWith('Worker module')) {
    paragraph = splitOnce(paragraph, '. ')?.[1] ?? '';
  }
  // `paragraph[len.min(280)..].find(". ")` works in UTF-8 bytes.
  const bytes = Buffer.from(paragraph, 'utf8');
  const from = Math.min(bytes.length, 280);
  const end = bytes.subarray(from).indexOf('. ');
  if (end !== -1) paragraph = bytes.subarray(0, from + end + 1).toString('utf8');
  return paragraph.replaceAll('"', "'");
}

/** The `*.js` files under `js/worker`, sorted by path, with line counts. */
export function collectWorkerFiles(cwd, warn = (line) => process.stderr.write(`${line}\n`)) {
  const directory = join(cwd, WORKER_DIR);
  let entries;
  try {
    entries = readDirEntries(directory);
  } catch {
    return [];
  }
  const files = [];
  for (const entry of entries) {
    if (!isFile(entry.path) || extension(entry.path)?.toLowerCase() !== 'js') continue;
    let content;
    try {
      content = readToString(entry.path);
    } catch (error) {
      if (!(error instanceof RustError)) throw error;
      warn(`Warning: Could not read ${entry.path}: ${error.message}`);
      continue;
    }
    files.push({ path: `${WORKER_DIR}/${entry.name}`, module: entry.name, lines: lineCount(content), summary: leadingSummary(content) });
  }
  return files.sort((left, right) => compareStrings(left.path, right.path));
}

function unquote(value) {
  const trimmed = trim(value);
  return trimmed.length >= 2 && trimmed.startsWith('"') && trimmed.endsWith('"') ? trimmed.slice(1, -1) : trimmed;
}

/** Read one shard; `null` when it names no module or its ceiling is not a number. */
export function parseBudget(source) {
  const budget = { module: '', ceiling: 0, rationale: '' };
  for (const line of lines(source)) {
    const parts = splitOnce(trim(line), ' ');
    if (!parts) continue;
    const [key, value] = parts;
    if (key === 'module') {
      budget.module = unquote(value);
    } else if (key === 'ceiling') {
      const parsed = parseUnsigned(trim(value));
      if (parsed.error) return null;
      budget.ceiling = parsed.value;
    } else if (key === 'rationale') {
      budget.rationale = unquote(value);
    }
  }
  return budget.module === '' ? null : budget;
}

/** Every shard keyed by module, the later of two in directory order winning. */
export function collectBudgets(cwd) {
  const budgets = new Map();
  let entries;
  try {
    entries = readDirEntries(join(cwd, BUDGET_DIR));
  } catch {
    return budgets;
  }
  for (const entry of entries) {
    if (extension(entry.path) !== 'lino') continue;
    let source;
    try {
      source = readToString(entry.path);
    } catch {
      continue;
    }
    const budget = parseBudget(source);
    if (budget) budgets.set(budget.module, budget);
  }
  return budgets;
}

/** `str::trim_end_matches(".js")`: every trailing `.js`, not just one. */
export function shardName(module) {
  let stem = module;
  while (stem.endsWith('.js')) stem = stem.slice(0, -3);
  return `${stem}.lino`;
}

function renderBudget(budget) {
  return `worker_module_budget\n  module "${budget.module}"\n  ceiling ${budget.ceiling}\n  rationale "${budget.rationale}"\n`;
}

/** Every way the recorded budgets and the mirror disagree. */
export function budgetFailures(files, budgets) {
  const failures = [];
  for (const file of files) {
    const budget = budgets.get(file.module);
    if (!budget) {
      failures.push(`${file.path} has no budget shard; add ${BUDGET_DIR}/${shardName(file.module)} recording why the module exists and how many lines it is allowed`);
    } else if (file.lines > budget.ceiling) {
      failures.push(`${file.path} grew to ${file.lines} lines, past its recorded ceiling of ${budget.ceiling}. Move logic into the Rust→WASM worker (js/wasm-worker) instead of growing the mirror, or re-baseline this one module with \`--write\` and explain the growth in ${BUDGET_DIR}/${shardName(file.module)}`);
    }
  }
  for (const module of sortedKeys(budgets)) {
    if (!files.some((file) => file.module === module)) {
      failures.push(`${BUDGET_DIR}/${shardName(module)} budgets \`${module}\`, which no longer exists; delete the shard`);
    }
  }
  return failures;
}

function rebaseline(cwd, files, budgets, out) {
  const directory = join(cwd, BUDGET_DIR);
  mkdirSync(directory, { recursive: true });
  for (const file of files) {
    const recorded = budgets.get(file.module)?.rationale;
    const rendered = renderBudget({ module: file.module, ceiling: file.lines, rationale: recorded || file.summary });
    const path = join(directory, shardName(file.module));
    let current = '';
    try {
      current = readFileSync(path, 'utf8');
    } catch {
      // A missing shard is written fresh.
    }
    if (current !== rendered) {
      writeFileSync(path, rendered);
      out(`  rebaselined  ${file.module} -> ${file.lines} lines`);
    }
  }
  for (const module of sortedKeys(budgets)) {
    if (!files.some((file) => file.module === module)) {
      try {
        rmSync(join(directory, shardName(module)));
      } catch {
        // `let _ = fs::remove_file(..)`: a shard that cannot be removed is left.
      }
      out(`  removed      ${module} (no longer in the mirror)`);
    }
  }
}

function main() {
  const out = (line) => process.stdout.write(`${line}\n`);
  out('\nChecking the UI-glue line budget for the split JavaScript worker...\n');
  const cwd = process.cwd();
  const files = collectWorkerFiles(cwd);
  if (files.length === 0) {
    out(`No worker JavaScript files found under ${WORKER_DIR}/ — nothing to check.\n`);
    return;
  }
  const budgets = collectBudgets(cwd);
  // `std::env::args()` includes the program name.
  if (process.argv.slice(1).includes('--write')) {
    rebaseline(cwd, files, budgets, out);
    out('\nBudget shards re-baselined. Review the diff and explain any growth.\n');
    return;
  }
  out(`Worker JavaScript line counts (${WORKER_DIR}/*.js):`);
  for (const file of files) {
    const ceiling = budgets.has(file.module) ? String(budgets.get(file.module).ceiling) : '  none';
    out(`  ${String(file.lines).padStart(6)} / ${ceiling.padStart(6)}  ${file.path}`);
  }
  const total = files.reduce((sum, file) => sum + file.lines, 0);
  const summed = [...budgets.values()].reduce((sum, budget) => sum + budget.ceiling, 0);
  out(`\n  total: ${total} lines (summed ceilings ${summed}, target ${TARGET_TOTAL_LINES})\n`);
  const failures = budgetFailures(files, budgets);
  if (failures.length > 0) {
    for (const failure of failures) out(`::error::${failure}`);
    out(`\n${failures.length} module budget violation(s). The mirror cannot silently regrow.\n`);
    process.exitCode = 1;
  } else if (total > TARGET_TOTAL_LINES) {
    out(`Every module is within its own ceiling. ${total - TARGET_TOTAL_LINES} line(s) above the ${TARGET_TOTAL_LINES}-line UI-glue target.`);
    out('Migrate more solver logic into the Rust→WASM worker, then run\n`rust-script scripts/check-worker-line-budget.rs --write` to lock in the drop.\n');
  } else {
    out(`Worker JavaScript is at or below the ${TARGET_TOTAL_LINES}-line UI-glue target.\n`);
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main();
