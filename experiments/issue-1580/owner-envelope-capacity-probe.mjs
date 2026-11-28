// Rejected bounded sensitivity experiment: pool owner maxima across historical
// source versions, including owners since split or repaired. All eight candidates
// fail; this is not the deployed capacity selector or a functional execution.
import {readFileSync, readdirSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import assert from 'node:assert/strict';
import {durationLookup, partitionProblems, planShards, isLongestFirst} from '../../scripts/lib/ci-speed-shards.mjs';
import {parseTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';
const [output] = process.argv.slice(2);
assert(output, 'provide a JSON output file');
const root = 'experiments/issue-1580/fixtures/';
const text = readFileSync('data/meta/javascript-test-durations.lino', 'utf8');
const weights = parseTestDurations(text);
const fallback = Number(text.match(/^ {2}default-seconds (\S+)$/mu)[1]);
const secondsOf = durationLookup(weights, fallback);
const files = readdirSync('rust/tests/web').filter(name => name.endsWith('.test.mjs')).map(name => 'rust/tests/web/' + name);
const cohorts = readdirSync(root).filter(name => name.startsWith('hosted-javascript-')).map(name => {
  const directory = join(root, name);
  let identity;
  try { identity = JSON.parse(readFileSync(join(directory, 'identity.json'), 'utf8')); }
  catch (error) { if (error.code !== 'ENOENT') throw error; identity = {}; }
  const rows = file => readFileSync(file, 'utf8').trim().split('\n').map(JSON.parse);
  const previous = identity.unobservedCostsFrom ? rows(join(root, identity.unobservedCostsFrom, 'durations.jsonl')) : [];
  const observed = rows(join(directory, 'durations.jsonl'));
  const costs = new Map([...previous, ...observed].map(row => [row.file, row.seconds]));
  if (name.endsWith('de79e6ec')) costs.delete('rust/tests/web/full-provenance-cli.test.mjs');
  if (name.endsWith('53816c9e-partial')) {
    const split = JSON.parse(readFileSync(root + 'meta-reasoner-owner-split/observation.json', 'utf8'));
    const sums = ['learning', 'ladder'].map(group => split.cases.filter(row => row.group === group)
      .reduce((sum, row) => sum + row.seconds, 0));
    costs.set('rust/tests/web/meta-reasoner.test.mjs', Math.max(...sums)
      + Math.max(0, split.fileSeconds - sums.reduce((a, b) => a + b, 0))
      + split.modeledIndependentGroupStartupAllowanceSeconds);
  }
  return {name, identity, costs};
});
const envelope = new Map();
for (const {costs} of cohorts) for (const [file, cost] of costs) envelope.set(file, Math.max(envelope.get(file) ?? 0, cost));
const candidates = [];
for (let count = 9; count <= 16; count++) {
  const {shards} = planShards(files, count, secondsOf);
  const queue = (shard, cost) => {
    const workers = [0, 0, 0, 0];
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += cost(file);
    return Math.max(...workers);
  };
  const results = cohorts.map(({name, identity, costs}) => {
    const cost = durationLookup(costs, identity.unobservedFallbackSeconds ?? 0.34);
    const loads = shards.map(shard => queue(
      name.endsWith('de79e6ec') ? shard.filter(file => costs.has(file)) : shard, cost)
      + (identity.runnerOverheadAllowanceSeconds ?? 0));
    return {name, loads, maximumSeconds: Math.max(...loads)};
  });
  const conservativeQueues = shards.map(shard => queue(shard, secondsOf));
  const envelopeLoads = shards.map(shard => queue(shard, durationLookup(envelope, 0.4)) + 30);
  const constraints = {
    retainedOwnerEnvelope: envelopeLoads.every(seconds => seconds <= 270),
    exactPartition: Object.values(partitionProblems(files, shards)).every(rows => rows.length === 0),
    longestFirst: isLongestFirst(shards, secondsOf),
    deterministic: JSON.stringify(planShards([...files].reverse(), count, secondsOf).shards) === JSON.stringify(shards),
    unchangedDeadline: conservativeQueues.every(seconds => seconds < 360),
    retainedCohorts: results.every(row => row.maximumSeconds <= 270),
  };
  candidates.push({count, constraints, fits: Object.values(constraints).every(Boolean), conservativeQueues, envelopeLoads, results});
}
const selected = candidates.find(row => row.fits);
writeFileSync(output, JSON.stringify({sourceRun: 45271423335, candidateLimit: 8,
  sourceOwners: files.length, selectedCount: selected?.count ?? null,
  selection: 'smallest group count that fits every retained cohort and unchanged queue deadline',
  executionProof: false, autonomousAuthorshipCredit: 0, candidates}, null, 2) + '\n');
assert(selected, 'no finite capacity candidate fits all retained observations');
console.log(JSON.stringify({selectedCount: selected.count, candidates: candidates.map(row => ({
  count: row.count, fits: row.fits, maximumSeconds: Math.max(...row.results.map(item => item.maximumSeconds)),
}))}));
