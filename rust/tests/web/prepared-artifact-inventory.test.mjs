import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const api = await import('../../../js/agentic/recipe_artifact_inventory.mjs');
const {
  preparedRecipeHygiene,
  preparedWorkHygiene
} = await import('../../../js/agentic/restart_feedback.mjs');
const evidenceRoot = api.recipeArtifactEvidenceRoot('https://github.com/owner/repository/issues/12');
const sourcePaths = ['main.rs'];
const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-inventory-'));
const git = (...argumentsValue) => execFileSync('git', argumentsValue, {
  cwd: workspace,
  timeout: 10000,
  stdio: 'pipe'
});
const write = (relative, bytes) => {
  fs.mkdirSync(path.dirname(path.join(workspace, relative)), {
    recursive: true
  });
  fs.writeFileSync(path.join(workspace, relative), bytes);
};
git('init', '-q');
write('main.rs', 'fn main() {}\n');
write('main', Buffer.concat([Buffer.from('7f454c46', 'hex'), Buffer.from('owned-output')]));
write(evidenceRoot + 'codex-stream.jsonl', '');
write(evidenceRoot + 'session-id.txt', 'formal-ai session 1234-abcd\nformal-ai model formal-ai/0.1.0\n');
write('dev/log/self-authored/issue-13/evidence/codex-stream.jsonl', 'foreign');
git('add', '.');
git('-c', 'user.name=Inventory fixture', '-c', 'user.email=inventory@example.invalid', 'commit', '-qm', 'fixture');
const command = api.recipeArtifactInventoryCommand(sourcePaths, evidenceRoot);
const observe = () => spawnSync('/bin/bash', ['-c', command], {
  cwd: workspace,
  timeout: 15000,
  maxBuffer: 1048576,
  encoding: 'utf8'
});
try {
  test('only exact validated issue roots are admitted', () => {
    assert.equal(evidenceRoot, 'dev/log/self-authored/issue-12/evidence/');
    for (const reference of ['https://github.com/owner/repository/pull/12', 'https://github.com/owner/repository/issues/0']) assert.equal(api.recipeArtifactEvidenceRoot(reference), null);
    assert.equal(api.recipeArtifactInventoryCommand(sourcePaths, 'dev/log/self-authored/'), null);
    assert.equal(api.recipeArtifactInventoryCommand(sourcePaths, evidenceRoot + '\n'), null);
  });
  const observed = observe();
  test('clean tracked output and exact producer metadata are discovered without effects', () => {
    assert.equal(observed.status, 0, observed.stderr);
    const value = api.recipeArtifactInventoryResult(observed.stdout, sourcePaths, evidenceRoot);
    assert.deepEqual(value.ownedPaths, [evidenceRoot + 'codex-stream.jsonl', evidenceRoot + 'session-id.txt', 'main']);
    assert.equal(value.sessionPolicy.path, evidenceRoot + 'session-id.txt');
    assert.equal(fs.readFileSync(path.join(workspace, 'main')).subarray(0, 4).toString('hex'), '7f454c46');
    assert.equal(fs.readFileSync(path.join(workspace, 'dev/log/self-authored/issue-13/evidence/codex-stream.jsonl'), 'utf8'), 'foreign');
    assert.equal(git('status', '--porcelain').toString(), '');
  });
  test('foreign, duplicate and mismatched source receipts refuse', () => {
    const value = JSON.parse(observed.stdout);
    for (const changed of [{
      ...value,
      evidenceRoot: evidenceRoot.replace('12', '13')
    }, {
      ...value,
      sourcePaths: ['other.rs']
    }, {
      ...value,
      ownedPaths: ['README.md']
    }, {
      ...value,
      ownedPaths: ['main', 'main']
    }, {
      ...value,
      complete: false
    }]) assert.equal(api.recipeArtifactInventoryResult(JSON.stringify(changed), sourcePaths, evidenceRoot), null);
  });
  test('ordinary source text under a compiler output spelling refuses', () => {
    write('main', 'ordinary source');
    assert.notEqual(observe().status, 0);
    write('main', Buffer.from('7f454c46', 'hex'));
  });
  test('nonempty stream is not discarded', () => {
    write(evidenceRoot + 'codex-stream.jsonl', 'valuable evidence');
    assert.notEqual(observe().status, 0);
    write(evidenceRoot + 'codex-stream.jsonl', '');
  });
  test('session prose and extra lines refuse', () => {
    write(evidenceRoot + 'session-id.txt', 'formal-ai session 123\nformal-ai model formal-ai/0.1\nextra\n');
    assert.notEqual(observe().status, 0);
    write(evidenceRoot + 'session-id.txt', 'formal-ai session 123\nformal-ai model formal-ai/0.1\n');
  });
  test('tracked symlink refuses without reading its foreign target', () => {
    fs.unlinkSync(path.join(workspace, 'main'));
    fs.symlinkSync('/etc/passwd', path.join(workspace, 'main'));
    assert.notEqual(observe().status, 0);
    fs.unlinkSync(path.join(workspace, 'main'));
    write('main', Buffer.from('7f454c46', 'hex'));
  });
  test('second pass excludes physically retained untracked artifacts', () => {
    git('rm', '--cached', '--', 'main', evidenceRoot + 'codex-stream.jsonl', evidenceRoot + 'session-id.txt');
    const second = observe();
    assert.equal(second.status, 0, second.stderr);
    assert.deepEqual(api.recipeArtifactInventoryResult(second.stdout, sourcePaths, evidenceRoot).ownedPaths, []);
    assert(fs.existsSync(path.join(workspace, 'main')));
  });
} finally {
  test.after(() => fs.rmSync(workspace, {
    recursive: true,
    force: true
  }));
}
test('command plan operand uses native serde canonical object-key ordering', () => {
  const prefix = 'command -v node >/dev/null 2>&1 && node --input-type=module -e';
  assert(command.startsWith(prefix));
  const extractor = "python3 -c 'import sys;print(sys.argv[-1])'" + command.slice(prefix.length);
  const operand = execFileSync('/bin/bash', ['-c', extractor], {
    encoding: 'utf8',
    timeout: 10000,
    maxBuffer: 1048576
  }).trimEnd();
  const ordered = value => Array.isArray(value) ? value.map(ordered) : value !== null && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key
    , ordered(value[key])])) : value;
  assert.equal(operand, JSON.stringify(ordered(JSON.parse(operand))));
});
function hygieneFixture() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-inventory-hygiene-'));
  const localGit = (...argumentsValue) => execFileSync('git', argumentsValue, {
    cwd: directory,
    timeout: 10000,
    stdio: 'pipe'
  }).toString();
  const save = (relative, bytes) => {
    const target = path.join(directory, relative);
    fs.mkdirSync(path.dirname(target), {
      recursive: true
    });
    fs.writeFileSync(target, bytes);
  };
  localGit('init', '-q');
  save('main.rs', 'fn main() {}\n');
  save('main', Buffer.from('7f454c46', 'hex'));
  save(evidenceRoot + 'session-id.txt', 'formal-ai session exact-123\nformal-ai model formal-ai/0.1.2\n');
  localGit('add', '.');
  localGit('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'fixture');
  const execute = commandValue => spawnSync('/bin/bash', ['-c', commandValue], {
    cwd: directory,
    timeout: 15000,
    maxBuffer: 1048576,
    encoding: 'utf8'
  });
  const result = execute(api.recipeArtifactInventoryCommand(sourcePaths, evidenceRoot));
  assert.equal(result.status, 0, result.stderr);
  const inventory = api.recipeArtifactInventoryResult(result.stdout, sourcePaths, evidenceRoot);
  return {
    directory,
    localGit,
    save,
    execute,
    inventory
  };
}
test('qualified clean tracked artifacts untrack without physical deletion or new source staging', () => {
  const fixture = hygieneFixture();
  try {
    const hygiene = preparedRecipeHygiene(sourcePaths, [], fixture.inventory);
    assert(hygiene);
    assert.deepEqual(hygiene.commitFiles, ['.gitignore']);
    const head = fixture.localGit('rev-parse', 'HEAD');
    const before = fs.readFileSync(path.join(fixture.directory, evidenceRoot + 'session-id.txt'));
    const observed = fixture.execute(hygiene.command);
    assert.equal(observed.status, 0, observed.stderr);
    assert.equal(fixture.localGit('ls-files', '--', 'main', evidenceRoot + 'session-id.txt'), '');
    assert.deepEqual(fs.readFileSync(path.join(fixture.directory, evidenceRoot + 'session-id.txt')), before);
    assert.equal(fs.readFileSync(path.join(fixture.directory, 'main')).toString('hex'), '7f454c46');
    assert.equal(fixture.localGit('rev-parse', 'HEAD'), head);
    assert.equal(fixture.localGit('diff', '--cached', '--name-only', '--', 'main.rs'), '');
  } finally {
    fs.rmSync(fixture.directory, {
      recursive: true,
      force: true
    });
  }
});
test('invented session policy, source mismatch and other issue scope are refused', () => {
  const fixture = hygieneFixture();
  try {
    assert.equal(preparedRecipeHygiene(sourcePaths, [], {
      ...fixture.inventory,
      sessionPolicy: {
        ...fixture.inventory.sessionPolicy,
        pattern: '.*'
      }
    }), null);
    assert.equal(preparedRecipeHygiene(['other.rs'], [], fixture.inventory), null);
    assert.equal(preparedRecipeHygiene(sourcePaths, ['dev/log/self-authored/issue-13/evidence/session-id.txt'], fixture.inventory), null);
    assert.equal(fs.existsSync(path.join(fixture.directory, '.gitignore')), false);
  } finally {
    fs.rmSync(fixture.directory, {
      recursive: true,
      force: true
    });
  }
});
test('session bytes are revalidated after inventory before any index or ignore effect', () => {
  const fixture = hygieneFixture();
  try {
    const hygiene = preparedRecipeHygiene(sourcePaths, [], fixture.inventory);
    fixture.save(evidenceRoot + 'session-id.txt', 'unrecognized valuable evidence\n');
    const index = fixture.localGit('ls-files', '--stage');
    const result = fixture.execute(hygiene.command);
    assert.notEqual(result.status, 0);
    assert.equal(fixture.localGit('ls-files', '--stage'), index);
    assert.equal(fs.existsSync(path.join(fixture.directory, '.gitignore')), false);
    const ordinary = preparedWorkHygiene([evidenceRoot + 'session-id.txt']);
    assert.notEqual(fixture.execute(ordinary.command).status, 0);
  } finally {
    fs.rmSync(fixture.directory, {
      recursive: true,
      force: true
    });
  }
});
