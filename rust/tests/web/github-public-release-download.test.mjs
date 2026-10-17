import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync,rmSync,readFileSync,existsSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';import {join} from 'node:path';
import {publicReleaseAsset,publicReleaseRedirect,downloadPublicReleaseAsset} from '../../../scripts/github-public-release-download.mjs';
const bytes=Buffer.from('public release bytes\n');
const repository='link-assistant/formal-ai',tag='v0.354.0';
const asset={id:7,name:'engine.tgz',state:'uploaded',size:bytes.length,digest:'sha256:'+createHash('sha256').update(bytes).digest('hex'),browser_download_url:'https://github.com/'+repository+'/releases/download/'+tag+'/engine.tgz'};
function options(directory,changes={}){return {asset:{...asset},repository,tag,destination:join(directory,asset.name),maxBytes:1024,deadline:Date.now()+10000,...changes};}
function isolated(run){const directory=mkdtempSync(join(tmpdir(),'formal-ai-public-download-unit-'));return Promise.resolve().then(()=>run(directory)).finally(()=>rmSync(directory,{recursive:true,force:true}));}
test('strict asset selection refuses wrong source URL, missing digest and oversized or unsafe names',()=>{
  assert.equal(publicReleaseAsset({asset,repository,tag,maxBytes:1024}),asset.browser_download_url);
  for(const change of [{browser_download_url:asset.browser_download_url.replace(tag,'v0.353.0')},{digest:null},{name:'../engine.tgz'},{state:'new'},{size:1025},{id:0}])assert.throws(()=>publicReleaseAsset({asset:{...asset,...change},repository,tag,maxBytes:1024}));
  assert.throws(()=>publicReleaseAsset({asset,repository,tag,maxBytes:undefined}));
});
test('redirects are HTTPS GitHub-only and never accept user credentials',()=>{
  assert.ok(publicReleaseRedirect('https://release-assets.githubusercontent.com/asset?signature=public',asset.browser_download_url).startsWith('https://release-assets.githubusercontent.com/'));
  for(const url of ['http://release-assets.githubusercontent.com/x','https://evil.example/x','https://github.com.evil.example/x','https://user:password@github.com/x','https://github.com:444/x','https://github.com/x#fragment'])assert.throws(()=>publicReleaseRedirect(url,asset.browser_download_url));
});
test('actual stream hashes and stores distinct public bytes without authorization or redirect forwarding',async t=>isolated(async directory=>{
  const requests=[];t.mock.method(globalThis,'fetch',async (url,request)=>{requests.push({url,request});return requests.length===1?new Response(null,{status:302,headers:{location:'https://release-assets.githubusercontent.com/pinned?signature=public'}}):new Response(bytes,{headers:{'content-length':String(bytes.length)}});});
  const result=await downloadPublicReleaseAsset(options(directory));
  assert.deepEqual(readFileSync(join(directory,asset.name)),bytes);assert.equal(result.completed,true);assert.equal(result.publicationAuthority,false);assert.equal(result.sha256,asset.digest.slice(7));assert.equal(requests.length,2);
  for(const {request}of requests){assert.equal(request.redirect,'manual');assert.equal(request.headers.Authorization,undefined);assert.equal(request.headers.authorization,undefined);assert.equal(request.headers['Accept-Encoding'],'identity');}
}));
for(const [label,payload,headers]of [['truncated',bytes.subarray(0,3),{}],['wrong hash',Buffer.alloc(bytes.length,1),{}],['oversized',Buffer.concat([bytes,bytes]),{}],['wrong content length',bytes,{'content-length':'1'}],['encoded payload',bytes,{'content-encoding':'gzip'}]])test('refuses '+label+' and leaves no partial successful file',async t=>isolated(async directory=>{
  t.mock.method(globalThis,'fetch',async()=>new Response(payload,{headers}));
  await assert.rejects(downloadPublicReleaseAsset(options(directory)));assert.equal(existsSync(join(directory,asset.name)),false);
}));
test('never overwrites or deletes an existing local file',async t=>isolated(async directory=>{
  const file=join(directory,asset.name);writeFileSync(file,'keep');t.mock.method(globalThis,'fetch',async()=>new Response(bytes));
  await assert.rejects(downloadPublicReleaseAsset(options(directory)),/EEXIST/u);assert.equal(readFileSync(file,'utf8'),'keep');
}));
test('redirect loops, missing assets and expired deadlines cannot produce success',async t=>isolated(async directory=>{
  t.mock.method(globalThis,'fetch',async()=>new Response(null,{status:302,headers:{location:'https://github.com/loop'}}));
  await assert.rejects(downloadPublicReleaseAsset(options(directory)),/redirects/u);assert.equal(existsSync(join(directory,asset.name)),false);
  await assert.rejects(downloadPublicReleaseAsset(options(directory,{deadline:Date.now()-1})),/deadline/u);
}));
