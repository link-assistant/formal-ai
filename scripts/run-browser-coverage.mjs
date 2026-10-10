#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import {
  browserInventory, browserSourceInventory, browserPlan, browserShardCount, collectBrowserShards,
  completedTestSummary, completeCoverageRecords, contentDigest,
} from './lib/browser-coverage-shards.mjs';

const root = process.cwd();
const childEnvironment = { ...process.env };
delete childEnvironment.NODE_TEST_CONTEXT;
const [mode, indexText, directoryText] = process.argv.slice(2);
if (mode === undefined) {
  const destination = path.join(root, 'coverage/browser-lcov.info');
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  const observed = spawnSync(process.execPath, [
    '--test', '--experimental-test-coverage',
    '--test-reporter=lcov', '--test-reporter-destination=coverage/browser-lcov.info',
    '--test-reporter=spec', '--test-reporter-destination=stdout',
    ...browserInventory(root).map((item) => item.path),
  ], { cwd: root, env: childEnvironment, stdio: 'inherit' });
  if (observed.error) throw observed.error;
  if (observed.signal !== null) throw new Error('whole browser coverage was interrupted');
  process.exit(observed.status ?? 1);
}
const directory = path.resolve(root, mode === 'run' ? directoryText : indexText);
const identity = {
  source: process.env.GITHUB_SHA,
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
};
if (Object.values(identity).some((value) => !value)) throw new Error('browser coverage requires actual workflow source/run/attempt');
const actualSource = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' });
if (actualSource.status !== 0 || actualSource.stdout.trim() !== identity.source) throw new Error('browser coverage checkout differs from workflow source');
const sourceInventory = browserSourceInventory(root);
identity.sourceDigest = contentDigest(JSON.stringify(sourceInventory));
const inventory = browserInventory(root);
const plan = browserPlan(inventory);

if (mode === 'run') {
  const index = Number(indexText);
  if (!Number.isInteger(index) || index < 1 || index > browserShardCount) throw new Error('invalid browser coverage shard');
  fs.mkdirSync(directory, { recursive: true });
  const coveragePath = path.join(directory, 'coverage.info');
  const stdoutPath = path.join(directory, 'stdout.txt');
  const stderrPath = path.join(directory, 'stderr.txt');
  for (const target of [coveragePath, stdoutPath, stderrPath, path.join(directory, 'receipt.json')]) {
    if (fs.existsSync(target)) throw new Error('browser shard destination already exists');
  }
  const argumentsList = [
    '--test', '--experimental-test-coverage',
    '--test-reporter=lcov', `--test-reporter-destination=${coveragePath}`,
    '--test-reporter=spec', '--test-reporter-destination=stdout',
    ...plan.shards[index - 1].map((relative) => path.join(root, relative)),
  ];
  const stdoutDescriptor = fs.openSync(stdoutPath, 'wx');
  const stderrDescriptor = fs.openSync(stderrPath, 'wx');
  const child = spawn(process.execPath, argumentsList, { cwd: root, env: childEnvironment, stdio: ['ignore', 'pipe', 'pipe'] });
  child.stdout.on('data', (bytes) => { fs.writeSync(stdoutDescriptor, bytes); process.stdout.write(bytes); });
  child.stderr.on('data', (bytes) => { fs.writeSync(stderrDescriptor, bytes); process.stderr.write(bytes); });
  const status = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('close', (exitCode, signal) => resolve({ exitCode, signal }));
  });
  fs.closeSync(stdoutDescriptor);
  fs.closeSync(stderrDescriptor);
  const stdout = fs.readFileSync(stdoutPath);
  const stderr = fs.readFileSync(stderrPath);
  const sourceUnchanged = JSON.stringify(browserInventory(root)) === JSON.stringify(inventory)
    && JSON.stringify(browserSourceInventory(root)) === JSON.stringify(sourceInventory);
  let summary = null;
  let coverage = null;
  let validationError = null;
  try {
    if (status.exitCode !== 0 || status.signal !== null || !sourceUnchanged) throw new Error('browser shard failed or mutated its source inventory');
    summary = completedTestSummary(new TextDecoder('utf-8', { fatal: true }).decode(stdout));
    coverage = fs.readFileSync(coveragePath);
    completeCoverageRecords(new TextDecoder('utf-8', { fatal: true }).decode(coverage));
  } catch (error) {
    validationError = String(error);
  }
  const receipt = {
    ...identity, shard: index, total: browserShardCount,
    planDigest: plan.digest, selected: plan.shards[index - 1], inventory,
    ...status, sourceUnchanged, summary, validationError,
    coverageDigest: coverage ? contentDigest(coverage) : null,
    stdoutDigest: contentDigest(stdout), stderrDigest: contentDigest(stderr),
    nodeVersion: process.version, executable: process.execPath, arguments: argumentsList,
  };
  fs.writeFileSync(path.join(directory, 'receipt.json'), `${JSON.stringify(receipt, null, 2)}\n`, { flag: 'wx' });
  if (validationError) { console.error(validationError); process.exitCode = 1; }
} else if (mode === 'collect') {
  const packets = fs.readdirSync(directory).map((name) => {
    const folder = path.join(directory, name);
    if (!fs.statSync(folder).isDirectory()) throw new Error('unexpected browser shard artifact entry');
    const names = fs.readdirSync(folder).sort();
    if (JSON.stringify(names) !== JSON.stringify(['coverage.info', 'receipt.json', 'stderr.txt', 'stdout.txt'])) {
      throw new Error('browser shard artifact members are missing or extra');
    }
    return {
      receipt: JSON.parse(fs.readFileSync(path.join(folder, 'receipt.json'), 'utf8')),
      coverage: fs.readFileSync(path.join(folder, 'coverage.info')),
      stdout: fs.readFileSync(path.join(folder, 'stdout.txt')),
      stderr: fs.readFileSync(path.join(folder, 'stderr.txt')),
    };
  });
  const merged = collectBrowserShards(plan, identity, packets);
  fs.mkdirSync(path.join(root, 'coverage'), { recursive: true });
  fs.writeFileSync(path.join(root, 'coverage/browser-lcov.info'), merged);
  fs.writeFileSync(path.join(root, 'coverage/browser-shard-manifest.json'), `${JSON.stringify({ ...identity, inventory, planDigest: plan.digest, shards: packets.map((packet) => packet.receipt) }, null, 2)}\n`);
} else {
  throw new Error('usage: run-browser-coverage.mjs run shard destination | collect artifact-directory');
}
