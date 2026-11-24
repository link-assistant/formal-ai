// Pinned full CLI execution with real Git and disposable local bare remotes.
import assert from 'node:assert/strict';
import test from 'node:test';
import {observeJavaProjectVersions} from './probe-java-project-version.mjs';

const expectedPom=observation=>observation.initialPom.replace(
  '<artifactId>my-package</artifactId><version>1.2.3</version>',
  '<artifactId>my-package</artifactId><version>1.2.4</version>').trim();
function assertSuccessfulRelease(observations){
  assert.equal(observations.length,4);
  for(const observation of observations){
    assert.equal(observation.exitCode,0,observation.stderr);
    assert.match(observation.outputs,/^released=true$/mu);
    assert.match(observation.outputs,/^new_version=1\.2\.4$/mu);
    assert.match(observation.taggedJava,/VERSION = "1\.2\.4"/u);
  }
}
test('unchanged Java release publishes the old project version for parent and comment POMs',{timeout:30000},()=>{
  const {observations}=observeJavaProjectVersions();
  assertSuccessfulRelease(observations);
  assert.equal(observations[0].taggedPom,expectedPom(observations[0]),'plain POM positive control');
  assert.deepEqual(observations.filter(observation=>observation.taggedPom!==expectedPom(observation)).map(observation=>observation.name),
    ['different-parent-version','equal-parent-version','comment-before-project-version']);
});
test('candidate POM writer preserves parent and comment bytes and publishes the project version',{timeout:30000},()=>{
  const candidate=process.env.ISSUE_1580_JAVA_DISABLE_PROJECT_VERSION_CANDIDATE!=='1';
  const {observations}=observeJavaProjectVersions({candidate});
  assertSuccessfulRelease(observations);
  for(const observation of observations)assert.equal(observation.taggedPom,expectedPom(observation),observation.name);
});
