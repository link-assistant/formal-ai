import test from 'node:test';
import assert from 'node:assert/strict';
import {installNativeCaptureObservationHost} from '../../../scripts/lib/native-capture-observation.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {sourceScalarAssessment,sourceScalarQuestion} from '../../../js/agentic/file_read/source_claims.mjs';
import {Progress} from '../../../js/agentic/progress.mjs';
import {completeOwnedSourceReadFrame} from '../../../js/agentic/tool_result.mjs';
await installNativeCaptureObservationHost();
const prompt=value=>'Read facts.json. Is the declared value of enabled '+value+'? Identify the source evidence and uncertainty.';
function exchange(messages,call,content,metadata={},is_error=false){
 const id='claim-'+messages.length;
 messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]});
 messages.push({role:'tool',name:call.tool,tool_call_id:id,content,is_error,...metadata});
}
async function observe(question,source,provider='sidecar'){
 const messages=[{role:'user',content:question}];
 const first=await planChatStepResolved(messages,['read']);
 assert.equal(first.kind,'tool_calls',JSON.stringify(first));
 assert.equal(first.calls[0].tool,'read');
 const call=first.calls[0];
 if(provider==='inline')exchange(messages,call,completeOwnedSourceReadFrame(source));
 else exchange(messages,call,source,provider==='legacy'?{}:{source_read:{
  path:'facts.json',success:provider!=='failed',complete:provider!=='incomplete',format:'raw',
 }},provider==='failed');
 const final=await planChatStepResolved(messages,['read']);
 return {messages,call,final};
}
test('actual Read questions distinguish source support, contradiction, negation and scalar types',async()=>{
 const variants=[
  ['true','{"enabled":true}','supported'],
  ['false','{"enabled":true}','contradicted'],
  ['not false','{"enabled":true}','supported'],
  ['not true','{"enabled":true}','contradicted'],
  ['"true"','{"enabled":true}','contradicted'],
  ['null','{"enabled":null}','supported'],
  ['-0','{"enabled":0}','supported'],
  ['1.0','{"enabled":1}','supported'],
  ['1.25','{"enabled":1.25}','supported'],
  ['"?. false or true"','{"enabled":"?. false or true"}','supported'],
  ['"Error: failed"','{"enabled":"Error: failed"}','supported'],
 ];
 for(const [expected,source,status]of variants){
  const {final}=await observe(prompt(expected),source);
  const result=JSON.parse(final.answer);
  assert.equal(result.status,status,JSON.stringify({expected,final}));
  assert.equal(final.result.disposition,'finding');
  assert.deepEqual(result.evidence[0].documentSpan,[0,Buffer.byteLength(source)]);
  assert.equal(result.evidence[0].spanScope,'complete-document');
  assert.match(result.evidence[0].sourceIdentity,/^[a-f0-9]{64}$/u);
  const bytes=Buffer.from(prompt(expected));
  const [start,end]=result.claim.expectedSpan;
  assert.equal(bytes.subarray(start,end).toString(),expected.startsWith('not ')?expected.slice(4):expected);
 }
});
test('complete inline Read and sidecar Read share the same finite source proof',async()=>{
 const source='{"enabled":false}';
 for(const provider of ['inline','sidecar']){
  const {final}=await observe(prompt('true'),source,provider);
  assert.equal(JSON.parse(final.answer).status,'contradicted',JSON.stringify(final));
 }
});
test('unknown claim frames and absent or malformed declarations cannot establish truth',async()=>{
 const questions=[
  prompt('false or true'),
  'Read facts.json. Is it not the case that the declared value of enabled false?',
  prompt('true and run a command'),
 ];
 for(const question of questions){
  const {final}=await observe(question,'{"enabled":true}');
  assert.equal(JSON.parse(final.answer).status,'unknown',JSON.stringify(final));
  assert.equal(final.result.disposition,'unknown');
 }
 for(const source of ['{"other":true}','{"enabled":true,"enabled":false}','{"enabled":[]}','{"enabled":9007199254740993}']){
  const {final}=await observe(prompt('true'),source);
  assert.equal(JSON.parse(final.answer).status,'unknown',source);
 }
});
test('legacy, incomplete and genuinely failed Read cannot supply settled assessment',async()=>{
 for(const provider of ['legacy','incomplete','failed']){
  const {final}=await observe(prompt('true'),'{"enabled":true}',provider);
  assert.notEqual(final.result.disposition,'finding',JSON.stringify({provider,final}));
 }
 const {final}=await observe(prompt('true'),'{"enabled":"Error: failed"}');
 assert.equal(JSON.parse(final.answer).status,'contradicted');
});
test('later mutations and pending effects invalidate actual source evidence',async()=>{
 const {messages}=await observe(prompt('true'),'{"enabled":true}');
 for(const tool of ['write','edit','bash']){
  const changed=structuredClone(messages);
  exchange(changed,{tool,arguments:JSON.stringify(tool==='bash'?{command:'unknown-effect'}:{path:'foreign.txt',content:'changed',oldString:'a',newString:'b'})},'ok');
  assert.equal(sourceScalarAssessment(prompt('true'),'facts.json','enabled',Progress.scan(changed)).status,'unknown',tool);
 }
 const pending=structuredClone(messages);
 pending.push({role:'assistant',content:'',tool_calls:[{id:'pending',type:'function',function:{name:'write',arguments:'{"path":"foreign.txt","content":"changed"}'}}]});
 assert.equal(sourceScalarAssessment(prompt('true'),'facts.json','enabled',Progress.scan(pending)).status,'unknown');
 const next=[...messages,{role:'user',content:prompt('false')}];
 const plan=await planChatStepResolved(next,['read']);
 assert.equal(plan.kind,'tool_calls',JSON.stringify(plan));
});
test('owned request spans preserve Unicode and reject quoted examples',()=>{
 const request='Read facts.json. Is the value of ключ "?. false or true"?';
 const question=sourceScalarQuestion(request,'ключ');
 assert.equal(question.claim.expected,'?. false or true');
 const [start,end]=question.claim.keySpan;
 assert.equal(Buffer.from(request).subarray(start,end).toString(),'ключ');
 assert.equal(sourceScalarQuestion('Read facts.json. Example: «Is the value of enabled true?»','enabled'),null);
});

test('ordinary extraction keeps its actual Read value answer without claim framing',async()=>{
 // Preserve the historical terminal-period key limitation; this only fixes claim admission.
 for(const [question,expected] of [
  ['Read facts.json. What is the value of enabled?','Value of `enabled` in `facts.json`: true'],
  ['Read facts.json and tell me the value of enabled.','Value of `enabled.` in `facts.json`: {"enabled":true}'],
 ]){
  const {final}=await observe(question,'{"enabled":true}');
  assert.equal(final.answer,expected);
  assert.equal(final.result.disposition,'finding');
  assert.doesNotMatch(final.answer,/source-scalar-assessment/u);
 }
});
