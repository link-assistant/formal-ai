// Keep the original fifteen cases in two isolated process groups. The ladder
// previously held this one file for 254.6s in run 45250430676.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const groups = ['learning', 'ladder'].map(name =>
  fileURLToPath(new URL('./support/meta-reasoner/' + name + '.test.mjs', import.meta.url)));
const env = {...process.env};
delete env.NODE_TEST_CONTEXT;
const result = spawnSync(process.execPath,
  ['--max-old-space-size=192', '--test', '--test-concurrency=2', '--test-timeout=210000', ...groups],
  {env, stdio: 'inherit', timeout: 225000});
assert.equal(result.error, undefined, 'both bounded meta-reasoner groups must finish');
assert.equal(result.status, 0, 'every original case must pass in its isolated group');
