import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { registeredRustTestFilters } from '../../scripts/lib/ladder-test-projection.mjs';

const [, , leavesPath] = process.argv;
if (!leavesPath) throw new Error('Usage: node probe-ladder-source-targets.mjs <leaves.tsv>');

const paths = new Map();
for (const line of readFileSync(leavesPath, 'utf8').split('\n').filter(Boolean)) {
  const [leaf, , source] = line.split('\t');
  if (!source?.startsWith('rust/')) continue;
  const leaves = paths.get(source) ?? [];
  leaves.push(leaf);
  paths.set(source, leaves);
}

const root = resolve('.');
for (const [source, leaves] of paths) {
  try {
    const selection = await registeredRustTestFilters(root, source);
    console.log(JSON.stringify({ source, leaves, jobs: selection.jobs, ignored: selection.ignoredJobs }));
  } catch (error) {
    console.log(JSON.stringify({ source, leaves, gap: error.message }));
  }
}
