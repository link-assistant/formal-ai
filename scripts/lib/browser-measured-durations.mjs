import fs from 'node:fs';
import path from 'node:path';
import { collectBrowserShards, contentDigest } from './browser-coverage-shards.mjs';

export function readBrowserDurationPackets(directory) {
  return fs.readdirSync(directory).sort().map(name => {
    const folder = path.join(directory, name);
    if (!fs.lstatSync(folder).isDirectory()) throw new Error('Unknown browser artifact folder');
    const names = fs.readdirSync(folder).sort();
    if (JSON.stringify(names) !== JSON.stringify(['coverage.info', 'durations.jsonl', 'receipt.json', 'stderr.txt', 'stdout.txt'])) {
      throw new Error('Unknown missing or extra browser duration artifact members');
    }
    for (const member of names) {
      if (!fs.lstatSync(path.join(folder, member)).isFile()) throw new Error('Unknown browser artifact member');
    }
    return { receipt: JSON.parse(fs.readFileSync(path.join(folder, 'receipt.json'), 'utf8')),
      coverage: fs.readFileSync(path.join(folder, 'coverage.info')),
      stdout: fs.readFileSync(path.join(folder, 'stdout.txt')),
      stderr: fs.readFileSync(path.join(folder, 'stderr.txt')),
      durations: fs.readFileSync(path.join(folder, 'durations.jsonl')) };
  });
}

export function measuredFileDurations(bytes, executionRoot, selected) {
  if (typeof executionRoot !== 'string' || !path.isAbsolute(executionRoot)) {
    throw new Error('Unknown browser duration execution root');
  }
  const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  if (!text.endsWith('\n')) throw new Error('Unknown truncated browser duration stream');
  const rows = text.trimEnd().split('\n').map(line => JSON.parse(line));
  const end = rows.pop();
  if (end?.schema !== 'browser-file-duration-stream/v1' || end.complete !== true) {
    throw new Error('Unknown incomplete browser duration stream');
  }
  const measured = new Map();
  for (const row of rows) {
    if (row.schema !== 'browser-file-duration/v1' || row.passed !== true || row.skipped !== false
        || row.todo !== false || !Number.isFinite(row.milliseconds) || row.milliseconds < 0
        || typeof row.file !== 'string' || !path.isAbsolute(row.file)) {
      throw new Error('Unknown failed or invalid browser file duration');
    }
    const relative = path.relative(executionRoot, row.file).split(path.sep).join('/');
    if (!selected.includes(relative) || measured.has(relative)
        || path.resolve(executionRoot, relative) !== row.file) {
      throw new Error('Unknown foreign or duplicate browser file duration');
    }
    measured.set(relative, row.milliseconds / 1000);
  }
  if (measured.size !== selected.length || new Set(selected).size !== selected.length) {
    throw new Error('Unknown missing browser file duration');
  }
  return measured;
}

export function deriveBrowserDurations(plan, identity, packets) {
  // Reuse every source/run/attempt/plan/exit/completion/raw-LCOV refusal.
  collectBrowserShards(plan, identity, packets);
  const measured = new Map();
  const receipts = [];
  let nodeVersion;
  for (const packet of [...packets].sort((left, right) => left.receipt.shard - right.receipt.shard)) {
    const receipt = packet.receipt;
    if (typeof receipt.nodeVersion !== 'string' || !/^v[0-9]+\.[0-9]+\.[0-9]+$/u.test(receipt.nodeVersion)
        || nodeVersion && nodeVersion !== receipt.nodeVersion) {
      throw new Error('Unknown mixed or invalid browser measurement runtime');
    }
    nodeVersion = receipt.nodeVersion;
    if (!identity.durationReporterDigest || receipt.durationReporterDigest !== identity.durationReporterDigest) {
      throw new Error('Unknown foreign duration reporter source');
    }
    if (!packet.durations || !receipt.durationsDigest
        || contentDigest(packet.durations) !== receipt.durationsDigest) {
      throw new Error('Unknown missing or counterfeit browser duration bytes');
    }
    const durations = measuredFileDurations(packet.durations, receipt.executionRoot, receipt.selected);
    for (const [file, seconds] of durations) {
      if (measured.has(file)) throw new Error('Unknown duplicate browser duration ownership');
      measured.set(file, seconds);
    }
    receipts.push({ shard: receipt.shard, durationsDigest: receipt.durationsDigest });
  }
  if (measured.size !== plan.inventory.length) throw new Error('Unknown incomplete browser duration census');
  const inventory = [...plan.inventory].sort((left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0);
  const lines = ['# Genuine instrumented file completion durations; never estimates.',
    'browser-test-durations', '  measurement instrumented-file-wall-seconds'];
  for (const key of ['source', 'run', 'attempt', 'sourceDigest', 'durationReporterDigest']) lines.push(`  ${key.replace(/[A-Z]/gu, value => '-' + value.toLowerCase())} ${JSON.stringify(identity[key])}`);
  lines.push(`  node-version ${JSON.stringify(nodeVersion)}`);
  lines.push(`  plan-digest ${JSON.stringify(plan.digest)}`);
  for (const receipt of receipts) lines.push(`  shard ${receipt.shard}\n    durations-digest ${JSON.stringify(receipt.durationsDigest)}`);
  for (const record of inventory) {
    lines.push(`  test ${JSON.stringify(record.path)}\n    source-sha256 ${JSON.stringify(record.sha256)}\n    seconds ${measured.get(record.path)}`);
  }
  return lines.join('\n') + '\n';
}

export function browserDurationStatus(plan, identity, packets) {
  try {
    return { status: 'Measured', table: deriveBrowserDurations(plan, identity, packets) };
  } catch (error) {
    return { status: 'Unknown', reason: String(error), source: identity.source,
      run: identity.run, attempt: identity.attempt };
  }
}
