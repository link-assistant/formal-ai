import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {feedbackNeedsChanges} from '../../../js/agentic/restart_feedback.mjs';

await installNodeHost(new WorkerHost());
const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/agentic/hive-own-session-feedback.json', import.meta.url)));
const check = feedback => feedbackNeedsChanges(JSON.stringify(feedback));

test('the actual Codex second-pass successful receipt contains only its own framework notices', () => {
  assert.equal(feedbackNeedsChanges(fixture.toolReceipt), false);
  assert.equal(check(fixture.feedback), false);
  // Raw GitHub account metadata, removed from public evidence, does not change the rule.
  const raw = structuredClone(fixture.feedback);
  for (const comment of raw.comments) comment.author = {login:'github-actions'};
  assert.equal(check(raw), false);
});

test('human, unknown, edited or appended feedback and requested reviews still require action', () => {
  for (const mutate of [
    value => {value.comments[0].viewerDidAuthor = false;},
    value => {delete value.comments[0].viewerDidAuthor;},
    value => {value.comments[0].includesCreatedEdit = true;},
    value => {value.comments[0].body += '\nPlease fix the verifier.';},
    value => {value.comments[0].body = 'Please fix the verifier.';},
    value => {value.comments.push({author:{login:'github-actions'},body:'CI failed: fix the verifier.'});},
    value => {value.reviews.push({state:'CHANGES_REQUESTED'});},
  ]) {
    const value = structuredClone(fixture.feedback); mutate(value);
    assert.equal(check(value), true);
  }
});

test('failed, unfinished and unauthenticated tool envelopes cannot authorize readiness', () => {
  for (const extra of [{exit_code:1},{complete:false},{signal:'SIGTERM'},{timed_out:true}]) {
    assert.equal(check({exit_code:0,complete:true,stdout:JSON.stringify(fixture.feedback),...extra}),true);
  }
  assert.equal(check({stdout:JSON.stringify(fixture.feedback)}),true);
  assert.equal(feedbackNeedsChanges('not JSON'),true);
});
