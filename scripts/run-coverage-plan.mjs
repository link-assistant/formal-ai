#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { executionBatches, executeBatches } from './lib/coverage-execution.mjs';

const [manifestPath, listingPath, planPath, resultPath] = process.argv.slice(2);
if (!resultPath) throw new Error('usage: run-coverage-plan.mjs manifest listing plan result');
if (!process.env.LLVM_PROFILE_FILE) throw new Error('LLVM_PROFILE_FILE is required');
const root = process.cwd();
const { readTestDurations } = await import(pathToFileURL(resolve(root, 'scripts/lib/ci-speed-durations.mjs')));
const { durationLookup } = await import(pathToFileURL(resolve(root, 'scripts/lib/ci-speed-shards.mjs')));
const lines = (path) => readFileSync(path, 'utf8').split('\n').filter(Boolean);
const manifest = lines(manifestPath);
const executables = new Map();
for (const line of manifest) {
  const columns = line.split('\t');
  if (columns.length !== 2 || !columns.every(Boolean)) throw new Error('invalid executable manifest row');
  const [target, path] = columns;
  if (executables.has(target)) throw new Error(`duplicate executable target: ${target}`);
  executables.set(target, resolve(root, path));
}
const native = readTestDurations(resolve(root, 'data/meta/test-durations.lino'));
const profile = readTestDurations(resolve(root, 'data/meta/coverage-test-weights.lino'));
const nativeSeconds = durationLookup(native, 0.092);
const secondsOf = (item) => Math.max(nativeSeconds(item), profile.get(item.slice(item.indexOf('\t') + 1)) ?? 0);
const listing = lines(listingPath);
const selected = lines(planPath);
const batches = executionBatches(selected, listing, executables, secondsOf);
const source = process.env.GITHUB_SHA ?? null;
const observed = await executeBatches(batches, executables, {
  cwd: resolve(root, 'rust'),
  onOutput: (chunk) => process.stdout.write(chunk),
  onStart: (batch) => console.log(`coverage batch ${batch.index}: ${batch.names.length} ${batch.target} case(s), priority ${batch.weight}s`),
  onFinish: (batch) => console.log(`coverage batch ${batch.index}: completed ${batch.completed}/${batch.selected} in ${batch.elapsedSeconds.toFixed(3)}s, exit ${batch.exitCode}, certified ${batch.succeeded}`),
});
writeFileSync(resultPath, `${JSON.stringify({ source, profile: 'instrumented', ...observed }, null, 2)}\n`);
console.log(`coverage shard ${process.env.SHARD_INDEX}/${process.env.SHARD_TOTAL}: completed ${observed.completed}/${observed.selected} selected of ${listing.length} listed test(s)`);
process.exitCode = observed.succeeded ? 0 : 1;
