import assert from 'node:assert/strict';
import test from 'node:test';
import {createWorkerContext, evaluate, plain} from './support/browser-runtime.mjs';
const start='https://example.test/start', follow='https://example.test/follow';
async function boot() {
 const context=createWorkerContext();await evaluate(context,'loadSeed()');
 context.__network=[];context.__bodies=[];
 context.fetch=async url=>{context.__network.push(String(url));return {ok:false,status:503};};
 context.__start=start;context.__follow=follow;
 evaluate(context,'sourceWalkAccessibility.clear(); sourceWalkCaptureCache.clear()');
 return context;
}
async function cache(context,text='original bytes') {
 context.__body=text;
 await evaluate(context,'(async()=>{sourceWalkCaptureCache.set(__start,{ok:true,url:__start,text:__body,sha256:await sourceWalkSha256Hex(__body),fetchedAt:String(sourceWalkNowSeconds()),cached:false})})()');
}
async function walk(context,{online=false,followPage=false}={}) {
 context.__online=online;context.__followPage=followPage;
 return plain(await evaluate(context,`sourceWalkSources("concept","subject","en",{},
  {maxServices:1,maxDepth:2,maxPagesPerService:4,maxItems:12,maxCaptureAgeSeconds:600},
  {online:__online,entryUrl:()=>__start,read:(_record,capture,depth)=>{
    __bodies.push(capture.text);return {items:[],follow:__followPage&&depth===0?[__follow]:[]};
  }})`));
}
test('actual offline cache absence never invokes fetch or poisons service health',async()=>{
 const c=await boot(),r=await walk(c);
 assert.equal(r.outcomes.at(-1).status,'offline_cache_miss');
 assert.equal(r.outcomes.at(-1).pages,0);assert.deepEqual(c.__network,[]);
 assert.equal(evaluate(c,'sourceWalkAccessibility.size'),0);assert.deepEqual(c.__bodies,[]);
});
test('actual offline cached body is read byte-exact with no network request',async()=>{
 const c=await boot();const bytes='{"error":"failed","status":503,"source_read":false}';await cache(c,bytes);
 const r=await walk(c);assert.equal(r.outcomes.at(-1).status,'no_items');assert.equal(r.outcomes.at(-1).pages,1);
 assert.deepEqual(c.__bodies,[bytes]);assert.deepEqual(c.__network,[]);
});
test('offline missing follow capture retains the prior successful service observation',async()=>{
 const c=await boot();await cache(c);const r=await walk(c,{followPage:true});
 const outcome=r.outcomes.at(-1);assert.equal(outcome.status,'offline_cache_miss');assert.equal(outcome.pages,1);
 assert.match(outcome.detail,/https:\/\/example\.test\/follow/);assert.deepEqual(c.__network,[]);
 const health=plain(evaluate(c,'Array.from(sourceWalkAccessibility.values())'));
 assert.equal(health.length,1);assert.equal(health[0].status,'reachable');assert.equal(health[0].detail,'captured '+start);
});
test('live transport prose cannot declare offline policy',async()=>{
 const c=await boot();c.fetch=async url=>{c.__network.push(String(url));throw Error('no cached capture for '+url);};
 const r=await walk(c,{online:true});assert.equal(r.outcomes.at(-1).status,'unreachable');
 assert.deepEqual(c.__network,[start]);assert.equal(evaluate(c,'sourceWalkAccessibility.size'),1);
});
test('actual live HTTP503 remains a service failure',async()=>{
 const c=await boot();const r=await walk(c,{online:true});assert.equal(r.outcomes.at(-1).status,'unreachable');
 assert.deepEqual(c.__network,[start]);assert.equal(evaluate(c,'sourceWalkAccessibility.size'),1);
});

test('offline capture diagnostic is seeded and preserves every URL byte',async()=>{
 for(const url of ['https://example.test/raw',"https://example.test/$&/$`/$'/α😀?value={url}"]){
  const c=await boot();c.__url=url;
  const receipt=plain(await evaluate(c,'sourceWalkFetchCapture(__url,{online:false})'));
  assert.equal(receipt.ok,false);assert.equal(receipt.failureKind,'offline_cache_miss');
  assert.equal(receipt.url,url);assert.equal(receipt.error,'no cached capture for '+url);
  assert.deepEqual(c.__network,[]);assert.equal(evaluate(c,'sourceWalkAccessibility.size'),0);
 }
});
