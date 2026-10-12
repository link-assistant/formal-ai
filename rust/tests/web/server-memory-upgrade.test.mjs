// The explicit persisted-memory upgrade of the JavaScript server
// (`node js/server/main.mjs memory upgrade-status|migrate`,
// js/server/memory-upgrade.mjs), case for case against
// rust/tests/integration/issue_982_memory_upgrade.rs over the same schema
// fixtures (issue #982: R982-1 to R982-9).

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { exportLinksNotation, memoryEvent, parseLinksNotation, preflightMemoryUpgrade } from '../../../js/server/memory-store.mjs';
import { holdExclusiveLock, memoryLockPath, migrateMemory } from '../../../js/server/memory-upgrade.mjs';
import { packageVersion } from '../../../js/server/seed.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const MAIN = path.join(HERE, '..', '..', '..', 'js', 'server', 'main.mjs');
const FIXTURES = path.join(HERE, '..', 'fixtures', 'memory');
const RELEASED = fs.readFileSync(path.join(FIXTURES, 'schema-1.lino'), 'utf8');
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const fixtureDir = (name) => fs.mkdtempSync(path.join(os.tmpdir(), `formal-ai-issue-982-js-cli-${name}-`));

/** `run_memory_command`: `formal-ai memory <arguments>`, through the JavaScript server's CLI. */
function memoryCommand(...args) {
  const result = spawnSync(process.execPath, [MAIN, 'memory', ...args], { encoding: 'utf8' });
  return { status: result.status, stdout: result.stdout, stderr: result.stderr, json: () => JSON.parse(result.stdout) };
}

test('fixtures cover every readable schema', () => {
  for (const expected of [1, 2]) {
    const dir = fixtureDir(`schema-${expected}`);
    const file = path.join(dir, 'memory.lino');
    fs.copyFileSync(path.join(FIXTURES, `schema-${expected}.lino`), file);
    const output = memoryCommand('upgrade-status', '--path', file, '--format', 'json');
    assert.equal(output.status, 0);
    const status = output.json();
    assert.equal(status.detected_schema_version, expected);
    assert.equal(status.compatible, true);
    assert.equal(status.migration_required, expected < status.target_schema_version);
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('upgrade-status prints the full machine-readable status and mutates nothing (R982-1, R982-2)', () => {
  const dir = fixtureDir('preflight');
  const file = path.join(dir, 'memory.lino');
  const released = 'demo_memory\n  event "event-1"\n    role "user"\n    content "keep me"\n    futureField "preserve me"\n';
  fs.writeFileSync(file, released);
  const output = memoryCommand('upgrade-status', '--path', file, '--format', 'json');
  assert.equal(output.status, 0, output.stderr);
  assert.equal(output.stdout, `${JSON.stringify({
    binary_version: packageVersion(),
    path_exists: true,
    detected_schema_version: 1,
    minimum_readable_schema_version: 1,
    maximum_readable_schema_version: 2,
    target_schema_version: 2,
    compatible: true,
    migration_required: true,
    migration_id: 'demo_memory_v1_to_v2',
    rollback_supported: true,
    migration_state: 'upgrade_required',
    event_count: 1,
    source_sha256: sha256(released),
  }, null, 2)}\n`);
  assert.equal(fs.readFileSync(file, 'utf8'), released);
  assert.deepEqual(fs.readdirSync(dir), ['memory.lino'], 'preflight creates no lock, backup, temp file or receipt');
  fs.rmSync(dir, { recursive: true, force: true });
});

test('released writer escapes are valid upgrade input and round-trip losslessly', async () => {
  const dir = fixtureDir('released-escapes');
  const file = path.join(dir, 'memory.lino');
  const backup = path.join(dir, 'rollback.lino');
  const inputs = '{"command":"set -eu\\nprintf \'%s\\\\n\' \\"migration canary\\"","description":""}';
  const content = 'quotes: \\"double\\" and \\ slash';
  const released = exportLinksNotation([memoryEvent({ id: 'released-tool-event', kind: 'tool_call', inputs, content })], 1);
  fs.writeFileSync(file, released);
  const status = preflightMemoryUpgrade(file);
  assert.equal(status.compatible, true);
  assert.equal(status.detected_schema_version, 1);
  assert.equal(status.event_count, 1);
  await migrateMemory(file, backup);
  assert.equal(fs.readFileSync(backup, 'utf8'), released);
  const migrated = fs.readFileSync(file, 'utf8');
  assert.equal(migrated, released.replace('demo_memory\n', 'demo_memory\n  schema_version "2"\n'));
  const [event] = parseLinksNotation(migrated);
  assert.equal(event.inputs, inputs);
  assert.equal(event.content, content);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('upgrade-status for a missing path creates nothing', () => {
  const root = fixtureDir('missing');
  fs.rmSync(root, { recursive: true, force: true });
  const output = memoryCommand('upgrade-status', '--path', path.join(root, 'nested', 'memory.lino'), '--format', 'json');
  assert.equal(output.status, 0);
  const status = output.json();
  assert.equal(status.path_exists, false);
  assert.equal(status.compatible, true);
  assert.equal(status.migration_required, false);
  assert.equal(status.migration_state, 'missing');
  assert.equal(status.event_count, 0);
  assert.equal(status.source_sha256, null);
  assert.equal(fs.existsSync(root), false, 'preflight must not create the parent tree');
});

test('a released zero-byte store is readable and upgradeable', async () => {
  const dir = fixtureDir('empty-released-store');
  const file = path.join(dir, 'memory.lino');
  const backup = path.join(dir, 'rollback.lino');
  fs.writeFileSync(file, '');
  const status = preflightMemoryUpgrade(file);
  assert.equal(status.detected_schema_version, 1);
  assert.equal(status.event_count, 0);
  assert.equal(status.compatible, true);
  assert.equal(status.migration_required, true);
  assert.equal((await migrateMemory(file, backup)).changed, true);
  assert.equal(fs.readFileSync(backup, 'utf8'), '');
  assert.equal(fs.readFileSync(file, 'utf8'), 'demo_memory\n  schema_version "2"\n');
  fs.rmSync(dir, { recursive: true, force: true });
});

test('migration is atomic, lossless, receipted, idempotent and rollback-safe (R982-5 to R982-8)', () => {
  const dir = fixtureDir('whole-flow');
  const file = path.join(dir, 'memory.lino');
  const backup = path.join(dir, 'rollback.lino');
  const receiptFile = path.join(dir, 'receipt.json');
  fs.writeFileSync(file, RELEASED);
  if (process.platform !== 'win32') fs.chmodSync(file, 0o600);

  const output = memoryCommand('migrate', '--path', file, '--backup', backup, '--receipt', receiptFile, '--format', 'json');
  assert.equal(output.status, 0, output.stderr);
  const migrated = fs.readFileSync(file);
  assert.deepEqual(output.json(), {
    binary_version: packageVersion(),
    changed: true,
    migration_id: 'demo_memory_v1_to_v2',
    from_schema_version: 1,
    to_schema_version: 2,
    memory_path: file,
    backup_path: backup,
    receipt_path: receiptFile,
    original_sha256: sha256(RELEASED),
    migrated_sha256: sha256(migrated),
    event_count: 2,
    rollback_supported: true,
    rollback_strategy: 'restore_backup_path',
  });
  assert.equal(fs.readFileSync(receiptFile, 'utf8'), output.stdout.trimEnd(), 'the durable receipt equals stdout');
  assert.equal(fs.readFileSync(backup, 'utf8'), RELEASED);
  if (process.platform !== 'win32') {
    assert.equal(fs.statSync(file).mode & 0o777, 0o600, 'atomic replacement keeps the memory permissions');
    assert.equal(fs.statSync(backup).mode & 0o777, 0o600, 'the backup keeps the memory permissions');
  }
  assert.equal(migrated.toString('utf8'), RELEASED.replace('demo_memory\n', 'demo_memory\n  schema_version "2"\n'));
  const events = parseLinksNotation(migrated.toString('utf8'));
  assert.deepEqual(events.map((event) => event.id), ['event-1', 'event-2']);
  assert.deepEqual(events[0].unknown_fields, [['futureField', 'preserve me']]);
  assert.deepEqual(events[1].evidence, ['source-a', 'source-b']);

  const retry = memoryCommand('migrate', '--path', file, '--backup', backup, '--receipt', receiptFile, '--format', 'json');
  assert.equal(retry.status, 0);
  assert.equal(retry.json().changed, false);
  assert.equal(retry.json().rollback_strategy, 'not_required');
  assert.deepEqual(fs.readFileSync(file), migrated, 'a second migration is a no-op');

  fs.copyFileSync(backup, file);
  const rolledBack = memoryCommand('upgrade-status', '--path', file, '--format', 'json').json();
  assert.equal(rolledBack.detected_schema_version, 1);
  assert.equal(rolledBack.event_count, 2);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('an interrupted migration keeps the original byte-identical and is retryable (R982-6)', async () => {
  const dir = fixtureDir('interrupted');
  const file = path.join(dir, 'memory.lino');
  const backup = path.join(dir, 'rollback.lino');
  const receiptFile = path.join(dir, 'receipt.json');
  fs.writeFileSync(file, RELEASED);
  await assert.rejects(migrateMemory(file, backup, receiptFile, (staged) => {
    assert.ok(fs.readFileSync(staged, 'utf8').includes('schema_version "2"'));
    throw new Error('simulated stop');
  }), (error) => error.code === 'migration_interrupted' && error.message === 'migration_interrupted:error=simulated stop');
  assert.equal(fs.readFileSync(file, 'utf8'), RELEASED);
  assert.equal(fs.readFileSync(backup, 'utf8'), RELEASED);
  assert.equal(fs.existsSync(receiptFile), false);
  assert.ok(fs.readdirSync(dir).every((name) => !name.includes('.migration.') && !name.includes('.receipt.')));
  assert.equal((await migrateMemory(file, backup, receiptFile)).changed, true);
  assert.equal(fs.existsSync(receiptFile), true);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('a conflicting backup is refused and the default backup is content-addressed (R982-5, R982-6)', async () => {
  const dir = fixtureDir('backup-contract');
  const file = path.join(dir, 'memory.lino');
  fs.writeFileSync(file, RELEASED);
  const conflicting = path.join(dir, 'conflicting.lino');
  fs.writeFileSync(conflicting, 'demo_memory\n');
  await assert.rejects(migrateMemory(file, conflicting), (error) => error.code === 'backup_conflict');
  assert.equal(fs.readFileSync(file, 'utf8'), RELEASED);
  assert.equal(fs.readFileSync(conflicting, 'utf8'), 'demo_memory\n');
  fs.rmSync(conflicting);

  const receipt = await migrateMemory(file);
  assert.equal(receipt.backup_path, path.join(dir, `memory.lino.schema-1.${sha256(RELEASED).slice(0, 12)}.backup`));
  assert.equal(receipt.receipt_path, path.join(dir, 'memory.lino.upgrade-receipt.json'));
  assert.equal(fs.readFileSync(receipt.backup_path, 'utf8'), RELEASED);
  assert.equal((await migrateMemory(file)).changed, false);
  assert.equal(fs.readdirSync(dir).filter((name) => name.endsWith('.backup')).length, 1, 'a retry writes no second backup');
  fs.rmSync(dir, { recursive: true, force: true });
});

test('a live writer lock causes a machine-readable refusal without modification (R982-4)', async () => {
  const dir = fixtureDir('locked');
  const file = path.join(dir, 'memory.lino');
  fs.writeFileSync(file, RELEASED);
  const lock = await holdExclusiveLock(memoryLockPath(file));
  try {
    const output = memoryCommand('migrate', '--path', file, '--format', 'json');
    assert.equal(output.status, 1, 'a live writer must block migration');
    const refusal = output.json();
    assert.equal(refusal.error.code, 'memory_locked');
    assert.equal(refusal.status.detected_schema_version, 1);
    assert.equal(output.stderr, 'Error: "persisted-memory migration refused to modify the file"\n');
    assert.equal(fs.readFileSync(file, 'utf8'), RELEASED);
  } finally {
    await lock.release();
  }
  fs.rmSync(dir, { recursive: true, force: true });
});

test('a future schema is refused nonzero and never modified (R982-4, R982-9)', () => {
  const dir = fixtureDir('future');
  const file = path.join(dir, 'memory.lino');
  const future = 'demo_memory\n  schema_version "99"\n  event "future"\n    content "untouched"\n';
  fs.writeFileSync(file, future);
  const preflight = memoryCommand('upgrade-status', '--path', file, '--format', 'json');
  assert.equal(preflight.status, 1);
  assert.equal(preflight.stderr, 'Error: "persisted-memory preflight refused an incompatible file"\n');
  const status = preflight.json();
  assert.equal(status.detected_schema_version, 99);
  assert.equal(status.compatible, false);
  assert.equal(status.refusal_code, 'schema_too_new');
  assert.equal(status.refusal_reason, 'schema_too_new:detected=99:maximum=2');
  const migration = memoryCommand('migrate', '--path', file, '--format', 'json');
  assert.equal(migration.status, 1);
  const refusal = migration.json();
  assert.equal(refusal.error.code, 'memory_incompatible');
  assert.equal(refusal.status.refusal_code, 'schema_too_new');
  assert.equal(fs.readFileSync(file, 'utf8'), future);
  assert.equal(fs.existsSync(path.join(dir, 'memory.lino.upgrade-receipt.json')), false);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('malformed memory is refused nonzero and never modified', () => {
  const dir = fixtureDir('malformed');
  const file = path.join(dir, 'memory.lino');
  const malformed = 'demo_memory\n  event "broken"\n    content "unterminated\n';
  fs.writeFileSync(file, malformed);
  const preflight = memoryCommand('upgrade-status', '--path', file, '--format', 'json');
  assert.equal(preflight.status, 1);
  assert.equal(preflight.json().refusal_code, 'memory_malformed');
  assert.equal(preflight.json().refusal_reason, 'memory_malformed:error=line=3:event_value_invalid');
  const migration = memoryCommand('migrate', '--path', file, '--format', 'json');
  assert.equal(migration.status, 1);
  assert.equal(migration.json().error.code, 'memory_incompatible');
  assert.equal(fs.readFileSync(file, 'utf8'), malformed);
  fs.rmSync(dir, { recursive: true, force: true });
});
