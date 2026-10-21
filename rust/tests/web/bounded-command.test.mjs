import test from 'node:test';
import assert from 'node:assert/strict';
import { boundedCommand, boundedCommandSync } from '../../../js/server/bounded-command.mjs';
const node = process.execPath;
for (const sync of [false, true]) {
  const run = sync ? boundedCommandSync : boundedCommand;
  const check = (name, source, options, verify) => {
    test(`${sync ? 'synchronous' : 'asynchronous'} command ${name}`, async () => {
      const started = performance.now();
      const result = await run(node, ['-e', source], { timeout: 1000, ...options });
      verify(result);
      assert.ok(performance.now() - started < 2500);
    });
  };
  check('binary', 'process.stdout.write(Buffer.from([0,255,128,10]));', {},
    (r) => { assert.equal(r.error, undefined); assert.deepEqual([...r.stdout], [0,255,128,10]); });
  check('blocked-input', 'setInterval(()=>{},1000)',
    { timeout: 500, input: Buffer.alloc(8 * 1024 * 1024) },
    (r) => { assert.equal(r.timed_out, true); assert.ok(r.error); });
  check('held-descendant-pipe',
    "require('node:child_process').spawn(process.execPath," +
    "['-e','setTimeout(()=>{},10000)'],{stdio:['ignore',1,2]});process.exit(0)",
    { timeout: 500 }, (r) => { assert.equal(r.timed_out, true); assert.ok(r.error); });
  check('bounded-output', 'process.stdout.write(Buffer.alloc(1024*1024));',
    { stdoutLimitBytes: 100 }, (r) => { assert.equal(r.error.code, 'ENOBUFS'); assert.equal(r.stdout.length, 100); });
  check('discard', 'process.stdout.write(Buffer.alloc(1024*1024));',
    { stdoutLimitBytes: 0 }, (r) => { assert.equal(r.error, undefined); assert.equal(r.stdout.length, 0); });
}

import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { ExecutionBox } from '../../../js/server/execution-box.mjs';
function fixture(t, name, code) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'prep-kernel-'));
  fs.writeFileSync(path.join(directory, name), `#!${process.execPath}\n${code}\n`, { mode: 0o700 });
  const original = process.env.PATH;
  process.env.PATH = directory + path.delimiter + original;
  t.after(() => {
    process.env.PATH = original;
    fs.rmSync(directory, { recursive: true, force: true });
  });
  return directory;
}
test('tar preparation preserves exact binary archive bytes', async t => {
  const workspace = fixture(t, 'tar', 'process.stdout.write(Buffer.from([0,255,128,10]));');
  const box = new ExecutionBox({ kind: 'box', image: 'test' }, 'denied', 1000, workspace);
  box.observeInvocation = (_invocation, input) => input;
  assert.deepEqual([...(await box.run('pass'))], [0,255,128,10]);
});
test('tar preparation includes descendant-held pipes in its deadline', async t => {
  const workspace = fixture(t, 'tar',
    "require('node:child_process').spawn(process.execPath," +
    "['-e','setInterval(()=>{},1000)'],{stdio:['ignore',1,2]});process.exit(0);");
  const box = new ExecutionBox({ kind: 'box', image: 'test' }, 'denied', 500, workspace);
  const started = performance.now();
  await assert.rejects(box.run('pass'), error => error.variant === 'Observed');
  assert.ok(performance.now() - started < 2000);
});
test('tar preparation rejects archives beyond its declared cap', async t => {
  const workspace = fixture(t, 'tar', 'process.stdout.write(Buffer.alloc(17*1024*1024));');
  const box = new ExecutionBox({ kind: 'box', image: 'test' }, 'denied', 2000, workspace);
  await assert.rejects(box.run('pass'));
});
test('synchronous Docker availability probe cannot hang', t => {
  const workspace = fixture(t, 'docker',
    "require('node:fs').writeFileSync(process.env.PREP_PROBE_MARKER,'started');" +
    'setInterval(()=>{},1000);');
  process.env.PREP_PROBE_MARKER = path.join(workspace, 'probe-marker');
  t.after(() => { delete process.env.PREP_PROBE_MARKER; });
  const previous = process.env.FORMAL_AI_EXECUTION_BACKEND;
  process.env.FORMAL_AI_EXECUTION_BACKEND = 'docker';
  t.after(() => {
    if (previous === undefined) delete process.env.FORMAL_AI_EXECUTION_BACKEND;
    else process.env.FORMAL_AI_EXECUTION_BACKEND = previous;
  });
  const started = performance.now();
  assert.throws(() => ExecutionBox.open(
    { kind: 'box', image: 'test' }, { network: 'denied', deadlineMs: 500 }),
  error => error.variant === 'NoDaemon');
  assert.equal(fs.readFileSync(process.env.PREP_PROBE_MARKER, 'utf8'), 'started');
  assert.ok(performance.now() - started < 2000);
});

test('normal command completion stops descendants in its owned process group', async t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'command-group-'));
  const heartbeat = path.join(directory, 'heartbeat');
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const background = `setInterval(()=>require('node:fs').appendFileSync(${JSON.stringify(heartbeat)},'x'),10);`;
  const source = "require('node:child_process').spawn(process.execPath," +
    `['-e',${JSON.stringify(background)}],{stdio:'ignore'});` +
    `const timer=setInterval(()=>{if(require('node:fs').existsSync(${JSON.stringify(heartbeat)}))` +
    "{clearInterval(timer);process.exit(0);}},5);";
  const result = await boundedCommand(process.execPath, ['-e', source], { timeout: 1500 });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0);
  const bytes = fs.statSync(heartbeat).size;
  await new Promise(resolve => setTimeout(resolve, 150));
  assert.equal(fs.statSync(heartbeat).size, bytes);
});

test('a blocked supervisor Worker cannot retain the caller after finite fallback', async t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'command-worker-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const preload = path.join(directory, 'blocked-worker.cjs');
  fs.writeFileSync(preload,
    "const threads=require('node:worker_threads');" +
    "if(threads.workerData?.boundedCommand)Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0);" +
    "if(threads.isMainThread)threads.Worker.prototype.terminate=()=>new Promise(()=>{});");
  const moduleUrl = new URL('../../../js/server/bounded-command.mjs', import.meta.url).href;
  const source = `import {boundedCommandSync} from ${JSON.stringify(moduleUrl)};` +
    "const result=boundedCommandSync(process.execPath,['-e',''],{timeout:100});" +
    "console.log('WORKER_RESULT:'+JSON.stringify({code:result.error?.code," +
    "cleanup:result.cleanup_complete,elapsed:String(result.elapsedNs)}));";
  const { executeBoundedSourceCommand } = await import('../../../scripts/check-native-source-operations.mjs');
  const result = await executeBoundedSourceCommand(process.execPath,
    ['--input-type=module', '-e', source], {
      env: { ...process.env, NODE_OPTIONS: `${process.env.NODE_OPTIONS ?? ''} --require=${preload}` },
    }, 2500);
  const line = result.output.split('\n').find(value => value.startsWith('WORKER_RESULT:'));
  assert.ok(line);
  const observed = JSON.parse(line.slice('WORKER_RESULT:'.length));
  assert.equal(observed.code, 'ECLEANUP');
  assert.equal(observed.cleanup, false);
  assert.ok(BigInt(observed.elapsed) >= 1000000000n);
});
