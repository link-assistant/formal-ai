import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {createWorkerContext,evaluate,plain} from './support/browser-runtime.mjs';
async function boot() {
  const context=createWorkerContext(); await evaluate(context,'loadSeed()');
  if(process.env.SOURCE_CACHE_CANDIDATE) evaluate(context,fs.readFileSync(process.env.SOURCE_CACHE_CANDIDATE,'utf8'));
  context.__calls=0; context.fetch=async()=>{context.__calls++;return {ok:true,text:async()=>'original bytes'};};
  return context;
}
const url='https://example.test/cache-integrity';
async function fetchCapture(context,online=true) {
  context.__url=url; context.__online=online;
  return evaluate(context,'sourceWalkFetchCapture(__url,{online:__online})');
}
test('controlled transport: returned captures cannot mutate retained bytes or metadata',async()=>{
  const c=await boot(),first=await fetchCapture(c);
  const original=plain(first); first.text='tampered';first.sha256='0'.repeat(64);first.url='https://wrong.test';
  const hit=await fetchCapture(c,false);assert.deepEqual(plain(hit),{...original,cached:true});
  hit.text='second mutation';hit.fetchedAt='0';
  assert.deepEqual(plain(await fetchCapture(c,false)),{...original,cached:true});assert.equal(c.__calls,1);
});
test('controlled transport: corrupt retained digest or source identity fails closed',async()=>{
  for(const field of ['text','sha256','url']) {
    const c=await boot();await fetchCapture(c);c.__field=field;
    evaluate(c,'sourceWalkCaptureCache.set(__url,{...sourceWalkCaptureCache.get(__url),[__field]:"corrupted"})');
    for(const online of [false,true]) assert.deepEqual(plain(await fetchCapture(c,online)),{ok:false,url,error:'invalid_capture'});
    assert.equal(c.__calls,1);
  }
});
test('controlled transport: deleting cache requires reacquisition and preserves provider failure',async()=>{
  const c=await boot();await fetchCapture(c);evaluate(c,'sourceWalkCaptureCache.delete(__url)');
  assert.equal((await fetchCapture(c,false)).failureKind,'offline_cache_miss');assert.equal(c.__calls,1);
  assert.equal((await fetchCapture(c)).cached,false);assert.equal(c.__calls,2);
  evaluate(c,'sourceWalkCaptureCache.delete(__url)');c.fetch=async()=>({ok:false,status:503});
  assert.deepEqual(plain(await fetchCapture(c)),{ok:false,url,error:'http_503'});
});
