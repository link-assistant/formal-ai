import test from 'node:test';
import assert from 'node:assert/strict';
import {dispatchedProducer, awaitCompletedProducer} from '../../../scripts/release-extension-dispatch.mjs';
const expected = {repository: 'link-assistant/formal-ai', runId: '123', attempt: '1'};
const environment = {EVENT: 'workflow_dispatch', REFERENCE: 'refs/heads/main', REPOSITORY: expected.repository};
const inputs = {'producer-run': '123', 'producer-attempt': '1'};
const run = {
 id: 123, run_attempt: 1, repository: {full_name: expected.repository},
 head_repository: {full_name: expected.repository}, head_branch: 'main',
 path: '.github/workflows/desktop-release.yml', head_sha: 'a'.repeat(40), status: 'completed'
};
test('routing hints require real main dispatch and exact complete identifiers', () => {
 assert.deepEqual(dispatchedProducer(environment, {inputs}), expected);
 assert.equal(dispatchedProducer(environment, {inputs: {}}), null);
 for (const EVENT of ['push', 'release', 'pull_request']) {
  assert.throws(() => dispatchedProducer({...environment, EVENT}, {inputs}));
 }
 for (const supplied of [{'producer-run': '123'}, {'producer-attempt': '1'}, {'producer-run': '../123', 'producer-attempt': '1'}]) {
  assert.throws(() => dispatchedProducer(environment, {inputs: supplied}));
 }
 assert.throws(() => dispatchedProducer({...environment, REFERENCE: 'refs/heads/foreign'}, {inputs}));
});
test('actual completed producer identity remains independent of routing hints', () => {
 assert.deepEqual(awaitCompletedProducer(expected, {json: () => run}), run);
 for (const mutate of [
  value => {value.id = 124;}, value => {value.run_attempt = 2;},
  value => {value.head_branch = 'foreign';}, value => {value.path = 'foreign.yml';},
  value => {value.head_sha = 'unknown';}, value => {value.repository.full_name = 'foreign/repository';},
  value => {value.head_repository.full_name = 'foreign/repository';}
 ]) {
  const changed = structuredClone(run);
  mutate(changed);
  assert.throws(() => awaitCompletedProducer(expected, {json: () => changed}));
 }
});
test('bounded completion observation refuses Unknown and retains current producer receipts', () => {
 assert.throws(() => awaitCompletedProducer(expected, {json: () => ({...run, status: 'in_progress'})}, {now: () => 0, deadline: 0}));
 let calls = 0;
 let elapsed = 0;
 const github = {json: () => ({...run, status: ++calls === 1 ? 'in_progress' : 'completed'})};
 assert.equal(awaitCompletedProducer(expected, github, {now: () => elapsed, deadline: 1000, sleep: value => {elapsed += value;}}).status, 'completed');
 assert.equal(calls, 2);
});
