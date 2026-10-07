// Issue #1165 R1165-10: the JavaScript server's solve path reads the procedure
// cache file the native `WriteProgram` branch of rust/src/solver.rs reads
// (`FORMAL_AI_PROCEDURE_CACHE`, else data/cache/coding-procedure-cache.lino)
// and calls `cachedWriteProgram` (js/server/procedure-cache.mjs). A verified
// row answers an unmodified catalog request with the cached entry and logs
// `procedure_cache outcome=hit ... content_id=0x...`; the committed, row-less
// cache keeps the miss; a customised request is never answered from the cache.
// A program the worker rediscovered from the documentation captures (R1165-1)
// is recorded in an explicitly configured runtime cache and reused as a hit;
// the committed cache file is never written by a solve.
// Cache files live under the OS temp directory, so the committed cache is never
// touched.

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, before, test } from 'node:test';

import { contentAddress, loadAt, rediscoverableRecipe, store } from '../../../js/agentic/crate/discovery_production.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { REPO_ROOT } from '../../../js/server/lino.mjs';
import { loadProcedureCache, procedureCachePath } from '../../../js/server/procedure-cache.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const ENV = 'FORMAL_AI_PROCEDURE_CACHE';
const PROMPT = 'Write a hello world program in Python';
const ENTRY = 'import sys\nsys.stdout.write("Hello, world!\\n")';
const TEMPLATE = 'print("Hello, world!")';

let host;
let directory;
const previous = process.env[ENV];

before(async () => {
  host = new WorkerHost();
  await installNodeHost(host);
  directory = mkdtempSync(path.join(tmpdir(), 'formal-ai-procedure-cache-'));
});

after(() => {
  if (previous === undefined) delete process.env[ENV];
  else process.env[ENV] = previous;
  rmSync(directory, { recursive: true, force: true });
});

function writeCache(name, rows) {
  const file = path.join(directory, name);
  const cache = loadAt(file, { readText: () => null, writeText: (target, text) => writeFileSync(target, text) });
  for (const row of rows) assert.deepEqual(store(cache, row), { ok: true });
  return file;
}

const pythonRow = () => rediscoverableRecipe({
  language: 'python',
  task: 'hello_world',
  rediscovery_query: 'python hello world',
  rediscovery_source: 'https://docs.python.org/3/library/sys.html#sys.stdout',
  entry: ENTRY,
  verified_output: 'Hello, world!',
});

const cacheEvents = (result) => (result.solverEvents || []).filter((event) => event.kind === 'procedure_cache');

test('the cache path mirrors default_cache_path: the override, else the repository file', () => {
  delete process.env[ENV];
  assert.equal(procedureCachePath(), path.join(REPO_ROOT, 'data/cache/coding-procedure-cache.lino'));
  process.env[ENV] = '  ';
  assert.equal(procedureCachePath(), path.join(REPO_ROOT, 'data/cache/coding-procedure-cache.lino'));
  process.env[ENV] = path.join(directory, 'elsewhere.lino');
  assert.equal(procedureCachePath(), path.join(directory, 'elsewhere.lino'));
  assert.deepEqual(loadProcedureCache().recipes, [], 'a missing file loads as an empty cache');
});

test('the committed cache has no rows, so an unmodified request stays the native miss', async () => {
  delete process.env[ENV];
  assert.deepEqual(loadProcedureCache().recipes, []);
  const result = await host.solve(PROMPT, []);
  assert.deepEqual(cacheEvents(result), [
    { kind: 'procedure_cache', payload: 'outcome=miss language=python task=hello_world research_missing=reviewer_approval' },
  ]);
  assert.ok(result.content.includes(TEMPLATE));
});

test('a verified row answers the unmodified request with the cached entry and logs the hit', async () => {
  process.env[ENV] = writeCache('hit.lino', [pythonRow()]);
  const result = await host.solve(PROMPT, []);
  const contentId = `0x${contentAddress(ENTRY).toString(16).padStart(16, '0')}`;
  assert.deepEqual(cacheEvents(result), [
    { kind: 'procedure_cache', payload: `outcome=hit language=python task=hello_world content_id=${contentId}` },
  ]);
  assert.ok(result.content.includes(ENTRY), 'the cached entry replaces the template');
  assert.ok(!result.content.includes(TEMPLATE));
  const kinds = result.solverEvents.map((event) => event.kind);
  assert.equal(kinds.indexOf('procedure_cache'), kinds.indexOf('legacy_intent') + 1);
});

test('a row for another pair leaves this request a miss', async () => {
  process.env[ENV] = writeCache('other.lino', [{ ...pythonRow(), language: 'ruby', entry: 'puts "x"' }]);
  const result = await host.solve(PROMPT, []);
  assert.equal(cacheEvents(result)[0]?.payload.split(' ')[0], 'outcome=miss');
  assert.ok(result.content.includes(TEMPLATE));
});

test('a row whose content_id drifted is dropped on load, as in Rust', async () => {
  const file = writeCache('drift.lino', [pythonRow()]);
  const drifted = (await import('node:fs')).readFileSync(file, 'utf8').replace(/content_id "0x[0-9a-f]+"/u, 'content_id "0x0000000000000001"');
  writeFileSync(file, drifted);
  process.env[ENV] = file;
  const result = await host.solve(PROMPT, []);
  assert.equal(cacheEvents(result)[0]?.payload.split(' ')[0], 'outcome=miss');
});

test('a customised request is never answered from the cache', async () => {
  process.env[ENV] = writeCache('custom.lino', [pythonRow()]);
  const result = await host.solve('Write a hello world program in Python and replace "Hello, world!" with "Hi there"', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), []);
  assert.ok(!result.content.includes(ENTRY));
});

test('R1165-1: a rediscovered row is recorded in the runtime cache and reused as a hit', async () => {
  const file = path.join(directory, 'runtime.lino');
  process.env[ENV] = file;
  const program = 'fn main() {\n    println!("Hello, world!");\n}';
  const contentId = `0x${contentAddress(program).toString(16).padStart(16, '0')}`;
  const first = await host.solve('Write a hello world program in Rust', []);
  assert.equal(cacheEvents(first)[0]?.payload.split(' ')[0], 'outcome=discovered');
  const row = loadProcedureCache().recipes.find((recipe) => recipe.language === 'rust');
  assert.ok(row, 'the rediscovered row is stored');
  assert.equal(row.entry, program);
  assert.equal(row.rediscovery_source, 'https://doc.rust-lang.org/book/ch01-02-hello-world.html');
  assert.equal(row.verified_output, 'Hello, world!');
  const second = await host.solve('Write a hello world program in Rust', []);
  assert.deepEqual(cacheEvents(second), [
    { kind: 'procedure_cache', payload: `outcome=hit language=rust task=hello_world content_id=${contentId}` },
  ]);
  assert.ok(second.content.includes(program));
});

test('the committed cache is never written by a solve', async () => {
  delete process.env[ENV];
  const committed = readFileSync(procedureCachePath(), 'utf8');
  await host.solve('Write a hello world program in Go', []);
  assert.equal(readFileSync(procedureCachePath(), 'utf8'), committed);
});
