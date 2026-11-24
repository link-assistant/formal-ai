// Finite pure scheduling replay: no functional tests, builds or subprocesses.
import {readFileSync, readdirSync, writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import {join} from 'node:path';
import {parseTestDurations, mergeJavaScriptDurations} from '../../scripts/lib/ci-speed-durations.mjs';
import {durationLookup, partitionProblems, isLongestFirst, planShards} from '../../scripts/lib/ci-speed-shards.mjs';

const [output] = process.argv.slice(2);
assert(output, 'provide a result directory');
const root = 'experiments/issue-1580/fixtures/';
const current = root + 'hosted-javascript-e46254ac/';
const prior = readFileSync(current + 'prior-scheduling-floor.lino', 'utf8');
const rows = readFileSync(current + 'durations.jsonl', 'utf8').trim().split('\n').map(JSON.parse);
const files = readdirSync('rust/tests/web/').filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
// Freeze this historical policy reconstruction at its original nine groups.
// New capacity evidence is replayed separately by probe-hosted-javascript-capacity.mjs.
const count = 9;
const capturedCohorts = ["hosted-javascript-2809ce65", "hosted-javascript-53816c9e-partial", "hosted-javascript-754fd314", "hosted-javascript-79a024bf", "hosted-javascript-8d8", "hosted-javascript-9e945", "hosted-javascript-a558f834", "hosted-javascript-bbc0ac4d", "hosted-javascript-c55a5f9c", "hosted-javascript-c90f94ca", "hosted-javascript-de79e6ec", "hosted-javascript-e46254ac"];
const cohorts = readdirSync(root).filter(name => capturedCohorts.includes(name)).map(name => {
  const identityPath = join(root, name, 'identity.json');
  let identity;
  try { identity = JSON.parse(readFileSync(identityPath, 'utf8')); }
  catch (error) { if (error.code !== 'ENOENT') throw error; identity = {}; }
  const observations = readFileSync(join(root, name, 'durations.jsonl'), 'utf8').trim().split('\n').map(JSON.parse);
  const fallbackRows = identity.unobservedCostsFrom
    ? readFileSync(join(root, identity.unobservedCostsFrom, 'durations.jsonl'), 'utf8').trim().split('\n').map(JSON.parse)
    : [];
  const measured = new Map([...fallbackRows, ...observations].map(row => [row.file, row.seconds]));
  if (name.endsWith('de79e6ec')) measured.delete('rust/tests/web/full-provenance-cli.test.mjs');
  // This owner now executes the unchanged case groups in two processes.
  // Retain each raw file timing; model the changed critical path explicitly.
  const split = JSON.parse(readFileSync(root + 'meta-reasoner-owner-split/observation.json', 'utf8'));
  const groupSeconds = ['learning', 'ladder'].map(group => split.cases.filter(row => row.group === group)
    .reduce((sum, row) => sum + row.seconds, 0));
  const modeled = Math.max(...groupSeconds) + Math.max(0, split.fileSeconds - groupSeconds.reduce((a, b) => a + b, 0))
    + split.modeledIndependentGroupStartupAllowanceSeconds;
  if (name.endsWith('53816c9e-partial')) measured.set('rust/tests/web/meta-reasoner.test.mjs', modeled);
  return {name, identity, measured};
});
const evaluate = text => {
  const fallback = Number(text.match(/^ {2}default-seconds (\S+)$/mu)[1]);
  const {shards} = planShards(files, count, durationLookup(parseTestDurations(text), fallback));
  return cohorts.map(({name, identity, measured}) => {
    const cost = durationLookup(measured, identity.unobservedFallbackSeconds ?? 0.34);
    const loads = shards.map(shard => {
      const workers = Array(identity.modeledWorkers ?? 4).fill(0);
      for (const file of shard) {
        if (name.endsWith('de79e6ec') && !measured.has(file)) continue;
        workers[workers.indexOf(Math.min(...workers))] += cost(file);
      }
      return Math.max(...workers) + (identity.runnerOverheadAllowanceSeconds ?? 0);
    });
    return {name, maximumSeconds: Math.max(...loads), loads};
  });
};
const candidates = [];
const priorWeights = parseTestDurations(prior);
const pathOf = name => 'rust/tests/web/' + name + '.test.mjs';
const full = pathOf('full-provenance-cli');
const fresh = pathOf('fresh-planner-recomputation');
const memory = pathOf('provenance-logical-memory-capture');
const reasoner = pathOf('meta-reasoner');
const production = pathOf('issue-1165-production-research');
const behavior = pathOf('issue-1021-behaviour-range');
const dialogueOwners = ['cnl-solver-dialogue', 'cnl-solver-dialogue-b', 'cnl-solver-dialogue-c']
  .map(name => 'rust/tests/web/' + name + '.test.mjs');
const dialogueAggregate = rows.filter(row => dialogueOwners.includes(row.file)).reduce((sum, row) => sum + row.seconds, 0);
assert.equal(rows.filter(row => dialogueOwners.includes(row.file)).length, dialogueOwners.length);
let selected = null;
for (let index = 0; index < 66; index++) {
  const factor = Number((1 + index * 0.01).toFixed(2));
  const reserved = new Map();
  // Keep every prior floor. Add an owner-specific scheduling allowance over
  // the existing family floor, using the complete measured child durations.
  for (const file of dialogueOwners) {
    const measured = rows.find(row => row.file === file).seconds;
    reserved.set(file, {seconds: Math.ceil((priorWeights.get(file) + measured * (factor - 1)) * 10) / 10, complete: true});
  }
  let text = mergeJavaScriptDurations(prior, reserved,
    {source: 45240466225, recordedDate: '2026-11-23', raiseOnly: true, floor: 0.401});
  for (const file of dialogueOwners) {
    const dialogueBound = reserved.get(file).seconds;
    const marker = '  test "' + file + '"\n    seconds ' + dialogueBound + '\n';
    const extra = '    estimate-kind observed-owner-allowance-over-prior-family-floor\n'
      + '    source-run 45240466225\n'
      + '    prior-family-floor ' + priorWeights.get(file) + '\n'
      + '    observed-owner-seconds ' + rows.find(row => row.file === file).seconds + '\n'
      + '    owner-allowance-factor ' + Number((factor - 1).toFixed(2)) + '\n';
    assert(text.includes(marker));
    if (dialogueBound > priorWeights.get(file)) text = text.replace(marker, marker + extra);
  }
  const results = evaluate(text);
  const weights = parseTestDurations(text);
  const fallback = Number(text.match(/^ {2}default-seconds (\S+)$/mu)[1]);
  const {shards} = planShards(files, count, durationLookup(weights, fallback));
  const owners = ['cnl-solver-dialogue', 'cnl-solver-dialogue-b', 'cnl-solver-dialogue-c']
    .map(name => 'rust/tests/web/' + name + '.test.mjs');
  const ownerShards = owners.map(file => shards.findIndex(shard => shard.includes(file)) + 1);
  const distinctDialogueOwners = new Set(ownerShards).size === owners.length;
  const secondsOf = durationLookup(weights, fallback);
  const shardOf = file => shards.findIndex(shard => shard.includes(file));
  const conservativeQueues = shards.map(shard => {
    const workers = [0, 0, 0, 0];
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += secondsOf(file);
    return Math.max(...workers);
  });
  const constraints = {
    exactPartition: Object.values(partitionProblems(files, shards)).every(list => list.length === 0),
    longestFirst: isLongestFirst(shards, secondsOf),
    deterministic: JSON.stringify(planShards([...files].reverse(), count, secondsOf).shards) === JSON.stringify(shards),
    fullBeforeFresh: secondsOf(full) > secondsOf(fresh),
    fullFirst: shards[shardOf(full)][0] === full,
    fullSeparate: [memory, fresh].every(file => shardOf(full) !== shardOf(file)),
    memoryObserved: secondsOf(memory) > fallback,
    reasonerFirst: shards[shardOf(reasoner)][0] === reasoner,
    reasonerSeparate: [full, fresh, production].every(file => shardOf(reasoner) !== shardOf(file)),
    reasonerObserved: secondsOf(reasoner) > secondsOf(pathOf('r1175-routing-probes')),
    behaviorFirstWave: shards[shardOf(behavior)].indexOf(behavior) < 4,
    behaviorObserved: secondsOf(behavior) > secondsOf(pathOf('r1175-routing-probes')),
    originalQueueBudget: conservativeQueues.every(seconds => seconds < 360),
    distinctDialogueOwners,
    retainedCohorts: results.every(row => row.maximumSeconds <= 270),
  };
  const fits = Object.values(constraints).every(Boolean);
  candidates.push({factor, fits, constraints, conservativeQueues, ownerShards, weightedAverageSeconds: files.reduce((sum, file) => sum + secondsOf(file), 0) / (count * 4), results});
  // Maximize the owner allowance within every unchanged scheduling contract.
  if (fits) selected = {factor, text};
}
const report = {candidateLimit: 66, candidates, baseline: evaluate(prior), selectedFactor: selected?.factor ?? null};
writeFileSync(join(output, 'reserve-probe.json'), JSON.stringify(report, null, 2) + '\n');
assert(selected, 'no bounded candidate fits every retained cohort');
writeFileSync(join(output, 'proposed-scheduling-floor.lino'),
  '# Scheduling weights retain prior floors; only the three dialogue owners gain\n'
  + '# an allowance of ' + Math.round((selected.factor - 1) * 100) + '% of their observed cost over their prior family floor.\n'
  + '# Complete nine-job observations and the unchanged runner allowance are retained in\n'
  + '# experiments/issue-1580/fixtures/hosted-javascript-e46254ac/.\n'
  + selected.text.split('\n').filter(line => !line.trimStart().startsWith('#')).join('\n'));
const selectedCandidate = candidates.find(row => row.factor === selected.factor);
writeFileSync(join(output, 'selected-reserve-policy.json'), JSON.stringify({
  sourceRun: 45240466225,
  priorFloorFixture: current + 'prior-scheduling-floor.lino',
  formula: 'ceil((priorFamilyFloor + observedOwnerSeconds * allowanceFactor) * 10) / 10',
  candidateLimit: 66,
  allowanceFactor: Number((selected.factor - 1).toFixed(2)),
  scope: dialogueOwners,
  constraints: selectedCandidate.constraints,
  conservativeQueues: selectedCandidate.conservativeQueues,
  cohortMaximumSeconds: Object.fromEntries(selectedCandidate.results.map(row => [row.name, row.maximumSeconds])),
  modeledSourceChanges: [{
    file: 'rust/tests/web/meta-reasoner.test.mjs',
    model: 'maximum of the two original case-group sums, retained setup residual, plus five seconds for the independent process',
    observation: root + 'meta-reasoner-owner-split/observation.json',
    executionProof: false,
  }],
  executionProof: false,
  autonomousAuthorshipCredit: 0,
}, null, 2) + '\n');
console.log(JSON.stringify({selectedFactor: selected.factor,
  candidates: candidates.map(row => ({factor: row.factor, fits: row.fits, maximumSeconds: Math.max(...row.results.map(value => value.maximumSeconds))})),
  baseline: report.baseline.find(row => row.name.endsWith('e46254ac'))}));
