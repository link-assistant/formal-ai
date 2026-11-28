import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
import YAML from 'yaml';
import {admission,compilerInputManifest} from '../../scripts/regenerate-worker-receipt.mjs';
import {createHash} from 'node:crypto';
import {parseManifest} from '../../scripts/wasm-worker-inputs.mjs';
const head='a'.repeat(40);
const environment={GITHUB_ACTIONS:'true',GITHUB_EVENT_NAME:'workflow_dispatch',
  FORMAL_AI_REGENERATION_TARGET:'worker-receipt',GITHUB_SHA:head,GITHUB_REPOSITORY:'link-assistant/formal-ai',GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'1'};
test('receipt regeneration refuses implicit, local and foreign execution',()=>{
  assert.doesNotThrow(()=>admission(environment,head));
  for(const [key,value] of [['GITHUB_ACTIONS','false'],['GITHUB_EVENT_NAME','pull_request'],
    ['GITHUB_EVENT_NAME','push'],['FORMAL_AI_REGENERATION_TARGET','translations'],
    ['GITHUB_SHA','b'.repeat(40)],['GITHUB_REPOSITORY','someone/fork'],['GITHUB_RUN_ID',''],['GITHUB_RUN_ATTEMPT','0']])
    assert.throws(()=>admission({...environment,[key]:value},head),key+' '+value);
});
test('hosted worker job selects only the explicit regeneration target',()=>{
  const job=YAML.parse(readFileSync(new URL('../../.github/workflows/layered-ci.yml',import.meta.url),'utf8')).jobs['regenerate-worker-receipt'];
  const expression=job.if.replace(/^\$\{\{\s*|\s*\}\}$/gu,'').replace('inputs.regeneration-target',"inputs['regeneration-target']");
  for(const event of ['pull_request','push','workflow_dispatch'])for(const mode of ['checks','regenerate-translations'])
    for(const target of ['translations','worker-receipt']){
      const actual=vm.runInNewContext(expression,{github:{event_name:event},inputs:{mode,'regeneration-target':target}});
      assert.equal(actual,event==='workflow_dispatch'&&mode==='regenerate-translations'&&target==='worker-receipt');
    }
  assert.deepEqual(job.permissions,{contents:'read'});
  assert.equal(job.steps[0].with['persist-credentials'],false);
  assert.equal(job.steps.find(step=>step.run).env.FORMAL_AI_REGENERATION_TARGET,'worker-receipt');
  assert.equal(job['timeout-minutes'],10);
  assert.equal(job.steps.find(step=>step.run).run,'bash scripts/run-with-budget-warning.sh 420 \"Explicit worker receipt regeneration\" node --max-old-space-size=256 scripts/regenerate-worker-receipt.mjs');
  assert.equal(job.steps.some(step=>/attest|cache|deploy|publish/u.test(step.uses??'')),false);
});

test('worker maintenance dispatch allocates exactly one job on either branch',()=>{
  const document=YAML.parse(readFileSync(new URL('../../.github/workflows/layered-ci.yml',import.meta.url),'utf8'));
  assert(document.on.workflow_dispatch.inputs['regeneration-target'].options.includes('worker-receipt'));
  for(const ref of ['refs/heads/main','refs/heads/issue-1580-8d61a32a61e1']) {
    const context={github:{event_name:'workflow_dispatch',ref,event:{}},
      inputs:{mode:'regenerate-translations','regeneration-target':'worker-receipt','diagnostic-native-run':''},
      vars:{},needs:{},cancelled:()=>false,always:()=>true};
    for(const [name,job] of Object.entries(document.jobs)) {
      const expression=job.if.trim().replace(/^\$\{\{\s*|\s*\}\}$/gu,'')
        .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu,(_,key)=>'['+JSON.stringify(key)+']');
      assert.equal(Boolean(vm.runInNewContext(expression,context,{timeout:100})),name==='regenerate-worker-receipt',name+' '+ref);
    }
  }
});

test('worker manifest preserves only actual compiler dependencies while the receipt binds metadata',()=>{
  const dependencyInfo=Buffer.from('js/formal_ai_worker.wasm: js/wasm-worker/src/lib.rs rust/src/../src/solver.rs\n');
  const record={dependencyInfoSha256:createHash('sha256').update(dependencyInfo).digest('hex'),
    output:{bytes:8,sha256:'a'.repeat(64)},compiler:'rustc 1.100.0\ncommit-hash: '+'b'.repeat(40)+'\n',
    inputs:['js/wasm-worker/src/lib.rs','rust/src/solver.rs','rust/Cargo.toml','js/wasm-worker/build.sh']
      .map(path=>({path,sha256:'c'.repeat(64)}))};
  const manifest=parseManifest(compilerInputManifest('/source',record,dependencyInfo));
  assert.deepEqual(manifest.inputs.map(input=>input.path),['js/wasm-worker/src/lib.rs','rust/src/solver.rs']);
  assert.equal(record.inputs.length,4,'the producer receipt retains its metadata bindings');
  const metadataChanged=structuredClone(record);metadataChanged.inputs[2].sha256='d'.repeat(64);
  assert.equal(compilerInputManifest('/source',metadataChanged,dependencyInfo),compilerInputManifest('/source',record,dependencyInfo));
  assert.throws(()=>compilerInputManifest('/source',record,Buffer.concat([dependencyInfo,Buffer.from('drift')])));
  assert.throws(()=>compilerInputManifest('/source',{...record,inputs:record.inputs.slice(1)},dependencyInfo));
});
