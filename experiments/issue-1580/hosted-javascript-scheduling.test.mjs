// Replay retained hosted costs without executing the functional suite.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import {durationLookup, partitionProblems, planShards} from '../../scripts/lib/ci-speed-shards.mjs';
import {parseTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';

for (const capture of ['8d8', '9e945']) test('deployed JavaScript plan retains every suite and fits hosted costs: ' + capture, () => {
  const identity = JSON.parse(readFileSync(new URL('./fixtures/hosted-javascript-' + capture + '/identity.json', import.meta.url), 'utf8'));
  const observations = readFileSync(new URL('./fixtures/hosted-javascript-' + capture + '/durations.jsonl', import.meta.url), 'utf8')
    .trimEnd().split('\n').map(line => JSON.parse(line));
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert(observations.every(row => row.complete && identity.jobs.some(job => job.job === row.job)));
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const secondsOf = durationLookup(parseTestDurations(recorded), Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1]));
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs'))
    .map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, secondsOf);
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  for (const [index, shard] of shards.entries()) {
    const load = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) {
      let worker = 0;
      for (let other = 1; other < load.length; other++) if (load[other] < load[worker]) worker = other;
      load[worker] += costs(file);
    }
    assert(Math.max(...load) <= 270, 'shard ' + (index + 1) + ' modeled seconds: ' + Math.max(...load));
  }
});

test('latest unchanged suite costs fit the plan and all split provenance scenarios start separately', () => {
  const fixture = new URL('./fixtures/hosted-javascript-de79e6ec/', import.meta.url);
  const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
  const partition = JSON.parse(readFileSync(new URL('scenario-partition.json', fixture), 'utf8'));
  const parent = 'rust/tests/web/full-provenance-cli.test.mjs';
  const parentCost = observations.find(row => row.file === parent).seconds;
  assert(parentCost > 270, 'the old indivisible suite cannot be repaired by reweighting');
  const measured = new Map(observations.filter(row => row.file !== parent).map(row => [row.file, row.seconds]));
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const weights = parseTestDurations(recorded);
  const secondsOf = durationLookup(weights, Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1]));
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, secondsOf);
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const selected = partition.scenarios.map(({path}) => shards.findIndex(shard => shard.includes(path)));
  assert.equal(new Set(selected).size, 3);
  for (const {path} of partition.scenarios) {
    assert(weights.get(path) >= parentCost, 'new scenario scheduling inherits the conservative parent bound');
    assert(shards.find(shard => shard.includes(path)).indexOf(path) < 4);
  }
  // Split-scenario durations remain unobserved here; hosted execution is required.
  // This replay asserts headroom only for the unchanged, measured source owners.
  for (const [index, shard] of shards.entries()) {
    const load = Array(4).fill(0);
    for (const file of shard.filter(file => measured.has(file))) {
      const worker = load.indexOf(Math.min(...load)); load[worker] += measured.get(file);
    }
    assert(Math.max(...load) <= 270, 'unchanged measured shard ' + (index + 1) + ': ' + Math.max(...load));
  }
});
