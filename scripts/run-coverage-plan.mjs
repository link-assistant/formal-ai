#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { inputSnapshots, createCoverageCheckpoints, assertFreshCoverageOutputs } from './lib/coverage-checkpoints.mjs';
import { batchTraceLine } from './lib/coverage-batch-trace.mjs';
import { pathToFileURL } from 'node:url';
import { executionBatches, executeBatches } from './lib/coverage-execution.mjs';

const [manifestPath, listingPath, planPath, resultPath] = process.argv.slice(2);
if (!resultPath) throw new Error('usage: run-coverage-plan.mjs manifest listing plan result');
if (!process.env.LLVM_PROFILE_FILE) throw new Error('LLVM_PROFILE_FILE is required');
const root = process.cwd();
const { parseTestDurations } = await import(pathToFileURL(resolve(root, 'scripts/lib/ci-speed-durations.mjs')));
const { durationLookup } = await import(pathToFileURL(resolve(root, 'scripts/lib/ci-speed-shards.mjs')));
const inputPaths = [manifestPath, listingPath, planPath,
  resolve(root, 'data/meta/test-durations.lino'), resolve(root, 'data/meta/coverage-test-weights.lino')];
const inputs = inputSnapshots(inputPaths);
const lines = (path) => inputs.get(path).toString('utf8').split('\n').filter(Boolean);
const manifest = lines(manifestPath);
const executables = new Map();
for (const line of manifest) {
  const columns = line.split('\t');
  if (columns.length !== 2 || !columns.every(Boolean)) throw new Error('invalid executable manifest row');
  const [target, path] = columns;
  if (executables.has(target)) throw new Error(`duplicate executable target: ${target}`);
  executables.set(target, resolve(root, path));
}
const native = parseTestDurations(inputs.get(inputPaths[3]).toString('utf8'));
const profile = parseTestDurations(inputs.get(inputPaths[4]).toString('utf8'));
const nativeSeconds = durationLookup(native, 0.092);
const secondsOf = (item) => Math.max(nativeSeconds(item), profile.get(item.slice(item.indexOf('\t') + 1)) ?? 0);
const listing = lines(listingPath);
const selected = lines(planPath);
const batches = executionBatches(selected, listing, executables, secondsOf);
const source = process.env.GITHUB_SHA ?? null;
const traceInputs = [manifestPath, listingPath, planPath,
  resolve(root, 'data/meta/test-durations.lino'), resolve(root, 'data/meta/coverage-test-weights.lino')];
const traceContext = { source, shardIndex: process.env.SHARD_INDEX ?? null,
  shardTotal: process.env.SHARD_TOTAL ?? null, runId: process.env.GITHUB_RUN_ID ?? null,
  runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
  inputs: traceInputs.map(path => ({ path,
    sha256: createHash('sha256').update(inputs.get(path)).digest('hex') })) };
const receiptDirectory = process.env.COVERAGE_EXECUTION_RECEIPTS ?? `${resultPath}.receipts`;
assertFreshCoverageOutputs(resultPath, receiptDirectory, inputs);
const checkpoints = createCoverageCheckpoints(
  receiptDirectory,
  inputs, traceContext, batches, executables);
const observed = await executeBatches(batches, executables, {
  cwd: resolve(root, 'rust'),
  onOutput: (chunk) => process.stdout.write(chunk),
  onStart: (batch) => {
    checkpoints.start(batch);
    console.log(`coverage batch ${batch.index}: ${batch.names.length} ${batch.target} case(s), priority ${batch.weight}s`);
  },
  onFinish: (batch) => {
    checkpoints.finish(batch);
    process.stdout.write(batchTraceLine(checkpoints.context, batch));
    console.log(`coverage batch ${batch.index}: completed ${batch.completed}/${batch.selected} in ${batch.elapsedSeconds.toFixed(3)}s, exit ${batch.exitCode}, certified ${batch.succeeded}`);
  },
});
writeFileSync(resultPath, `${JSON.stringify({ source, profile: 'instrumented', ...observed }, null, 2)}\n`, { flag: 'wx' });
console.log(`coverage shard ${process.env.SHARD_INDEX}/${process.env.SHARD_TOTAL}: completed ${observed.completed}/${observed.selected} selected of ${listing.length} listed test(s)`);
process.exitCode = observed.succeeded ? 0 : 1;
