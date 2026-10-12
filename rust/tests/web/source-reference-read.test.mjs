import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {createNodeSourceSessionHost,registeredPhysicalReadObservation} from '../../../js/server/node-source-session-host.mjs';
import {preparePhysicalRustRename} from '../../../experiments/formal_ai_subagent/physical-rust-transform-input.mjs';
const original = [
  "Atomic task L31: In the file rust/src/learning_adoption_ledger.rs, rename the constant UNKNOWN_INTEN",
  "T to UNKNOWN_INTENT_NAME. Change only that file and keep it valid Rust.\n\nThis is recursive binary-tr",
  "ee node 2.2.2.2.1 at depth 5. Solve only this node's task in this fresh temporary repository. Its ha",
  "rness-evaluated completion criterion is: tracked_source_change. Apply the change to the tracked file",
  " `rust/src/learning_adoption_ledger.rs` itself -- the file has to end up modified in the Git worktre",
  "e, and nothing else may change. Then create `agent-ladder-effects/node-2.2.2.2.1.lino` with these ex",
  "act field lines: `node_path=2.2.2.2.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by ",
  "at least four words that state the change you made and that contain the exact text UNKNOWN_INTENT_NA",
  "ME. Leave supporting evidence in .agent-ladder/node-2.2.2.2.1-proof.md. The first line must be exact",
  "ly node_path=2.2.2.2.1 and the body must state the concrete result. The harness rejects proof withou",
  "t the separate Git effect. Use web research when it materially improves factual accuracy. Do not cla",
  "im success without evidence.\n",
].join('');
async function controls(){
 await installNodeHost(new WorkerHost());
 const root=fs.realpathSync(new URL('../../..',import.meta.url));
 const session=createNodeSourceSessionHost(p=>fs.readFileSync(root+'/'+p,'utf8'));
 installHost({...host(),sourceSession:session,sourceOperation:session.sourceOperation});
 const workspace=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-reference-read-')));
 const invoke=(request,operand,id='reference-read',argumentsText=JSON.stringify({file_path:operand}))=>{
  const call={tool:'read',arguments:argumentsText};
  const declaration={id,name:call.tool,arguments:call.arguments};
  const messages=[{role:'user',content:request},{role:'assistant',tool_calls:[{id,function:{name:call.tool,arguments:call.arguments}}]}];
  return {call,declaration,messages};
 };
 let assertions=0;const check=(value)=>{assert.equal(value,true);assertions++;};
 try{
  const init=spawnSync('git',['init','--quiet',workspace],{encoding:'utf8'});
  assert.equal(init.status,0);
  for(const [operand,previous,replacement] of [
   ['rust/src/learning_adoption_ledger.rs','UNKNOWN_INTENT','UNKNOWN_INTENT_NAME'],
   ['rust/src/domain_catalog.rs','UNRESOLVED_KIND','UNRESOLVED_KIND_NAME']]){
   const request=original.replaceAll('rust/src/learning_adoption_ledger.rs',operand)
     .replaceAll('UNKNOWN_INTENT_NAME',replacement).replaceAll('UNKNOWN_INTENT',previous);
   const source = previous === 'UNKNOWN_INTENT'
     ? fs.readFileSync(root + '/rust/src/learning_adoption_ledger.rs', 'utf8')
     : `const ${previous}: usize = 7;\nfn observed() -> usize { return ${previous}; }\n`;
   const expected = source.split(previous).join(replacement);
   fs.mkdirSync(path.dirname(workspace+'/'+operand),{recursive:true});fs.writeFileSync(workspace+'/'+operand,source);
   assert.equal(spawnSync('git',['-C',workspace,'add','--',operand]).status,0);
   const input=invoke(request,operand);let escaped;
   await session.run({request,workspace,tools:['read','write','bash']},async()=>{
    check(session.sourceOperation.hasContext()===false);
    const read=session.executeResult(input.call,input.messages,()=>{throw Error('ForbiddenReaderSubstitute');});
    check(read.content===source);check(read.is_error!==true);escaped=read.owned_source_read;
    const fact=registeredPhysicalReadObservation(escaped,request,workspace,input.declaration);
    check(fact.referenceRequest.clauses.length===12);
    check(fact.referenceRequest.originalRequest===request);
    const candidate=await preparePhysicalRustRename(escaped,request,workspace,input.declaration);
    check(candidate.candidate.content===expected);
    check(candidate.pendingNeeds.length===12&&candidate.writeAuthority===false);
    check(candidate.trackedGitEffect==='Missing'&&candidate.compilerValidity==='Unknown');
    assert.throws(()=>registeredPhysicalReadObservation({},request,workspace,input.declaration));assertions++;
    assert.throws(()=>registeredPhysicalReadObservation(escaped,request+' ',workspace,input.declaration));assertions++;
    assert.throws(()=>session.executeResult({tool:'write',arguments:JSON.stringify({file_path:operand,content:'bad'})},input.messages,()=>{throw Error('ForbiddenFallback');}),/MissingReferenceEffectAuthority/);assertions++;
    const foreign=invoke(request,'rust/src/foreign.rs','foreign-read');
    check(session.executeResult(foreign.call,foreign.messages,()=>{throw Error('ForbiddenFallback');}).is_error===true);
    const malformed=invoke(request,operand,'extra-read',JSON.stringify({file_path:operand,extra:true}));
    check(session.executeResult(malformed.call,malformed.messages,()=>{throw Error('ForbiddenFallback');}).is_error===true);
    assert.throws(()=>registeredPhysicalReadObservation(escaped,request,workspace,input.declaration));assertions++;
    check(fs.readFileSync(workspace+'/'+operand,'utf8')===source);
   });
   assert.throws(()=>registeredPhysicalReadObservation(escaped,request,workspace,input.declaration));assertions++;
   check(fs.readFileSync(workspace+'/'+operand,'utf8')===source);
  }
  await assert.rejects(session.run({request:original,workspace,tools:['write']},async()=>{
   throw Error('ForbiddenMissingReadExecution');
  }),/MissingReferenceReadCapability/);assertions++;
  const forgedSeedSession = createNodeSourceSessionHost(p =>
   fs.readFileSync(root + '/' + p, 'utf8') + (p === 'data/seed/source-authoring-grammar.lino' ? '\n' : ''));
  await assert.rejects(forgedSeedSession.run({request:original,workspace,tools:['read']},async()=>{
   throw Error('ForbiddenForgedSeedExecution');
  }),/DifferentPhysicalReferenceRequestSeed/);assertions++;
  for(const request of ['Do not read. '+original,'Do not write. '+original,original+' Publish unrelated result.',original.slice(0,original.indexOf('Then create'))]){
   await session.run({request,workspace,tools:['read']},async()=>{
    assert.throws(()=>registeredPhysicalReadObservation({},request,workspace,{}));assertions++;
   });
  }
  console.log(JSON.stringify({assertions,physicalRead:true,trackedInputs:true,effects:false,nativeExecution:false}));
 }finally{fs.rmSync(workspace,{recursive:true,force:true});}
}
if(process.env.NODE_TEST_CONTEXT&& !process.argv.includes('--physical-reference-child')){
 test('real source reference Read controls',()=>{
  const env={...process.env};delete env.NODE_TEST_CONTEXT;
  const result=spawnSync(process.execPath,[fileURLToPath(import.meta.url),'--physical-reference-child'],{env,encoding:'utf8'});
  console.log(result.stdout);
  assert.equal(result.status,0,result.stderr+result.stdout);
 });
}else await controls();
