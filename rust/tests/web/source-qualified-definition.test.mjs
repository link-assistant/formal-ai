// The unchanged native citation pin plus genuine offline capture, subject,
// disambiguation and provenance contracts over the production worker hooks.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { createHash } from 'node:crypto';
import { WorkerHost, REPO_ROOT } from '../../../js/server/worker-host.mjs';
import { specificationCases, checkFailure } from '../../../scripts/lib/rust-specification-cases.mjs';
import { installSourceDefinitionCaptures, sourceDefinitionCaptures } from './support/source-definition-captures.mjs';
const original = 'Cite a definition of associative memory from Wikipedia';
const url = 'https://en.wikipedia.org/api/rest_v1/page/summary/associative%20memory';
const digest = 'd8c37a8c6e2e0abcb15cab4ece37127ab3a1d6335319be0fc79be0be66f5fa09';
const meanings = [
  'Associative memory (psychology), the ability to learn and remember the relationship between unrelated items',
  'Associative storage, or content-addressable memory, a type of computer memory used in certain very high speed searching applications',
  'Autoassociative memory, all computer memories that enable one to retrieve a piece of data from only a tiny sample of itself',
  'Bidirectional associative memory, a type of recurrent neural network',
  'Hopfield network, a form of recurrent artificial neural network',
  'Transderivational search in psychology or cybernetics, a search for a fuzzy match across a broad field',
];
const expected = 'associative memory — Wikipedia (disambiguation):\n'
  + meanings.map((meaning,index) => `  ${index + 1}. ${meaning}\n`).join('')
  + `Source: ${url} (sha256 ${digest}; captured-at 1791508773; cached true; CC BY-SA 4.0; https://creativecommons.org/licenses/by-sa/4.0/)`;
let host;
before(async () => {
  host = new WorkerHost();
  await installSourceDefinitionCaptures(host);
  host.context.fetch = () => { throw new Error('offline test must replay verified bytes'); };
});

test('a fresh worker needs an actual retrievable capture for a previously recorded subject', async () => {
  const fixture = sourceDefinitionCaptures().find(capture => capture.url === url);
  let requests = 0;
  const fresh = new WorkerHost({}, { networkFetch: async requested => {
    assert.equal(String(requested), url);
    requests++;
    return new Response(fixture.text);
  } });
  const capture = online => fresh.run('sourceWalkFetchCapture(__url, { online: __online })', { __url: url, __online: online });
  assert.equal((await capture(false)).failureKind, 'offline_cache_miss');
  assert.equal(requests, 0);
  const retrieved = await capture(true);
  assert.equal(retrieved.sha256, digest);
  assert.equal(retrieved.cached, false);
  assert.deepEqual(await capture(false), { ...retrieved, cached: true });
  assert.equal(requests, 1);
});

test('unchanged native original pin and all six real meanings', async () => {
  const answer = await host.solve(original);
  assert.equal(answer.intent,'concept_lookup');
  assert.equal(answer.content,expected);
  assert.ok(answer.evidence.includes(`source:http:${url} fetched_at=1791508773 sha256=${digest} cached=true`));
  assert.ok(answer.evidence.includes(`cache_hit:${url}`));
  const item = specificationCases(REPO_ROOT).cases.find(item => item.id.endsWith('source_cache.rs::external_lookups_record_source_url'));
  assert.ok(item,'the original native assertion remains carried');
  for (const check of item.checks) assert.equal(checkFailure(check,new Map(item.asks.map(ask => [ask.binding,answer]))),null);
});

test('genuine held-out English and Russian entries keep actual subjects and capture times', async () => {
  const records = sourceDefinitionCaptures();
  for (const [prompt,subject,language] of [['Cite a definition of entropy from Wikipedia','entropy','en'],['Приведи определение энтропия из Wikipedia','энтропия','ru']]) {
    const answer = await host.solve(prompt);
    const record = records.find(record => record.url.startsWith(`https://${language}.`) && !record.url.endsWith('memory'));
    const value = JSON.parse(record.text);
    assert.equal(value.type,'standard');
    assert.equal(createHash('sha256').update(record.text).digest('hex'),record.sha256);
    assert.ok(answer.content.startsWith(`${subject} — Wikipedia (definition):\n`),answer.content);
    assert.ok(answer.content.includes(value.extract.replace(/\s+/gu,' ').trim()));
    assert.ok(answer.content.includes(`captured-at ${record.fetchedAt}; cached true`));
    assert.ok(answer.evidence.includes(`source:${record.url}`));
    assert.ok(!answer.content.includes(value.timestamp),'article revision time is not fetch time');
  }
});

test('explicit source controls, cache misses and unrelated prompts never invent citations', async () => {
  for (const [prompt,preferences,status] of [[original,{externalServiceMediawikiFamily:false},'disabled-source'],['Cite a definition of entropy from Unknown provider',{},'missing-source'],['Cite a definition of uncaptured arbitrary subject from Wikipedia',{},'no-capture']]) {
    const answer = await host.run('trySourceQualifiedDefinition(__prompt,__preferences)',{__prompt:prompt,__preferences:preferences});
    assert.equal(answer.content,`No verified definition of ${prompt === original ? 'associative memory' : prompt.includes('Unknown') ? 'entropy' : 'uncaptured arbitrary subject'} from ${prompt.includes('Unknown') ? 'Unknown provider' : 'Wikipedia'}: ${status}.`);
    assert.ok(!answer.evidence.some(item => item.startsWith('source:http') || item.startsWith('cache_hit:')));
  }
  assert.equal(await host.run('trySourceQualifiedDefinition("Hi",{})'),null);
});

test('five language frames, reversed order and delimiter words bind distinct slots', async () => {
  const probes = [
    ['According to Wikipedia, define entropy','entropy','en'],
    ['Cita una definición de red de nodos de Wikipedia','red de nodos','es'],
    ['Приведи определение энтропия из Wikipedia','энтропия','ru'],
    ['Wikipedia से ऊर्जा की परिभाषा उद्धृत करें','ऊर्जा','hi'],
    ['引用Wikipedia对熵的定义','熵','zh'],
  ];
  for (const [prompt,term,language] of probes) assert.deepEqual(await host.run('sourceQualifiedRequest(__prompt,sourceQualifiedFrames(),sourceWalkRegistry())',{__prompt:prompt}),{term,source:'Wikipedia',language});
  assert.equal(await host.run('sourceQualifiedRequest("Cite a definition of \\\"\\\" from Wikipedia",sourceQualifiedFrames(),sourceWalkRegistry())'),null);
});

test('arbitrary providers and subjects use actual captured extractor bytes', async () => {
  const outcome = await host.run(`observeSourceQualifiedDefinition(__prompt,__frames,__sources,{
    allowed: sourceWalkServiceAllowed.bind(null,{}),entryUrl:sourceWalkEntryUrl,
    capture:async url=>({ok:true,url,text:__body,fetchedAt:"1720000000",sha256:await sourceWalkSha256Hex(__body),cached:true}),
    sha256:bytes=>sourceWalkSha256Hex(new TextDecoder().decode(bytes)),read:sourceQualifiedProjection})`,{
      __prompt:'cita una definición de red de nodos de Archivo de pruebas',
      __frames:[{text:'cita una definición de {term} de {source}',language:'es'}],
      __sources:[{id:'lab-archive',name:'Archivo de pruebas',defaultEnabled:true,api:'https://example.test/{lemma}',apiLanguages:['es'],extractor:'wiktionary_entry_v1',licenseName:'synthetic fixture',licenseUrl:'https://example.test/license'}],
      __body:JSON.stringify([{word:'red de nodos',meanings:[{partOfSpeech:'noun',definitions:[{definition:'An arbitrary fixture definition.'}]}]}]),
    });
  assert.equal(outcome.status,'captured');
  assert.equal(outcome.request.term,'red de nodos');
  assert.deepEqual(outcome.projection.items,['An arbitrary fixture definition.']);
  assert.equal(outcome.provenance.sourceUrl,'https://example.test/red%20de%20nodos');
});

test('cache replay refuses corruption, invalid times and duplicate records', async () => {
  const bytes='arbitrary body';const sha=createHash('sha256').update(bytes).digest('hex');
  const raw=`source-captures\n  capture ${sha}\n    url https://example.test/entry\n    fetched-at 1720000000\n    sha256 ${sha}\n    body "${bytes}"\n`;
  const read = value => host.run('sourceWalkSeedCapture("https://example.test/entry",__raw)',{__raw:value});
  assert.equal((await read(raw)).text,bytes);
  assert.equal((await read(raw.replace(bytes,'tampered'))).error,'invalid_capture');
  for (const time of ['0', '+1720000000', '18446744073709551616']) assert.equal((await read(raw.replace('1720000000',time))).error,'invalid_capture');
  assert.equal((await read(raw+raw)).error,'ambiguous_capture');
  assert.equal(await host.run('sourceWalkSeedCapture("https://example.test/missing",__raw)',{__raw:raw}),null);
});

test('observer validates capture URL and hash before advertising provenance', async () => {
  for(const fault of ['url','hash']){
    const outcome=await host.run(`observeSourceQualifiedDefinition(__prompt,sourceQualifiedFrames(),sourceWalkRegistry(),{
      entryUrl:sourceWalkEntryUrl,capture:async url=>({ok:true,url:__fault==="url"?url+"/wrong":url,text:"{}",fetchedAt:"1720000000",sha256:__fault==="hash"?"0".repeat(64):await sourceWalkSha256Hex("{}"),cached:true}),
      sha256:bytes=>sourceWalkSha256Hex(new TextDecoder().decode(bytes)),read:()=>({items:["must never escape"]})})`,{__prompt:original,__fault:fault});
    assert.equal(outcome.status,'invalid-capture');assert.equal(outcome.provenance,undefined);
  }
});

test('all disambiguation list entries survive the source projection', async()=>{
  const html=Array.from({length:15},(_,index)=>`<li>Arbitrary meaning ${index+1}</li>`).join('');
  const projection=await host.run('sourceQualifiedProjection({extractor:"mediawiki_summary_v1"},{text:JSON.stringify({type:"disambiguation",title:"Arbitrary",extract_html:__html})},"Arbitrary")',{__html:html});
  assert.deepEqual(projection.items,Array.from({length:15},(_,index)=>`Arbitrary meaning ${index+1}`));
});
