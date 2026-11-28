// Current upstream bytes are immutable fixtures; the repair runs only in a copy.
import test, {after} from 'node:test';
import assert from 'node:assert/strict';
import {cpSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {evaluate as original} from './fixtures/go-current-pipeline/scripts/ci/check-pipeline-status.mjs';

const fixture=fileURLToPath(new URL('./fixtures/go-current-pipeline/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}
const directory=mkdtempSync(join(tmpdir(),'issue-1580-go-needs-'));
after(()=>rmSync(directory,{recursive:true,force:true}));
cpSync(join(fixture,'scripts'),join(directory,'scripts'),{recursive:true});
const script=join(directory,'scripts/ci/check-pipeline-status.mjs');
const before=readFileSync(script,'utf8');
const boundary='export function blockingNeeds(needs, event, releaseMode) {\n';
assert.equal(before.split(boundary).length,2);
const guard=`  const results = new Set(['success', 'failure', 'cancelled', 'skipped']);
  const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
  if (!object(needs) || !Object.keys(needs).length ||
      !Object.values(needs).every(value => object(value) && results.has(value.result)))
    return ['Invalid or missing completed pipeline result records'];
`;
writeFileSync(script,process.env.FORMAL_AI_DISABLE_GO_NEEDS_CANDIDATE==='1'?before:before.replace(boundary,boundary+guard));
const {evaluate:candidate}=await import(pathToFileURL(script).href);

function observe(evaluate,needs,event='pull_request',releaseMode){
  let output='';
  try{
    const status=evaluate({needs,event,releaseMode,jobs:()=>[],classify:()=>[],githubActions:false,stdout:{write:text=>output+=text}});
    return {status,output};
  }catch(error){return {status:1,output:String(error)};}
}

for(const [name,needs] of [
  ['empty object',{}],['empty array',[]],['array of records',[{result:'success'}]],
  ['number',42],['string','unknown'],['missing result',{lint:{}}],
  ['null record',{lint:null}],['string record',{lint:'success'}],
  ['unknown result',{lint:{result:'neutral'}}],
  ['partial evidence',{lint:{result:'success'},test:{result:null}}],
]){
  test(`Go PR pipeline accepts ${name}; a complete schema rejects it`,{timeout:3000},()=>{
    assert.equal(observe(original,needs).status,0,'preserve the actual upstream false success');
    const repaired=observe(candidate,needs);
    assert.equal(repaired.status,1,repaired.output);
    assert.match(repaired.output,/Invalid or missing completed pipeline result records/);
    assert.doesNotMatch(repaired.output,/Every required job succeeded/);
  });
}
for(const result of ['success','skipped','failure','cancelled']){
  test(`Go complete ${result} result keeps its original outcome`,{timeout:3000},()=>{
    const needs={lint:{result}};
    const expected=['success','skipped'].includes(result)?0:1;
    assert.equal(observe(original,needs).status,expected);
    assert.equal(observe(candidate,needs).status,expected);
  });
}
test('Go root null is already rejected and stays rejected',{timeout:3000},()=>{
  assert.equal(observe(original,null).status,1);
  assert.equal(observe(candidate,null).status,1);
});
for(const [event,releaseMode,required] of [
  ['push',undefined,['detect-changes','lint','test','build','auto-release']],
  ['workflow_dispatch','instant',['lint','test','build','instant-release']],
  ['workflow_dispatch','changeset',['lint','test','build','changeset-release']],
]){
  test(`Go ${event} ${releaseMode??'auto'} still requires successful release jobs`,{timeout:3000},()=>{
    const needs=Object.fromEntries(required.map(name=>[name,{result:'success'}]));
    for(const evaluate of [original,candidate]){
      assert.equal(observe(evaluate,needs,event,releaseMode).status,0);
      for(const result of ['skipped','failure','cancelled'])
        assert.equal(observe(evaluate,{...needs,[required.at(-1)]:{result}},event,releaseMode).status,1);
    }
  });
}
