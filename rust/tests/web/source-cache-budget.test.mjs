import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {SOURCE_CACHE_POLICY, sourceCacheUsage, writeBudgetedSourceCapture, boundedSourceCacheIo}
  from '../../../js/provenance/source-cache-budget.mjs';
import {readBoundedFilePath} from '../../../js/provenance/bounded-file.mjs';
import {cacheKey, cachedSourceFetchResult} from '../../../js/agentic/crate/source_cache.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const policy = overrides => ({...SOURCE_CACHE_POLICY, maxBytes: 4096, maxFiles: 16, maxEntries: 2,
  maxBodyBytes: 128, maxMetadataBytes: 2048, ...overrides});
function fixture(t) {
  const directory = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'formal-ai-source-budget-'));
  t.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  return {directory, root: path.join(directory, 'source-cache')};
}
function record(url, text = 'exact source bytes') {
  const bytes = Buffer.from(text);
  return {source_url: url, fetched_at: '1800000000', sha256: hash(bytes), bytes};
}
function write(root, value, limits) {
  return writeBudgetedSourceCapture(root, path.join(root, cacheKey(value.source_url) + '.meta'), value, limits);
}

test('shared installed policy has explicit positive byte, entry, file and per-record limits', () => {
  assert.equal(SOURCE_CACHE_POLICY.format, 'source-cache-budget/v1');
  for (const key of ['maxBytes', 'maxFiles', 'maxEntries', 'maxBodyBytes', 'maxMetadataBytes'])
    assert.ok(Number.isSafeInteger(SOURCE_CACHE_POLICY[key]) && SOURCE_CACHE_POLICY[key] > 0);
  assert.ok(SOURCE_CACHE_POLICY.maxBodyBytes <= SOURCE_CACHE_POLICY.maxBytes);
});
test('actual capture adapter supports cold acquisition and offline byte-identical warm reuse', t => {
  const {directory, root} = fixture(t);
  let calls = 0;
  const io = boundedSourceCacheIo(() => {calls++; return Buffer.from('original bytes');});
  const online = cachedSourceFetchResult({cacheDir: directory, online: true, io, now: () => 1800000000});
  const first = online('https://example.test/a');
  assert.equal(first.kind, 'capture');
  assert.equal(first.capture.cached, false);
  const second = cachedSourceFetchResult({cacheDir: directory, online: false, io})('https://example.test/a');
  assert.equal(second.kind, 'capture');
  assert.equal(second.capture.text, 'original bytes');
  assert.equal(second.capture.sha256, first.capture.sha256);
  assert.equal(second.capture.cached, true);
  assert.equal(calls, 1);
  assert.equal(sourceCacheUsage(root).entries, 1);
});
test('aggregate entry cap refuses additional metadata without evicting shared body or first capture', t => {
  const {root} = fixture(t), limits = policy({maxEntries: 1});
  const first = record('https://example.test/first'), next = record('https://example.test/next');
  write(root, first, limits);
  const before = sourceCacheUsage(root, limits);
  assert.throws(() => write(root, next, limits), /source_cache_budget_exceeded/u);
  assert.deepEqual(sourceCacheUsage(root, limits), before);
  assert.equal(fs.readFileSync(path.join(root, 'objects', first.sha256 + '.body'), 'utf8'), 'exact source bytes');
  assert.equal(fs.existsSync(path.join(root, '.source-cache-write.lock')), false);
});
test('content-addressed reuse counts one body and independent URL metadata', t => {
  const {root} = fixture(t), limits = policy();
  write(root, record('https://example.test/a'), limits);
  write(root, record('https://example.test/b'), limits);
  assert.equal(fs.readdirSync(path.join(root, 'objects')).length, 1);
  assert.equal(sourceCacheUsage(root, limits).entries, 2);
});
test('peak byte admission reserves the complete metadata temporary and preserves previous bytes on refusal', t => {
  const {root} = fixture(t), limits = policy(), first = record('https://example.test/a');
  write(root, first, limits);
  const before = sourceCacheUsage(root, limits);
  assert.throws(() => write(root, first, policy({maxBytes: before.bytes + 1})), /source_cache_budget_exceeded/u);
  assert.deepEqual(sourceCacheUsage(root, limits), before);
});
test('aggregate file accounting includes orphan bodies and interrupted-write remnants', t => {
  const {root} = fixture(t), limits = policy({maxFiles: 2});
  fs.mkdirSync(path.join(root, 'objects'), {recursive: true});
  fs.writeFileSync(path.join(root, 'objects', 'interrupted.tmp'), 'orphan');
  assert.throws(() => write(root, record('https://example.test/a'), limits), /source_cache_budget_exceeded/u);
  assert.equal(sourceCacheUsage(root, limits).files, 1);
});
test('pre-existing cross-runtime lock fails immediately and preserves lock and unrelated user file', t => {
  const {directory, root} = fixture(t);
  fs.mkdirSync(root);
  const lock = path.join(root, '.source-cache-write.lock'), unrelated = path.join(directory, 'keep.txt');
  fs.writeFileSync(lock, 'owned by another writer'); fs.writeFileSync(unrelated, 'personal fixture');
  const start = performance.now();
  assert.throws(() => write(root, record('https://example.test/a'), policy()), /EEXIST/u);
  assert.ok(performance.now() - start < 1000);
  assert.equal(fs.readFileSync(lock, 'utf8'), 'owned by another writer');
  assert.equal(fs.readFileSync(unrelated, 'utf8'), 'personal fixture');
});
test('symlinked object directory is refused without modifying its target', t => {
  const {directory, root} = fixture(t), target = path.join(directory, 'unrelated');
  fs.mkdirSync(root); fs.mkdirSync(target); fs.writeFileSync(path.join(target, 'keep.txt'), 'keep');
  fs.symlinkSync(target, path.join(root, 'objects'), 'dir');
  assert.throws(() => write(root, record('https://example.test/a'), policy()), /source_cache_directory_type/u);
  assert.deepEqual(fs.readdirSync(target), ['keep.txt']);
});
test('body and metadata per-record bounds refuse before creating cache directories', t => {
  const {root} = fixture(t);
  assert.throws(() => write(root, record('https://example.test/a', 'x'.repeat(129)), policy()), /source_cache_body_budget_exceeded/u);
  assert.throws(() => write(root, record('https://example.test/' + 'a'.repeat(500)), policy({maxMetadataBytes: 32})), /source_cache_metadata_budget_exceeded/u);
  assert.equal(fs.existsSync(root), false);
});
test('inconsistent content identity and invalid policy are refused before effects', t => {
  const {root} = fixture(t), value = record('https://example.test/a');
  assert.throws(() => write(root, {...value, sha256: '0'.repeat(64)}, policy()), /source_cache_capture_identity/u);
  assert.throws(() => write(root, value, policy({maxEntries: NaN})), /source_cache_policy_invalid/u);
  assert.equal(fs.existsSync(root), false);
});
test('oversized sparse metadata is refused by actual adapter before transport and without full allocation', t => {
  const {directory, root} = fixture(t), url = 'https://example.test/a';
  fs.mkdirSync(root);
  const filename = path.join(root, cacheKey(url) + '.meta'), descriptor = fs.openSync(filename, 'wx');
  fs.ftruncateSync(descriptor, SOURCE_CACHE_POLICY.maxMetadataBytes + 1); fs.closeSync(descriptor);
  let calls = 0;
  const result = cachedSourceFetchResult({cacheDir: directory, online: true,
    io: boundedSourceCacheIo(() => {calls++; return Buffer.from('new');})})(url);
  assert.equal(result.kind, 'error');
  assert.match(result.error, /RuntimeArchiveSizeOrType/u);
  assert.equal(calls, 0);
});
test('corrupt retained body refuses refetch; deleted cache allows genuine fresh controlled acquisition', t => {
  const {directory, root} = fixture(t), url = 'https://example.test/a'; let calls = 0;
  const io = boundedSourceCacheIo(() => {calls++; return Buffer.from('original');});
  const fetch = cachedSourceFetchResult({cacheDir: directory, online: true, io, now: () => 1800000000});
  const first = fetch(url);
  fs.writeFileSync(path.join(root, 'objects', first.capture.sha256 + '.body'), 'changed');
  assert.equal(fetch(url).kind, 'error'); assert.equal(calls, 1);
  fs.rmSync(root, {recursive: true});
  assert.equal(fetch(url).capture.cached, false); assert.equal(calls, 2);
});

test('partial stage write failure removes only owned temporary bytes and preserves its original error', t => {
  const {root} = fixture(t), value = record('https://example.test/partial');
  const originalWrite = fs.writeFileSync;
  let injected = false;
  fs.writeFileSync = function(target, bytes, ...options) {
    if (typeof target === 'number' && !injected) {
      injected = true;
      fs.writeSync(target, bytes, 0, Math.min(3, bytes.length), 0);
      throw Error('controlled_partial_capture_failure');
    }
    return originalWrite.call(fs, target, bytes, ...options);
  };
  try { assert.throws(() => write(root, value, policy()), /controlled_partial_capture_failure/u); }
  finally { fs.writeFileSync = originalWrite; }
  assert.equal(injected, true);
  assert.deepEqual(sourceCacheUsage(root, policy()), {bytes: 0, files: 0, entries: 0});
  assert.equal(fs.existsSync(path.join(root, '.source-cache-write.lock')), false);
});
test('replaced stage custody is preserved while the primary action failure remains diagnostic authority', t => {
  const {root} = fixture(t), originalSync = fs.fsyncSync;
  let replacement;
  fs.fsyncSync = function() {
    const objects = path.join(root, 'objects');
    const pending = fs.readdirSync(objects).find(name => name.endsWith('.tmp'));
    assert.ok(pending);
    replacement = path.join(objects, pending);
    fs.renameSync(replacement, replacement + '.displaced');
    fs.writeFileSync(replacement, 'replacement-owned-by-another-writer');
    throw Error('controlled_capture_sync_failure');
  };
  let failure;
  try { write(root, record('https://example.test/replaced-stage'), policy()); }
  catch (error) { failure = error; }
  finally { fs.fsyncSync = originalSync; }
  assert.ok(failure instanceof AggregateError);
  assert.match(failure.message, /controlled_capture_sync_failure/u);
  assert.match(failure.message, /source_cache_temporary_custody/u);
  assert.equal(failure.cause.message, 'controlled_capture_sync_failure');
  assert.equal(fs.readFileSync(replacement, 'utf8'), 'replacement-owned-by-another-writer');
  assert.equal(fs.existsSync(path.join(root, '.source-cache-write.lock')), false);
});
test('replaced lock is never unlinked and cleanup context does not conceal the failed capture', t => {
  const {root} = fixture(t), originalSync = fs.fsyncSync;
  const lock = path.join(root, '.source-cache-write.lock');
  fs.fsyncSync = function() {
    fs.renameSync(lock, lock + '.displaced');
    fs.writeFileSync(lock, 'another-writer-lock');
    throw Error('controlled_capture_primary_failure');
  };
  let failure;
  try { write(root, record('https://example.test/replaced-lock'), policy()); }
  catch (error) { failure = error; }
  finally { fs.fsyncSync = originalSync; }
  assert.ok(failure instanceof AggregateError);
  assert.match(failure.message, /controlled_capture_primary_failure/u);
  assert.match(failure.message, /source_cache_lock_custody/u);
  assert.equal(failure.cause.message, 'controlled_capture_primary_failure');
  assert.equal(fs.readFileSync(lock, 'utf8'), 'another-writer-lock');
  assert.deepEqual(fs.readdirSync(path.join(root, 'objects')), []);
});
test('nonregular paths refuse before opening and ordinary reads use available nonblocking protection', t => {
  const {directory} = fixture(t), filename = path.join(directory, 'regular');
  fs.writeFileSync(filename, 'bounded');
  let opened = 0;
  const io = {...fs, openSync(target, flags) {
    opened++;
    if (typeof fs.constants.O_NONBLOCK === 'number') assert.equal(flags & fs.constants.O_NONBLOCK, fs.constants.O_NONBLOCK);
    return fs.openSync(target, flags);
  }};
  assert.throws(() => readBoundedFilePath(directory, 16, {io}), /RuntimeArchiveSizeOrType/u);
  assert.equal(opened, 0);
  assert.equal(readBoundedFilePath(filename, 16, {io}).toString(), 'bounded');
  assert.equal(opened, 1);
});


test('invalid UTF8 metadata cannot impersonate a legitimate replacement-character URL or trigger refetch', t => {
  const {directory, root} = fixture(t), url = 'https://example.test/\uFFFD'; let calls = 0;
  const io = boundedSourceCacheIo(() => {calls++; return Buffer.from('retained exact source');});
  const fetch = cachedSourceFetchResult({cacheDir: directory, online: true, io, now: () => 1800000000});
  assert.equal(fetch(url).kind, 'capture');
  const filename = path.join(root, cacheKey(url) + '.meta'), metadata = fs.readFileSync(filename);
  const marker = Buffer.from('\uFFFD'), position = metadata.indexOf(marker); assert.ok(position >= 0);
  fs.writeFileSync(filename, Buffer.concat([metadata.subarray(0, position), Buffer.from([255]), metadata.subarray(position + marker.length)]));
  const result = fetch(url); assert.equal(result.kind, 'error'); assert.equal(calls, 1);
});
