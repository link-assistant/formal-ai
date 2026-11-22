import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const forbiddenWasm={instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}};
const source = 'https://www.nhs.uk/conditions/constipation/';
const document = readFileSync(new URL('../fixtures/practical-help/toilet.html', import.meta.url), 'utf8');
const guidance = 'Do not delay if you feel the urge to poo.';
const worker = (networkFetch = null) => new WorkerHost({WebAssembly: forbiddenWasm}, {networkFetch});
const sourcedWorker = () => worker(async url => {
  assert.equal(url, source);
  return new Response(document);
});
const cases = [
  {language: 'en', prompt: 'I really need to poop, what should I do?', label: 'Source guidance'},
  {language: 'ru', prompt: 'Хочу какать, что делать?', label: 'Рекомендации источника'},
  {language: 'hi', prompt: 'मुझे शौच करना है', label: 'स्रोत के निर्देश'},
  {language: 'zh', prompt: '我快要大便了，怎么办？', label: '来源指引'},
  {language: 'es', prompt: 'Necesito ir al baño, ¿qué debo hacer?', label: 'Consejos de la fuente'},
];
for (const {language, prompt, label} of cases) test('real solver gives source-backed practical guidance in ' + language, async () => {
  const result = await sourcedWorker().solve(prompt, [], {forcedResponseLanguage: language});
  assert.equal(result.intent, 'procedural_how_to');
  assert.ok(result.content.includes(`[${label}](${source})`), result.content);
  assert.ok(result.content.includes(guidance), result.content);
  assert.ok(!/reciprocal rank|source_count|teach|unknown|зауч|Tió|Cheese/iu.test(result.content));
});
test('Spanish detection and follow-up preserve the captured guidance', async () => {
  const host = sourcedWorker();
  const example = cases.find(row => row.language === 'es');
  const initial = await host.solve(example.prompt);
  assert.ok(initial.content.includes(example.label), initial.content);
  const history = [{role: 'user', content: example.prompt}, {role: 'assistant', content: initial.content}];
  const followup = await host.solve('¡Ayúdame!', history);
  assert.equal(followup.content, initial.content);
});
const unavailableCases=[
 {
  "language": "en",
  "prompt": "I need to poop",
  "expected": "It sounds urgent. Do you want help finding a toilet, or are you unable to go?"
 },
 {
  "language": "ru",
  "prompt": "Хочу какать, что делать?",
  "expected": "Похоже, это срочно. Хотите, чтобы я помог найти туалет, или у вас не получается сходить?"
 },
 {
  "language": "hi",
  "prompt": "मुझे शौच करना है",
  "expected": "यह ज़रूरी लग रहा है। क्या आप शौचालय ढूँढने में मदद चाहते हैं, या शौच नहीं हो पा रहा है?"
 },
 {
  "language": "zh",
  "prompt": "我快要大便了，怎么办？",
  "expected": "听起来很着急。是否需要帮你找厕所，还是你排不出来？"
 },
 {
  "language": "es",
  "prompt": "Necesito ir al baño, ¿qué debo hacer?",
  "expected": "Suena urgente. ¿Quieres ayuda para encontrar un baño o no puedes evacuar?"
 }
];
for(const {language,prompt,expected} of unavailableCases)test('unavailable guidance asks for the user choice in '+language,async()=>{
 const host=worker();await host.boot();host.context.fetch=async()=>assert.fail('offline practical help reached the transport');
 const result=await host.run('solve(__prompt, [], {greetingVariations:false, offline:true}, {}, [], {forcedResponseLanguage:__language})',{__prompt:prompt,__language:language});
 assert.equal(result.intent,'procedural_how_to');assert.equal(result.confidence,0);assert.equal(result.content,expected);
});
test('the urgent report and a short follow-up preserve the actual goal',async()=>{
 const host=sourcedWorker();const urgent=await host.solve('Сейчас обкакаюсь! Помоги!');assert(urgent.content.includes(guidance),urgent.content);
 const followup=await host.solve('Помоги!',[{role:'user',content:'Хочу какать, что делать?'},{role:'assistant',content:'Уточните, что происходит.'}]);
 assert(followup.content.includes(guidance),followup.content);
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
 return host.run('tryPracticalHelp("I need to poop","en")');
}
test('an empty cache requires retrieval and records verified provenance for reuse', async () => {
  const host = worker();
  await host.boot();
  host.context.fetch = async () => assert.fail('offline practical help reached the transport');
  const missed = await host.run('tryPracticalHelp("I need to poop", "en", {offline: true})');
  assert.equal(missed.confidence, 0);
  assert.ok(!missed.content.includes(source));

  let calls = 0;
  const result = await cold(host, async (url, options) => {
    calls++;
    assert.equal(url, source);
    assert.ok(options.signal);
    return new Response(document);
  });
  const digest = createHash('sha256').update(document).digest('hex');
  assert.equal(calls, 1);
  assert.ok(result.content.includes(source));
  assert.ok(result.evidence.includes('practical_help:source_mode:fetched-source'));
  assert.ok(result.solverEvents.some(event => event.kind === 'source:http'
    && event.payload.includes(`sha256=${digest} cached=false`)));

  const cached = await host.run('tryPracticalHelp("I need to poop", "en", {offline: true})');
  assert.ok(cached.evidence.includes('practical_help:source_mode:cached-source'));
  assert.ok(cached.solverEvents.some(event => event.kind === 'source:http'
    && event.payload.includes(`sha256=${digest} cached=true`)));
  assert.equal(cached.content, result.content);
  assert.equal(calls, 1);
});

test('an available capture component can supply the same source when a browser cannot read it directly', async () => {
  const host = worker();
  let requests = 0;
  const result = await cold(host, async () => { throw new TypeError('Failed to fetch'); }, {
    FormalAIWebSearchComponent: {
      package: 'test-capture',
      async fetchUrl(url) {
        assert.equal(url, source);
        requests++;
        return new Response(document);
      },
    },
  });
  const digest = createHash('sha256').update(document).digest('hex');
  assert.ok(result.content.includes(guidance), result.content);
  assert.ok(result.solverEvents.some(event => event.kind === 'source:http'
    && event.payload.includes(`${source} fetched_at=`) && event.payload.includes(`sha256=${digest}`)));
  const cached = await host.run('tryPracticalHelp("I need to poop", "en", {offline: true})');
  assert.ok(cached.evidence.includes('practical_help:source_mode:cached-source'));
  assert.equal(cached.content, result.content);
  assert.equal(requests, 1);
});

test('failed, irrelevant, oversized and invalid UTF-8 sources elicit a clarification', async () => {
  const transports = [
    async () => { throw new Error('offline'); },
    async () => new Response('Unrelated festival text'),
    async () => new Response(`<!-- <h3>Improve your toilet routine</h3><p>${guidance}</p> --><h3>Unrelated section</h3><p>${guidance}</p>`),
    async () => new Response(`<h3>Improve your toilet routine</h3><p>Regular times are helpful.</p><h3>Another section</h3><p>${guidance}</p>`),
    async () => new Response('x'.repeat(2097153)),
    async () => new Response('雪'.repeat(800000)),
    async () => new Response('', { status: 503 }),
    async () => new Response(Buffer.concat([Buffer.from(document), Buffer.from([0xff])])),
  ];
  for (const fetch of transports) {
    const result = await cold(worker(), fetch);
    assert.equal(result.confidence, 0);
    assert.ok(result.evidence.includes('practical_help:source_mode:unavailable'));
    assert.ok(!result.content.includes(source));
    assert.ok(result.content.includes('?'), result.content);
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
 assert.equal(reads,17);assert.equal(cancelled,1);assert.equal(result.confidence,0);
});
test('the same strategy handles a new data-defined goal without a prompt-specific code branch', async () => {
  const excerpt = 'Plan a short rest before returning to the task.';
  const host = worker(async () => new Response(`<h3>Taking breaks</h3><p>${excerpt}</p>`));
  await host.run(`globalThis.originalPracticalPolicy = practicalHelpPolicy;
    practicalHelpPolicy = key => Object.hasOwn(__profile, key) ? __profile[key] : originalPracticalPolicy(key);
    true;`, {
    __profile: {
      goals: 'rest', leads: 'i need to', 'rest-forms': 'rest', 'rest-source': source,
      'rest-heading': 'Taking breaks', 'rest-excerpt-terms': 'rest',
      'rest-unavailable': 'practical-help-toilet-unavailable',
    },
  });
  const result = await host.run('tryPracticalHelp("I need to rest", "en")');
  assert.ok(result.evidence.includes('practical_help:goal:rest'));
  assert.equal(result.confidence, 0.8);
  assert.ok(result.content.includes(excerpt), result.content);
});

test('an empty matcher operand terminates within the same finite contract as Rust',async()=>{
 const host=worker(),context=await host.boot();
 assert.equal(vm.runInContext('practicalHelpContains("letters", "")',context,{timeout:1000}),false);
});

test('each practical answer quotes the guidance retrieved from the matching section', async () => {
  for (const advice of [
    'When you feel the urge, allow yourself time to use a toilet.',
    'Use the closest available toilet when the urge returns.',
  ]) {
    const host = worker();
    await host.boot();
    let requests = 0;
    host.context.fetch = async url => {
      assert.equal(url, source);
      requests++;
      return new Response(`<main><h3>Improve your toilet routine</h3><p>${advice}</p></main>`);
    };
    const result = await host.solve('I need to poop');
    assert.equal(requests, 1);
    assert.ok(result.content.includes(advice), result.content);
    assert.ok(result.content.includes(source), result.content);
  }
});
