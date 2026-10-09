#!/usr/bin/env node
// Join complete producer packets against the release checkout, with exact disjoint source coverage.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { ownedProductionSources } from './check-source-networks.mjs';
import { workflowPin } from './translate-js-rust.mjs';
import { verifySourceDistribution } from './lib/source-network-packets.mjs';

const root = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
const directory = resolve(process.argv[2] ?? '');
if (!process.argv[2]) throw new Error('an extracted source-network distribution directory is required');
const dirty = execFileSync('git', ['diff', '--name-only', 'HEAD', '--', 'rust/src', 'js', 'ts'],
  { cwd: root, encoding: 'utf8' }).trim();
if (dirty) throw new Error('distribution must be checked against a committed source tree');
const sourceHead = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
const upstreamCommit = workflowPin(readFileSync(join(root, '.github/workflows/layered-ci.yml'), 'utf8'));
const reports = Array.from({ length: 8 }, (_, index) => JSON.parse(
  readFileSync(join(directory, 'shard-' + index, 'source-network-receipts.json'), 'utf8')));
const result = verifySourceDistribution({
  reports, sourcePaths: ownedProductionSources(root), sourceHead, upstreamCommit,
  sourceBytes: (path) => readFileSync(join(root, path)),
  packetBytes: (index, path) => readFileSync(join(directory, 'shard-' + index, path)),
});
console.log(JSON.stringify(result));
