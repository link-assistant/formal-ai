// Synthetic protocol receipts prove model behavior, not real provider execution.
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host, installHost} from '../../../js/agentic/host.mjs';
import {parseChatRequest} from '../../../js/server/chat-request.mjs';
import {Progress} from '../../../js/agentic/progress.mjs';
import {composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {ownedCreationObservation, ownedCreationReadbackCommand} from '../../../js/agentic/general_execution.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {finalResult, FinalDisposition} from '../../../js/agentic/final_result.mjs';
import {completeOwnedSourceReadFrame} from '../../../js/agentic/tool_result.mjs';
import {projectedCreationPath, creationPathMatches} from '../../../js/agentic/progress_command_readback.mjs';

const root='/private/tmp/formal-ai-host-root-test';
const target='issue-708-literal-payload.txt', payload='prefix rename X to Y suffix';
const task='Create file '+target+' with exactly this content:\n'+payload;
let installed;
before(async()=>{await installNodeHost(new WorkerHost(),{sourceUsageRustGrammar:false});installed=host();});
async function atRoot(value, run) {
  installHost({...installed,currentDirectory:typeof value==='function'?value:()=>value});
  try { return await run(); } finally { installHost(installed); }
}
function call(id,name,args) {
  return {role:'assistant',content:'',tool_calls:[{id,type:'function',
    function:{name,arguments:JSON.stringify(args)}}]};
}
function result(id,name,content,is_error=false) {
  return {role:'tool',tool_call_id:id,name,content,is_error};
}
function messages(path,read=true) {
  const rows=[{role:'user',content:task},
    call('write','write',{filePath:path,content:payload}),result('write','write','')];
  if(read) rows.push(call('read','read',{filePath:path}),
    result('read','read',completeOwnedSourceReadFrame(payload)));
  return parseChatRequest(JSON.stringify({model:'formal-ai',cwd:root,messages:rows})).messages;
}
function proof(rows,request=task) {
  return ownedCreationObservation(composeGeneralChangePlan(request),Progress.scan(rows));
}

test('host-root absolute Write and Read verify the original 27-byte issue708 literal',()=>atRoot(root,async()=>{
  assert.equal(Buffer.byteLength(payload),27);
  const rows=messages(root+'/'+target), raw=rows[1].tool_calls[0].function.arguments;
  const progress=Progress.scan(rows), mutation=progress.creationMutationFor(target);
  assert.equal(mutation.path,target);
  assert.equal(mutation.observedPath,root+'/'+target);
  assert.equal(mutation.workspaceRelativePath,target);
  assert.equal(mutation.arguments,raw);
  assert.equal(JSON.parse(raw).filePath,root+'/'+target);
  const observed=proof(rows);
  assert.equal(observed.source,payload);
  assert.equal(observed.target,target);
  assert.equal(observed.command,null);
  assert.equal(observed.exitCode,null);
  assert.equal(finalResult(await planChatStepResolved(rows,['read','write'])).disposition,FinalDisposition.Finding);
}));

test('an observed absolute Write still requests independent target verification',()=>atRoot(root,async()=>{
  const step=await planChatStepResolved(messages(root+'/'+target,false),['read','write']);
  assert.equal(step.kind,'tool_calls');
  assert.equal(step.calls[0].tool,'read');
  assert.equal(JSON.parse(step.calls[0].arguments).path,target);
  assert.equal(proof(messages(root+'/'+target,false)),null);
}));

test('foreign roots and conflicting aliases cannot borrow the trusted root',()=>atRoot(root,()=>{
  for(const path of ['/private/tmp/foreign-root/'+target,root+'-other/'+target]) {
    assert.equal(proof(messages(path)),null);
  }
  const aliases=messages(root+'/'+target);
  aliases[1].tool_calls[0].function.arguments=JSON.stringify({
    path:target,filePath:root+'/'+target,content:payload});
  assert.equal(proof(aliases),null);
  for(const variant of ['pending','wrong-id','failed','duplicate']) {
    const rows=messages(root+'/'+target);
    if(variant==='pending')rows.splice(2,1);
    if(variant==='wrong-id')rows[2].tool_call_id='foreign';
    if(variant==='failed')rows[2].is_error=true;
    if(variant==='duplicate')rows.splice(3,0,structuredClone(rows[2]));
    assert.equal(proof(rows),null,variant);
  }
}));

test('null or throwing host scope cannot grant an absolute-path equivalence',async()=>{
  for(const value of [null,()=>{throw Error('UnavailableCwd');}]) await atRoot(value,()=>{
    assert.equal(proof(messages(root+'/'+target)),null);
    assert.equal(proof(messages(target)).source,payload,'exact relative proof remains supported');
  });
});

test('only exact POSIX host projections are admitted',()=>{
  for(const [relative,scope] of [
    ['../x.txt',root],['./x.txt',root],['nested//x.txt',root],
    ['C:x.txt',root],['nested\\x.txt',root],['x.txt','/C:foo']
  ]) assert.equal(projectedCreationPath(relative,scope),null);
  assert.equal(creationPathMatches(target,root+'/'+target,root),true);
  assert.equal(creationPathMatches(target,root+'//'+target,root),false);
});

test('disjoint absolute writes retain exact relative framed-run proof and command identity',()=>atRoot(root,()=>{
  const request='First, create a.txt with exactly this content «alpha». Second, create b.txt with exactly this content «beta».';
  for(const failed of [false,true]) {
    const rows=[{role:'user',content:request}];
    for(const [path,content] of [['a.txt','alpha'],['b.txt','beta']]) rows.push(
      call('write-'+path,'write',{filePath:root+'/'+path,content}),result('write-'+path,'write','Success'));
    for(const [path,content] of [['a.txt','alpha'],['b.txt','beta']]) rows.push(
      call('run-'+path,'bash',{command:ownedCreationReadbackCommand(path)}),
      result('run-'+path,'bash','Output: ('+content+')\nExit Code: '+(failed&&path==='b.txt'?1:0),failed&&path==='b.txt'));
    const progress=Progress.scan(rows);
    const first=ownedCreationObservation(composeGeneralChangePlan('create a.txt with exactly this content «alpha».'),progress);
    assert.equal(first.source,'alpha');
    assert.equal(first.command,ownedCreationReadbackCommand('a.txt'),'actual relative command retained');
    assert.equal(first.exitCode,0);
    const second=ownedCreationObservation(composeGeneralChangePlan('create b.txt with exactly this content «beta».'),progress);
    assert.equal(second!==null,!failed);
  }
}));
