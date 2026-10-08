import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { cloneRepositoryWorkspace, nodeRepositoryIo, runRepositoryCase } from '../../../js/server/repository-workspace.mjs';

const directories = [];
before(async () => installNodeHost(new WorkerHost()));
after(() => directories.forEach((directory) => rmSync(directory, { recursive: true, force: true })));
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-node-protocol-')); directories.push(root);
  const git = (...argumentsList) => execFileSync('git', argumentsList, { cwd: root, encoding: 'utf8' });
  git('init', '--quiet');
  writeFileSync(join(root, 'lists.rs'), 'pub const TEST_NAMES: &[&str] = &["alpha"];\n');
  writeFileSync(join(root, 'package.json'), JSON.stringify({ scripts: {
    test: "node -e \"if(!require('fs').readFileSync('lists.rs','utf8').includes('beta'))process.exit(1)\"",
  } }));
  git('add', '--all');
  git('-c', 'user.name=Protocol Test', '-c', 'user.email=protocol@example.invalid', 'commit', '--quiet', '-m', 'base');
  return { root, task: { requirement: 'Edit the TEST_NAMES list to add "beta"',
    clone: { base_commit: git('rev-parse', 'HEAD').trim() }, tests: { line: 'npm test', names: ['list membership'] } } };
}

test('live Node SWE-bench, solve and ladder callers observe source, named tests and shared trace', async () => {
  for (const caller of ['swe_bench', 'solve', 'coding_ladder']) {
    const { root, task } = fixture();
    const outcome = await runRepositoryCase(root, task, { caller });
    assert.equal(outcome.stopped_at, null, outcome.open.join('\n'));
    assert.ok(outcome.diff.includes('"beta"'));
    assert.ok(outcome.observations.some((evidence) => evidence.produced_by === 'repository_workspace_named_tests'
      && evidence.exit_code === 0));
    assert.ok(outcome.trace_document.includes(caller));
    assert.equal(readFileSync(join(root, 'repository-protocol.lino'), 'utf8'), outcome.trace_document);
    assert.equal(outcome.trace.stages.find((stage) => stage.id === 'commit').status, 'refused');
  }
});

test('Node file ports refuse parent escapes and internal or broken symlinks before a write', async () => {
  const { root } = fixture(); const io = nodeRepositoryIo();
  await assert.rejects(io.write(root, '../escaped.txt', 'bad'), /escapes/u);
  symlinkSync('missing-target.txt', join(root, 'broken-link.txt'));
  await assert.rejects(io.write(root, 'broken-link.txt', 'bad'), /symlink/u);
  symlinkSync(join(root, 'lists.rs'), join(root, 'source-link.rs'));
  await assert.rejects(io.read(root, 'source-link.rs'), /symlink/u);
  assert.ok(!readFileSync(join(root, 'lists.rs'), 'utf8').includes('bad'));
});

test('bounded Node processes distinguish missing executables and real deadline exhaustion', async () => {
  const { root } = fixture(); const io = nodeRepositoryIo();
  const missing = await io.run(root, 'formal-ai-does-not-exist-test', [], { deadline_seconds: 1 });
  assert.equal(missing.missing, true); assert.equal(missing.exit_code, 127);
  const timed = await io.run(root, process.execPath, ['-e', 'setTimeout(()=>{},10000)'], { deadline_seconds: 0.05 });
  assert.equal(timed.timed_out, true); assert.equal(timed.exit_code, null);
  assert.ok(timed.elapsed_seconds < 3);
});


test('clone-at-base materializes only a tiny local fixture and refuses moving references before creation', async () => {
  const { root: origin, task } = fixture();
  const parent = mkdtempSync(join(tmpdir(), 'formal-ai-node-clone-')); directories.push(parent);
  const refused = join(parent, 'refused');
  await assert.rejects(cloneRepositoryWorkspace({ origin, base_commit: 'main' }, refused), /forty/u);
  assert.equal(existsSync(refused), false);
  const root = join(parent, 'isolated');
  const before = readFileSync(join(origin, 'lists.rs'), 'utf8');
  const result = await runRepositoryCase(root, { ...task, clone: { ...task.clone, origin } }, { caller: 'swe_bench' });
  assert.equal(result.stopped_at, null, result.open.join('\n'));
  assert.ok(result.diff.includes('"beta"'));
  assert.equal(readFileSync(join(origin, 'lists.rs'), 'utf8'), before);
  assert.equal(execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(), task.clone.base_commit);
});


test('bounded process output retains UTF-8 characters split across stdout and stderr chunks', async () => {
  const { root } = fixture();
  const script = "process.stdout.write(Buffer.from([0xe2]));process.stderr.write(Buffer.from([0xe2]));"
    + "setTimeout(()=>{process.stdout.write(Buffer.from([0x82,0xac]));process.stderr.write(Buffer.from([0x82,0xac]));},20)";
  const result = await nodeRepositoryIo().run(root, process.execPath, ['-e', script], { deadline_seconds: 2 });
  assert.equal(result.exit_code, 0); assert.equal(result.stdout, '€'); assert.equal(result.stderr, '€');
});


test('bounded output overflow never reports a successful process exit', async () => {
  const { root } = fixture();
  const result = await nodeRepositoryIo().run(root, process.execPath, ['-e', "process.stdout.write('abcdefghijklmnop')"],
    { deadline_seconds: 2, output_limit_bytes: 4 });
  assert.equal(result.output_limit_exceeded, true); assert.equal(result.exit_code, null);
  assert.ok(Buffer.byteLength(result.stdout) <= 4);
});
