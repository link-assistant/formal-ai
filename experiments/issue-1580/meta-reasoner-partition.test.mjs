import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import vm from 'node:vm';
import {fileURLToPath} from 'node:url';

const fixture = new URL('./fixtures/meta-reasoner-owner-split/', import.meta.url);
const observation = JSON.parse(readFileSync(new URL('observation.json', fixture), 'utf8'));
const original = readFileSync(new URL('original.test-source.mjs', fixture), 'utf8');
const owners = ['learning', 'ladder'];
function registrations(source) {
  const cases = [];
  vm.runInNewContext(source.replace(/^import [^\n]*\n/gmu, '').replaceAll('import.meta.url', '"file:///unexecuted-registration-context.mjs"'), {
    captured: () => false,
    test: (title, options, callback) => {
      const body = typeof options === 'function' ? options : callback;
      cases.push({title, body: body.toString(), skipped: typeof options === 'object' && Boolean(options.skip)});
    },
  }, {timeout: 1000});
  return cases;
}

test('all fifteen original meta-reasoner callbacks and skip controls occur exactly once in isolated groups', () => {
  assert.equal(createHash('sha256').update(original).digest('hex'), observation.originalSourceSha256);
  const before = registrations(original.slice(original.indexOf('\ntest(')));
  assert.equal(before.length, 15);
  const sources = owners.map(owner => readFileSync(
    new URL('../../rust/tests/web/support/meta-reasoner/' + owner + '.test.mjs', import.meta.url), 'utf8'));
  for (const block of original.slice(original.indexOf('\ntest(')).trim().split(/(?=^test\()/gmu))
    assert.equal(sources.filter(source => source.includes(block.trim())).length, 1, 'unaltered original source block');
  const groups = sources.map(registrations);
  assert.deepEqual(groups.map(group => group.length), [14, 1]);
  const after = groups.flat().sort((a, b) => a.title.localeCompare(b.title));
  assert.deepEqual(after, before.sort((a, b) => a.title.localeCompare(b.title)));
  assert.equal(new Set(after.map(row => row.title)).size, 15);
  assert.equal(after.filter(row => row.skipped).length, 2);
  assert.deepEqual(groups[1].map(row => row.title), ['every rung of the task ladder is derived, never handled']);
});

test('the bounded two-process owner propagates child failures and preserves every group', () => {
  const url = new URL('../../rust/tests/web/meta-reasoner.test.mjs', import.meta.url);
  const source = readFileSync(url, 'utf8').replace(/^import [^\n]*\n/gmu, '')
    .replaceAll('import.meta.url', JSON.stringify(url.href));
  for (const result of [{status: 0}, {status: 1}, {status: null, error: new Error('bounded child failure')}]) {
    const env = {NODE_TEST_CONTEXT: 'child-v8', retained: 'yes'};
    let calls = 0;
    const run = () => vm.runInNewContext(source, {
      assert, URL, fileURLToPath, process: {execPath: '/node', env},
      spawnSync: (executable, args, options) => {
        calls++;
        assert.equal(executable, '/node');
        assert.deepEqual(Array.from(args.slice(0, 4)),
          ['--max-old-space-size=192', '--test', '--test-concurrency=2', '--test-timeout=210000']);
        assert.deepEqual(Array.from(args.slice(4)), owners.map(name => fileURLToPath(
          new URL('../../rust/tests/web/support/meta-reasoner/' + name + '.test.mjs', import.meta.url))));
        assert.equal(options.env.NODE_TEST_CONTEXT, undefined);
        assert.equal(options.env.retained, 'yes');
        assert.equal(options.stdio, 'inherit');
        assert.equal(options.timeout, 225000);
        return result;
      },
    }, {timeout: 1000});
    if (result.status === 0) run();
    else assert.throws(run);
    assert.equal(calls, 1);
    assert.equal(env.NODE_TEST_CONTEXT, 'child-v8');
  }
});

test('the failed hosted cohort retains 819 durations, one failed file and 103 unexecuted files', () => {
  const base = new URL('./fixtures/hosted-javascript-53816c9e-partial/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', base), 'utf8'));
  const raw = readFileSync(new URL('durations.jsonl', base), 'utf8');
  const rows = raw.trim().split('\n').map(JSON.parse);
  const run = JSON.parse(readFileSync(new URL('run-observation.json', base), 'utf8'));
  assert.equal(identity.cohortComplete, false);
  assert.equal(identity.executionProof, false);
  assert.equal(identity.runConclusion, 'failure');
  assert.equal(identity.run, run.id);
  assert.equal(identity.sourceCommit, run.head_sha);
  assert.equal(identity.completeObservations, 819);
  assert.equal(rows.length, 819);
  assert.equal(new Set(rows.map(row => row.file)).size, 819);
  assert.deepEqual(rows.filter(row => row.result === 'failed').map(row => row.file),
    ['rust/tests/web/ci-speed.test.mjs']);
  assert.equal(identity.unobservedFiles.length, 103);
  assert.equal(new Set([...rows.map(row => row.file), ...identity.unobservedFiles]).size, 922);
  assert.deepEqual(identity.warnings.map(row => row.shard), [5]);
  assert.equal(identity.unobservedCostsFrom, 'hosted-javascript-e46254ac');
  assert.equal(createHash('sha256').update(raw).digest('hex'), identity.durationsSha256);
  assert.equal(createHash('sha256').update(readFileSync(new URL('prior-scheduling-floor.lino', base))).digest('hex'),
    identity.priorSchedulingFloorSha256);
});

test('the changed critical-path model retains the original over-threshold owner and every case timing', () => {
  assert.equal(observation.cases.length, 15);
  assert.equal(observation.cases.filter(row => row.skipped).length, 2);
  assert(observation.fileSeconds + 30 > 270, 'the original owner cannot fit by repartitioning alone');
  const groups = owners.map(group => observation.cases.filter(row => row.group === group)
    .reduce((sum, row) => sum + row.seconds, 0));
  const setup = Math.max(0, observation.fileSeconds - groups.reduce((a, b) => a + b, 0));
  assert.equal(observation.modeledIndependentGroupStartupAllowanceSeconds, 5);
  assert(Math.max(...groups) + setup + 5 + 30 < 270);
  assert.equal(observation.executionProof, false, 'a modeled parallel run is not execution evidence');
});
