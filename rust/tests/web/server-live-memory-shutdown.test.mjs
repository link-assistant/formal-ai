// Live startup uses released schema fixtures; original incompatible volume bytes are unavailable.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { EventEmitter } from 'node:events';
import { createServer } from 'node:http';
import net from 'node:net';
import { resolveLiveMemory } from '../../../js/server/live-memory.mjs';
import { memoryHealthStatus } from '../../../js/server/memory.mjs';
import { SyncStore, preflightMemoryUpgrade, sharedMemoryPath } from '../../../js/server/memory-store.mjs';
import { startServer, installShutdownHandlers } from '../../../js/server/main.mjs';
import { boundedCommand } from '../../../js/server/bounded-command.mjs';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function fixture(t, bytes) {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-live-memory-'));
  t.after(() => fs.rmSync(home, {
    recursive: true,
    force: true
  }));
  const file = path.join(home, 'memory.lino');
  if (bytes !== undefined) fs.writeFileSync(file, bytes, {
    mode: 0o600
  });
  return {
    home,
    file,
    env: {
      ...process.env,
      HOME: home,
      FORMAL_AI_MEMORY_PATH: file
    }
  };
}
const released = fs.readFileSync(new URL('../fixtures/memory/schema-1.lino', import.meta.url));
test('live legacy startup migrates losslessly once with a verified original backup', async t => {
  const f = fixture(t, released),
    events = [];
  const logger = {
    log(...args) {
      events.push(args);
    }
  };
  const selected = await resolveLiveMemory(f.env, logger);
  assert.equal(selected.path, f.file);
  assert.equal(selected.mode, 'migrated');
  const bytes = fs.readFileSync(f.file);
  assert.equal(preflightMemoryUpgrade(f.file).detected_schema_version, 2);
  assert.equal(bytes.toString(), released.toString().replace('demo_memory\n', 'demo_memory\n  schema_version "2"\n'));
  const backups = fs.readdirSync(f.home).filter(name => name.endsWith('.backup'));
  assert.equal(backups.length, 1);
  assert.deepEqual(fs.readFileSync(path.join(f.home, backups[0])), released);
  assert.equal((await resolveLiveMemory(f.env, logger)).mode, 'current');
  assert.deepEqual(fs.readFileSync(f.file), bytes);
  assert.equal(events.length, 1);
});
test('incompatible live volume remains exact beside an idempotent writable private sibling', async t => {
  for (const bytes of [Buffer.from('demo_memory\n  schema_version "99"\n  event "future"\n    content "untouched"\n'), Buffer.from([255, 0, 1])]) {
    const f = fixture(t, bytes),
      events = [];
    const logger = {
      log(...args) {
        events.push(args);
      }
    };
    const selected = await resolveLiveMemory(f.env, logger);
    assert.equal(selected.mode, 'preserved-incompatible');
    assert.equal(selected.path, f.file + '.schema-2.' + hash(bytes) + '.lino');
    assert.deepEqual(fs.readFileSync(f.file), bytes);
    const store = SyncStore.open({
      ...f.env,
      FORMAL_AI_MEMORY_PATH: selected.path
    });
    assert.equal(store.events.length, 0);
    assert(store.recordChatExchangeWithTools('new prompt', 'new answer') > 0);
    const recorded = fs.readFileSync(selected.path);
    assert(recorded.includes(Buffer.from('new prompt')));
    assert.deepEqual(fs.readFileSync(f.file), bytes);
    assert.equal((await resolveLiveMemory(f.env, logger)).path, selected.path);
    assert.deepEqual(fs.readFileSync(selected.path), recorded);
    assert.equal(events[0][2].original_preserved, true);
  }
});
test('fallback collision and source symlinks refuse without replacing any bytes', async t => {
  const f = fixture(t, 'not memory');
  const selected = f.file + '.schema-2.' + hash(fs.readFileSync(f.file)) + '.lino';
  fs.writeFileSync(selected, 'occupied', {
    mode: 0o600
  });
  await assert.rejects(resolveLiveMemory(f.env), /memory_live_sibling_incompatible/u);
  assert.equal(fs.readFileSync(selected, 'utf8'), 'occupied');
  assert.equal(fs.readFileSync(f.file, 'utf8'), 'not memory');
  fs.unlinkSync(selected);
  const target = path.join(f.home, 'external');
  fs.writeFileSync(target, 'external');
  fs.symlinkSync(target, selected);
  await assert.rejects(resolveLiveMemory(f.env), /memory_live_regular_file_required/u);
  assert.equal(fs.readFileSync(target, 'utf8'), 'external');
  fs.unlinkSync(selected);
  fs.unlinkSync(f.file);
  fs.symlinkSync(target, f.file);
  await assert.rejects(resolveLiveMemory(f.env), /memory_live_regular_file_required/u);
  assert.equal(fs.readFileSync(target, 'utf8'), 'external');
});
test('live startup binds active memory and health to the selected store before any worker request', async t => {
  const f = fixture(t, 'unrecognized old format');
  const records = [];
  const logger = {
    log(...args) {
      records.push(args);
    },
    close() {}
  };
  const worker = {
    async boot() {},
    async run() {
      throw Error('worker must not execute');
    }
  };
  const running = await startServer({
    env: f.env,
    worker,
    logger
  });
  t.after(() => new Promise(resolve => running.server.close(resolve)));
  const health = await fetch(running.url + '/health', {
    signal: AbortSignal.timeout(1000)
  });
  assert.equal(health.status, 200);
  const observed = await health.json();
  assert.equal(observed.memory.compatible, true);
  assert.equal(observed.memory.schema_version, 2);
  assert.equal(observed.memory.active_path, running.ctx.env.FORMAL_AI_MEMORY_PATH);
  assert.equal(observed.memory.configured_path, f.file);
  assert.notEqual(observed.memory.active_path, observed.memory.configured_path);
  assert.notEqual(running.ctx.env.FORMAL_AI_MEMORY_PATH, f.file);
  assert.equal(running.ctx.liveMemory.originalPath, f.file);
  assert(running.ctx.memory.recordExchange([{
    role: 'user',
    content: 'new request',
    tool_calls: []
  }], 'new answer') > 0);
  assert.equal(fs.readFileSync(f.file, 'utf8'), 'unrecognized old format');
  assert.equal(preflightMemoryUpgrade(running.ctx.env.FORMAL_AI_MEMORY_PATH).compatible, true);
});
test('shutdown closes active sockets within finite grace and unregisters both signal handlers', async () => {
  const server = createServer(() => {}),
    signals = new EventEmitter(),
    ctx = {};
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const socket = net.createConnection(server.address().port, '127.0.0.1');
  await new Promise(resolve => socket.once('connect', resolve));
  socket.write('GET / HTTP/1.1\r\nHost: localhost\r\n\r\n');
  await new Promise(resolve => setTimeout(resolve, 10));
  const closed = new Promise(resolve => server.once('close', resolve));
  installShutdownHandlers(server, ctx, {
    graceMs: 25,
    signals
  });
  signals.emit('SIGTERM');
  await Promise.race([closed, new Promise((_, reject) => {
    const timer = setTimeout(() => reject(Error('shutdown exceeded bound')), 1000);
    timer.unref();
  })]);
  socket.destroy();
  assert.equal(ctx.shutdownSignal, 'SIGTERM');
  assert.equal(signals.listenerCount('SIGTERM'), 0);
  assert.equal(signals.listenerCount('SIGINT'), 0);
});
test('grace override refuses zero, nonfinite and expanded timeout values', () => {
  for (const graceMs of [0, -1, Infinity, 5001, 1.5]) assert.throws(() => installShutdownHandlers({}, {}, {
    graceMs
  }), /server_shutdown_grace_invalid/u);
});
test('real Node SIGTERM reaches maintained shutdown callback and exits zero', async () => {
  const entry = new URL('../../../js/server/main.mjs', import.meta.url).href;
  const source = ["import {createServer} from 'node:http';", `import {installShutdownHandlers} from ${JSON.stringify(entry)};`, 'const context={};const server=createServer();', 'installShutdownHandlers(server,context);', "server.once('close',()=>console.log(context.shutdownSignal));", "server.listen(0,'127.0.0.1',()=>process.kill(process.pid,'SIGTERM'));"].join('');
  const result = await boundedCommand(process.execPath, ['--input-type=module', '--eval', source], {
    timeout: 5000,
    maxBuffer: 65536
  });
  assert.equal(result.error, undefined);
  assert.equal(result.cleanup_complete, true);
  assert.equal(result.status, 0);
  assert.equal(result.stdout.toString().trim(), 'SIGTERM');
});
import { writeAtomic } from '../../../js/server/memory-store.mjs';
import { holdExclusiveLock } from '../../../js/server/memory-upgrade.mjs';
test('atomic memory writer preserves modes and refuses foreign staging or zero progress', t => {
  const f = fixture(t, 'before');
  writeAtomic(f.file, 'after');
  assert.equal(fs.readFileSync(f.file, 'utf8'), 'after');
  if (process.platform !== 'win32') assert.equal(fs.statSync(f.file).mode & 0o777, 0o600);
  const staged = f.file + '.tmp.' + process.pid;
  fs.writeFileSync(staged, 'foreign');
  assert.throws(() => writeAtomic(f.file, 'unexpected'));
  assert.equal(fs.readFileSync(staged, 'utf8'), 'foreign');
  assert.equal(fs.readFileSync(f.file, 'utf8'), 'after');
  fs.unlinkSync(staged);
  const original = fs.writeSync;
  try {
    fs.writeSync = () => 0;
    assert.throws(() => writeAtomic(f.file, 'unexpected'), /memory_atomic_write_no_progress/u);
  } finally {
    fs.writeSync = original;
  }
  assert.equal(fs.existsSync(staged), false);
  assert.equal(fs.readFileSync(f.file, 'utf8'), 'after');
});
test('migration lock handshake and held lease are finite with owned child cleanup', async t => {
  const f = fixture(t),
    bin = path.join(f.home, 'bin');
  fs.mkdirSync(bin);
  const executable = path.join(bin, 'perl'),
    previous = process.env.PATH;
  try {
    process.env.PATH = bin + path.delimiter + previous;
    fs.writeFileSync(executable, '#!/bin/sh\nsleep 10\n', {
      mode: 0o700
    });
    await assert.rejects(holdExclusiveLock(f.file + '.lock', {
      handshakeMs: 30,
      lifetimeMs: 300,
      settleMs: 100
    }), error => error.message === 'memory_lock_handshake_timeout' && typeof error.cleanup_complete === 'boolean');
    fs.writeFileSync(executable, '#!/bin/sh\nprintf "locked\\n"\nsleep 10\n', {
      mode: 0o700
    });
    const lock = await holdExclusiveLock(f.file + '.lock', {
      handshakeMs: 1000,
      lifetimeMs: 1500,
      settleMs: 100
    });
    await new Promise(resolve => setTimeout(resolve, 1700));
    assert.throws(() => lock.assertHeld(), /memory_lock_lease_expired/u);
    await assert.rejects(lock.release(), /memory_lock_release_failed/u);
  } finally {
    process.env.PATH = previous;
  }
});
test('startup migration completes positive short writes and refuses zero without losing original bytes', async t => {
  const original = fs.writeSync;
  const f = fixture(t, released);
  let shortWrites = 0;
  try {
    fs.writeSync = (descriptor, buffer, offset, length, ...rest) => {
      shortWrites++;
      return original(descriptor, buffer, offset, Math.max(1, Math.floor(length / 2)), ...rest);
    };
    await resolveLiveMemory(f.env);
  } finally {
    fs.writeSync = original;
  }
  assert(shortWrites > 3);
  assert.equal(fs.readFileSync(f.file, 'utf8'), released.toString().replace('demo_memory\n', 'demo_memory\n  schema_version "2"\n'));
  const failed = fixture(t, released);
  try {
    fs.writeSync = () => 0;
    await assert.rejects(resolveLiveMemory(failed.env));
  } finally {
    fs.writeSync = original;
  }
  assert.deepEqual(fs.readFileSync(failed.file), released);
});

test('health reports missing configured paths without creating a memory file', t => {
  const f = fixture(t);
  const first = memoryHealthStatus(f.env);
  assert.equal(first.active_path, f.file);
  assert.equal(first.configured_path, f.file);
  assert.equal(first.schema_version, null);
  assert.equal(first.migration_state, 'missing');
  assert.deepEqual(memoryHealthStatus(f.env), first);
  assert.equal(fs.existsSync(f.file), false);
});
test('health reports the platform default path without creating its directory', t => {
  const f = fixture(t);
  const env = { ...f.env, APPDATA: f.home };
  delete env.FORMAL_AI_MEMORY_PATH;
  const expected = sharedMemoryPath(env);
  const before = fs.readdirSync(f.home);
  const observed = memoryHealthStatus(env);
  assert.equal(observed.active_path, expected);
  assert.equal(observed.configured_path, expected);
  assert.deepEqual(fs.readdirSync(f.home), before);
});
