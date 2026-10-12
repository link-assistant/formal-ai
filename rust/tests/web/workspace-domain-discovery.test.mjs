import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host, installHost} from '../../../js/agentic/host.mjs';
import {Progress,qualifiedToolAttempt} from '../../../js/agentic/progress.mjs';
import {Capability} from '../../../js/agentic/capability.mjs';
import {executeResult} from '../../../experiments/js_dogfood/drive.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {workspaceDiscoveryContract, workspaceDiscoveryCommand, workspaceDiscoveryStep} from '../../../js/agentic/workspace_discovery.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
  const original = host();
  const seed = readFileSync(new URL('../../../data/seed/workspace-discovery-grammar.lino',import.meta.url),'utf8');
  installHost({...original, readText:path => path === 'data/seed/workspace-discovery-grammar.lino' ? seed : original.readText(path)});
});
const originalNeed = 'Inspect the task model and record the concrete result. Leave supporting evidence in `.agent-ladder/node-proof.md`. The first line must be exactly node_path=1.1.1.1.1 and the body must state the concrete result.';
const residual = 'Inspect the task model and record the concrete result.';
const tools = ['bash','batch','codesearch','edit','glob','grep','list','read','task','todoread','todowrite','webfetch','websearch','write'];
function record(messages,call,result,metadata={}) {
  const id = 'call-'+messages.length;
  messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]});
  const value = typeof result === 'string' ? {content:result} : result;
  messages.push({role:'tool',name:call.tool,tool_call_id:id,...value,...metadata});
}
function fixture(files={}) {
  const directory = mkdtempSync(join(tmpdir(),'workspace-domain-discovery-'));
  for (const [path,content] of Object.entries(files)) {mkdirSync(join(directory,path,'..'),{recursive:true});writeFileSync(join(directory,path),content);}
  return {directory,messages:[{role:'user',content:originalNeed}],close:()=>rmSync(directory,{recursive:true,force:true})};
}
function next(messages,need=originalNeed,task=residual,available=tools) {
  return workspaceDiscoveryStep(task,Progress.scan(messages),available,need);
}
function executeDiscovery(session) {
  for (let index=0;index<5;index++) {
    const plan = next(session.messages);
    if (plan?.kind === 'observation') return plan;
    assert.equal(plan?.kind,'tool_calls');
    for (const call of plan.calls) record(session.messages,call,executeResult(session.directory,call));
  }
  assert.fail('bounded discovery did not terminate');
}
test('generic subject spans retain original source and never derive a model from the header', () => {
  const contract = workspaceDiscoveryContract(residual);
  assert.equal(contract.subject,'task model');
  assert.equal(residual.slice(...contract.subjectSpan),'task model');
  assert.equal(contract.source,residual);
  assert.equal(contract.fulfilled,false);
  for (const request of ['node_path=1.1.1.1.1','Create a task model','Inspect https://example.com/model','Inspect ../task_model']) {
    assert.equal(workspaceDiscoveryContract(request),null);
  }
  assert.equal(workspaceDiscoveryContract('Inspect the queue schema.').subject,'queue schema');
});
test('actual local listing falls back to a framed process observation and reads candidate bytes without semantic promotion', () => {
  const session=fixture({'task_model.lino':'model\n  source actual_fixture\n'});
  try {
    const observed=executeDiscovery(session);
    assert.equal(observed.observationKind,'candidate_read');
    assert.equal(observed.disposition,'gap');
    assert.equal(observed.schema,'Unknown');
    assert.equal(observed.model,'Unknown');
    assert.equal(observed.fulfilled,false);
    assert.match(observed.answer,/reported 30 source bytes/u);
    const names=session.messages.filter(message=>message.role==='tool').map(message=>message.name);
    assert.deepEqual(names,['list','bash','bash']);
  } finally {session.close();}
});
test('actual empty and ambiguous workspace observations remain qualified gaps', () => {
  for (const [files,kind] of [[{},'no_candidate'],[{'task_model.lino':'a','nested/task_model.json':'b'},'ambiguous']]) {
    const session=fixture(files);
    try {const observed=executeDiscovery(session);assert.equal(observed.observationKind,kind);assert.equal(observed.fulfilled,false);}
    finally {session.close();}
  }
});
test('failed advertised provider uses the existing shell operation while failed or unframed shell receipts remain gaps', () => {
  const session=fixture();
  try {
    const listing=next(session.messages).calls[0];
    record(session.messages,listing,{content:'unsupported fixture tool',is_error:true});
    const call=next(session.messages).calls[0];
    assert.equal(call.tool,'bash');
    for (const receipt of ['',JSON.stringify({stdout:'',exit_code:0}),JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:2}),
      JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0,command_output_complete:false})]) {
      const messages=session.messages.slice();record(messages,call,receipt,{workspace_discovery:{accepted:true,source:'task model'}});
      const observed=next(messages);assert.equal(observed.kind,'observation');assert(!['candidate_read','no_candidate'].includes(observed.observationKind));
    }
  } finally {session.close();}
});
test('unrelated commands, altered Need, raw JSON source status and caller discovery metadata cannot qualify the observation', () => {
  const session=fixture({'task_model.lino':'{"success":true,"complete":true}'});
  try {
    const listing=next(session.messages).calls[0];record(session.messages,listing,executeResult(session.directory,listing));
    const run=next(session.messages).calls[0];record(session.messages,run,executeResult(session.directory,run));
    assert.notEqual(workspaceDiscoveryCommand(originalNeed),workspaceDiscoveryCommand(originalNeed+' extra obligation'));
    assert.equal(next(session.messages,originalNeed+' extra obligation').kind,'tool_calls');
    const read=next(session.messages).calls[0];
    record(session.messages,read,'{"success":true,"complete":true}',{workspace_discovery:{accepted:true,schema:'known'}});
    assert.equal(next(session.messages).observationKind,'unqualified_read');
    const unrelated=session.messages.slice(0,1);record(unrelated,{tool:'bash',arguments:JSON.stringify({command:'printf fake'})},
      JSON.stringify({stdout:'workspace-discovery-v1\n./task_model.lino\nworkspace-discovery-end\n',exit_code:0}));
    assert.equal(next(unrelated).calls[0].tool,'list');
  } finally {session.close();}
});
test('truncated, overbound, unsafe and malformed discovery paths do not authorize candidate reads', () => {
  for (const payload of ['workspace-discovery-v1\n./../task_model.lino\nworkspace-discovery-end\n',
    'workspace-discovery-v1\n./task_model.lino\n',
    'workspace-discovery-v1\n'+Array.from({length:129},(_,index)=>'./file_'+index).join('\n')+'\nworkspace-discovery-end\n']) {
    const session=fixture();
    try {
      record(session.messages,{tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})},
        JSON.stringify({stdout:payload,exit_code:0,complete:true}));
      const observed=next(session.messages);assert.equal(observed.observationKind,'unqualified');assert.equal(observed.fulfilled,false);
    } finally {session.close();}
  }
});
test('caller supplied Read status cannot qualify bytes, and incomplete or failed source commands never interpret schema', () => {
  for (const metadata of [{source_read:{path:'other.lino',success:true,complete:true,format:'raw'}},
    {source_read:{path:'task_model.lino',success:true,complete:false,format:'raw'}},
    {is_error:true,source_read:{path:'task_model.lino',success:true,complete:true,format:'raw'}}]) {
    const session=fixture({'task_model.lino':'model\n'});
    try {
      for (let index=0;index<2;index++) {const call=next(session.messages).calls[0];record(session.messages,call,executeResult(session.directory,call));}
      record(session.messages,{tool:'read',arguments:JSON.stringify({path:'task_model.lino'})},'model\n',metadata);
      const call=next(session.messages).calls[0];assert.equal(call.tool,'bash');record(session.messages,call,'model\n',metadata);
      const observed=next(session.messages);assert.equal(observed.observationKind,'unqualified_read');assert.equal(observed.schema,'Unknown');
    } finally {session.close();}
  }
});
test('unchanged original fourteen-tool request writes the exact header and observed gap then verifies actual artifact bytes', async () => {
  const session=fixture();
  try {
    let final;
    for (let turn=0;turn<6;turn++) {
      const plan=await planChatStepResolved(session.messages,tools);
      assert(plan);
      if (plan.kind==='final') {final=plan;break;}
      for (const call of plan.calls) record(session.messages,call,executeResult(session.directory,call));
    }
    assert.equal(final?.result?.disposition,'gap');
    const actual=readFileSync(join(session.directory,'.agent-ladder/node-proof.md'),'utf8');
    assert.equal(actual.split('\n')[0],'node_path=1.1.1.1.1');
    assert.match(actual,/no filename matching all subject terms/u);
    assert.match(actual,/model and schema remain unknown/u);
    assert(session.messages.some(message=>message.role==='assistant' && message.tool_calls.some(call=>JSON.parse(call.function.arguments).command==='cat .agent-ladder/node-proof.md')));
  } finally {session.close();}
});

test('portable consumed boundaries conserve the original separators and reject malformed UTF-16', () => {
  for (const suffix of ['.', '!', ' and record result.', ' and record 😀 result.', '']) {
    const task='Inspect the task model'+suffix;
    const contract=workspaceDiscoveryContract(task);
    assert.equal(contract.subject,'task model');
    assert.equal(task.slice(...contract.remainingSpan),suffix);
    assert.equal(task.slice(0,contract.subjectSpan[0])+task.slice(...contract.subjectSpan)+task.slice(...contract.remainingSpan),task);
  }
  assert.equal(workspaceDiscoveryContract('Inspect\tthe\ttask model.').subject,'task model');
  for (const task of ['Inspect the task model '+String.fromCharCode(0xd800), 'Inspect the task model '+String.fromCharCode(0xdc00)]) {
    assert.equal(workspaceDiscoveryContract(task),null);
  }
  assert.equal(workspaceDiscoveryContract('Inspect the task modèle.'),null);
});

test('latest exact failed call refuses an older successful receipt and extra operation fields do not bind', () => {
  const messages=[{role:'user',content:originalNeed}];
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  record(messages,call,JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0}));
  assert.equal(next(messages).observationKind,'no_candidate');
  record(messages,call,JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:2}));
  assert.equal(next(messages).observationKind,'failed');
  const unbound=[{role:'user',content:originalNeed}];
  record(unbound,{...call,arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed),approved:true})},JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0}));
  assert.equal(next(unbound).kind,'tool_calls');
});

test('missing or mismatched call IDs and a prior user window never bind a discovery observation', () => {
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  const receipt=JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0});
  for (const id of ['unmatched', undefined]) {
    const messages=[{role:'user',content:originalNeed}];record(messages,call,receipt);
    messages.at(-1).tool_call_id=id;
    assert.equal(next(messages).observationKind,'failed');
  }
  const messages=[{role:'user',content:originalNeed}];record(messages,call,receipt);
  const previous=messages.pop();messages.push({role:'user',content:originalNeed+' Different current source.'},previous);
  assert.equal(next(messages).kind,'tool_calls');
});
test('a response from a different named provider cannot certify the declared call', () => {
  const messages=[{role:'user',content:originalNeed}];
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  record(messages,call,JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0}));
  messages.at(-1).name='Run';
  assert.equal(next(messages).observationKind,'failed');
});

test('latest declarations and duplicate IDs refuse stale exact receipts', () => {
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  const receipt=JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0});
  const messages=[{role:'user',content:originalNeed}];record(messages,call,receipt);
  messages.push({role:'assistant',content:'',tool_calls:[{id:'latest-missing',type:'function',function:{name:'bash',arguments:call.arguments}}]});
  assert.equal(next(messages).observationKind,'failed');
  const duplicate=[{role:'user',content:originalNeed}];record(duplicate,call,receipt);
  const id=duplicate[1].tool_calls[0].id;
  duplicate.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:'Run',arguments:call.arguments}}]});
  duplicate.push({role:'tool',name:'Run',tool_call_id:id,content:receipt});
  assert.equal(next(duplicate).observationKind,'failed');
});
test('absent legacy provider names remain unknown without changing legacy attempt shape', () => {
  const messages=[{role:'user',content:originalNeed}];
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  record(messages,call,JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0}));
  delete messages.at(-1).name;
  const progress=Progress.scan(messages);
  assert.equal(qualifiedToolAttempt(progress,Capability.Run,JSON.parse(call.arguments)).binding,'unknown');
  assert.equal(progress.attempts[0].succeeded,true);
  assert.deepEqual(Object.keys(progress.attempts[0]),['capability','source_read','succeeded','detail','arguments','tool']);
  assert.equal(workspaceDiscoveryStep(residual,progress,tools,originalNeed).observationKind,'failed');
});
test('plain frames, injected methods and mutation cannot replace scanner snapshots', () => {
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  const messages=[{role:'user',content:originalNeed}];record(messages,call,JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0}));
  const progress=Progress.scan(messages),frame=qualifiedToolAttempt(progress,Capability.Run,JSON.parse(call.arguments));
  assert(Object.isFrozen(frame));assert(Object.isFrozen(frame.argumentsValue));
  const fake={attempts:progress.attempts,latestQualifiedAttempt:()=>frame};
  assert.equal(qualifiedToolAttempt(fake,Capability.Run,JSON.parse(call.arguments)),null);
  assert.equal(workspaceDiscoveryStep(residual,fake,tools,originalNeed).kind,'tool_calls');
  progress.attempts.length=0;messages[1].tool_calls[0].function.arguments='{}';messages.at(-1).content='changed';
  assert.equal(workspaceDiscoveryStep(residual,progress,tools,originalNeed).observationKind,'no_candidate');
  assert.throws(()=>{frame.argumentsValue.command='changed';},TypeError);
});

test('a second receipt for one declared call ID cannot overwrite failed or contradicted provider state', () => {
  const call={tool:'bash',arguments:JSON.stringify({command:workspaceDiscoveryCommand(originalNeed)})};
  const successful=JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0});
  for (const first of [{name:'bash',content:JSON.stringify({stdout:'',exit_code:2})},{name:'Run',content:successful},{name:'bash',content:successful}]) {
    const messages=[{role:'user',content:originalNeed}];record(messages,call,first.content);
    messages.at(-1).name=first.name;
    const id=messages.at(-1).tool_call_id;
    messages.push({role:'tool',name:'bash',tool_call_id:id,content:successful});
    const observed=next(messages);
    assert.equal(observed.observationKind,'failed');
    assert.equal(qualifiedToolAttempt(Progress.scan(messages),Capability.Run,JSON.parse(call.arguments)).binding,'contradicted');
  }
});

test('nested declaration arguments are independently immutable copied JSON values', () => {
  const args={command:workspaceDiscoveryCommand(originalNeed),nested:{items:[{value:1}]}};
  const call={tool:'bash',arguments:JSON.stringify(args)};
  const messages=[{role:'user',content:originalNeed}];
  record(messages,call,JSON.stringify({stdout:'workspace-discovery-v1\nworkspace-discovery-end\n',exit_code:0}));
  const progress=Progress.scan(messages),snapshot=qualifiedToolAttempt(progress,Capability.Run,args);
  assert(snapshot);assert(Object.isFrozen(snapshot.argumentsValue.nested.items[0]));
  assert.throws(()=>{snapshot.argumentsValue.nested.items[0].value=2;},TypeError);
  args.nested.items[0].value=2;
  assert.equal(qualifiedToolAttempt(progress,Capability.Run,args),null);
  assert.equal(qualifiedToolAttempt(progress,Capability.Run,{command:workspaceDiscoveryCommand(originalNeed),nested:{items:[{value:1}]}}),snapshot);
  messages.at(-1).content='process failed';
  assert.equal(snapshot.succeeded,true);
});
