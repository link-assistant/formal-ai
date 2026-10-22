import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { syncBuiltinESMExports } from 'node:module';
import { grepCapture } from '../../../experiments/js_dogfood/grep-capture.mjs';
import * as shell from '../../../experiments/js_dogfood/shell-capture.mjs';
import { exclusiveCreate } from '../../../experiments/js_dogfood/exclusive-create.mjs';
import { boundedCommandSync } from '../../../js/server/bounded-command.mjs';
import { quoteShellWord } from '../../../js/agentic/portable/string_contracts.mjs';

const node = source => quoteShellWord(process.execPath) + ' -e ' + quoteShellWord(source);
const fallback = 'setTimeout(()=>process.exit(99),5000);';
function fixture(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'dogfood-deadline-'));
  t.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  return directory;
}
function executable(t, name, source) {
  const directory = fixture(t);
  const marker = path.join(directory, 'started');
  fs.writeFileSync(path.join(directory, name), `#!${process.execPath}\n` +
    `require('node:fs').writeFileSync(${JSON.stringify(marker)},'started');` + source,
  {mode: 0o700});
  const previous = process.env.PATH;
  process.env.PATH = directory + path.delimiter + previous;
  t.after(() => { process.env.PATH = previous; });
  return {directory, marker};
}
const held = "require('node:child_process').spawn(process.execPath," +
  "['-e','setInterval(()=>{},1000);setTimeout(()=>process.exit(99),3000)']," +
  "{stdio:['ignore',1,2]});process.exit(0);";
function cleanup(t, record) {
  t.after(() => fs.rmSync(path.dirname(record.stdout_capture.path), {recursive: true, force: true}));
}

test('grep refuses descendant-held output even after parent success', t => {
  const {marker} = executable(t, 'rg',
    'process.stdout.write(JSON.stringify({type:"summary",data:{stats:{matched_lines:0}}})+"\\n");' + held);
  const started = performance.now();
  assert.throws(() => grepCapture(['--json', 'private-fixture'], {timeoutMs: 700}),
    error => error.code === 'ETIMEDOUT');
  assert.equal(fs.readFileSync(marker, 'utf8'), 'started');
  assert.ok(performance.now() - started < 2000);
});

test('grep forcefully terminates an ignored SIGTERM within its configured lower bound', t => {
  const {marker} = executable(t, 'rg', "process.on('SIGTERM',()=>{});setInterval(()=>{},1000);" + fallback);
  assert.throws(() => grepCapture(['private-fixture'], {timeoutMs: 700}),
    error => error.code === 'ETIMEDOUT');
  assert.equal(fs.readFileSync(marker, 'utf8'), 'started');
});

test('grep raw capture cap is independent of its preview limits', t => {
  executable(t, 'rg', 'process.stdout.write(Buffer.alloc(4096));');
  assert.throws(() => grepCapture(['private-fixture'], {stdoutLimitBytes: 128}),
    error => error.code === 'ENOBUFS');
});

test('shell refuses inherited output handles after a zero parent status', t => {
  const record = JSON.parse(shell.shellCapture(node(held), {timeoutMs: 700}));
  cleanup(t, record);
  assert.equal(record.exit_code, 0);
  assert.equal(record.is_error, true);
  assert.equal(record.command_output_complete, false);
  assert.equal(record.timeout_ms, 700);
});

test('shell ignored SIGTERM still ends with bounded incomplete output evidence', t => {
  const record = JSON.parse(shell.shellCapture(node(
    "process.on('SIGTERM',()=>{});process.stdout.write('started');setInterval(()=>{},1000);" + fallback),
  {timeoutMs: 700}));
  cleanup(t, record);
  assert.equal(record.is_error, true);
  assert.equal(record.command_output_complete, false);
  assert.equal(fs.readFileSync(record.stdout_capture.path, 'utf8'), 'started');
});

test('shell hard stderr capture cap remains independent of preview and stdout', t => {
  const record = JSON.parse(shell.shellCapture(node(
    "process.stdout.write('kept');process.stderr.write(Buffer.alloc(65*1024*1024,120));"),
  {timeoutMs: 10000, maxOutputBytes: 1024}));
  cleanup(t, record);
  assert.equal(record.is_error, true);
  assert.equal(record.command_output_complete, false);
  assert.equal(record.stderr_capture.bytes, 64 * 1024 * 1024);
  assert.equal(fs.statSync(record.stderr_capture.path).size, 64 * 1024 * 1024);
  assert.equal(fs.readFileSync(record.stdout_capture.path, 'utf8'), 'kept');
});

test('exclusive creation preserves caller-owned input larger than the output cap', t => {
  const root = fixture(t), text = 'λ'.repeat(40 * 1024);
  const result = exclusiveCreate(root, 'large.txt', text);
  assert.equal(result.source_creation.success, true);
  assert.equal(fs.readFileSync(path.join(root, 'large.txt'), 'utf8'), text);
});

test('exclusive physical creation is not certified when a genuine receipt pipe stays open', t => {
  const observed = boundedCommandSync('python3', ['-c', 'import sys;print(sys.executable)'],
    {timeout: 3000, maxBuffer: 65536});
  assert.equal(observed.error, undefined);
  const realPython = observed.stdout.toString().trim();
  const {directory, marker} = executable(t, 'python3',
    "const api=require('node:child_process');const chunks=[];" +
    `const child=api.spawn(${JSON.stringify(realPython)},process.argv.slice(2),{stdio:['pipe','pipe',2]});` +
    'process.stdin.pipe(child.stdin);child.stdout.on("data",bytes=>chunks.push(bytes));' +
    'child.on("close",code=>process.stdout.write(Buffer.concat(chunks),()=>{' + held + '}));' + fallback);
  const result = exclusiveCreate(directory, 'physical.txt', 'actual bytes', undefined, {timeoutMs: 700});
  assert.equal(fs.readFileSync(marker, 'utf8'), 'started');
  assert.equal(fs.readFileSync(path.join(directory, 'physical.txt'), 'utf8'), 'actual bytes');
  assert.equal(result.is_error, true);
  assert.equal(result.source_creation.success, false);
  assert.equal(result.source_creation.external_namespace_stability, 'unknown');
});

test('shared raw writer refuses every invalid progress count without unchanged retries', t => {
  const directory = fixture(t), original = fs.writeSync;
  try {
    for (const [index, value] of [0, -1, NaN, 5].entries()) {
      let calls = 0;
      fs.writeSync = () => { calls += 1; return value; };
      syncBuiltinESMExports();
      assert.throws(() => shell.writeCaptureBytes(path.join(directory, String(index)), Buffer.from('data')),
        /InvalidCaptureWriteProgress/u);
      assert.equal(calls, 1);
    }
  } finally { fs.writeSync = original; syncBuiltinESMExports(); }
});
