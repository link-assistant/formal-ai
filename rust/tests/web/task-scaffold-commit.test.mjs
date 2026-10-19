import assert from 'node:assert/strict';
import test from 'node:test';
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { installHost } from '../../../js/agentic/host.mjs';
import { recipeCommitCommand } from '../../../js/agentic/git_commit.mjs';
installHost({ readText: (path) => readFileSync(new URL('../../../' + path, import.meta.url), 'utf8'),
  parseLino: () => { throw new Error('this commit route reads shared template data only'); }, realm: {} });
const gitEnvironment = { ...process.env, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_TERMINAL_PROMPT: '0' };
const reference = 'https://github.com/example/held-out-project/issues/73';
const branch = 'feature/held-out-$symbol';
const timestamp = '2024-02-29T23:59:59.123Z';
const marker = (date = timestamp, destination = branch, target = reference) =>
  '# .gitkeep file auto-generated at ' + date + ' for PR creation at branch ' + destination + ' for issue ' + target;
const cases = [
  { name: 'exact owned marker without final newline', content: marker(), remove: true },
  { name: 'a nested tool CWD never grants root-scaffold ownership', content: marker(), nested: true, remove: false },
  { name: 'exact owned marker with one final newline', content: marker() + '\n', remove: true },
  { name: 'ordinary manual root marker', content: 'Keep this directory.\n', remove: false },
  { name: 'a different branch owns its generated marker', content: marker(timestamp, 'foreign-branch'), remove: false },
  { name: 'a different work item owns its generated marker', content: marker(timestamp, branch, reference + '9'), remove: false },
  { name: 'multiple lines cannot become a scaffold protocol', content: marker() + '\nmanual\n', remove: false },
  { name: 'a second blank line remains caller-owned', content: marker() + '\n\n', remove: false },
  { name: 'an invalid Gregorian date is not canonical', content: marker('2026-02-30T01:02:03.004Z'), remove: false },
  { name: 'century leap rules remain exact', content: marker('2100-02-29T01:02:03.004Z'), remove: false },
  { name: 'noncanonical hour is not a timestamp', content: marker('2024-02-29T24:00:00.000Z'), remove: false },
  { name: 'a symlink never grants scaffold ownership', content: marker(), symlink: true, remove: false },
  { name: 'an untracked generated-looking file stays untracked', content: marker(), untracked: true, remove: false },
  { name: 'a staged caller edit stays staged', content: marker(), staged: true, remove: false },
  { name: 'a working-tree caller edit stays untouched', content: marker(), dirty: true, remove: false },
  { name: 'an explicitly declared recipe marker is retained', content: marker(), declared: true, remove: false },
  { name: 'a refused commit restores only its clean owned marker', content: marker(), hookFailure: true, remove: false },
  { name: 'a missing declared path restores the owned marker', content: marker(), missing: true, remove: false },
];
for (const example of cases) test(example.name, () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-task-scaffold-'));
  const workspace = join(directory, 'workspace'), remote = join(directory, 'remote.git');
  mkdirSync(workspace);
  const git = (...args) => execFileSync('git', args, { cwd: workspace, env: gitEnvironment, timeout: 5000, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  try {
    execFileSync('git', ['init', '--bare', '-q', remote], { env: gitEnvironment, timeout: 5000 });
    git('init', '-q', '--initial-branch=' + branch);
    git('config', 'user.name', 'Scaffold fixture'); git('config', 'user.email', 'fixture@example.invalid');
    const emptyHooks = join(directory, 'empty-hooks'); mkdirSync(emptyHooks); git('config', 'core.hooksPath', emptyHooks);
    git('config', 'commit.gpgSign', 'false');
    writeFileSync(join(workspace, 'foreign.txt'), 'original caller bytes\n');
    const location = example.nested ? join(workspace, 'subdirectory') : workspace;
    if (example.nested) mkdirSync(location);
    const scaffoldPath = example.nested ? 'subdirectory/.gitkeep' : '.gitkeep';
    const artifactPath = example.nested ? 'subdirectory/nested/owned.txt' : 'nested/owned.txt';
    const scaffold = join(location, '.gitkeep');
    if (!example.untracked) {
      if (example.symlink) { writeFileSync(join(workspace, 'marker-target'), example.content); symlinkSync('marker-target', scaffold); }
      else writeFileSync(scaffold, example.content);
    }
    git('add', '-A'); git('commit', '-q', '-m', 'initial fixture');
    git('remote', 'add', 'origin', remote); git('push', '-q', '-u', 'origin', branch);
    const originalHead = git('rev-parse', 'HEAD');
    const originalScaffoldBlob = example.untracked ? null : git('rev-parse', 'HEAD:' + scaffoldPath);
    if (example.untracked) writeFileSync(scaffold, example.content);
    if (example.staged || example.dirty) {
      writeFileSync(scaffold, example.content + '\ncaller-owned change');
      if (example.staged) git('add', '--', '.gitkeep');
    }
    writeFileSync(join(workspace, 'foreign.txt'), 'unrelated staged caller bytes\n'); git('add', '--', 'foreign.txt');
    const foreignIndex = git('rev-parse', ':foreign.txt');
    mkdirSync(join(location, 'nested')); writeFileSync(join(location, 'nested/owned.txt'), 'new declared source\n');
    const recipe = { path: 'nested/owned.txt', supporting_files: example.declared ? [{ path: '.gitkeep', source: example.content }] : [] };
    if (example.missing) rmSync(join(workspace, 'nested/owned.txt'));
    if (example.hookFailure) {
      const hooks = join(directory, 'hooks'); mkdirSync(hooks);
      writeFileSync(join(hooks, 'pre-commit'), '#!/bin/sh\nexit 17\n', { mode: 0o700 }); git('config', 'core.hooksPath', hooks);
    }
    const beforeScaffold = example.symlink ? null : readFileSync(scaffold, 'utf8');
    const command = recipeCommitCommand(recipe, { branch, reference, bindingRefused: false });
    const result = spawnSync('/bin/sh', ['-c', command], { cwd: location, env: gitEnvironment, encoding: 'utf8', timeout: 15000 });
    if (example.hookFailure || example.missing) {
      assert.notEqual(result.status, 0); assert.equal(git('rev-parse', 'HEAD'), originalHead);
      assert.equal(readFileSync(scaffold, 'utf8'), beforeScaffold); assert.equal(git('diff', '--name-only', '--', '.gitkeep'), '');
      assert.equal(git('diff', '--cached', '--name-only', '--', '.gitkeep'), '');
    } else {
      assert.equal(result.status, 0, result.stderr);
      assert.equal(git('show', 'HEAD:' + artifactPath), 'new declared source');
      assert.equal(git('rev-parse', 'origin/' + branch), git('rev-parse', 'HEAD'));
      if (example.remove) {
        assert.equal(existsSync(scaffold), false);
        assert.notEqual(spawnSync('git', ['cat-file', '-e', 'HEAD:.gitkeep'], { cwd: workspace, env: gitEnvironment, timeout: 5000 }).status, 0);
        assert.match(git('diff-tree', '--no-commit-id', '--name-status', '-r', 'HEAD'), /D\s+\.gitkeep/u);
      } else if (example.symlink) {
        assert.equal(lstatSync(scaffold).isSymbolicLink(), true);
        assert.equal(readFileSync(join(workspace, 'marker-target'), 'utf8'), example.content);
      } else assert.equal(readFileSync(scaffold, 'utf8'), beforeScaffold);
      if (!example.remove && !example.declared && !example.untracked) {
        assert.equal(git('rev-parse', 'HEAD:' + scaffoldPath), originalScaffoldBlob);
      }
      if (example.staged) assert.equal(git('diff', '--cached', '--name-only', '--', '.gitkeep'), '.gitkeep');
      if (example.untracked) assert.match(git('status', '--porcelain', '--', '.gitkeep'), /^\?\? /u);
    }
    assert.equal(git('show', 'HEAD:foreign.txt'), 'original caller bytes');
    assert.equal(git('rev-parse', ':foreign.txt'), foreignIndex);
    assert.equal(readFileSync(join(workspace, 'foreign.txt'), 'utf8'), 'unrelated staged caller bytes\n');
    assert.equal(git('diff', '--cached', '--name-only', '--', 'foreign.txt'), 'foreign.txt');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('ambiguous request authority never emits cleanup or Git effects', () => {
  const command = recipeCommitCommand({ path: 'owned.txt', supporting_files: [] }, { bindingRefused: true, branch, reference });
  assert.doesNotMatch(command, /rm --|git add|git commit|git push/u);
  assert.match(command, /false$/u);
});
