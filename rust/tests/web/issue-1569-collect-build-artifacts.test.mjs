// Issue #1569: Rust 1.100 moved Cargo's test executables from
// `target/release/deps/<name>-<hash>` to the build-dir layout
// `target/release/build/<package>/<hash>/out/<name>-<hash>`, and every main
// pipeline failed in `build-artifacts`. scripts/collect-build-artifacts.sh now
// takes each executable from Cargo's `--message-format=json` report and, with
// no report, searches both layouts. These cases run the actual script against
// a scratch tree; nothing is compiled.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';

import { boundedCommand } from '../../../js/server/bounded-command.mjs';

const SCRIPT = path.resolve('scripts/collect-build-artifacts.sh');
const TARGETS = ['unit', 'integration', 'source'];

function scratch(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-collect-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

function executable(root, relative, body, mtimeSeconds) {
  const file = path.join(root, relative);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, body, { mode: 0o755 });
  if (mtimeSeconds !== undefined) fs.utimesSync(file, mtimeSeconds, mtimeSeconds);
  return file;
}

function newLayout(target, hash) {
  return `rust/target/release/build/formal-ai/${hash}/out/${target}-${hash}`;
}

function cargoReport(root, entries) {
  const rows = entries.map(([target, relative]) => ({
    reason: 'compiler-artifact', target: { name: target, kind: ['test'] },
    profile: { test: true }, executable: path.join(root, relative),
  }));
  rows.push({ reason: 'build-finished', success: true });
  const file = path.join(root, 'native-build-messages.jsonl');
  fs.writeFileSync(file, rows.map((row) => JSON.stringify(row)).join('\n') + '\n');
  return file;
}

async function collect(root, argumentsValue = [], env = {}) {
  const result = await boundedCommand('bash', [SCRIPT, ...argumentsValue], {
    cwd: root, timeout: 30000, maxBuffer: 1024 * 1024,
    env: { PATH: process.env.PATH, HOME: process.env.HOME, ...env },
  });
  assert.equal(result.error, undefined, String(result.stderr));
  assert.equal(result.cleanup_complete, true);
  return { status: result.status, stdout: String(result.stdout), stderr: String(result.stderr) };
}

const copied = (root, target) => fs.readFileSync(path.join(root, 'dist/tests', target), 'utf8');

test('Cargo report selects the Rust 1.100 build-dir executables over a newer stale hash', async (t) => {
  const root = scratch(t);
  executable(root, 'rust/target/release/formal-ai', 'binary');
  const entries = TARGETS.map((target) => [target, newLayout(target, 'c5518f19ec441de8')]);
  for (const [target, relative] of entries) {
    executable(root, relative, `current ${target}`, 1_000_000);
    // A stale hash restored from cache, newer on disk than this build's.
    executable(root, newLayout(target, '0000000000000000'), `stale ${target}`, 2_000_000);
  }
  const report = cargoReport(root, entries);
  const result = await collect(root, ['dist', report], { FORMAL_AI_COLLECT_DEBUG: '1' });
  assert.equal(result.status, 0, result.stderr);
  for (const target of TARGETS) assert.equal(copied(root, target), `current ${target}`);
  assert.equal(fs.readFileSync(path.join(root, 'dist/formal-ai'), 'utf8'), 'binary');
  assert.match(result.stderr, /unit: Cargo reported .*build\/formal-ai\/c5518f19ec441de8\/out\/unit-c5518f19ec441de8/u);
  assert.match(result.stdout, /collected 3 test executables/u);
});

test('the report is found under RUNNER_TEMP where the build step writes it', async (t) => {
  const root = scratch(t);
  const entries = TARGETS.map((target) => [target, newLayout(target, 'd7589adc145d7f21')]);
  for (const [target, relative] of entries) executable(root, relative, `reported ${target}`);
  cargoReport(root, entries);
  const result = await collect(root, [], { RUNNER_TEMP: root });
  assert.equal(result.status, 0, result.stderr);
  for (const target of TARGETS) assert.equal(copied(root, target), `reported ${target}`);
});

test('without a report the newest executable is found in either layout', async (t) => {
  for (const layout of ['old', 'new']) {
    const root = scratch(t);
    for (const target of TARGETS) {
      const place = (hash) => (layout === 'old' ? `rust/target/release/deps/${target}-${hash}` : newLayout(target, hash));
      executable(root, place('1111111111111111'), `older ${target}`, 1_000_000);
      executable(root, place('2222222222222222'), `newer ${target}`, 2_000_000);
      if (layout === 'old') fs.writeFileSync(path.join(root, place('2222222222222222') + '.d'), 'deps');
    }
    const result = await collect(root);
    assert.equal(result.status, 0, `${layout}: ${result.stderr}`);
    for (const target of TARGETS) assert.equal(copied(root, target), `newer ${target}`, layout);
  }
});

test('a missing test executable still fails loudly with an annotation', async (t) => {
  const root = scratch(t);
  executable(root, newLayout('unit', 'aaaaaaaaaaaaaaaa'), 'unit');
  executable(root, newLayout('integration', 'aaaaaaaaaaaaaaaa'), 'integration');
  // A same-named file outside both layouts is not a Cargo test executable.
  executable(root, 'rust/target/release/build/formal-ai/aaaaaaaaaaaaaaaa/source-aaaaaaaaaaaaaaaa', 'misplaced');
  const result = await collect(root);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /::error title=Missing test executable::.*'source'/su);
});
