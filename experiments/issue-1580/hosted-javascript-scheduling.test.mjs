// Replay retained hosted costs without executing the functional suite.
import test from 'node:test';
import {createHash} from 'node:crypto';
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

test('complete current hosted costs fit the deployed eight-shard plan without discarding failed-shard observations', () => {
  const fixture = new URL('./fixtures/hosted-javascript-2809ce65/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
  const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert.equal(identity.runConclusion, 'failure');
  assert(observations.some(row => identity.jobs.find(job => job.job === row.job)?.conclusion === 'failure'));
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const weights = parseTestDurations(recorded);
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, durationLookup(weights, Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  for (const [index, shard] of shards.entries()) {
    const load = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) {
      const worker = load.indexOf(Math.min(...load)); load[worker] += costs(file);
    }
    assert(Math.max(...load) <= 270, 'current measured shard ' + (index + 1) + ': ' + Math.max(...load));
  }
});

test('complete c55 hosted costs retain failed-shard observations and fit the unchanged warning threshold', () => {
  const fixture = new URL('./fixtures/hosted-javascript-c55a5f9c/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
  const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(identity.runConclusion, 'failure');
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert(observations.every(row => row.complete));
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const weights = parseTestDurations(recorded);
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, durationLookup(weights, Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  for (const [index, shard] of shards.entries()) {
    const workers = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += costs(file);
    assert(Math.max(...workers) <= 270, 'c55 measured shard ' + (index + 1) + ': ' + Math.max(...workers));
  }
});

test('complete 754 hosted costs retain the failed file and both budget warnings below the modeled threshold', () => {
  const fixture = new URL('./fixtures/hosted-javascript-754fd314/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
  const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(identity.runConclusion, 'failure');
  assert.equal(identity.executionProof, false);
  assert.equal(identity.warnings.length, 2);
  assert.equal(identity.budgetSeconds, 360);
  assert.equal(identity.warningSeconds, 270);
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert(observations.every(row => row.complete));
  assert(observations.some(row => row.file === 'rust/tests/web/recipe-resume-protocols.test.mjs' && row.result === 'failed'));
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const weights = parseTestDurations(recorded);
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, durationLookup(weights, Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  for (const [index, shard] of shards.entries()) {
    const workers = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += costs(file);
    assert(Math.max(...workers) <= identity.warningSeconds, '754 measured shard ' + (index + 1) + ': ' + Math.max(...workers));
  }
});

test('complete a558 hosted costs preserve all failed observations and fit the unchanged warning threshold', () => {
  const fixture = new URL('./fixtures/hosted-javascript-a558f834/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
  const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(identity.runConclusion, 'failure');
  assert.equal(identity.executionProof, false);
  assert.equal(identity.warnings.length, 1);
  assert.equal(identity.budgetSeconds, 360);
  assert.equal(identity.warningSeconds, 270);
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert(observations.every(row => row.complete));
  assert.deepEqual(observations.filter(row => row.result === 'failed').map(row => row.file).sort(), identity.failedObservations.slice().sort());
  assert.equal(identity.failedObservations.length, 3);
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, durationLookup(parseTestDurations(recorded), Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  for (const [index, shard] of shards.entries()) {
    const workers = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += costs(file);
    assert(Math.max(...workers) + identity.runnerOverheadAllowanceSeconds <= identity.warningSeconds, 'a558 measured shard plus runner overhead ' + (index + 1) + ': ' + Math.max(...workers));
  }
});


test('complete bbc hosted costs retain the warning and fit the fixed runner-overhead allowance', () => {
  const fixture = new URL('./fixtures/hosted-javascript-bbc0ac4d/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
  const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(identity.runConclusion, 'success');
  assert.equal(identity.executionProof, false);
  assert.equal(identity.warnings[0].seconds, 319);
  assert.equal(identity.warningSeconds, 270);
  assert.equal(identity.budgetSeconds, 360);
  assert.equal(identity.runnerOverheadAllowanceSeconds, 30);
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert(observations.every(row => row.complete && identity.jobs.some(job => job.job === row.job)));
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, durationLookup(parseTestDurations(recorded), Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  for (const [index, shard] of shards.entries()) {
    const workers = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += costs(file);
    assert(Math.max(...workers) + identity.runnerOverheadAllowanceSeconds <= identity.warningSeconds, 'bbc measured shard plus runner overhead ' + (index + 1) + ': ' + Math.max(...workers));
  }
});


test('complete 79a hosted costs preserve both warnings and the fixed runner allowance', () => {
  const fixture = new URL('./fixtures/hosted-javascript-79a024bf/', import.meta.url);
  const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
  const raw = readFileSync(new URL('durations.jsonl', fixture), 'utf8');
  const observations = raw.trim().split('\n').map(line => JSON.parse(line));
  const run = JSON.parse(readFileSync(new URL('run-observation.json', fixture), 'utf8'));
  assert.equal(run.id, identity.run);
  assert.equal(run.head_sha, identity.sourceCommit);
  assert.equal(run.conclusion, 'failure');
  assert.equal(identity.runConclusion, run.conclusion);
  assert.deepEqual(observations.filter(row => row.result === 'failed').map(row => row.file).sort(),
    ['rust/tests/web/translate-es.test.mjs', 'rust/tests/web/typescript-runtime.test.mjs']);
  assert.deepEqual(identity.failedObservations.map(row => row.file).sort(),
    observations.filter(row => row.result === 'failed').map(row => row.file).sort());
  for (const row of identity.failedObservations) {
    assert.equal(identity.jobs.find(job => job.job === row.job)?.conclusion, 'failure');
    assert(observations.some(observed => observed.file === row.file && observed.job === row.job && observed.result === 'failed'));
  }
  assert.equal(identity.executionProof, false);
  assert.equal(identity.schedulingFactor, 1.4);
  assert.deepEqual(identity.warnings.map(row => row.seconds), [302, 283]);
  assert.equal(identity.warningSeconds, 270);
  assert.equal(identity.budgetSeconds, 360);
  assert.equal(identity.runnerOverheadAllowanceSeconds, 30);
  assert.equal(createHash('sha256').update(raw).digest('hex'), identity.durationsSha256);
  assert.equal(observations.length, identity.completeObservations);
  assert.equal(new Set(observations.map(row => row.file)).size, observations.length);
  assert(observations.every(row => row.complete && identity.jobs.some(job => job.job === row.job)));
  const recorded = readFileSync(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url), 'utf8');
  const files = readdirSync(new URL('../../rust/tests/web/', import.meta.url)).filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
  const {shards} = planShards(files, 8, durationLookup(parseTestDurations(recorded), Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files, shards), {missing: [], repeated: [], extra: []});
  const costs = durationLookup(new Map(observations.map(row => [row.file, row.seconds])), identity.unobservedFallbackSeconds);
  for (const [index, shard] of shards.entries()) {
    const workers = Array(identity.modeledWorkers).fill(0);
    for (const file of shard) workers[workers.indexOf(Math.min(...workers))] += costs(file);
    assert(Math.max(...workers) + identity.runnerOverheadAllowanceSeconds <= identity.warningSeconds, '79a measured shard plus runner overhead ' + (index + 1) + ': ' + Math.max(...workers));
  }
});


test('complete c90 costs retain the assertion-only file and fit the unchanged warning and runner allowance', () => {
  const fixture=new URL('./fixtures/hosted-javascript-c90f94ca/',import.meta.url);
  const identity=JSON.parse(readFileSync(new URL('identity.json',fixture),'utf8'));
  const run=JSON.parse(readFileSync(new URL('run-observation.json',fixture),'utf8'));
  const raw=readFileSync(new URL('durations.jsonl',fixture),'utf8');
  const observations=raw.trim().split('\n').map(line=>JSON.parse(line));
  assert.equal(run.id,identity.run);
  assert.equal(run.head_sha,identity.sourceCommit);
  assert.equal(run.conclusion,'success');
  assert.equal(identity.runConclusion,run.conclusion);
  assert.equal(identity.executionProof,false);
  assert.equal(identity.observedSchedulingFactor,1.4);
  assert.equal(identity.schedulingFactor,1.65);
  assert.equal(identity.warningSeconds,270);
  assert.equal(identity.budgetSeconds,360);
  assert.equal(identity.runnerOverheadAllowanceSeconds,30);
  assert.deepEqual(identity.warnings.map(row=>({shard:row.shard,seconds:row.seconds})),[{shard:3,seconds:296}]);
  assert.equal(createHash('sha256').update(raw).digest('hex'),identity.durationsSha256);
  assert.equal(observations.length,916);
  assert.equal(observations.length,identity.completeObservations);
  assert.equal(new Set(observations.map(row=>row.file)).size,observations.length);
  assert(observations.every(row=>row.complete&&row.result==='completed'&&identity.jobs.some(job=>job.job===row.job&&job.conclusion==='success')));
  assert.deepEqual(observations.filter(row=>row.completionSource!=='node-file-summary').map(row=>({file:row.file,source:row.completionSource})),
    [{file:'rust/tests/web/server-dialog-lino.test.mjs',source:'node-spec-file-pass-without-summary'}]);
  const recorded=readFileSync(new URL('../../data/meta/javascript-test-durations.lino',import.meta.url),'utf8');
  const priorText=readFileSync(new URL('prior-scheduling-floor.lino',fixture),'utf8');
  const prior=parseTestDurations(priorText);
  const weights=parseTestDurations(recorded);
  const defaultSeconds=Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1]);
  assert.equal(defaultSeconds,Number(priorText.match(/^ {2}default-seconds (\S+)$/mu)[1]));
  for(const [file,seconds]of prior)assert(weights.get(file)>=seconds,'prior explicit floor '+file);
  for(const row of observations){
    const reserved=Math.ceil(row.seconds*identity.schedulingFactor*10)/10;
    assert((weights.get(row.file)??defaultSeconds)>=reserved,'observed reserve '+row.file);
    if(!prior.has(row.file)&&weights.has(row.file))assert(weights.get(row.file)>defaultSeconds,'redundant default override '+row.file);
  }
  const files=readdirSync(new URL('../../rust/tests/web/',import.meta.url)).filter(file=>file.endsWith('.test.mjs')).map(file=>'rust/tests/web/'+file);
  const {shards}=planShards(files,8,durationLookup(parseTestDurations(recorded),Number(recorded.match(/^ {2}default-seconds (\S+)$/mu)[1])));
  assert.deepEqual(partitionProblems(files,shards),{missing:[],repeated:[],extra:[]});
  const costs=durationLookup(new Map(observations.map(row=>[row.file,row.seconds])),identity.unobservedFallbackSeconds);
  for(const [index,shard]of shards.entries()){
    const workers=Array(identity.modeledWorkers).fill(0);
    for(const file of shard)workers[workers.indexOf(Math.min(...workers))]+=costs(file);
    assert(Math.max(...workers)+identity.runnerOverheadAllowanceSeconds<=identity.warningSeconds,
      'c90 measured shard plus runner overhead '+(index+1)+': '+Math.max(...workers));
  }
});
