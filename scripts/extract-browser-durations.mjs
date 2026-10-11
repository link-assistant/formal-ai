#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { browserInventory, browserSourceInventory, browserPlan, browserDurationModel,
  browserShardCount, contentDigest } from './lib/browser-coverage-shards.mjs';
import { browserDurationStatus, readBrowserDurationPackets } from './lib/browser-measured-durations.mjs';

const [mode, directory, output] = process.argv.slice(2);
if (!['--write', '--check'].includes(mode) || !directory || !output) {
  throw new Error('usage: extract-browser-durations.mjs --write|--check shard-directory table-path');
}
const root = process.cwd();
const identity = { source: process.env.GITHUB_SHA, run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT };
if (Object.values(identity).some(value => !value)) throw new Error('Unknown actual workflow identity');
const actual = spawnSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' });
if (actual.status !== 0 || actual.stdout.trim() !== identity.source) throw new Error('Unknown stale checkout');
identity.sourceDigest = contentDigest(JSON.stringify(browserSourceInventory(root)));
identity.durationReporterDigest = contentDigest(fs.readFileSync(path.join(root, 'scripts/browser-duration-reporter.mjs')));
const plan = browserPlan(browserInventory(root), browserShardCount, browserDurationModel(root));
let status;
try {
  const packets = readBrowserDurationPackets(directory);
  status = browserDurationStatus(plan, identity, packets);
} catch (error) {
  status = { status: 'Unknown', reason: String(error), ...identity };
}
if (status.status === 'Unknown') {
  // Preserve the previous reviewed table. Record refusal, never fill gaps.
  console.error(JSON.stringify(status));
  process.exitCode = 1;
} else if (mode === '--check') {
  if (fs.readFileSync(output, 'utf8') !== status.table) throw new Error('measured browser duration table drift');
  console.log('complete source-bound browser duration table agrees');
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, status.table);
  console.log('complete source-bound browser duration table written');
}
