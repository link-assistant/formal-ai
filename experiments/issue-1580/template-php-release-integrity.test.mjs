// Evaluate the real pinned job guards with bounded, dependency-result controls.
// This is an offline expression model, not an observed hosted publication.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {Script} from 'node:vm';
import YAML from 'yaml';

const fixture=new URL('./fixtures/php-template/',import.meta.url);
const provenance=JSON.parse(readFileSync(new URL('source.json',fixture),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(new URL(item.path,fixture));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}
const workflow=YAML.parse(readFileSync(new URL('.github/workflows/release.yml',fixture),'utf8'));
const prerequisites=['lint','test','build','release-preflight'];
const unsuccessful=['failure','cancelled','skipped'];

function admitted(job,{results={},event='push',ref='refs/heads/main',cancelled=false}={}){
  const needs=Object.fromEntries(prerequisites.map(name=>[name,{result:results[name]??'success'}]));
  const success=()=>Object.values(needs).every(value=>value.result==='success');
  const source=String(job.if).trim().replace(/^\$\{\{\s*|\s*\}\}$/gu,'')
    .replaceAll('needs.release-preflight',"needs['release-preflight']");
  const explicitStatus=/\b(?:always|success|failure|cancelled)\s*\(/u.test(source);
  const value=new Script(source).runInNewContext({
    github:{event_name:event,ref},needs,always:()=>true,cancelled:()=>cancelled,
    success,failure:()=>Object.values(needs).some(value=>value.result==='failure'),
  },{timeout:1000});
  return Boolean(value)&&(explicitStatus||success());
}

const automatic=workflow.jobs['auto-release'];
const manual=workflow.jobs['manual-release'];
const repaired={...automatic,if:automatic.if+' &&\n'+
  ['lint','test','build'].map(name=>`needs.${name}.result == \'success\'`).join(' &&\n')};

test('PHP automatic release admits unsuccessful quality checks in the actual pinned guard',()=>{
  assert.deepEqual(automatic.needs,prerequisites);
  assert.equal(admitted(automatic),true,'successful default-branch push is admitted');
  for(const name of ['lint','test','build']){
    for(const result of unsuccessful){
      assert.equal(admitted(automatic,{results:{[name]:result}}),true,`${name}: ${result}`);
    }
  }
  for(const result of unsuccessful){
    assert.equal(admitted(automatic,{results:{'release-preflight':result}}),false);
  }
});

test('explicit success checks block all unsuccessful prerequisites while retaining successful release',()=>{
  assert.equal(admitted(repaired),true);
  for(const name of prerequisites){
    for(const result of unsuccessful){
      assert.equal(admitted(repaired,{results:{[name]:result}}),false,`${name}: ${result}`);
    }
  }
});

test('implicit success gating blocks failed checks, while retaining cancelled() still overrides it',()=>{
  const implicit={...automatic,if:automatic.if.replace('always() && !cancelled() &&','')};
  const cancellationOnly={...automatic,if:automatic.if.replace('always() && ','')};
  assert.equal(admitted(implicit),true);
  for(const name of ['lint','test','build']){
    assert.equal(admitted(implicit,{results:{[name]:'failure'}}),false);
    assert.equal(admitted(cancellationOnly,{results:{[name]:'failure'}}),true);
  }
});

test('the PHP automatic-release candidate retains event, branch and cancellation controls',()=>{
  for(const job of [automatic,repaired]){
    assert.equal(admitted(job,{event:'pull_request'}),false);
    assert.equal(admitted(job,{event:'workflow_dispatch'}),false);
    assert.equal(admitted(job,{ref:'refs/heads/feature'}),false);
    assert.equal(admitted(job,{cancelled:true}),false);
  }
});

test('the unchanged PHP manual-release guard already requires every prerequisite to succeed',()=>{
  assert.deepEqual(manual.needs,prerequisites);
  assert.equal(admitted(manual,{event:'workflow_dispatch'}),true);
  assert.equal(admitted(manual),false);
  assert.equal(admitted(manual,{event:'workflow_dispatch',cancelled:true}),false);
  for(const name of prerequisites){
    for(const result of unsuccessful){
      assert.equal(admitted(manual,{event:'workflow_dispatch',results:{[name]:result}}),false);
    }
  }
});
