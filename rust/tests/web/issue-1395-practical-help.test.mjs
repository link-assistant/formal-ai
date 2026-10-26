import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const forbiddenWasm={instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}};
const worker=()=>new WorkerHost({WebAssembly:forbiddenWasm});
const cases=[['en','I really need to poop, what should I do?','Use a toilet'],['ru','Хочу какать, что делать?','Идите в туалет'],['hi','मुझे शौच करना है','शौचालय'],['zh','我快要大便了，怎么办？','厕所']];
const source='https://www.nhs.uk/conditions/constipation/';
for(const [language,prompt,expected] of cases)test('real solver gives source-backed practical guidance in '+language,async()=>{
 const host=worker(),result=await host.solve(prompt,[],{forcedResponseLanguage:language});
 assert.equal(result.intent,'procedural_how_to');assert(result.content.includes(expected),result.content);assert(result.content.includes(source));
 assert(!/reciprocal rank|source_count|teach|unknown|зауч|Tió|Cheese/iu.test(result.content));
});
test('the urgent report and a short follow-up preserve the actual goal',async()=>{
 const host=worker();const urgent=await host.solve('Сейчас обкакаюсь! Помоги!');assert(urgent.content.includes('Идите в туалет'),urgent.content);
 const followup=await host.solve('Помоги!',[{role:'user',content:'Хочу какать, что делать?'},{role:'assistant',content:'Уточните, что происходит.'}]);
 assert(followup.content.includes('Идите в туалет'),followup.content);
});
test('negations, quotations and unrelated requests do not become personal needs',async()=>{
 const host=worker();
 for(const prompt of ['"I need to poop" is a quotation','I need to not poop','Мне нужно не какать','मुझे शौच नहीं करना है','我不想大便','I need to write about poop','I want to understand the toilet history','I needed to poop yesterday'])
  assert.equal(await host.run('practicalHelpGoal(__prompt)',{__prompt:prompt}),null,prompt);
 const history=[{role:'user',content:'I need to poop'},{role:'assistant',content:'Use a toilet.'},{role:'user',content:'Tell me about stars'},{role:'assistant',content:'Stars are astronomical objects.'}];
 assert.equal(await host.run('tryPracticalHelpFollowup("help me","en",__history)',{__history:history}),null);
});
async function cold(host,fetch,extra={}){
 await host.boot();Object.assign(host.context,{fetch,...extra});
 return host.run('tryPracticalHelp("I need to poop","en",{practicalHelpCache:false})');
}
test('an empty cache discovers the registered source and labels fetched evidence honestly',async()=>{
 const host=worker();await host.boot();const proof=await host.run('practicalHelpPolicy("toilet-proof")');let calls=0;
 const result=await cold(host,async(url,options)=>{calls++;assert.equal(url,source);assert(options.signal);return new Response('<p>'+proof+'</p>');});
 assert.equal(calls,1);assert(result.content.includes(source));assert(result.evidence.includes('practical_help:source_mode:fetched-source'));
 const cached=await host.run('tryPracticalHelp("I need to poop","en")');assert(cached.evidence.includes('practical_help:source_mode:curated-excerpt'));assert.equal(calls,1);
});
test('failed, irrelevant and oversized sources produce a useful question rather than unsupported advice',async()=>{
 for(const fetch of [async()=>{throw Error('offline');},async()=>new Response('Unrelated festival text'),async()=>new Response('x'.repeat(2097153)),async()=>new Response('雪'.repeat(800000)),async()=>new Response('',{status:503})]){
  const result=await cold(worker(),fetch);assert.equal(result.confidence,0);assert(result.evidence.includes('practical_help:source_mode:unavailable'));assert(!result.content.includes(source));assert(result.content.includes('?'),result.content);
 }
});
test('both fetch and body-read stalls have a deadline even when transport ignores abort',async()=>{
 for(const fetch of [()=>new Promise(()=>{}),async()=>({ok:true,body:{getReader:()=>({read:()=>new Promise(()=>{}),cancel:()=>Promise.resolve()})}})]){
  let calls=0;const result=await cold(worker(),(...args)=>{calls++;return fetch(...args);},{setTimeout:(fn,ms)=>setTimeout(fn,Math.min(ms,20))});
  assert.equal(calls,1);assert.equal(result.confidence,0);
 }
});
test('zero-byte stream chunks have a finite read budget and are cancelled',async()=>{
 let reads=0,cancelled=0;const result=await cold(worker(),async()=>({ok:true,body:{getReader:()=>({read:async()=>{reads++;return{done:false,value:new Uint8Array()};},cancel:async()=>{cancelled++;}})}}));
 assert.equal(reads,1024);assert.equal(cancelled,1);assert.equal(result.confidence,0);
});
test('the same strategy handles a new data-defined goal without a prompt-specific code branch',async()=>{
 const host=worker();await host.boot();await host.run('globalThis.originalPracticalPolicy=practicalHelpPolicy;MULTILINGUAL_ANSWERS["practical-help-test-rest"]={en:"Take a rest. [{source}]({source})"};practicalHelpPolicy=key=>Object.hasOwn(__profile,key)?__profile[key]:originalPracticalPolicy(key);true',{__profile:{goals:'rest',leads:'i need to','rest-forms':'rest','rest-source':source,'rest-proof':'A verified rest instruction.','rest-cached-proof':'A verified rest instruction.','rest-response':'practical-help-test-rest','rest-unavailable':'practical-help-toilet-unavailable'}});
 const result=await host.run('tryPracticalHelp("I need to rest","en")');assert(result.evidence.includes('practical_help:goal:rest'));assert.equal(result.confidence,0.8);assert(result.content.startsWith('Take a rest.'));
});

test('an empty matcher operand terminates within the same finite contract as Rust',async()=>{
 const host=worker(),context=await host.boot();
 assert.equal(vm.runInContext('practicalHelpContains("letters", "")',context,{timeout:1000}),false);
});
