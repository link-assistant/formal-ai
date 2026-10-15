import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
import {deferReleaseRecoveryAuthoring} from '../../../scripts/check-release-recovery-authoring-deferral.mjs';

const repository = 'link-assistant/formal-ai';
function event() {
  return {number: 1196, repository: {full_name: repository},
    pull_request: {number: 1196, state: 'open',
      head: {ref: 'fix/postmerge-release-ci-1188', repo: {full_name: repository}},
      base: {ref: 'main', repo: {full_name: repository}}}};
}
const evaluate = value => deferReleaseRecoveryAuthoring('pull_request', repository, value);
test('only the exact open recovery PR receives a zero-credit deferral', () => {
  assert.equal(evaluate(event()), true);
});
for (const eventName of ['push', 'schedule', 'workflow_dispatch', 'pull_request_target', '', undefined]) {
  test('ordinary event retains authoring gates: ' + String(eventName), () => {
    assert.equal(deferReleaseRecoveryAuthoring(eventName, repository, event()), false);
  });
}
for (const [label, change] of [
  ['wrong event repository', value => {value.repository.full_name = 'foreign/formal-ai';}],
  ['missing event repository', value => {delete value.repository;}],
  ['wrong PR', value => {value.number = 1197;}],
  ['string PR', value => {value.number = '1196';}],
  ['wrong nested PR', value => {value.pull_request.number = 1197;}],
  ['closed PR', value => {value.pull_request.state = 'closed';}],
  ['different branch', value => {value.pull_request.head.ref = 'main';}],
  ['different base', value => {value.pull_request.base.ref = 'release';}],
  ['foreign head repository', value => {value.pull_request.head.repo.full_name = 'fork/formal-ai';}],
  ['foreign base repository', value => {value.pull_request.base.repo.full_name = 'fork/formal-ai';}],
  ['missing head repository', value => {delete value.pull_request.head.repo;}],
  ['missing base repository', value => {delete value.pull_request.base.repo;}],
  ['missing request', value => {delete value.pull_request;}],
]) {
  test('scope refuses ' + label, () => {
    const value = event(); change(value); assert.equal(evaluate(value), false);
  });
}
test('runner repository mismatch and absent payload cannot defer', () => {
  assert.equal(deferReleaseRecoveryAuthoring('pull_request', 'fork/formal-ai', event()), false);
  assert.equal(evaluate(null), false);
  assert.equal(evaluate({}), false);
});
const helper = fileURLToPath(new URL('../../../scripts/check-release-recovery-authoring-deferral.mjs', import.meta.url));
function invoke(eventName, payload, malformed = false) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'pr1196-deferral-'));
  try {
    const eventPath = path.join(dir, 'event.json'), output = path.join(dir, 'output'), summary = path.join(dir, 'summary');
    fs.writeFileSync(eventPath, malformed ? payload : JSON.stringify(payload));
    const result = spawnSync(process.execPath, [helper], {encoding: 'utf8', timeout: 10000,
      env: {...process.env, GITHUB_EVENT_NAME: eventName, GITHUB_REPOSITORY: repository,
        GITHUB_EVENT_PATH: eventPath, GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: summary}});
    return {result, output: fs.existsSync(output) ? fs.readFileSync(output, 'utf8') : null,
      summary: fs.existsSync(summary) ? fs.readFileSync(summary, 'utf8') : null};
  } finally {fs.rmSync(dir, {recursive: true, force: true});}
}
test('actual helper output says deferred rather than satisfied or age-relaxed', () => {
  const observed = invoke('pull_request', event());
  assert.equal(observed.result.status, 0); assert.equal(observed.result.signal, null);
  assert.deepEqual(JSON.parse(observed.result.stdout), {defer: true,
    status: 'DeferredForReleaseRecovery_PR1196', qualifyingAuthorship: 0,
    floorSatisfied: false, restorationDebt: 'PR1196-release-recovery-authoring'});
  assert.equal(observed.output, 'defer=true\n');
  assert.match(observed.summary, /qualifyingAuthorship=0; floorSatisfied=false/u);
  assert.match(observed.summary, /#pr1196-release-recovery-authoring/u);
});
test('dispatch retains normal gates even when payload is malformed', () => {
  const observed = invoke('workflow_dispatch', '{', true);
  assert.equal(observed.result.status, 0); assert.equal(observed.output, 'defer=false\n');
  assert.deepEqual(JSON.parse(observed.result.stdout), {defer: false, status: 'NotDeferred'});
  assert.equal(observed.summary, null);
});
test('malformed PR context fails before writing a deferral output', () => {
  const observed = invoke('pull_request', '{', true);
  assert.notEqual(observed.result.status, 0); assert.equal(observed.output, null);
  assert.equal(observed.summary, null);
});
function workflow(name) {
  return YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/' + name, import.meta.url), 'utf8'));
}
const gate = '$' + "{{ steps.release-recovery-authoring.outputs.defer != 'true' }}";
test('invalid trailers still fail before the optional positive-authoring step', () => {
  const definition = workflow('evidence-check.yml'), steps = definition.jobs['evidence-check'].steps;
  const strict = steps.find(step => step.name === 'Validate Formal-AI trailers on this pull request');
  const positive = steps.find(step => step.name === 'Formal AI authored part of this pull request');
  const deferral = steps.find(step => step.id === 'release-recovery-authoring');
  assert.equal(strict.if, undefined); assert.equal(strict['continue-on-error'], undefined);
  assert.match(strict.run, /set -euo pipefail/u);
  assert.match(strict.run, /rust-script scripts\/self-hosting-metric.rs --since "\$merge_base" --until HEAD/u);
  assert.ok(steps.indexOf(strict) < steps.indexOf(deferral));
  assert.ok(steps.indexOf(deferral) < steps.indexOf(positive));
  assert.equal(positive.if, gate);
  assert.match(positive.run, /scripts\/check-formal-ai-contribution.rs/u);
  assert.equal(deferral.if, undefined); assert.equal(deferral['continue-on-error'], undefined);
  assert.equal(deferral.run, 'node scripts/check-release-recovery-authoring-deferral.mjs');
  const status = definition.jobs['evidence-status'];
  assert.equal(status.if, 'always()'); assert.deepEqual(status.needs, ['evidence-check']);
  assert.equal(status.steps.at(-1).run, 'bash scripts/check-pipeline-status.sh');
});
test('measurement and debt failures still fail before the optional prospective floor', () => {
  const definition = workflow('self-development-status.yml');
  const steps = definition.jobs['self-development-status'].steps;
  const measure = steps.find(step => step.name === 'Measure the share since the last tag (metric version 3)');
  const debt = steps.find(step => step.name === 'Debt ratchet holds (ceilings move down, never up)');
  const positive = steps.find(step => step.name === 'Self-development floor (issue #924, #1066, #1085)');
  const deferral = steps.find(step => step.id === 'release-recovery-authoring');
  for (const step of [measure, debt, deferral]) {
    assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
  }
  assert.match(measure.run, /scripts\/self-hosting-metric.rs --since "\$since" --until HEAD/u);
  assert.equal(debt.run, 'rust-script scripts/check-debt-ratchet.rs --base origin/main');
  assert.ok(steps.indexOf(measure) < steps.indexOf(deferral));
  assert.ok(steps.indexOf(debt) < steps.indexOf(deferral));
  assert.ok(steps.indexOf(deferral) < steps.indexOf(positive));
  assert.equal(positive.if, gate);
  assert.equal(positive.run, 'rust-script scripts/check-self-development-release.rs');
  assert.deepEqual(definition.on.push.branches, ['main']);
  assert.equal(definition.on.schedule[0].cron, '17 6 * * *');
});


function manifestRepairEvent() {
  const value = event(); value.number = 1197; value.pull_request.number = 1197;
  value.pull_request.head.ref = 'fix/release-manifest-byte-witness'; return value;
}
test('only the exact open PR1197 manifest repair receives its named zero-credit deferral', () => {
  assert.equal(evaluate(manifestRepairEvent()), true);
  const observed = invoke('pull_request', manifestRepairEvent());
  assert.equal(observed.result.status, 0); assert.equal(observed.result.signal, null);
  assert.deepEqual(JSON.parse(observed.result.stdout), {defer: true,
    status: 'DeferredForReleaseRecovery_PR1197', qualifyingAuthorship: 0,
    floorSatisfied: false, restorationDebt: 'PR1197-release-recovery-authoring'});
  assert.equal(observed.output, 'defer=true\n');
  assert.match(observed.summary, /qualifyingAuthorship=0; floorSatisfied=false/u);
  assert.match(observed.summary, /#pr1197-release-recovery-authoring/u);
});
for (const [label, change] of [
  ['wrong repository', value => {value.repository.full_name = 'fork/formal-ai';}],
  ['unknown PR', value => {value.number = value.pull_request.number = 1198;}],
  ['string PR', value => {value.number = value.pull_request.number = '1197';}],
  ['mismatched nested PR', value => {value.pull_request.number = 1196;}],
  ['old PR with new branch', value => {value.number = value.pull_request.number = 1196;}],
  ['new PR with old branch', value => {value.pull_request.head.ref = 'fix/postmerge-release-ci-1188';}],
  ['closed PR', value => {value.pull_request.state = 'closed';}],
  ['branch suffix', value => {value.pull_request.head.ref += '-other';}],
  ['missing branch', value => {delete value.pull_request.head.ref;}],
  ['wrong base', value => {value.pull_request.base.ref = 'release';}],
  ['foreign head', value => {value.pull_request.head.repo.full_name = 'fork/formal-ai';}],
  ['foreign base', value => {value.pull_request.base.repo.full_name = 'fork/formal-ai';}],
  ['missing head repository', value => {delete value.pull_request.head.repo;}],
  ['missing base repository', value => {delete value.pull_request.base.repo;}],
  ['missing request', value => {delete value.pull_request;}],
]) test('PR1197 scope refuses ' + label, () => {
  const value = manifestRepairEvent(); change(value); assert.equal(evaluate(value), false);
});
for (const name of ['push', 'schedule', 'workflow_dispatch', 'pull_request_target', '', undefined]) {
  test('PR1197 cannot defer ordinary event ' + String(name), () => {
    assert.equal(deferReleaseRecoveryAuthoring(name, repository, manifestRepairEvent()), false);
  });
}

function dockerRecoveryEvent() {
 const value=event();
 value.number=value.pull_request.number=1200;
 value.pull_request.head.ref='fix/release-docker-export';
 return value;
}
test('PR1200 exact open Docker recovery has zero-credit CLI status and named debt',()=>{
 assert.equal(evaluate(dockerRecoveryEvent()),true);
 const observed=invoke('pull_request',dockerRecoveryEvent());
 assert.equal(observed.result.status,0);
 assert.equal(observed.result.signal,null);
 assert.deepEqual(JSON.parse(observed.result.stdout),{defer:true,status:'DeferredForReleaseRecovery_PR1200',qualifyingAuthorship:0,floorSatisfied:false,restorationDebt:'PR1200-release-recovery-authoring'});
 assert.equal(observed.output,'defer=true\n');
 assert.match(observed.summary,/#pr1200-release-recovery-authoring/u);
 assert.match(observed.summary,/qualifyingAuthorship=0; floorSatisfied=false/u);
});
for(const[label,change]of[
 ['string number',value=>{value.number=value.pull_request.number='1200';
 }],
 ['unknown number',value=>{value.number=value.pull_request.number=1201;
 }],

 ['nested mismatch',value=>{value.pull_request.number=1197;
 }],
 ['old number new branch',value=>{value.number=value.pull_request.number=1197;
 }],
 ['new number old branch',value=>{value.pull_request.head.ref='fix/release-manifest-byte-witness';
 }],

 ['closed',value=>{value.pull_request.state='closed';
 }],
 ['branch suffix',value=>{value.pull_request.head.ref+='-other';
 }],
 ['missing branch',value=>{delete value.pull_request.head.ref;
 }],
 ['wrong base',value=>{value.pull_request.base.ref='release';
 }],

 ['event repository',value=>{value.repository.full_name='fork/formal-ai';
 }],
 ['head repository',value=>{value.pull_request.head.repo.full_name='fork/formal-ai';
 }],
 ['base repository',value=>{value.pull_request.base.repo.full_name='fork/formal-ai';
 }],

 ['missing head repository',value=>{delete value.pull_request.head.repo;
 }],
 ['missing base repository',value=>{delete value.pull_request.base.repo;
 }],
 ['missing request',value=>{delete value.pull_request;
 }]
])test('PR1200 scope refuses '+label,()=>{const value=dockerRecoveryEvent();
 change(value);
 assert.equal(evaluate(value),false);
 });
for(const name of ['push','schedule','workflow_dispatch','pull_request_target','',undefined])test('PR1200 ordinary event remains strict '+String(name),()=>{assert.equal(deferReleaseRecoveryAuthoring(name,repository,dockerRecoveryEvent()),false);
 });
test('PR1200 runner repository mismatch and closed CLI retain normal gates',()=>{
 assert.equal(deferReleaseRecoveryAuthoring('pull_request','fork/formal-ai',dockerRecoveryEvent()),false);
 const value=dockerRecoveryEvent();
 value.pull_request.state='closed';
 const observed=invoke('pull_request',value);
 assert.equal(observed.result.status,0);
 assert.deepEqual(JSON.parse(observed.result.stdout),{defer:false,status:'NotDeferred'});
 assert.equal(observed.output,'defer=false\n');
 assert.equal(observed.summary,null);
});

function nextRecoveryEvent() {
 const value=event();value.number=value.pull_request.number=1202;
 value.pull_request.head.ref='fix/staged-image-job-budget';return value;
}
test('exact next open image-budget recovery is a named zero-credit policy deferral',()=>{
 assert.equal(evaluate(nextRecoveryEvent()),true);
 const observed=invoke('pull_request',nextRecoveryEvent());
 assert.equal(observed.result.status,0);
 assert.deepEqual(JSON.parse(observed.result.stdout),{defer:true,
  status:'DeferredForReleaseRecovery_PR1202',qualifyingAuthorship:0,
  floorSatisfied:false,restorationDebt:'PR1202-release-recovery-authoring'});
});
for(const [label,change] of [
 ['string PR',x=>{x.number=x.pull_request.number='1202';}],
 ['unknown PR',x=>{x.number=x.pull_request.number=900002;}],
 ['nested mismatch',x=>{x.pull_request.number=1196;}],
 ['closed',x=>{x.pull_request.state='closed';}],
 ['foreign event repository',x=>{x.repository.full_name='fork/formal-ai';}],
 ['foreign head',x=>{x.pull_request.head.repo.full_name='fork/formal-ai';}],
 ['foreign base',x=>{x.pull_request.base.repo.full_name='fork/formal-ai';}],
 ['missing head repository',x=>{delete x.pull_request.head.repo;}],
 ['branch suffix',x=>{x.pull_request.head.ref+='-other';}],
 ['old recovery branch',x=>{x.pull_request.head.ref='fix/release-docker-export';}],
 ['wrong base',x=>{x.pull_request.base.ref='release';}]
])test('next image recovery refuses '+label,()=>{
 const value=nextRecoveryEvent();change(value);assert.equal(evaluate(value),false);
});
for(const name of ['push','schedule','workflow_dispatch','pull_request_target'])
 test('next image recovery does not defer '+name,()=>{
  assert.equal(deferReleaseRecoveryAuthoring(name,repository,nextRecoveryEvent()),false);
 });
