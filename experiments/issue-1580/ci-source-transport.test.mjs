import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';
import test from 'node:test';
import YAML from 'yaml';
const root = fileURLToPath(new URL('../../', import.meta.url));
const read = name => readFileSync(join(root, name), 'utf8');

test('the actual layered sparse checkout retains consumed source fixtures', { timeout: 20_000 }, t => {
  const workflow = YAML.parse(read('.github/workflows/layered-ci.yml'));
  const candidates = Object.values(workflow.jobs).flatMap(job => job.steps ?? []);
  const checkout = candidates.find(step => step.with?.['sparse-checkout']?.includes('native-reasoning-budget'));
  assert.ok(checkout);
  const files = ['seed-record-readers.test.mjs', 'application-conversation-list.test.mjs'].flatMap(file => {
    const source = read('experiments/issue-1580/' + file);
    return [...source.matchAll(/new URL\('([^']+\.gz)'/gu)].map(match =>
      relative(root, fileURLToPath(new URL(match[1], new URL('./' + file, import.meta.url)))));
  });
  const captures = JSON.parse(read('experiments/issue-1580/fixtures/immutable-source-captures.json'));
  files.push(...captures.map(row => row.archivePath));
  assert.equal(files.length, 10);
  const directory = mkdtempSync(join(tmpdir(), 'issue-1580-sparse-fixtures-'));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const git = (...args) => execFileSync('git', args, { cwd: directory, stdio: 'pipe', timeout: 10_000,
    env: { ...process.env, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null' } });
  git('init', '-q');git('config', 'user.name', 'Fixture');git('config', 'user.email', 'fixture@example.invalid');
  const archive = 'experiments/issue-1580/evidence/checks/unconsumed-history.log.gz';
  for (const path of [...files, archive]) {
    mkdirSync(dirname(join(directory, path)), { recursive: true });
    writeFileSync(join(directory, path), 'bounded sparse-checkout operand');
  }
  git('add', '.');git('commit', '-qm', 'Sparse fixture input');
  git('sparse-checkout', 'init', '--no-cone');
  writeFileSync(join(directory, '.git/info/sparse-checkout'), checkout.with['sparse-checkout']);
  git('read-tree', '-mu', 'HEAD');
  for (const path of files) assert.ok(existsSync(join(directory, path)), path);
  assert.equal(existsSync(join(directory, archive)), false);
});

test('the structured-output VM loads the actual seed-reader prerequisite', () => {
  const source = read('rust/tests/web/issue-1586-structured-output.test.mjs');
  const scripts = source.match(/for \(const script of (\[[^\n]+\])\)/u)?.[1];
  assert.ok(scripts);
  const context = vm.createContext({ self: {} });
  for (const file of vm.runInNewContext(scripts)) vm.runInContext(read('js/' + file), context);
  assert.equal(typeof context.self.FormalAiSeed.parse, 'function');
  assert.equal(context.self.FormalAiSeed.parse('record').name, 'record');
});
