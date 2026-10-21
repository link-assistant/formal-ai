import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host, installHost} from '../../../js/agentic/host.mjs';
import {planRestart} from '../../../js/agentic/restart_feedback.mjs';
import {boundedCommand} from '../../../js/server/bounded-command.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
  const installed = host();
  const policy = fs.readFileSync(new URL('../../../data/meta/restart-hygiene.lino', import.meta.url), 'utf8');
  installHost({...installed, readText: file => file === 'data/meta/restart-hygiene.lino'
    ? policy : installed.readText(file)});
});
const originalRestart = fs.readFileSync(new URL('../fixtures/issue-1364-claude-restart.txt', import.meta.url), 'utf8');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
async function execute(directory, file, argumentsValue) {
  return boundedCommand(file, argumentsValue, {cwd:directory, timeout:10000,
    stdoutLimitBytes:1048576, stderrLimitBytes:1048576});
}
async function git(directory, ...argumentsValue) {
  const result = await execute(directory, 'git', argumentsValue);
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr.toString('utf8'));
  assert.equal(result.cleanup_complete, true);
  return result.stdout.toString('utf8').trim();
}
async function fixture(prepare) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-restart-hygiene-'));
  const directory = path.join(root, 'work');
  const origin = path.join(root, 'origin.git');
  fs.mkdirSync(directory);
  await git(root, 'init', '--bare', origin);
  await git(directory, 'init', '-b', 'actual-work');
  await git(directory, 'config', 'user.name', 'Hermetic fixture');
  await git(directory, 'config', 'user.email', 'fixture@example.invalid');
  await git(directory, 'config', 'core.autocrlf', 'false');
  fs.writeFileSync(path.join(directory, 'README.md'), 'Owned fixture source.\n');
  await prepare(directory, false);
  await git(directory, 'add', '--', '.');
  await git(directory, 'commit', '-qm', 'fixture initial source');
  await git(directory, 'remote', 'add', 'origin', origin);
  await git(directory, 'push', '-qu', 'origin', 'actual-work');
  await prepare(directory, true);
  return {root, directory, origin};
}
function prompt(listing) {
  return 'Issue to solve: https://github.com/example/fixture/issues/1\n'
    + 'Your prepared branch: prose-must-not-select-this-branch\n'
    + 'Your prepared Pull Request: https://github.com/example/fixture/pull/2\n'
    + 'UNCOMMITTED CHANGES DETECTED\n```\n' + listing + '\n```\n'
    + 'For EVERY listed file, commit solution source, ignore generated output, or delete only if unneeded.\n'
    + 'Continue until every numbered issue requirement is verified.';
}
async function recover(directory, task) {
  const messages = [{role:'user', content:task}];
  const commands = [];
  for (let attempt = 0; attempt < 6; attempt += 1) {
    const plan = planRestart(task, messages, ['bash']);
    if (plan === null || plan.kind === 'final') return {plan, commands};
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls.length, 1);
    const call = plan.calls[0];
    const command = JSON.parse(call.arguments).command;
    commands.push(command);
    const result = await execute(directory, '/bin/sh', ['-c', command]);
    const id = String(attempt);
    messages.push({role:'assistant', content:'', tool_calls:[{
      id, function:{name:call.tool, arguments:call.arguments},
    }]}, {role:'tool', name:call.tool, tool_call_id:id, content:JSON.stringify({
      exit_code:result.status, stdout:result.stdout.toString('utf8'),
      stderr:result.stderr.toString('utf8'), complete:result.cleanup_complete,
      ...(result.error ? {error:result.error.message} : {}),
    })});
  }
  assert.fail('restart recovery did not terminate within six owned commands');
}
test('original Kotlin restart shape preserves jar bytes and repairs Git before issue handoff', async () => {
  const artifact = Buffer.concat([Buffer.from('504b0304', 'hex'), Buffer.alloc(1024, 7)]);
  const originalIgnore = Buffer.from('# Preserve this existing rule\n*.cache\n');
  const owned = await fixture((directory, changed) => {
    if (!changed) {
      fs.writeFileSync(path.join(directory, 'Main.kt'), '// source owner\nfun main() { println("Hello, World!") }\n');
      fs.writeFileSync(path.join(directory, '.gitignore'), originalIgnore);
    } else fs.writeFileSync(path.join(directory, 'Main.jar'), artifact);
  });
  try {
    const result = await recover(owned.directory, originalRestart);
    assert.equal(result.plan, null, 'working-tree repair must resume full issue coverage');
    assert.match(result.commands[0], /git status --porcelain --untracked-files=all/);
    assert.equal(result.commands.some(command => command.includes('gh pr ready')), false);
    assert.equal(sha(fs.readFileSync(path.join(owned.directory, 'Main.jar'))), sha(artifact));
    assert.equal(await git(owned.directory, 'ls-files', '--', 'Main.jar'), '');
    assert.equal(await git(owned.directory, 'status', '--porcelain', '--untracked-files=all'), '');
    const ignore = fs.readFileSync(path.join(owned.directory, '.gitignore'));
    assert.deepEqual(ignore.subarray(0, originalIgnore.length), originalIgnore);
    assert.match(ignore.toString('utf8'), /^\/Main\.jar$/mu);
    assert.equal(await git(owned.directory, 'branch', '--show-current'), 'actual-work');
    assert.equal(await git(owned.directory, 'rev-parse', 'HEAD'),
      await git(owned.directory, 'rev-parse', 'origin/actual-work'));
    const head = await git(owned.directory, 'rev-parse', 'HEAD');
    assert.equal((await recover(owned.directory, prompt('?? Main.jar'))).plan, null);
    assert.equal(await git(owned.directory, 'rev-parse', 'HEAD'), head, 'clean resumed recovery is a no-op');
  } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
});
test('tracked Rust compiler output is removed only from index and committed without re-adding bytes', async () => {
  const artifact = Buffer.concat([Buffer.from('7f454c46', 'hex'), Buffer.alloc(4_600_000, 1)]);
  const owned = await fixture((directory, changed) => {
    if (!changed) {
      fs.writeFileSync(path.join(directory, 'main.rs'), '// source owner\nfn main() { println!("Hello, World!"); }\n');
      fs.writeFileSync(path.join(directory, 'main'), Buffer.from('7f454c4600', 'hex'));
    } else fs.writeFileSync(path.join(directory, 'main'), artifact);
  });
  try {
    const result = await recover(owned.directory, prompt(' M main'));
    assert.equal(result.plan, null);
    assert.equal(sha(fs.readFileSync(path.join(owned.directory, 'main'))), sha(artifact));
    assert.equal(await git(owned.directory, 'ls-files', '--', 'main'), '');
    assert.equal(await git(owned.directory, 'status', '--porcelain', '--untracked-files=all'), '');
    assert.match(await git(owned.directory, 'show', '--format=', '--name-status', 'HEAD'), /D\s+main/u);
  } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
});
test('collapsed owned internal-log directory expands to empty evidence without deleting bytes', async () => {
  const relative = 'dev/log/self-authored/issue-1/evidence/claude-stream.jsonl';
  const owned = await fixture((directory, changed) => {
    if (changed) {
      fs.mkdirSync(path.dirname(path.join(directory, relative)), {recursive:true});
      fs.writeFileSync(path.join(directory, relative), '');
    }
  });
  try {
    const result = await recover(owned.directory, prompt('?? dev/'));
    assert.equal(result.plan, null);
    assert.equal(fs.statSync(path.join(owned.directory, relative)).size, 0);
    assert.equal(await git(owned.directory, 'ls-files', '--', relative), '');
    assert.equal(await git(owned.directory, 'status', '--porcelain', '--untracked-files=all'), '');
  } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
});
test('unknown extra staged scope refuses before ignore or index changes', async () => {
  const owned = await fixture((directory, changed) => {
    if (!changed) fs.writeFileSync(path.join(directory, 'Main.kt'), 'fun main() {}\n');
    else {
      fs.writeFileSync(path.join(directory, 'Main.jar'), Buffer.from('504b030400', 'hex'));
      fs.writeFileSync(path.join(directory, 'unrelated.txt'), 'must remain staged\n');
    }
  });
  try {
    await git(owned.directory, 'add', '--', 'unrelated.txt');
    const head = await git(owned.directory, 'rev-parse', 'HEAD');
    const staged = await git(owned.directory, 'diff', '--cached', '--name-status');
    const result = await recover(owned.directory, prompt('?? Main.jar'));
    assert.equal(result.plan.kind, 'final');
    assert.equal(result.commands.length, 1);
    assert.equal(fs.existsSync(path.join(owned.directory, '.gitignore')), false);
    assert.equal(await git(owned.directory, 'rev-parse', 'HEAD'), head);
    assert.equal(await git(owned.directory, 'diff', '--cached', '--name-status'), staged);
  } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
});
for (const scenario of ['unknown-binary', 'missing-source', 'nonempty-internal-evidence', 'symbolic-output']) {
  test('unsafe recovery refuses without changing source/index/ignore: ' + scenario, async () => {
    let relative = scenario === 'unknown-binary' ? 'asset.bin' : 'Main.jar';
    if (scenario === 'nonempty-internal-evidence') relative = 'dev/log/self-authored/issue-1/evidence/codex-stream.jsonl';
    const owned = await fixture((directory, changed) => {
      if (!changed && scenario !== 'missing-source') fs.writeFileSync(path.join(directory, 'Main.kt'), 'fun main() {}\n');
      if (changed) {
        fs.mkdirSync(path.dirname(path.join(directory, relative)), {recursive:true});
        if (scenario === 'symbolic-output') {
          const outside = path.join(path.dirname(directory), 'outside-bytes');
          fs.writeFileSync(outside, Buffer.from('504b030400', 'hex'));
          fs.symlinkSync(outside, path.join(directory, relative));
        } else fs.writeFileSync(path.join(directory, relative), scenario === 'nonempty-internal-evidence'
          ? '{"retained":"content requires policy"}\n' : Buffer.from('504b030400', 'hex'));
      }
    });
    try {
      const head = await git(owned.directory, 'rev-parse', 'HEAD');
      const before = fs.readFileSync(path.join(owned.directory, relative));
      const result = await recover(owned.directory, prompt('?? ' + relative));
      assert.equal(result.plan.kind, 'final');
      assert.equal(fs.existsSync(path.join(owned.directory, '.gitignore')), false);
      assert.deepEqual(fs.readFileSync(path.join(owned.directory, relative)), before);
      assert.equal(await git(owned.directory, 'rev-parse', 'HEAD'), head);
      assert.equal(await git(owned.directory, 'diff', '--cached', '--name-status'), '');
    } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
  });
}
test('ordinary empty source and literal Git pathspec-looking names are committed rather than hidden', async () => {
  const owned = await fixture((directory, changed) => {
    if (changed) fs.writeFileSync(path.join(directory, ':source.txt'), '');
  });
  try {
    const result = await recover(owned.directory, prompt('?? :source.txt'));
    assert.equal(result.plan, null);
    assert.equal(fs.existsSync(path.join(owned.directory, '.gitignore')), false);
    assert.equal(await git(owned.directory, 'ls-files', '--', ':(literal):source.txt'), ':source.txt');
    assert.equal(await git(owned.directory, 'status', '--porcelain', '--untracked-files=all'), '');
  } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
});

test('replacement metacharacters stay literal in owned source/index operands', async () => {
  for (const relative of ['$&source.txt', '$`source.txt', "$'source.txt", '$$source.txt']) {
    const owned = await fixture((directory, changed) => {
      if (changed) fs.writeFileSync(path.join(directory, relative), 'literal source bytes\n');
    });
    try {
      const result = await recover(owned.directory, prompt('?? ' + relative));
      assert.equal(result.plan, null, relative);
      assert.equal(await git(owned.directory, 'ls-files', '--', ':(literal)' + relative), relative);
      assert.equal(await git(owned.directory, 'status', '--porcelain', '--untracked-files=all'), '');
    } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
  }
});
test('replacement metacharacters stay literal when untracking owned empty internal evidence', async () => {
  const relative = 'dev/log/self-authored/issue-1/evidence/$&-stream.jsonl';
  const owned = await fixture((directory, changed) => {
    fs.mkdirSync(path.dirname(path.join(directory, relative)), {recursive:true});
    fs.writeFileSync(path.join(directory, relative), changed ? '' : '{"legacy":"internal log"}\n');
  });
  try {
    const result = await recover(owned.directory, prompt(' M ' + relative));
    assert.equal(result.plan, null);
    assert.equal(fs.statSync(path.join(owned.directory, relative)).size, 0);
    assert.equal(await git(owned.directory, 'ls-files', '--', ':(literal)' + relative), '');
    assert.match(await git(owned.directory, 'show', '--format=', '--name-status', 'HEAD'), /D\s+dev\/log\/self-authored/u);
    assert.equal(await git(owned.directory, 'status', '--porcelain', '--untracked-files=all'), '');
  } finally { fs.rmSync(owned.root, {recursive:true, force:true}); }
});
