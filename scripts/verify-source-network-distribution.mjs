#!/usr/bin/env node
// Join complete producer packets against the release checkout, with exact disjoint source coverage.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { ownedProductionSources, SOURCE_ROOTS } from './check-source-networks.mjs';
import { workflowPin } from './translate-js-rust.mjs';
import { verifySourceDistribution } from './lib/source-network-packets.mjs';

const args = process.argv.slice(2);
const value = (name, fallback) => args.includes(name) ? args[args.indexOf(name) + 1] : fallback;
const root = resolve(value('--source-root', '') || execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim());
const directory = resolve(process.argv[2] ?? '');
if (!process.argv[2]) throw new Error('an extracted source-network distribution directory is required');
const dirty = execFileSync('git', ['diff', '--name-only', 'HEAD', '--', ...SOURCE_ROOTS],
  { cwd: root, encoding: 'utf8' }).trim();
if (dirty) throw new Error('distribution must be checked against a committed source tree');
const sourceHead = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
const upstreamCommit = value('--upstream-commit', '') || workflowPin(readFileSync(join(root, '.github/workflows/layered-ci.yml'), 'utf8'));
const shardCount = Number(value('--shard-count', '8'));
if (!Number.isSafeInteger(shardCount) || shardCount < 1) throw new Error('invalid distribution shard count');
const reports = Array.from({ length: shardCount }, (_, index) => JSON.parse(
  readFileSync(join(directory, 'shard-' + index, 'source-network-receipts.json'), 'utf8')));
const result = verifySourceDistribution({
  reports, sourcePaths: ownedProductionSources(root), sourceHead, upstreamCommit, shardCount,
  requireReadableCst: true,
  sourceBytes: (path) => readFileSync(join(root, path)),
  packetBytes: (index, path) => readFileSync(join(directory, 'shard-' + index, path)),
});
console.log(JSON.stringify(result));
