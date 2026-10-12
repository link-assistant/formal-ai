import assert from 'node:assert/strict';
import test from 'node:test';
import {resolveProductionPackageRelease, productionPackageObserver, resolvePackageRelease}
  from '../../../scripts/resolve-package-release.mjs';
const repository = 'owner/repository', head = 'a'.repeat(40);
const environment = {EVENT:'workflow_run', REPOSITORY:repository, RUN_BRANCH:'main',
  RUN_REPOSITORY:repository, RUN_HEAD:head, RUN_CONCLUSION:'success',
  RUN_ID:'42', RUN_ATTEMPT:'1', RUN_WORKFLOW_ID:'7'};
const baseRun = {id:42, run_attempt:1, workflow_id:7, status:'completed', conclusion:'success',
  path:'.github/workflows/release.yml', head_sha:head, head_branch:'main',
  repository:{full_name:repository}, head_repository:{full_name:repository}, event:'push'};
const release = {id:17, url:'https://api.github.com/repos/owner/repository/releases/17',
  tag_name:'v1.2.3', draft:false, prerelease:false, published_at:'2020-01-02T03:04:05Z'};
function provider(run=baseRun, selected=release) {
  const calls=[];
  const read=endpoint=> {
    calls.push(endpoint);
    if(endpoint.includes('/actions/runs/')) return run;
    if(endpoint.includes('/actions/workflows/')) return {id:7,path:baseRun.path};
    if(endpoint.includes('/commits/')) return {sha:head,parents:[]};
    if(endpoint.includes('/releases/tags/')) return selected;
    if(endpoint.includes('/releases?')) return [selected];
    throw new Error('Unknown fixture endpoint');
  };
  return {calls,read};
}
for(const event of ['push','workflow_dispatch']) test('production observation admits source policy for '+event,()=>{
  const p=provider({...baseRun,event});
  assert.deepEqual(resolveProductionPackageRelease(environment,p.read),{tag:'v1.2.3',publish:true,build:true});
});
for(const event of ['pull_request','workflow_run','release','unknown',undefined]) test('unsupported producer refuses before release source '+event,()=>{
  const p=provider({...baseRun,event});
  assert.throws(()=>resolveProductionPackageRelease(environment,p.read),/production release event/);
  assert.deepEqual(p.calls,['repos/owner/repository/actions/runs/42']);
});
for(const changed of [{published_at:undefined},{published_at:null},{published_at:'not-a-date'},
  {published_at:'2999-01-01T00:00:00Z'},{published_at:'2020-02-30T00:00:00Z'},
  {tag_name:'v01.2.3'},{id:0},{url:'https://api.github.com/repos/foreign/repository/releases/17'}]) {
  test('published metadata drift refuses '+JSON.stringify(changed),()=>{
    const p=provider(baseRun,{...release,...changed});
    assert.throws(()=>resolveProductionPackageRelease(environment,p.read),/published metadata/);
    assert.equal(p.calls.some(x=>x.includes('/commits/')),false);
  });
}
test('PR and checks dispatch retain no-network check behavior',()=>{
  for(const event of ['pull_request','workflow_dispatch']) {
    assert.deepEqual(resolveProductionPackageRelease({EVENT:event,REPOSITORY:repository},()=>{
      throw new Error('No fixture API permitted');
    }),{tag:'',publish:false,build:true});
  }
});
test('unknown network observation remains an error, never absence or publication',()=>{
  assert.throws(()=>resolveProductionPackageRelease(environment,()=>{throw new Error('Unavailable')}),/Unavailable/);
  assert.throws(()=>productionPackageObserver(environment,()=>{throw new Error('No foreign API')})('repos/foreign/repository/releases?per_page=30'),/Foreign/);
});
test('legacy structural fixtures remain nonproduction and retain their original interface',()=>{
  const p=provider({...baseRun,event:'pull_request'},{tag_name:'v1.2.3',draft:false,prerelease:false});
  assert.equal(resolvePackageRelease(environment,p.read).publish,true);
});

for (const event of ['release', 'workflow_dispatch']) {
  test('explicit production tag response is bound to the requested version for ' + event, () => {
    const requested = {EVENT:event, REPOSITORY:repository, RELEASE_TAG:'v1.2.3',
      INPUT_TAG:'v1.2.3', PUBLISH:'true', REFERENCE:'refs/heads/main'};
    const valid = provider();
    assert.equal(resolveProductionPackageRelease(requested, valid.read).tag, 'v1.2.3');
    const foreign = provider(baseRun, {...release, tag_name:'v9.9.9'});
    assert.throws(() => resolveProductionPackageRelease(requested, foreign.read),
      /Requested release tag does not match/);
    assert.deepEqual(foreign.calls, ['repos/owner/repository/releases/tags/v1.2.3']);
  });
}
for (const field of ['draft', 'prerelease']) {
  for (const value of [undefined, null, 0, 'false']) {
    test('release observation requires a boolean ' + field + ': ' + String(value), () => {
      const p = provider(baseRun, {...release, [field]:value});
      assert.throws(() => resolveProductionPackageRelease(environment, p.read), /published metadata/);
      assert.equal(p.calls.some(endpoint => endpoint.includes('/commits/')), false);
    });
  }
}
test('explicit tag observations reject malformed responses without selecting another source', () => {
  for (const selected of [null, undefined, [], {...release, tag_name:'v01.2.3'}]) {
    const calls = [];
    const p = {calls, read:endpoint => {calls.push(endpoint); return selected;}};
    const requested = {EVENT:'release', REPOSITORY:repository, RELEASE_TAG:'v1.2.3'};
    assert.throws(() => resolveProductionPackageRelease(requested, p.read), /published metadata/);
    assert.equal(p.calls.some(endpoint => endpoint.includes('/commits/')), false);
  }
});
