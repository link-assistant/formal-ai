// Check the deployed workflow's network step boundaries without invoking GitHub.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
test('every desktop producer and consumer checkout has a short explicit deadline',()=>{
  const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/desktop-release.yml',import.meta.url),'utf8'));
  const checked=[];
  for(const [name,job] of Object.entries(workflow.jobs))for(const step of job.steps??[]) {
    if(!step.uses?.startsWith('actions/checkout@'))continue;
    assert.ok(Number.isFinite(step['timeout-minutes'])&&step['timeout-minutes']>0&&step['timeout-minutes']<=5,name);
    checked.push(name);
  }
  for(const name of ['resolve','native-source','native','build','cli','vscode','finalize'])assert.ok(checked.includes(name),name);
});

test('desktop step deadlines preserve the job margin in every declared matrix leg',()=>{
 const name='desktop-release.yml';let checked=0;
 {
  const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/'+name,import.meta.url),'utf8'));
  for(const [id,job] of Object.entries(workflow.jobs??{}))for(const step of job.steps??[]){
   const budget=step['timeout-minutes'];if(typeof budget!=='number')continue;
   const raw=job['timeout-minutes'];let caps;
   if(typeof raw==='number')caps=[raw];
   else{
    const match=/^\$\{\{ matrix\.([A-Za-z0-9_]+) \}\}$/u.exec(raw??'');assert.ok(match,name+':'+id);
    assert.ok(Array.isArray(job.strategy?.matrix?.include)&&job.strategy.matrix.include.length>0,name+':'+id);
    caps=job.strategy.matrix.include.map(row=>row[match[1]]);
   }
   for(const cap of caps){
    assert.ok(Number.isSafeInteger(cap)&&cap>0,name+':'+id);
    assert.ok(Math.floor(budget*100/cap)<=70,name+':'+id+' step exceeds 70% of '+cap+'m');
    checked++;
   }
  }
 }
 assert.ok(checked>20);
});
