import assert from 'node:assert/strict';
import test from 'node:test';
import {cachedSourceFetch,cachedSourceFetchResult,cacheKey} from '../../../js/agentic/crate/source_cache.mjs';
import {sha256Hex} from '../../../js/agentic/crate/source_fetch.mjs';
const url='https://example.test/item',bytes=new TextEncoder().encode('original bytes');
const hash=sha256Hex(bytes),root='/synthetic/source-cache',meta=root+'/'+cacheKey(url)+'.meta',body=root+'/objects/'+hash+'.body';
function setup({metadata,bodyBytes=bytes,readError=false,bodyError=false}={}) {
  let calls=0;const files=new Map();if(metadata!==undefined)files.set(meta,metadata);files.set(body,bodyBytes);
  const io={readText:file=>{if(readError)throw Error('permission denied');return files.get(file)??null;},
    readBytes:file=>{if(bodyError)throw Error('body denied');return files.get(file)??null;},
    get:()=>{calls++;return bytes;},createDirAll:()=>{},writeBytes:(file,value)=>files.set(file,value)};
  return {io,files,calls:()=>calls};
}
const valid=(time='100')=>'source-capture-v2\nurl='+url+'\nfetched_at='+time+'\nsha256='+hash+'\n';
for(const online of [false,true]) {
  for(const [label,options,key] of [
    ['unsupported',{metadata:'unknown'},'source_cache_unsupported_metadata'],
    ['missing field',{metadata:'source-capture-v2'},'source_cache_missing_field'],
    ['wrong URL',{metadata:valid().replace(url,url+'/wrong')},'source_cache_url_mismatch'],
    ['invalid digest',{metadata:valid().replace(hash,'invalid')},'source_cache_invalid_content_hash'],
    ['missing body',{metadata:valid(),bodyBytes:null},'source_cache_missing_body'],
    ['poisoned body',{metadata:valid(),bodyBytes:new TextEncoder().encode('poison')},'source_cache_content_hash_mismatch'],
    ['metadata read error',{readError:true},'permission denied'],
    ['body read error',{metadata:valid(),bodyError:true},'body denied'],
  ]) test('synthetic '+label+' fails closed online='+online,()=>{
    const c=setup(options),optionsFor={cacheDir:'/synthetic',online,io:c.io,now:()=>100};
    const result=cachedSourceFetchResult(optionsFor)(url);assert.equal(result.kind,'error');assert.ok(result.error.includes(key),result.error);
    if(label==='missing body')assert.equal(result.error,'source_cache_missing_body:'+body);
    assert.equal(cachedSourceFetch(optionsFor)(url),null);assert.equal(c.calls(),0);
  });
}
test('synthetic genuine miss remains offline miss or online live capture',()=>{
  for(const online of [false,true]){const c=setup(),r=cachedSourceFetchResult({cacheDir:'/synthetic',online,io:c.io,now:()=>100})(url);
    assert.equal(r.kind,online?'capture':'miss');assert.equal(c.calls(),online?1:0);if(online){assert.equal(r.capture.cached,false);assert.equal(r.capture.fetched_at,'100');}}
});
test('synthetic valid cache preserves original timestamp and exact bytes without transport',()=>{
  for(const online of [false,true]){const c=setup({metadata:valid()}),r=cachedSourceFetchResult({cacheDir:'/synthetic',online,io:c.io,now:()=>101})(url);
    assert.equal(r.kind,'capture');assert.equal(r.capture.cached,true);assert.equal(r.capture.fetched_at,'100');assert.equal(r.capture.sha256,hash);assert.deepEqual(r.capture.bytes,bytes);assert.equal(c.calls(),0);}
});
test('synthetic stale valid cache refreshes only online and retains offline history',()=>{
  for(const online of [false,true]){const c=setup({metadata:valid()}),r=cachedSourceFetchResult({cacheDir:'/synthetic',online,io:c.io,now:()=>1000,ttlSeconds:10})(url);
    assert.equal(r.kind,'capture');assert.equal(r.capture.cached,!online);assert.equal(r.capture.fetched_at,online?'1000':'100');assert.equal(c.calls(),online?1:0);}
});
test('synthetic legacy body layout and CRLF metadata remain readable',()=>{
  const c=setup({metadata:valid().replace('v2','v1').replaceAll('\n','\r\n')});c.files.set(meta.replace(/\.meta$/u,'.body'),bytes);
  assert.equal(cachedSourceFetch({cacheDir:'/synthetic',online:true,io:c.io,now:()=>101})(url).sha256,hash);assert.equal(c.calls(),0);
});
test('synthetic unavailable transport is not invented HTTP metadata',()=>{
  const c=setup();c.io.get=()=>null;assert.deepEqual(cachedSourceFetchResult({cacheDir:'/synthetic',online:true,io:c.io})(url),{kind:'unavailable'});
});

for (const online of [false, true]) test('bare final CR is an invalid digest before any body read online=' + online, () => {
  let bodyReads = 0;
  const c = setup({metadata: valid().slice(0, -1) + '\r'});
  const originalRead = c.io.readBytes;
  c.io.readBytes = file => { bodyReads++; return originalRead(file); };
  const result = cachedSourceFetchResult({cacheDir: '/synthetic', online, io: c.io, now: () => 101})(url);
  assert.equal(result.kind, 'error');
  assert.ok(result.error.includes('source_cache_invalid_content_hash'));
  assert.equal(bodyReads, 0);
  assert.equal(c.calls(), 0);
});
