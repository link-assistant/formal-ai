import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';

test('SQLite consumers provision a runtime without the experimental import warning', () => {
  for (const [path, name] of [
    ['.github/workflows/layered-ci.yml', 'js'],
    ['.github/workflows/native-type-check.yml', 'clippy'],
  ]) {
    const job = YAML.parse(readFileSync(path, 'utf8')).jobs[name];
    assert(job, `${path}: missing consumer ${name}`);
    const setup = job.steps.find(step => step.uses === 'actions/setup-node@v7');
    assert(setup, `${path}: consumer must not depend on the runner's ambient Node`);
    const version = String(setup.with['node-version']).split('.').map(Number);
    assert(version[0] === 24 && version[1] >= 15,
      `${path}: SQLite requires the Node 24 release-candidate implementation`);
  }
});

test('the actual runtime reads and writes SQLite without hiding process warnings', {timeout: 5000}, () => {
  const result = spawnSync(process.execPath, ['--input-type=module', '-e', `
    import {DatabaseSync} from 'node:sqlite';
    const database = new DatabaseSync(':memory:');
    database.exec('CREATE TABLE sample (value TEXT NOT NULL)');
    database.prepare('INSERT INTO sample VALUES (?)').run('preserved');
    if (database.prepare('SELECT value FROM sample').get().value !== 'preserved') {
      throw new Error('SQLite round trip failed');
    }
    database.close();
    process.emitWarning('diagnostic sentinel', {code: 'CI_DIAGNOSTIC_SENTINEL'});
  `], {encoding: 'utf8', timeout: 3000, maxBuffer: 65536,
    env: {...process.env, NODE_OPTIONS: '--max-old-space-size=64'}});
  assert.equal(result.error, undefined, String(result.error));
  assert.equal(result.status, 0, result.stderr);
  assert.doesNotMatch(result.stderr, /ExperimentalWarning/u);
  assert.match(result.stderr, /CI_DIAGNOSTIC_SENTINEL.*diagnostic sentinel/u,
    'the warning channel must remain active');
});
