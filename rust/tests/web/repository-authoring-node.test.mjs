import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { runNodeAuthoring } from '../../../js/server/repository-authoring.mjs';
import { runRepositoryCommandLine } from '../../../js/server/repository-workspace-command-line.mjs';
import { loadProtocol } from '../../../js/agentic/crate/repository_workspace.mjs';

const directories = [];
before(async () => installNodeHost(new WorkerHost()));
after(() => directories.forEach((directory) => rmSync(directory, { recursive: true, force: true })));
function fixture() {
  const parent = mkdtempSync(join(tmpdir(), 'formal-ai-node-authoring-')); directories.push(parent);
  const repository = join(parent, 'repository'); const workspace = join(parent, 'workspace'); const seed = join(parent, 'seed');
  mkdirSync(repository); mkdirSync(seed); mkdirSync(join(repository, 'out'));
  const binary = Buffer.from([255, 0, 128, 65]);
  writeFileSync(join(seed, 'result.txt'), 'seed bytes'); writeFileSync(join(seed, 'extra.bin'), binary);
  writeFileSync(join(repository, 'out/result.txt'), 'destination bytes');
  const git = (...argumentsList) => execFileSync('git', argumentsList, { cwd: repository, encoding: 'utf8' });
  git('init', '--quiet'); git('config', 'user.name', 'Authoring Test'); git('config', 'user.email', 'authoring@example.invalid');
  git('add', '--all'); git('commit', '--quiet', '-m', 'base');
  const args = { repository, workspace, seed, task: 'Write the declared artifact', message: 'feat: authored test',
    pull_request: 'https://github.com/example/project/pull/1', produces: ['result.txt', 'extra.bin'],
    into: ['out/result.txt', 'out/extra.bin'], contains: ['authored'], evidence: 'evidence' };
  const calls = [];
  const ports = {
    startServer: async () => { calls.push('serve'); return { url: 'http://127.0.0.1:12345', close: async () => calls.push('close') }; },
    runSession: async ({ root, configuration }) => {
      calls.push('edit'); assert.equal(configuration.model, 'formalai/formal-ai');
      writeFileSync(join(root, 'result.txt'), 'authored bytes');
      return { exit_code: 0, stderr: '', stdout: 'progress\n{"type":"start","session_id":"ses_replay"}\n{"type":"finish"}\n' };
    },
    classifyStderr: async (stderr) => stderr === '',
  };
  return { args, ports, calls, git, binary };
}

test('authoring observes a framed session and byte-preserving artifact contract, then lands without committing', async () => {
  const { args, ports, calls, binary } = fixture();
  const result = await runNodeAuthoring(args, ports);
  assert.equal(result.stopped_at, null, result.open.join('\n')); assert.equal(result.committed, false);
  assert.equal(result.session_id, 'ses_replay');
  assert.equal(readFileSync(join(args.repository, 'out/result.txt'), 'utf8'), 'authored bytes');
  assert.deepEqual(readFileSync(join(args.repository, 'out/extra.bin')), binary);
  assert.deepEqual(calls, ['serve', 'edit', 'close']);
  assert.match(readFileSync(join(args.repository, 'evidence/session-id.txt'), 'utf8'), /formal-ai model formal-ai\/javascript/u);
});

test('explicit commit contains all four trailers and the already-opened trace gate', async () => {
  const { args, ports, git } = fixture(); args.commit = true;
  const result = await runNodeAuthoring(args, ports);
  assert.equal(result.stopped_at, null, result.open.join('\n')); assert.equal(result.committed, true);
  const body = git('log', '-1', '--format=%B');
  for (const trailer of ['Session', 'Model', 'Evidence', 'Pull-Request']) assert.ok(body.includes('Formal-AI-' + trailer + ':'));
  const trace = git('show', 'HEAD:evidence/repository-protocol.lino');
  assert.ok(trace.includes('requested')); assert.equal(git('status', '--porcelain'), '');
});

test('missing contains, unchanged seed and missing session each stop before landing and close the server', async () => {
  for (const failure of ['contains', 'unchanged', 'session']) {
    const { args, ports, calls } = fixture();
    if (failure === 'contains') args.contains = ['absent'];
    if (failure === 'unchanged') { args.contains = []; ports.runSession = async () => ({ exit_code: 0, stderr: '', stdout: '{"session_id":"ses_replay"}\n' }); }
    if (failure === 'session') ports.runSession = async () => ({ exit_code: 0, stderr: '', stdout: '{"type":"finish"}\n' });
    const result = await runNodeAuthoring(args, ports);
    assert.ok(result.stopped_at); assert.deepEqual(result.destinations, []); assert.equal(result.committed, false);
    assert.equal(readFileSync(join(args.repository, 'out/result.txt'), 'utf8'), 'destination bytes');
    assert.equal(calls.at(-1), 'close');
  }
});

test('a removed serve stage starts no server and an ambient workspace or hosted model is refused', async () => {
  const { args, ports, calls } = fixture();
  const result = await runNodeAuthoring(args, { ...ports, steps: loadProtocol().filter((step) => step.id !== 'serve') });
  assert.equal(result.stopped_at.id, 'edit'); assert.deepEqual(calls, []);
  await assert.rejects(runNodeAuthoring({ ...args, workspace: args.repository }, ports), /outside/u);
  await assert.rejects(runNodeAuthoring({ ...args, model: 'hosted/example' }, ports), /Formal AI/u);
});

test('live CLI routes coding-ladder task JSON and refuses unknown callers before host setup', async () => {
  const { args, git } = fixture();
  writeFileSync(join(args.repository, 'lists.rs'), 'pub const TEST_NAMES: &[&str] = &["alpha"];\n');
  git('add', '--all'); git('commit', '--quiet', '-m', 'structural fixture');
  const task = { requirement: 'Edit the TEST_NAMES list to add "beta"', clone: { base_commit: git('rev-parse', 'HEAD').trim() } };
  const taskFile = join(args.repository, 'task.json'); writeFileSync(taskFile, JSON.stringify(task));
  const result = await runRepositoryCommandLine(['coding-ladder', '--workspace', args.repository, '--task', taskFile], { install: async () => {} });
  assert.equal(result.code, 0); assert.ok(result.outcome.diff.includes('"beta"'));
  let installed = false;
  await assert.rejects(runRepositoryCommandLine(['unknown'], { install: async () => { installed = true; } }), /expected/u);
  assert.equal(installed, false);
});
