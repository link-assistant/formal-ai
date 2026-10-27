import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const forbiddenWasm={instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}};
const worker=()=>new WorkerHost({WebAssembly:forbiddenWasm});
const cases=[
 {language:'en',prompt:'I really need to poop, what should I do?',expected:'Use a toilet as soon as you can rather than holding it. If you cannot go or have pain, tell me what is happening. [NHS guidance](https://www.nhs.uk/conditions/constipation/)'},
 {language:'ru',prompt:'Хочу какать, что делать?',expected:'Идите в туалет при первой возможности, не терпите. Если не получается сходить или есть боль, уточните, что происходит. [Рекомендации NHS](https://www.nhs.uk/conditions/constipation/)'},
 {language:'hi',prompt:'मुझे शौच करना है',expected:'जितनी जल्दी हो सके शौचालय जाएँ, रोककर न रखें। यदि शौच नहीं हो रहा है या दर्द है, तो बताएँ कि क्या हो रहा है। [NHS की सलाह](https://www.nhs.uk/conditions/constipation/)'},
 {language:'zh',prompt:'我快要大便了，怎么办？',expected:'尽快去厕所，不要憋着。如果排不出来或疼痛，请说明具体情况。[NHS 指引](https://www.nhs.uk/conditions/constipation/)'},
 {language:'es',prompt:'Necesito ir al baño, ¿qué debo hacer?',expected:'Ve al baño en cuanto puedas y no te aguantes. Si no puedes evacuar o tienes dolor, explica qué ocurre. [Consejos del NHS](https://www.nhs.uk/conditions/constipation/)'}
];
const source='https://www.nhs.uk/conditions/constipation/';
for(const {language,prompt,expected} of cases)test('real solver gives source-backed practical guidance in '+language,async()=>{
 const host=worker(),result=await host.solve(prompt,[],{forcedResponseLanguage:language});
 assert.equal(result.intent,'procedural_how_to');assert.equal(result.content,expected);assert(result.content.includes(source));
 assert(!/reciprocal rank|source_count|teach|unknown|зауч|Tió|Cheese/iu.test(result.content));
});
test('Spanish detection, follow-up and unavailable guidance use the same strategy',async()=>{
 const host=worker(),example=cases.find(row=>row.language==='es');
 const initial=await host.solve(example.prompt);assert.equal(initial.content,example.expected);
 const history=[{role:'user',content:example.prompt},{role:'assistant',content:initial.content}];
 const followup=await host.solve('¡Ayúdame!',history);assert.equal(followup.content,example.expected);
 await host.boot();host.context.fetch=async()=>new Response('unrelated text');
 const unavailable=await host.run('tryPracticalHelp(__prompt,"es",{practicalHelpCache:false})',{__prompt:example.prompt});
 assert.equal(unavailable.confidence,0);assert.equal(unavailable.content,'Suena urgente. ¿Necesitas encontrar un baño o no puedes evacuar?');
});
test('the urgent report and a short follow-up preserve the actual goal',async()=>{
 const host=worker();const urgent=await host.solve('Сейчас обкакаюсь! Помоги!');assert(urgent.content.includes('Идите в туалет'),urgent.content);
 const followup=await host.solve('Помоги!',[{role:'user',content:'Хочу какать, что делать?'},{role:'assistant',content:'Уточните, что происходит.'}]);
 assert(followup.content.includes('Идите в туалет'),followup.content);
});
test('negations, quotations and unrelated requests do not become personal needs',async()=>{
 const host=worker();
 for(const prompt of ['"I need to poop" is a quotation','I need to not poop','Мне нужно не какать','मुझे शौच नहीं करना है','我不想大便','Necesito no ir al baño','No necesito ir al baño','I need to write about poop','I want to understand the toilet history','I needed to poop yesterday'])
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
