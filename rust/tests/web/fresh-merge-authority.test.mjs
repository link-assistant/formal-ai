import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import YAML from 'yaml';
import { runInNewContext } from 'node:vm';
const root=new URL('../../../',import.meta.url);
function read(path){return readFileSync(new URL(path,root),'utf8');}
function invokingSteps(workflow){
 const parsed=YAML.parse(workflow);const steps=[];
 for(const [jobName,job] of Object.entries(parsed.jobs??{}))for(const step of job.steps??[]){
  const invocations=String(step.run??'').split('\n').filter(line=>/^(?:bash\s+)?scripts\/simulate-fresh-merge\.sh(?:\s|$)/u.test(line.trim()));
  if(invocations.length){const pin=step.env?.BASE_COMMIT;steps.push({jobName,count:invocations.length,pin,valid:typeof pin==='string'&&['${{needs.base.outputs.commit}}','${{inputs.base-commit}}'].includes(pin.replaceAll(/\s/gu,''))});}
 }return steps;
}
test('all six real merge invocations bind the exact shared resolver in their own steps',()=>{
 let count=0;
 for(const name of readdirSync(new URL('.github/workflows/',root)).filter(name=>/\.ya?ml$/u.test(name))){
  const observed=invokingSteps(read('.github/workflows/'+name));
  assert.ok(observed.every(step=>step.valid),name);
  assert.ok(new Set(observed.map(step=>step.pin.replaceAll(/\s/gu,''))).size<=1,name);
  count+=observed.reduce((sum,step)=>sum+step.count,0);
 }assert.equal(count,6);
});
test('the six original native binding negatives cannot certify an invoking step',()=>{
 const source=read('rust/tests/unit/ci-cd/fresh_merge_bindings.rs');const block=source.slice(source.indexOf('let cases = ['),source.indexOf('];',source.indexOf('let cases = [')));
 const cases=Array.from(block.matchAll(/"((?:\\.|[^"\\])*)"/gu),match=>JSON.parse('"'+match[1]+'"'));assert.equal(cases.length,6);
 for(const fixture of cases){
  const yaml='jobs:\n  probe:\n    steps:\n'+fixture;
  try{const observed=invokingSteps(yaml);assert.equal(observed.length,1);assert.equal(observed[0].valid,false,fixture);}
  catch(error){assert.match(error.message,/Map keys must be unique/u);}
 }
});
test('one step pin binds repeated invocations while metadata prose is not a command',()=>{
 const fixture='jobs:\n  probe:\n    steps:\n      - run: echo scripts/simulate-fresh-merge.sh\n      - env:\n          BASE_COMMIT: ${{ needs.base.outputs.commit }}\n        run: |\n          bash scripts/simulate-fresh-merge.sh\n          bash scripts/simulate-fresh-merge.sh';
 assert.deepEqual(invokingSteps(fixture),[{jobName:'probe',count:2,pin:'${{ needs.base.outputs.commit }}',valid:true}]);
});
test('native container publication requires explicit event, main ref, mode, matching success and nonempty tag',()=>{
 const condition=YAML.parse(read('.github/workflows/release.yml')).jobs['native-container-images'].if;

 const expression=condition.replaceAll('needs.auto-release','needs["auto-release"]').replaceAll('needs.manual-release','needs["manual-release"]').replaceAll('.outputs.container-tag','.outputs["container-tag"]');
 const evaluate=(event,ref,mode,autoSucceeded,manualSucceeded,autoTag,manualTag,isCancelled=false)=>runInNewContext(expression,{github:{event_name:event,ref,event:{inputs:{release_mode:mode}}},needs:{'auto-release':{result:autoSucceeded?'success':'failure',outputs:{'container-tag':autoTag}},'manual-release':{result:manualSucceeded?'success':'failure',outputs:{'container-tag':manualTag}}},cancelled:()=>isCancelled},{timeout:100});
 assert.equal(evaluate('push','refs/heads/main','checks',true,false,'v1',''),true);
 assert.equal(evaluate('workflow_dispatch','refs/heads/main','instant',false,true,'','v1'),true);
 for(const event of ['pull_request','workflow_dispatch','schedule'])assert.equal(evaluate(event,'refs/heads/main','checks',true,true,'v1','v1'),false,event);
 assert.equal(evaluate('workflow_dispatch','refs/heads/branch','instant',true,true,'v1','v1'),false);
 assert.equal(evaluate('push','refs/heads/branch','instant',true,true,'v1','v1'),false);
 assert.equal(evaluate('workflow_dispatch','refs/heads/main','instant',true,false,'v1','v1'),false);
 assert.equal(evaluate('push','refs/heads/main','checks',false,true,'v1','v1'),false);
 assert.equal(evaluate('workflow_dispatch','refs/heads/main','instant',false,true,'',''),false);
 assert.equal(evaluate('push','refs/heads/main','checks',true,false,'','v1'),false);
 assert.equal(evaluate('workflow_dispatch','refs/heads/main','instant',false,true,'','v1',true),false);
});
