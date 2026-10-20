import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { createInstalledSourceCommandDescriptor, installedSourceScripts }
  from '../../../scripts/create-installed-source-command-descriptor.mjs';
import { installedOperationArguments, executeBoundedSourceCommand } from '../../../scripts/check-native-source-operations.mjs';
import { quoteShellWord } from '../../../js/agentic/portable/string_contracts.mjs';

const hash = value => createHash('sha256').update(value).digest('hex');
const template = '{node} {script} --request {task} --expected {digest}';
function fixture(t) {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'installed-source-test-')));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.mkdirSync(path.join(directory, 'scripts'));
  for (const relative of Object.values(installedSourceScripts)) {
    fs.writeFileSync(path.join(directory, relative), '// identity-only fixture; never executed\n');
  }
  return directory;
}

test('descriptor pins current executable and both fixed installed entrypoints', t => {
  const directory = fixture(t);
  const descriptor = createInstalledSourceCommandDescriptor(directory);
  assert.equal(descriptor.resourceHome, directory);
  assert.equal(descriptor.nodeExecutable, fs.realpathSync(process.execPath));
  assert.match(descriptor.nodeSha256, /^[a-f0-9]{64}$/u);
  for (const [key, relative] of Object.entries(installedSourceScripts)) {
    assert.equal(descriptor[key], hash(fs.readFileSync(path.join(directory, relative))));
  }
  assert.equal(Object.keys(descriptor).length, 5);
});

test('descriptor rejects relative homes, symlink entrypoints and oversized scripts', t => {
  const directory = fixture(t);
  assert.throws(() => createInstalledSourceCommandDescriptor('.'));
  const relative = installedSourceScripts.unaryConsolidationSha256;
  const filename = path.join(directory, relative);
  fs.unlinkSync(filename);
  fs.symlinkSync(path.join(directory, installedSourceScripts.sourceCstTransformSha256), filename);
  assert.throws(() => createInstalledSourceCommandDescriptor(directory));
  fs.unlinkSync(filename);
  fs.writeFileSync(filename, Buffer.alloc(1024 * 1024 + 1));
  assert.throws(() => createInstalledSourceCommandDescriptor(directory));
});

test('command admission preserves literal placeholders and refuses task-selected producers', () => {
  const descriptor = { nodeExecutable: '/trusted/node', resourceHome: '/trusted/bundle' };
  const task = "literal '{digest}' $HOME; never interpreted";
  const before = 'a'.repeat(64);
  const script = path.join(descriptor.resourceHome, installedSourceScripts.unaryConsolidationSha256);
  const command = `${quoteShellWord(descriptor.nodeExecutable)} ${quoteShellWord(script)} ` +
    `--request ${quoteShellWord(task)} --expected ${before}`;
  assert.deepEqual(installedOperationArguments(command, descriptor, 'unary', task, before, template),
    [script, '--request', task, '--expected', before]);
  assert.throws(() => installedOperationArguments(command + '; extra',
    descriptor, 'unary', task, before, template));
  assert.throws(() => installedOperationArguments(command,
    descriptor, 'task-selected', task, before, template));
});

test('bounded commands terminate a Node-only timeout without waiting for open pipes', async () => {
  const started = Date.now();
  await assert.rejects(executeBoundedSourceCommand(process.execPath,
    ['-e', 'setInterval(() => {}, 1000)'], {}, 50), /SourceCommandTimeout/u);
  assert.ok(Date.now() - started < 2000);
});

test('bounded commands reject output overflow from a Node-only child', async () => {
  await assert.rejects(executeBoundedSourceCommand(process.execPath,
    ['-e', "process.stdout.write('x'.repeat(1048577)); setInterval(() => {}, 1000)"], {}, 2000),
  /SourceCommandOutputBounds/u);
});

test('bounded commands kill a Node-only descendant retaining the output pipe', async t => {
  const marker = path.join(fixture(t), 'descendant-started');
  const child = "require('node:child_process').spawn(process.execPath," +
    "['-e','setInterval(() => {}, 1000)'],{stdio:['ignore',1,2]});" +
    `require('node:fs').writeFileSync(${JSON.stringify(marker)},'started');process.exit(0)`;
  const started = Date.now();
  await assert.rejects(executeBoundedSourceCommand(process.execPath, ['-e', child], {}, 500),
    /SourceCommandTimeout/u);
  assert.ok(Date.now() - started < 2000);
  assert.equal(fs.readFileSync(marker, 'utf8'), 'started');
});

test('the independent coverage oracle rejects both unchanged fixture preimages', async () => {
  const { verifyNativeSourceOperation } =
    await import('../fixtures/native-source-operations/oracle.mjs');
  for (const [kind, basename] of [['unary', 'unary-consumer'], ['parameter', 'parameter-consumer']]) {
    const filename = new URL(`../fixtures/native-source-operations/${basename}.mjs`, import.meta.url);
    await assert.rejects(verifyNativeSourceOperation(kind, fileURLToPath(filename)),
      assert.AssertionError);
  }
});
