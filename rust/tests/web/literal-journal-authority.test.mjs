import {host,installHost} from '../../../js/agentic/host.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {driverRequestAuthority,withDriverRequestAuthority,literalJournalAuthority,literalJournalCallsMatch} from '../../../js/provenance/literal-journal-authority.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {ownedDeclaredCreateFrame} from '../../../js/agentic/planner/owned_goals.mjs';
test('actual execution and replay derive the same whole-file journal authority without granting Create overwrite',async()=>{
 await installNodeHost(new WorkerHost({WebAssembly:{compile(){throw Error('ProductWasmDisabled');},
  instantiate(){throw Error('ProductWasmDisabled');},instantiateStreaming(){throw Error('ProductWasmDisabled');}}}));
 const prompt='Set the contents of retained.mjs to exactly this content: «export const sample = 97;».';
 const authority=literalJournalAuthority(prompt,false);
 assert.equal(authority.target,'retained.mjs');assert.equal(authority.content,'export const sample = 97;');
 assert.deepEqual(literalJournalAuthority(prompt,ownedDeclaredCreateFrame(prompt)!==null),authority);
 const requestAuthority=driverRequestAuthority(prompt);
 assert.deepEqual(requestAuthority.literalJournal,authority);assert.deepEqual(requestAuthority.createContracts,[]);
 const original='Create file retained.mjs containing export const sample = 97;';
 assert.notEqual(ownedDeclaredCreateFrame(original),null);
 assert.equal(literalJournalAuthority(original,true),null);
 const creation=driverRequestAuthority(original);assert.equal(creation.literalJournal,null);
 assert.equal(creation.createContracts.length,1);assert.equal(creation.createContract.target,'retained.mjs');
 assert.equal(literalJournalAuthority('Read file retained.mjs.',false),null);
 assert.equal(literalJournalAuthority('Create file a.txt containing A. Create file b.txt containing B.',false),null);
});
test('replay journal calls retain exact maintained tool, target, bytes and atomic request identity',()=>{
 const authority={target:'retained.mjs',content:'export const sample = 97;'};
 const argumentsValue={path:'plan.lino',content:'retained record',append_mode:'atomic_record_append',append_request_id:'owned-request'};
 const call={tool:'write',arguments:JSON.stringify(argumentsValue)},canonical={kind:'tool_calls',calls:[call]};
 assert.equal(literalJournalCallsMatch(authority,[call],canonical),true);
 for(const field of ['path','content','append_request_id'])assert.equal(literalJournalCallsMatch(authority,[{
  ...call,arguments:JSON.stringify({...argumentsValue,[field]:'forged'})}],canonical),false);
 assert.equal(literalJournalCallsMatch(authority,[{...call,tool:'bash'}],canonical),false);
 assert.equal(literalJournalCallsMatch(authority,[call],{kind:'final',answer:'done'}),false);
 assert.equal(literalJournalCallsMatch(null,[call],null),true);
});

const heldoutAuthority={target:'retained.mjs',content:'owned'},heldoutCall={tool:'write',arguments:'{"path":"owned","content":"bytes"}'};
test('literal journal refuses an empty recorded batch',()=>{
 assert.equal(literalJournalCallsMatch(heldoutAuthority,[],{kind:'tool_calls',calls:[heldoutCall]}),false);
});
test('literal journal refuses reusing one canonical call more than once',()=>{
 assert.equal(literalJournalCallsMatch(heldoutAuthority,[heldoutCall,heldoutCall],{kind:'tool_calls',calls:[heldoutCall]}),false);
 const other={tool:'read',arguments:'{"path":"owned"}'},canonical={kind:'tool_calls',calls:[heldoutCall,other]};
 assert.equal(literalJournalCallsMatch(heldoutAuthority,[heldoutCall,heldoutCall],canonical),false);
 assert.equal(literalJournalCallsMatch(heldoutAuthority,[other,heldoutCall],canonical),true);
 assert.equal(literalJournalCallsMatch(heldoutAuthority,[other],canonical),true);
});
test('literal journal refuses a malformed canonical call batch',()=>{
 assert.equal(literalJournalCallsMatch(heldoutAuthority,[heldoutCall],{kind:'tool_calls',calls:null}),false);
});

test('replay derives real request authority only inside its strict source-session scope',async()=>{
 const installed=host();let active=false,reads=0;
 const strict={...installed,readText(relative){if(!active)throw Error('StrictRequestScopeRequired');reads++;return installed.readText(relative);}};
 const request='Set the contents of scoped.mjs to exactly this content: «export const scoped = 17;».';
 try{
  installHost(strict);
  assert.throws(()=>driverRequestAuthority(request),/StrictRequestScopeRequired/u);
  installHost(strict);
  const session={async run(context,action){assert.equal(context.request,request);active=true;try{return await action();}finally{active=false;}}};
  await withDriverRequestAuthority(session,{request,workspace:'/synthetic',tools:['read','write']},authority=>{
   assert.equal(active,true);assert.equal(authority.literalJournal.target,'scoped.mjs');
   assert.equal(authority.literalJournal.content,'export const scoped = 17;');
  });
  assert.ok(reads>0);assert.equal(active,false);
 }finally{installHost(installed);}
});
