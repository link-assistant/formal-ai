import assert from 'node:assert/strict';
import test from 'node:test';
import {observePagesAdoption} from './probe-pages-adoption.mjs';

for (const language of ['js', 'python']) {
  test(`${language} pinned Pages retry can adopt an older artifact of the same commit`, {timeout: 5000}, async () => {
    const observed = await observePagesAdoption(language);
    assert.equal(observed.requestedArtifactId, 42);
    assert.equal(observed.observedArtifactId, 41);
    assert.equal(observed.posts, 1, 'the new artifact was never submitted successfully');
  });
  test(`${language} artifact-bound candidate retries a request that never created a deployment`, {timeout: 5000}, async () => {
    const observed = await observePagesAdoption(language, {artifactBound: process.env.ISSUE_1580_DISABLE_PAGES_BINDING !== '1'});
    assert.equal(observed.observedArtifactId, 42);
    assert.equal(observed.posts, 2);
  });
  test(`${language} artifact-bound candidate adopts its own deployment after a lost response`, {timeout: 5000}, async () => {
    const observed = await observePagesAdoption(language, {artifactBound: true, acceptedBeforeDisconnect: true});
    assert.equal(observed.observedArtifactId, 42);
    assert.equal(observed.posts, 1, 'no duplicate after the server accepted the first request');
  });
}
