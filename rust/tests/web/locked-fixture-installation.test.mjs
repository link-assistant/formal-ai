// Captured registry failure; never installs dependencies or launches Bun locally.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
import {installLockedFixtures} from '../../../scripts/install-locked-fixture-dependencies.mjs';
const observed='error: GET https://registry.npmjs.org/@babel/generator/-/generator-7.29.7.tgz - 409\n';
function harness(results) {
  const calls=[], delays=[], log=[];
  const options={stdout:text=>log.push(String(text)),stderr:text=>log.push(String(text)),
    wait:async milliseconds=>delays.push(milliseconds),invoke:async(file,args,bounds)=>{
      calls.push({file,args,bounds});
      const result=results[Math.min(calls.length-1,results.length-1)];
      bounds.onStderr(result.stderr ?? '');
      return {cleanup_complete:true,status:1,stderr:'',...result};
    }};
  return {calls,delays,log,options};
}
test('the captured tarball conflict recovers once while keeping frozen and script-free installation', async () => {
  const h=harness([{stderr:observed},{status:0}]);
  await installLockedFixtures(h.options);
  assert.equal(h.calls.length,2);
  assert.deepEqual(h.delays,[1000]);
  assert(h.log.join('').includes(observed));
  for (const call of h.calls) {
    assert.equal(call.file,'bun');
    assert.deepEqual(call.args,['install','--frozen-lockfile','--ignore-scripts']);
    assert.equal(call.bounds.timeout,120000);
  }
});
test('persistent conflict stops after three attempts and other failures never retry', async () => {
  const repeated=harness([{stderr:observed}]);
  await assert.rejects(installLockedFixtures(repeated.options));
  assert.equal(repeated.calls.length,3);
  assert.deepEqual(repeated.delays,[1000,2000]);
  for (const result of [{stderr:'lockfile changed'}, {stderr:observed.replace('409','401')},
    {stderr:observed.replace('registry.npmjs.org','example.invalid')},
    {stderr:observed,cleanup_complete:false}, {stderr:observed,error:Error('spawn')},
    {stderr:observed,timed_out:true}, {stderr:observed,signal:'SIGTERM'},
    {stderr:observed,status:null}, {stderr:observed+'error: integrity mismatch\n'},
    {stderr:observed,status:2}, {stderr:observed,stdout:'error: integrity mismatch\n'}]) {
    const h=harness([result]);
    await assert.rejects(installLockedFixtures(h.options));
    assert.equal(h.calls.length,1);
    assert.deepEqual(h.delays,[]);
  }
});
test('the main native fixture step uses the bounded frozen installer', () => {
  const jobs=YAML.parse(readFileSync('.github/workflows/release.yml','utf8')).jobs;
  const step=jobs.test.steps.find(step=>step.name==='Install actual locked native fixture dependencies');
  assert.equal(step.run,'node scripts/install-locked-fixture-dependencies.mjs');
});
