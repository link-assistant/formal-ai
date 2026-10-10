import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { planShards, partitionProblems } from './ci-speed-shards.mjs';

export const contentDigest = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
export const browserShardCount = 6;

export function browserInventory(root) {
  return fs.readdirSync(path.join(root, 'rust/tests/web'))
    .filter((name) => name.endsWith('.test.mjs'))
    .sort()
    .map((name) => {
      const relative = `rust/tests/web/${name}`;
      const bytes = fs.readFileSync(path.join(root, relative));
      return { path: relative, sha256: contentDigest(bytes), bytes: bytes.length };
    });
}

export function browserSourceInventory(root) {
  const records = [];
  const walk = (directory) => {
    for (const name of fs.readdirSync(path.join(root, directory)).sort()) {
      const relative = `${directory}/${name}`;
      const physical = path.join(root, relative);
      const status = fs.lstatSync(physical);
      if (status.isSymbolicLink()) throw new Error('browser source inventory contains a symlink');
      if (status.isDirectory()) walk(relative);
      else if (/\.(?:mjs|js|jsx|ts|lino)$/.test(name)) {
        const bytes = fs.readFileSync(physical);
        records.push({ path: relative, sha256: contentDigest(bytes), bytes: bytes.length });
      }
    }
  };
  walk('js');
  walk('data/seed');
  return records;
}

export function browserPlan(inventory, count = browserShardCount) {
  if (!Array.isArray(inventory) || inventory.length === 0) throw new Error('browser test inventory is empty');
  const names = inventory.map((item) => item.path);
  if (new Set(names).size !== names.length) throw new Error('duplicate browser test path');
  const { shards } = planShards(names, count, () => 1);
  const problems = partitionProblems(names, shards);
  if (Object.values(problems).some((items) => items.length)) throw new Error('browser test partition is incomplete');
  if (shards.some((items) => items.length === 0)) throw new Error('empty browser test shard');
  return { inventory, shards, digest: contentDigest(JSON.stringify({ inventory, shards })) };
}

export function completedTestSummary(output) {
  const summary = {};
  const expression = /^(?:ℹ|#) (tests|pass|fail|cancelled|skipped|todo|duration_ms) ([0-9]+(?:\.[0-9]+)?)(?:\s|$)/gm;
  for (const match of output.matchAll(expression)) summary[match[1]] = Number(match[2]);
  for (const key of ['tests', 'pass', 'fail', 'cancelled', 'skipped', 'todo', 'duration_ms']) {
    if (!Number.isFinite(summary[key])) throw new Error(`missing browser test summary ${key}`);
  }
  if (summary.tests <= 0 || summary.fail !== 0 || summary.cancelled !== 0
      || summary.pass + summary.skipped + summary.todo !== summary.tests) {
    throw new Error('browser test run did not complete successfully');
  }
  return summary;
}

export function completeCoverageRecords(text) {
  if (!text || !text.endsWith('\n')) throw new Error('browser LCOV report is empty or truncated');
  let active = false;
  let records = 0;
  for (const line of text.split('\n')) {
    if (line.startsWith('SF:')) {
      if (active || line.length === 3) throw new Error('invalid browser LCOV source boundary');
      active = true;
    } else if (line === 'end_of_record') {
      if (!active) throw new Error('unowned browser LCOV record boundary');
      active = false;
      records += 1;
    }
  }
  if (active || records === 0) throw new Error('browser LCOV report has incomplete records');
  return records;
}

export function collectBrowserShards(plan, identity, packets) {
  if (packets.length !== plan.shards.length) throw new Error('browser shard receipts are missing or extra');
  const ordered = Array(plan.shards.length);
  for (const packet of packets) {
    const { receipt, coverage, stdout, stderr } = packet;
    const index = receipt.shard - 1;
    if (!Number.isInteger(index) || index < 0 || index >= ordered.length || ordered[index]) {
      throw new Error('duplicate or invalid browser shard index');
    }
    for (const key of ['source', 'run', 'attempt', 'sourceDigest']) {
      if (!identity[key] || receipt[key] !== identity[key]) throw new Error(`foreign browser shard ${key}`);
    }
    if (receipt.planDigest !== plan.digest || receipt.total !== plan.shards.length
        || JSON.stringify(receipt.selected) !== JSON.stringify(plan.shards[index])) {
      throw new Error('browser shard inventory or plan differs');
    }
    if (receipt.exitCode !== 0 || receipt.signal !== null || receipt.sourceUnchanged !== true) {
      throw new Error('browser shard process did not complete successfully');
    }
    for (const [key, bytes] of [['coverageDigest', coverage], ['stdoutDigest', stdout], ['stderrDigest', stderr]]) {
      if (receipt[key] !== contentDigest(bytes)) throw new Error(`browser shard ${key} differs`);
    }
    const decodedOutput = new TextDecoder('utf-8', { fatal: true }).decode(stdout);
    const summary = completedTestSummary(decodedOutput);
    if (JSON.stringify(summary) !== JSON.stringify(receipt.summary)) throw new Error('browser test summary differs');
    completeCoverageRecords(new TextDecoder('utf-8', { fatal: true }).decode(coverage));
    ordered[index] = packet;
  }
  if (ordered.some((packet) => !packet)) throw new Error('browser shard receipt is absent');
  // The maintained ratchet unions repeated SF/DA/FN/FNDA records. Keeping
  // every raw record also preserves cold declarations and all denominators.
  return Buffer.concat(ordered.map((packet) => packet.coverage));
}
