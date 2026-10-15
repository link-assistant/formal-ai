import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync, fork } from 'node:child_process';
import { test } from 'node:test';
import { Worker } from 'node:worker_threads';
import { installHost } from '../../../js/agentic/host.mjs';
import { parseLino, readRepoFile } from '../../../js/server/lino.mjs';
import { emptyDerivation, toLino } from '../../../js/agentic/crate/derivation.mjs';
import { runtimeCachePaths, persistRuntimeDerivation, loadRuntimeDerivation } from '../../../js/server/runtime-cache.mjs';
import { runExplain, finalizeServerAnswer } from '../../../js/server/derivation-store.mjs';
import { startServer } from '../../../js/server/main.mjs';

installHost({ parseLino, readText: readRepoFile });
const temporary = run => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-cache-resolver-'));
  return Promise.resolve().then(() => run(directory)).finally(() => fs.rmSync(directory, { recursive: true, force: true }));
};

test('runtime resolver precedence is independent of implicit caller cwd', () => {
  const cases = [
    [{ env: { FORMAL_AI_CACHE_DIR: '/explicit', XDG_CACHE_HOME: '/xdg', HOME: '/home/a' }, cwd: '/work', platform: 'linux' }, '/explicit'],
    [{ env: { FORMAL_AI_CACHE_DIR: 'chosen' }, cwd: '/work', platform: 'linux' }, '/work/chosen'],
    [{ env: { FORMAL_AI_CACHE_DIR: './chosen/../cache' }, cwd: '/work', platform: 'linux' }, '/work/cache'],
    [{ env: { XDG_CACHE_HOME: '/xdg', HOME: '/home/a' }, cwd: '/work', platform: 'linux' }, '/xdg/formal-ai'],
    [{ env: { XDG_CACHE_HOME: 'relative', HOME: '/home/a' }, cwd: '/work', platform: 'linux' }, '/home/a/.cache/formal-ai'],
    [{ env: { HOME: '/home/a' }, cwd: '/another', platform: 'linux' }, '/home/a/.cache/formal-ai'],
    [{ env: { HOME: '/Users/a' }, cwd: '/work', platform: 'darwin' }, '/Users/a/Library/Caches/formal-ai'],
    [{ env: { LOCALAPPDATA: 'C:\\Users\\a\\AppData\\Local' }, cwd: 'C:\\work', platform: 'win32' }, 'C:\\Users\\a\\AppData\\Local\\formal-ai\\Cache'],
    [{ env: { USERPROFILE: 'C:\\Users\\a' }, cwd: 'C:\\work', platform: 'win32' }, 'C:\\Users\\a\\AppData\\Local\\formal-ai\\Cache'],
  ];
  for (const [options, expected] of cases) {
    const resolved = runtimeCachePaths({ ...options, readOnlySeedRoots: ['/bundled/seed'] });
    assert.equal(resolved.runtimeRoot, expected);
    assert.deepEqual(resolved.readOnlySeedRoots, ['/bundled/seed']);
  }
  for (const env of [{}, { HOME: 'relative' }, { FORMAL_AI_CACHE_DIR: '', HOME: '/home/a' }]) {
    assert.throws(() => runtimeCachePaths({ env, cwd: '/work', platform: 'linux' }), /runtime_cache_/);
  }
});

test('immutable traces deduplicate identical records and explain ambiguity with exact selectors', () => temporary(directory => {
  const env = { FORMAL_AI_CACHE_DIR: directory };
  const first = emptyDerivation('answer_same');
  const a = persistRuntimeDerivation(first, { env });
  assert.deepEqual(persistRuntimeDerivation(first, { env }), a);
  assert.deepEqual(loadRuntimeDerivation('answer_same', { env }), first);
  const second = { ...first, search_queries: ['distinct source trace'] };
  const b = persistRuntimeDerivation(second, { env });
  assert.notEqual(a.path, b.path);
  assert.throws(() => loadRuntimeDerivation('answer_same', { env }), /ambiguous/);
  assert.deepEqual(loadRuntimeDerivation(a.locator, { env }), first);
  assert.deepEqual(loadRuntimeDerivation(b.locator, { env }), second);
  assert.equal(runExplain(['answer_same'], { env, cwd: directory }).code, 1);
  assert.equal(runExplain([a.locator, '--format', 'links'], { env, cwd: directory }).stdout, toLino(first));
  assert.throws(() => loadRuntimeDerivation('../escape', { env }), /unusable/);
  fs.writeFileSync(a.path, 'modified');
  assert.throws(() => loadRuntimeDerivation(a.locator, { env }), /custody/);
}));

test('concurrent sessions retain distinct immutable records without choosing a winner', () => temporary(async directory => {
  const worker = path.join(directory, 'writer.mjs');
  const moduleUrl = new URL('../../../js/server/runtime-cache.mjs', import.meta.url).href;
  const hostUrl = new URL('../../../js/agentic/host.mjs', import.meta.url).href;
  const linoUrl = new URL('../../../js/server/lino.mjs', import.meta.url).href;
  fs.writeFileSync(worker, [
    'import { installHost } from ' + JSON.stringify(hostUrl) + ';',
    'import { parseLino, readRepoFile } from ' + JSON.stringify(linoUrl) + ';',
    'import { persistRuntimeDerivation } from ' + JSON.stringify(moduleUrl) + ';',
    'installHost({parseLino,readText:readRepoFile});',
    'process.send(persistRuntimeDerivation(JSON.parse(process.argv[2])));',
    'process.disconnect();',
  ].join('\n'));
  const first = emptyDerivation('answer_concurrent');
  const records = [first, { ...first, search_queries: ['another session source'] }];
  const locations = await Promise.all(records.map(record => new Promise((resolve, reject) => {
    const child = fork(worker, [JSON.stringify(record)], { env: { ...process.env, FORMAL_AI_CACHE_DIR: directory }, stdio: ['ignore', 'ignore', 'pipe', 'ipc'] });
    let location, stderr = '';
    child.stderr.on('data', bytes => { stderr += bytes; });
    child.on('message', message => { location = message; });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? resolve(location) : reject(Error(stderr)));
  })));
  const env = { FORMAL_AI_CACHE_DIR: directory };
  assert.equal(new Set(locations.map(value => value.locator)).size, 2);
  assert.throws(() => loadRuntimeDerivation('answer_concurrent', { env }), /ambiguous/);
  for (let i = 0; i < records.length; i++) assert.deepEqual(loadRuntimeDerivation(locations[i].locator, { env }), records[i]);
}));

test('default writer preserves error evidence instead of falling back into the caller', () => temporary(async directory => {
  const blocked = path.join(directory, 'not-a-directory');
  fs.writeFileSync(blocked, 'blocked fixture');
  const result = await finalizeServerAnswer({ env: { FORMAL_AI_CACHE_DIR: blocked } }, { answer: 'cache failure fixture', evidence_links: [], links_notation: '' }, { solverEvents: [] });
  assert.ok(result.evidence_links.some(value => value.startsWith('derivation:persistence_failed:')));
  assert.ok(!result.evidence_links.some(value => value.startsWith('derivation_record:')));
  assert.ok(result.links_notation.includes('derivation'));
}));

test('default actual server caches outside a clean caller Git repository and explain works elsewhere', () => temporary(async directory => {
  const caller = path.join(directory, 'caller'), elsewhere = path.join(directory, 'elsewhere');
  fs.mkdirSync(caller); fs.mkdirSync(elsewhere);
  execFileSync('git', ['init', '--quiet'], { cwd: caller });
  const env = { ...process.env, HOME: path.join(directory, 'home'), XDG_CACHE_HOME: path.join(directory, 'xdg'), FORMAL_AI_MEMORY_PATH: path.join(directory, 'state/memory.lino'), FORMAL_AI_SILENT: '1' };
  delete env.FORMAL_AI_CACHE_DIR;
  const originalCwd = process.cwd(); let running;
  try {
    process.chdir(caller);
    running = await startServer({ env });
    const response = await fetch(running.url + '/v1/chat/completions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ model: 'formal-ai', messages: [{ role: 'user', content: '1 + 1' }] }) });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.match(result.choices[0].message.content, /2/);
    assert.equal(execFileSync('git', ['status', '--porcelain', '--untracked-files=all'], { cwd: caller, encoding: 'utf8' }), '');
    const root = runtimeCachePaths({ env }).derivationDirectory;
    const ids = fs.readdirSync(root).filter(name => /^[A-Za-z0-9_-]+$/u.test(name));
    assert.ok(ids.length > 0);
    const answerId = ids[0], names = fs.readdirSync(path.join(root, answerId));
    const recordPath = path.join(root, answerId, names[0]);
    const locator = answerId + '@' + names[0].slice(0, -5);
    const explained = runExplain([locator, '--format', 'links'], { env, cwd: elsewhere });
    assert.equal(explained.code, 0, explained.stderr);
    assert.equal(explained.stdout, fs.readFileSync(recordPath, 'utf8'));
    assert.equal(fs.existsSync(path.join(caller, 'data')), false);
  } finally {
    process.chdir(originalCwd);
    if (running) await new Promise(resolve => running.server.close(resolve));
  }
}));

async function heldWriters(directory, count) {
  const record = emptyDerivation('answer_atomic');
  const control = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * count);
  const workerFile = path.join(directory, 'held-writer.mjs');
  const moduleUrl = new URL('../../../js/server/runtime-cache.mjs', import.meta.url).href;
  const hostUrl = new URL('../../../js/agentic/host.mjs', import.meta.url).href;
  const linoUrl = new URL('../../../js/server/lino.mjs', import.meta.url).href;
  fs.writeFileSync(workerFile, [
    'import fs from "node:fs";',
    'import {parentPort,workerData} from "node:worker_threads";',
    'import {installHost} from ' + JSON.stringify(hostUrl) + ';',
    'import {parseLino,readRepoFile} from ' + JSON.stringify(linoUrl) + ';',
    'import {persistRuntimeDerivation} from ' + JSON.stringify(moduleUrl) + ';',
    'installHost({parseLino,readText:readRepoFile});',
    'const original=fs.writeFileSync; let held=false;',
    'fs.writeFileSync=(fd,bytes,...rest)=>{',
    ' if(typeof fd==="number" && Buffer.isBuffer(bytes) && !held){',
    '  held=true; const half=Math.floor(bytes.length/2);',
    '  fs.writeSync(fd,bytes,0,half); parentPort.postMessage({phase:"partial"});',
    '  Atomics.wait(new Int32Array(workerData.control),workerData.index,0);',
    '  return original(fd,bytes.subarray(half),...rest);',
    ' } return original(fd,bytes,...rest);',
    '};',
    'const location=persistRuntimeDerivation(workerData.record,{env:{FORMAL_AI_CACHE_DIR:workerData.directory}});',
    'parentPort.postMessage({phase:"complete",location});',
  ].join('\n'));
  const states = Array.from({ length: count }, (_, index) => {
    const worker = new Worker(workerFile, { workerData: { control, index, record, directory } });
    let readyResolve, readyReject, doneResolve, doneReject, location;
    const ready = new Promise((resolve, reject) => { readyResolve = resolve; readyReject = reject; });
    const done = new Promise((resolve, reject) => { doneResolve = resolve; doneReject = reject; });
    // Handle early errors without creating unhandled rejections while awaiting readiness.
    done.catch(() => {});
    worker.on('message', message => {
      if (message.phase === 'partial') readyResolve();
      if (message.phase === 'complete') location = message.location;
    });
    worker.on('error', error => { readyReject(error); doneReject(error); });
    worker.on('exit', code => {
      if (code !== 0 || !location) { const error = Error('held writer exited: ' + code); readyReject(error); doneReject(error); }
      else doneResolve(location);
    });
    return { worker, ready, done };
  });
  try {
    await Promise.all(states.map(state => state.ready));
    const env = { FORMAL_AI_CACHE_DIR: directory };
    assert.equal(loadRuntimeDerivation(record.answer_id, { env }), null, 'an incomplete staging object is never published');
    assert.deepEqual(fs.readdirSync(path.join(directory, 'derivations', record.answer_id)), []);
    const cells = new Int32Array(control);
    for (let index = 0; index < count; index++) { Atomics.store(cells, index, 1); Atomics.notify(cells, index); }
    const locations = await Promise.all(states.map(state => state.done));
    assert.equal(new Set(locations.map(location => location.locator)).size, 1);
    assert.equal(fs.readdirSync(path.join(directory, 'derivations', record.answer_id)).length, 1);
    assert.deepEqual(loadRuntimeDerivation(record.answer_id, { env }), record);
    assert.deepEqual(fs.readdirSync(path.join(directory, 'derivations', '.pending')), []);
  } finally {
    const cells = new Int32Array(control);
    for (let index = 0; index < count; index++) { Atomics.store(cells, index, 1); Atomics.notify(cells, index); }
    await Promise.all(states.map(state => state.worker.terminate()));
  }
}

test('reader sees no partially written object before atomic publication', { timeout: 10_000 }, () => temporary(directory => heldWriters(directory, 1)));
test('same-record simultaneous writers publish exactly one complete object', { timeout: 10_000 }, () => temporary(directory => heldWriters(directory, 2)));

test('corrupted staging bytes and unavailable atomic publication refuse without final objects', () => temporary(directory => {
  const originalSync = fs.fsyncSync, originalLink = fs.linkSync;
  const record = emptyDerivation('answer_refusal'), env = { FORMAL_AI_CACHE_DIR: directory };
  try {
    fs.fsyncSync = fd => {
      originalSync(fd);
      fs.writeSync(fd, Buffer.from('corrupt'), 0, 7, 0);
    };
    assert.throws(() => persistRuntimeDerivation(record, { env }), /custody/);
    assert.equal(loadRuntimeDerivation(record.answer_id, { env }), null);
    assert.deepEqual(fs.readdirSync(path.join(directory, 'derivations', '.pending')), []);
    fs.fsyncSync = originalSync;
    fs.linkSync = () => { throw Object.assign(Error('atomic publication unavailable'), { code: 'ENOTSUP' }); };
    assert.throws(() => persistRuntimeDerivation(record, { env }), /atomic publication unavailable/);
    assert.equal(loadRuntimeDerivation(record.answer_id, { env }), null);
    assert.deepEqual(fs.readdirSync(path.join(directory, 'derivations', '.pending')), []);
  } finally { fs.fsyncSync = originalSync; fs.linkSync = originalLink; }
}));
