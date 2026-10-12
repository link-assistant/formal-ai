// Provider-owned source observations keep raw file bytes separate from status.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { SourceReadStatus, sourceReadObservation, observedBytesMatch, observedDigestMatches, failureMessage } from '../../../js/agentic/tool_result.mjs';
import { Progress } from '../../../js/agentic/progress.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const path = '.formal-ai/general-change-plan.lino';
const metadata = { path, success: true, complete: true, format: 'raw' };
const source = 'goal "Create a file status.txt containing exactly: failed"\n';
const jsonSource = JSON.stringify({ schema: 'source-read-receipt/v1', path, success: true, complete: true, content: 'not the file', error: 'failed', exit_code: 9 });
function history(content, extra = {}, args = {path}, callId = 'read-now', tool = 'read') {
  return [{role:'user',content:'Create a file status.txt containing exactly: failed'},
    {role:'assistant',tool_calls:[{id:'read-now',type:'function',function:{name:'read',arguments:JSON.stringify(args)}}]},
    {role:'tool',name:tool,tool_call_id:callId,content,...extra}];
}
for (const body of [source, jsonSource, JSON.stringify({success:false,complete:false,error:"failed",signal:"SIGTERM"}), JSON.stringify({success:true,complete:true,content:'fake',is_error:true})]) {
  test('raw authored source does not provide its own metadata: '+body.slice(0,35), () => {
    const observation = sourceReadObservation(body,false,null,path);
    assert.equal(observation.status,SourceReadStatus.Unknown);
    assert.equal(observation.complete,false);assert.equal(observation.source,body);assert.equal(observation.error,null);
    assert.equal(Progress.scan(history(body)).sourceReadFor(path).source,body);
    assert.equal(Progress.scan(history(body)).attempts[0].succeeded,true);
  });
  test('outer provider success retains exact source bytes: '+body.slice(0,35), () => {
    const observation=sourceReadObservation(body,false,metadata,path);
    assert.equal(observation.status,SourceReadStatus.Success);assert.equal(observation.complete,true);
    assert.equal(observation.source,body);assert.equal(observation.error,null);
    assert.equal(Progress.scan(history(body,{source_read:metadata})).sourceReadFor(path).source,body);
  });
}
for (const patch of [{complete:false},{stream_complete:false},{truncated:true},{timed_out:true},{aborted:true},{signal:'SIGTERM'},{success:false},{is_error:true},{exit_code:1},{success:'true'},{complete:'true'},{format:'numbered'},{path:'elsewhere/'+path},{path:null}]) {
  test('provider refusal or partial observation cannot certify source '+JSON.stringify(patch), () => {
    assert.equal(sourceReadObservation(source,false,{...metadata,...patch},path).complete,false);
  });
}
test('outer tool error remains failure even when metadata declares success', () => {
  const observation=sourceReadObservation(source,true,metadata,path);
  assert.equal(observation.status,SourceReadStatus.Failure);assert.equal(observation.source,null);assert.equal(observation.complete,false);
  assert.equal(Progress.scan(history(source,{is_error:true,source_read:metadata})).attempts[0].succeeded,false);
});
test('known complete numbered format is separate from process status', () => {
  const framed='<file>\n1| failed\n2| authored\n\n(End of file - total 2 lines)\n</file>';
  const observation=sourceReadObservation(framed,false,null,path);
  assert.equal(observation.source,'failed\nauthored');assert.equal(observation.complete,true);assert.equal(observation.status,SourceReadStatus.Unknown);
  for(const body of [framed.replace('1|','3|'),framed.replace('total 2','total 3')])assert.equal(sourceReadObservation(body,false,null,path).complete,false);
  assert.equal(sourceReadObservation(framed,false,{...metadata,path:'wrong'},path).complete,false);
});
test('current call identity and exact nonconflicting path are required', () => {
  for(const [args,callId,tool] of [[{path},'unmatched','read'],[{path,filePath:'other'},'read-now','read'],[{path:'elsewhere/'+path},'read-now','read'],[{path},'read-now','write'],[{path},'read-now','read_file']]) {
    assert.equal(Progress.scan(history(source,{source_read:metadata},args,callId,tool)).sourceReadFor(path),null);
  }
  const old=history(source,{source_read:metadata});old.push({role:'user',content:'Read another file'},old[2]);
  assert.equal(Progress.scan(old).sourceReadFor(path),null);
  const late=history(source,{source_read:metadata});late.push({...late[2],is_error:true});
  assert.equal(Progress.scan(late).sourceReadFor(path).status,SourceReadStatus.Failure);
});
test('explicit successful process receipt retains authored failure vocabulary', () => {
  assert.equal(failureMessage(JSON.stringify({exit_code:0,stdout:'failed'}),false,true),null);
  assert.notEqual(failureMessage(JSON.stringify({exit_code:1,stdout:'failed'}),false,true),null);
  assert.notEqual(failureMessage(JSON.stringify({exit_code:0,stdout:'failed',is_error:true}),false,true),null);
});
for(const patch of [{complete:false},{stream_complete:false},{truncated:true},{timed_out:true},{aborted:true},{signal:'SIGTERM'}]) {
  test('partial shell receipts veto exact bytes and digest '+JSON.stringify(patch), () => {
    assert.equal(observedBytesMatch(JSON.stringify({exit_code:0,stdout:'failed',...patch}),'failed'),false);
    assert.equal(observedDigestMatches(JSON.stringify({exit_code:0,stdout:'abc  file',...patch}),'abc'),false);
  });
}
test('full successful shell bytes still require exact output and real status', () => {
  assert.equal(observedBytesMatch(JSON.stringify({exit_code:0,stdout:'failed',complete:true}),'failed'),true);
  assert.equal(observedBytesMatch(JSON.stringify({exit_code:0,stdout:'failed',complete:true}),'failed\n'),false);
  assert.equal(observedBytesMatch(JSON.stringify({exit_code:1,stdout:'failed',complete:true}),'failed'),false);
});
test('original authored failure goal proceeds beyond actual bare source read', async () => {
  const next=await planChatStep(history(source),['read','write','bash']);
  assert.ok(next && (next.kind !== 'final' || !next.answer.startsWith('The command failed:')));
  const failed=await planChatStep(history(source,{is_error:true}),['read','write','bash']);
  assert.ok(failed?.kind==='final' && failed.answer.includes('failed'));
});

test('a later declined Read observation cannot retain an older source certificate', () => {
  const messages=history(source,{source_read:metadata});
  messages.push({...messages[2],name:'read_file'});
  assert.equal(Progress.scan(messages).sourceReadFor(path),null);
});
