// Replay retained hosted costs without executing the functional suite.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import {durationLookup, partitionProblems, planShards} from '../../scripts/lib/ci-speed-shards.mjs';
import {parseTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';

test('deployed JavaScript plan retains every suite and fits the retained four-worker cost model', () => {
  const identity = JSON.parse(readFileSync(new URL('./fixtures/hosted-javascript-8d8/identity.json', import.meta.url), 'utf8'));
  const observations = readFileSync(new URL('./fixtures/hosted-javascript-8d8/durations.jsonl', import.meta.url), 'utf8')
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
