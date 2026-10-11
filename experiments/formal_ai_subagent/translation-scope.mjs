#!/usr/bin/env node
// Measures the js -> rust translation (scripts/translate-js-rust.mjs) over
// JavaScript roots outside its committed scope, without touching the ledger or
// the projections: per root, how many top-level items meta-language translates
// and carries, and which constructs it refuses most often.
//
// Usage:
//   node experiments/formal_ai_subagent/translation-scope.mjs --meta-language DIR [--jobs N] [--json OUT] [ROOT...]
//
// A ROOT is `dir` or `dir:regex` (the regex filters file names); the default
// roots are the agentic planner, the server and the browser worker outside the
// committed scope. DIR is a meta-language checkout at the commit the ledger
// pins (scripts/translate-js-rust.mjs --fetch downloads one).

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { availableParallelism } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { Worker, isMainThread, parentPort, workerData } from 'node:worker_threads';
import { measure, upstreamBlocks } from '../../scripts/translate-js-rust.mjs';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const DIAGNOSED = new Set(['unsupported', 'syntax', 'type']);
const DEFAULT_ROOTS = [
  'js/agentic:^[a-z0-9_]+\\.mjs$',
  'js/agentic/file_read:\\.mjs$',
  'js/agentic/learning_report:\\.mjs$',
  'js/agentic/planner:\\.mjs$',
  'js/server:^[a-z0-9_.-]+\\.mjs$',
  'js/worker:^formal_ai_worker(?!_meta_)[a-z0-9_]*\\.js$',
];

if (!isMainThread) {
  const api = await import(pathToFileURL(join(workerData.dir, 'js', 'src', 'index.js')).href);
  parentPort.on('message', ({ path, text }) => {
    let code;
    try {
      ({ code } = api.selfTranslate(text, 'JavaScript', 'Rust'));
    } catch (error) {
      parentPort.postMessage({ path, failed: String(error?.message ?? error) });
      return;
    }
    const details = upstreamBlocks(code).blocks.map((block) => (block.kind === 'carried' && DIAGNOSED.has(block.reason)
      ? (api.translateProgram(block.source, 'JavaScript', 'Rust').diagnostic?.message ?? null)
      : null));
    parentPort.postMessage({ path, code, details });
  });
} else {
  const argv = process.argv.slice(2);
  const option = (name, fallback) => (argv.includes(name) ? argv[argv.indexOf(name) + 1] : fallback);
  const dir = option('--meta-language', process.env.FORMAL_AI_META_LANGUAGE);
  if (!dir) throw new Error('name the meta-language checkout with --meta-language DIR');
  const jobs = Number(option('--jobs', Math.min(4, availableParallelism())));
  const valued = new Set(['--meta-language', '--jobs', '--json']);
  const named = argv.filter((arg, index) => !arg.startsWith('--') && !valued.has(argv[index - 1]));
  const roots = (named.length ? named : DEFAULT_ROOTS).map((spec) => {
    const [root, pattern] = spec.split(/:(.*)/su);
    return { root, name: new RegExp(pattern ?? '\\.m?js$', 'u') };
  });
  const tracked = execFileSync('git', ['ls-files', '--', ...roots.map((entry) => entry.root)], { cwd: REPO, encoding: 'utf8' })
    .split('\n').filter(Boolean);
  const rootOf = (path) => roots.find((entry) => entry.root === path.slice(0, path.lastIndexOf('/')) && entry.name.test(path.slice(path.lastIndexOf('/') + 1)));
  const queue = tracked.filter((path) => rootOf(path)).map((path) => ({ path, text: readFileSync(join(REPO, path), 'utf8') }))
    .sort((a, b) => b.text.length - a.text.length);
  const total = queue.length;
  const results = [];
  await new Promise((resolve, reject) => {
    let running = 0;
    for (let index = 0; index < Math.max(1, Math.min(jobs, total)); index += 1) {
      const worker = new Worker(fileURLToPath(import.meta.url), { workerData: { dir } });
      running += 1;
      const next = () => {
        const task = queue.shift();
        if (task) worker.postMessage(task);
        else worker.terminate();
      };
      worker.on('message', (message) => {
        results.push(message);
        process.stderr.write(`measured ${results.length}/${total} ${message.path}\n`);
        next();
      });
      worker.on('error', reject);
      worker.on('exit', () => {
        running -= 1;
        if (running === 0) resolve();
      });
      next();
    }
  });
  const report = roots.map(({ root }) => ({ root, modules: 0, failed: [], translated: 0, functions: 0, carried: 0, fully: 0, refused: 0, refusals: new Map() }));
  for (const result of results) {
    const entry = report[roots.indexOf(rootOf(result.path))];
    entry.modules += 1;
    if (result.failed) {
      entry.failed.push(`${result.path}: ${result.failed}`);
      continue;
    }
    const measured = measure(result.path, result);
    entry.translated += measured.translated;
    // A translated block that emits a function, not only a constant.
    entry.functions += measured.projection.split('// meta-language:').filter((block) => block.startsWith('translated') && /^pub fn /mu.test(block)).length;
    entry.carried += measured.carried;
    if (measured.carried === 0 && measured.translated > 0) entry.fully += 1;
    if (measured.translated === 0) entry.refused += 1;
    for (const refusal of measured.refusals) entry.refusals.set(refusal, (entry.refusals.get(refusal) ?? 0) + 1);
  }
  const json = report.map((entry) => ({ ...entry, refusals: [...entry.refusals].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])) }));
  if (argv.includes('--json')) writeFileSync(option('--json'), `${JSON.stringify(json, null, 2)}\n`);
  for (const entry of json) {
    const items = entry.translated + entry.carried;
    console.log(`${entry.root}: ${entry.modules} modules, ${entry.translated}/${items} items translated (${entry.functions} with a function), ${entry.carried} carried, ${entry.fully} fully, ${entry.refused} refused${entry.failed.length ? `, ${entry.failed.length} failed` : ''}`);
    for (const [refusal, count] of entry.refusals.slice(0, 12)) console.log(`  ${count} ${refusal}`);
    for (const failure of entry.failed) console.log(`  failed ${failure}`);
  }
}
