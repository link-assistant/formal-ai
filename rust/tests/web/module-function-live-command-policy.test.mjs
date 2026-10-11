import {before,test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,realpathSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installDefaultNodeSourceHost} from '../../../js/server/default-node-source-bootstrap.mjs';
import {queryOwnedInferredCommandPolicy} from '../../../js/server/node-source-session-host.mjs';
import {host} from '../../../js/agentic/host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
before(async()=>installDefaultNodeSourceHost(new WorkerHost()));
const tools=['read','write','edit','bash'];
const cases=[
 {name:'product',expression:'left*right',module:'alpha.mjs',test:'alpha.test.mjs'},
 {name:'difference',expression:'left-right',module:'beta.mjs',test:'beta.test.mjs'},
];
for(const contract of cases) for(const restricted of [false,true]){
 test((restricted?'restricted':'unrestricted')+' actual source-owned verification '+contract.name,async()=>{
  const dir=realpathSync(mkdtempSync(join(tmpdir(),'formal-policy-profile-')));
  try{
   const requested='node --test '+contract.test;
   const task='Add a function '+contract.name+'(left,right) that returns '+contract.expression+' to '+contract.module
    +', add a test for it to '+contract.test+', and run '+requested+'.';
   const options={steps:12,tools};if(restricted) options.allowedCommands=[requested];
   const result=await drive(planChatStep,dir,task,options);
   assert.equal(result.stop,'final');
   assert.equal(result.deniedCalls?.length??0,0);
   assert.deepEqual(result.transcript.filter(row=>row.tool==='bash').map(row=>JSON.parse(row.arguments).command),
    restricted?[requested]:['node --check '+contract.module,requested]);
  }finally{rmSync(dir,{recursive:true,force:true});}
 });
}
test('public source-session context cannot manufacture a restrictive policy',async()=>{
 const dir=realpathSync(mkdtempSync(join(tmpdir(),'formal-forged-policy-')));
 const session=host().sourceSession;
 try{const result=await session.run({request:'Describe this source.',workspace:dir,tools,allowedCommands:[]},
  ()=>queryOwnedInferredCommandPolicy(session,'Describe this source.','node --check alpha.mjs',tools));
 assert.equal(result,null);
 assert.equal(queryOwnedInferredCommandPolicy({...session},'Describe this source.','node --check alpha.mjs',tools),null);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('live policy lookup rejects changed requests, tool sets, copied APIs, and stale contexts',async()=>{
 const dir=realpathSync(mkdtempSync(join(tmpdir(),'formal-live-policy-'))),request='Describe this source.';
 let deferred,stale;const barrier=new Promise(resolve=>deferred=resolve);
 try{
  const result=await drive(async()=>{
   const session=host().sourceSession;
   assert.equal(queryOwnedInferredCommandPolicy(session,request,'node --check alpha.mjs',tools),false);
   assert.equal(queryOwnedInferredCommandPolicy(session,request+' changed','node --check alpha.mjs',tools),null);
   assert.equal(queryOwnedInferredCommandPolicy(session,request,'node --check alpha.mjs',['read']),null);
   assert.equal(queryOwnedInferredCommandPolicy({...session},request,'node --check alpha.mjs',tools),null);
   stale=barrier.then(()=>queryOwnedInferredCommandPolicy(session,request,'node --check alpha.mjs',tools));
   return{kind:'final',answer:'Recorded source-owned policy observations.'};
  },dir,request,{steps:1,tools,allowedCommands:['node --test alpha.test.mjs']});
  assert.equal(result.stop,'final');deferred();assert.equal(await stale,null);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('NoBash advertises no command capability and policy observation grants none',async()=>{
 const dir=realpathSync(mkdtempSync(join(tmpdir(),'formal-no-bash-policy-'))),request='Describe this source.';
 try{const result=await drive(async(messages,actualTools)=>{
  assert.equal(actualTools.includes('bash'),false);
  return{kind:'tools',calls:[{tool:'bash',arguments:JSON.stringify({command:'node --test alpha.test.mjs'})}]};
 },dir,request,{steps:1,tools:['read','write','edit'],allowedCommands:['node --test alpha.test.mjs']});
 assert.equal(result.transcript.filter(row=>row.tool==='bash').length,0);
 assert.notEqual(result.stop,'final');
 }finally{rmSync(dir,{recursive:true,force:true});}
});
