// The persisted-memory compatibility contract the JavaScript server keeps
// (js/server/memory-store.mjs `inspectMemoryBytes`, `preflightMemory`,
// `SyncStore`; js/server/memory.mjs `memoryHealthStatus`), case for case
// against rust/tests/integration/issue_982_memory_upgrade.rs (issue #982:
// R982-1, R982-2, R982-3, R982-4, R982-7, R982-9). The explicit
// `formal-ai memory migrate` transaction is a Rust CLI command with no
// JavaScript root; its cases stay in the Rust suite.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  MAXIMUM_READABLE_SCHEMA, MINIMUM_READABLE_SCHEMA, SyncStore, TARGET_SCHEMA, exportLinksNotation, parseLinksNotation,
  preflightMemory,
} from '../../../js/server/memory-store.mjs';
import { memoryHealthStatus } from '../../../js/server/memory.mjs';

const FIXTURES = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'fixtures', 'memory');
const RELEASED = fs.readFileSync(path.join(FIXTURES, 'schema-1.lino'), 'utf8');
const FUTURE = 'demo_memory\n  schema_version "99"\n  event "future"\n    content "untouched"\n';

function fixtureDir(name) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `formal-ai-issue-982-js-${name}-`));
}

const env = (file) => ({ ...process.env, FORMAL_AI_MEMORY_PATH: file, FORMAL_AI_RECORD_CHAT: '1' });

test('the readable schema range is 1..2 and the target is 2 (R982-2)', () => {
  assert.deepEqual([MINIMUM_READABLE_SCHEMA, MAXIMUM_READABLE_SCHEMA, TARGET_SCHEMA], [1, 2, 2]);
});

test('a fixture exists for every readable schema and preflight reads it without writing (R982-1, R982-9)', () => {
  const fixtures = fs.readdirSync(FIXTURES).filter((name) => /^schema-\d+\.lino$/.test(name)).sort();
  assert.deepEqual(fixtures, ['schema-1.lino', 'schema-2.lino']);
  for (const name of fixtures) {
    const expected = Number(name.match(/\d+/)[0]);
    const dir = fixtureDir(`schema-${expected}`);
    const file = path.join(dir, 'memory.lino');
    const bytes = fs.readFileSync(path.join(FIXTURES, name));
    fs.writeFileSync(file, bytes);
    assert.deepEqual(preflightMemory(file), {
      detected_schema_version: expected,
      compatible: true,
      migration_required: expected < TARGET_SCHEMA,
      migration_state: expected < TARGET_SCHEMA ? 'upgrade_required' : 'ready',
    });
    assert.deepEqual(fs.readFileSync(file), bytes);
    assert.deepEqual(fs.readdirSync(dir), ['memory.lino'], 'preflight must not create a lock, backup, or temp file');
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('preflight of a missing path reports "missing" and creates nothing (R982-1)', () => {
  const dir = fixtureDir('missing');
  const file = path.join(dir, 'nested', 'memory.lino');
  assert.deepEqual(preflightMemory(file), {
    detected_schema_version: null,
    compatible: true,
    migration_required: false,
    migration_state: 'missing',
  });
  assert.deepEqual(fs.readdirSync(dir), []);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('/health reports detected, minimum, maximum and target schema without migrating (R982-2, R982-3)', () => {
  const dir = fixtureDir('health');
  const file = path.join(dir, 'memory.lino');
  fs.writeFileSync(file, RELEASED);
  assert.deepEqual(memoryHealthStatus(env(file)), {
    compatible: true,
    maximum_readable_schema_version: 2,
    migration_required: true,
    migration_state: 'upgrade_required',
    minimum_readable_schema_version: 1,
    schema_version: 1,
    target_schema_version: 2,
  });
  assert.equal(fs.readFileSync(file, 'utf8'), RELEASED);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('an ordinary write keeps the released schema, unknown metadata and event order (R982-3, R982-7)', () => {
  const dir = fixtureDir('ordinary-write');
  const file = path.join(dir, 'memory.lino');
  fs.writeFileSync(file, RELEASED);
  const store = SyncStore.open(env(file), file);
  assert.ok(store.recordChatExchangeWithTools('new prompt', 'new answer') > 0);
  const after = fs.readFileSync(file, 'utf8');
  assert.ok(after.startsWith('demo_memory\n  event "event-1"\n'), after);
  assert.ok(!after.includes('schema_version'), 'an ordinary write must not migrate');
  assert.ok(after.includes('    futureField "preserve me"\n'));
  assert.ok(after.indexOf('event "event-1"') < after.indexOf('event "event-2"'));
  assert.equal(preflightMemory(file).detected_schema_version, 1);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('unknown fields, evidence and ids round-trip through parse and export (R982-7)', () => {
  const events = parseLinksNotation(RELEASED);
  assert.deepEqual(events.map((event) => event.id), ['event-1', 'event-2']);
  assert.deepEqual(events[0].unknown_fields, [['futureField', 'preserve me']]);
  assert.deepEqual(events[1].evidence, ['source-a', 'source-b']);
  // A parsed released event counts its one observed write; nothing else is added.
  const exported = exportLinksNotation(events, 1);
  assert.equal(exported, RELEASED
    .replace('    futureField "preserve me"\n', '    futureField "preserve me"\n    writeCount "1"\n')
    .concat('    writeCount "1"\n'));
  assert.deepEqual(parseLinksNotation(exported), events);
  assert.equal(exportLinksNotation(events, 2), exported.replace('demo_memory\n', 'demo_memory\n  schema_version "2"\n'));
});

test('the first write to a new store uses the target schema (R982-3)', () => {
  const dir = fixtureDir('new-store');
  const file = path.join(dir, 'nested', 'memory.lino');
  const store = SyncStore.open(env(file), file);
  assert.equal(fs.readFileSync(file, 'utf8'), '', 'opening a missing store creates no synthetic event');
  store.importLinksNotation('demo_memory\n  event "new-event"\n    role "user"\n    content "new memory"\n');
  assert.ok(fs.readFileSync(file, 'utf8').startsWith('demo_memory\n  schema_version "2"\n'));
  assert.equal(preflightMemory(file).migration_required, false);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('a future schema is refused and never modified, even by a server write (R982-4, R982-9)', () => {
  const dir = fixtureDir('future');
  const file = path.join(dir, 'memory.lino');
  fs.writeFileSync(file, FUTURE);
  assert.deepEqual(preflightMemory(file), {
    detected_schema_version: 99,
    compatible: false,
    migration_required: false,
    migration_state: 'incompatible',
  });
  const store = SyncStore.open(env(file), file);
  assert.throws(() => store.recordChatExchangeWithTools('must not write', 'future schema'));
  assert.equal(fs.readFileSync(file, 'utf8'), FUTURE);
  fs.rmSync(dir, { recursive: true, force: true });
});

test('malformed memory is incompatible and never modified (R982-4)', () => {
  const dir = fixtureDir('malformed');
  const file = path.join(dir, 'memory.lino');
  const malformed = 'demo_memory\n  event "broken"\n    content "unterminated\n';
  fs.writeFileSync(file, malformed);
  assert.equal(preflightMemory(file).compatible, false);
  assert.equal(preflightMemory(file).migration_state, 'incompatible');
  const store = SyncStore.open(env(file), file);
  assert.throws(() => store.recordChatExchangeWithTools('must not write', 'malformed'));
  assert.equal(fs.readFileSync(file, 'utf8'), malformed);
  fs.rmSync(dir, { recursive: true, force: true });
});
