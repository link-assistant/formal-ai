import {test,before} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installDefaultNodeSourceHost} from '../../../js/server/default-node-source-bootstrap.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
before(async()=>installDefaultNodeSourceHost(new WorkerHost()));
const task='Add a function product(a,b) that returns a*b to p.mjs, add a test for it to p.test.mjs, and run node --test p.test.mjs.';
test('explicit verification stays inside the original command allowlist',async()=>{
 const workspace=mkdtempSync(join(tmpdir(),'formal-command-profile-'));
 try {const result=await drive(planChatStep,workspace,task,{steps:12,tools:['read','write','edit','bash'],allowedCommands:['node --test p.test.mjs']});
 assert.equal(result.stop,'final');
 assert.deepEqual(result.transcript.filter(x=>x.tool==='bash').map(x=>JSON.parse(x.arguments).command),['node --test p.test.mjs']);
 assert.equal(result.deniedCalls?.length??0,0);
 }finally{rmSync(workspace,{recursive:true,force:true});}
});
test('an unapproved explicit verification is refused without substituting a check',async()=>{
 const workspace=mkdtempSync(join(tmpdir(),'formal-command-denied-'));
 try {const result=await drive(planChatStep,workspace,task,{steps:12,tools:['read','write','edit','bash'],allowedCommands:['node --check p.mjs']});
 assert.equal(result.stop,'command-policy-denied');
 assert.equal(result.transcript.filter(x=>x.tool==='bash').length,0);
 assert.deepEqual(result.deniedCalls.map(x=>JSON.parse(x.arguments).command),['node --test p.test.mjs']);
 }finally{rmSync(workspace,{recursive:true,force:true});}
});
