import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {createWorkerContext,evaluate} from './support/browser-runtime.mjs';
const worker=createWorkerContext(); await evaluate(worker,'loadSeed()');
const fixtures=JSON.parse(readFileSync(new URL('../../../data/parity/issue-1588-intent-spans.json',import.meta.url),'utf8'));
for(const row of fixtures) test(row.prompt,{timeout:15000},async()=>{
 const result=await worker.solve(row.prompt,[],{},{},[],{});
 if(row.kind==='document') {
  assert.notEqual(result.intent,'unit_conversion',JSON.stringify(result));
  assert.notEqual(result.intent,'calendar_create_event',JSON.stringify(result));
 } else if(row.kind==='event') {
  assert.equal(result.intent,'calendar_create_event',JSON.stringify(result));
  assert.ok(result.content.includes('BEGIN:VEVENT'),result.content);
  assert.ok(result.evidence.some(item=>item.startsWith('calendar:intent_span:')),JSON.stringify(result));
 } else {
  assert.equal(result.intent,row.kind==='date'?'calendar_date_offset':'unit_conversion',JSON.stringify(result));
  assert.equal(result.content,row.answer);
  if(row.text) assert.ok(result.content.includes(row.text),result.content);
  if(row.result) assert.ok(result.evidence.includes('calendar:result_date:'+row.result),JSON.stringify(result));
 }
});
test('quantities cannot borrow a number across words',()=>{
 assert.equal(evaluate(worker,'tryUnitConversion("Convert 2 years of experience, hours to days", "convert 2 years of experience hours to days", "en")'),null);
});

test('calendar day numbers stop at intervening words',()=>{
 assert.equal(evaluate(worker,'extractDayNumber("experience of 2 years full day")'),null);
 assert.equal(evaluate(worker,'extractDayNumber("2 years full day")'),null);
 assert.equal(evaluate(worker,'extractDayNumber("schedule on 12 day")'),12);
});

test('shared fixtures cover English, Russian, Hindi, Chinese and Spanish',()=>{
 for (const language of ['en','ru','hi','zh','es']) {
  assert.ok(fixtures.some(row=>row.language===language),language);
 }
});


test('original API requests preserve document intent and exact requested answers', {timeout:30000}, async()=>{
 const {mkdtempSync,rmSync}=await import('node:fs');
 const {tmpdir}=await import('node:os');
 const {join}=await import('node:path');
 const {startServer}=await import('../../../js/server/main.mjs');
 const home=mkdtempSync(join(tmpdir(),'formal-ai-issue1588-'));
 const env={...process.env,HOME:home,FORMAL_AI_API_BEARER_TOKEN:'issue1588',
  FORMAL_AI_MEMORY_PATH:join(home,'memory.lino'),FORMAL_AI_CACHE_DIR:join(home,'cache'),
  FORMAL_AI_DIALOG_LOG_DIR:join(home,'dialogs'),FORMAL_AI_RECORD_CHAT:'0',
  FORMAL_AI_LIVE_FETCH:'0',FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR:'false'};
 let running;
 try {
  running=await startServer({host:'127.0.0.1',port:0,env});
  for(const row of fixtures.slice(0,6)) {
   const response=await fetch(running.url+'/v1/chat/completions',{
    method:'POST',headers:{authorization:'Bearer issue1588','content-type':'application/json'},
    body:JSON.stringify({model:'formal-ai',messages:[{role:'user',content:row.prompt}]}),
    signal:AbortSignal.timeout(10000)});
   assert.equal(response.status,200);
   const body=await response.json();
   const answer=body.choices[0].message.content;
   if(row.kind==='document') assert.doesNotMatch(answer,/BEGIN:VCALENDAR|1461\/2|1\/12/);
   else assert.equal(answer,row.answer);
  }
 } finally {
  if(running) await new Promise(resolve=>running.server.close(resolve));
  rmSync(home,{recursive:true,force:true});
 }
});

test('event fields come only from their scheduling request span',async()=>{
 const answer=await worker.solve('Background: 12 day shifts. Create an event tomorrow.',[],{},{},[],{});
 assert.ok(answer.evidence.includes('calendar:intent_span:create an event tomorrow'));
 assert.equal(evaluate(worker,'calendarCreateRequestSpan("Create a poem about tomorrow.")'),null);
 assert.equal(evaluate(worker,'calendarCreateRequestSpan("Schedule a meeting. The document mentions tomorrow.")'),null);
 assert.equal(evaluate(worker,'calendarCreateRequestSpan("Create an event tomorrow. Schedule a call tomorrow.")'),null);
});
