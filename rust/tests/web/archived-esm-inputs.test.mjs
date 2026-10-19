import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const descriptor=text=>({bytes:Buffer.from(text).toString('base64'),
 sha256:createHash('sha256').update(text).digest('hex'),size:Buffer.byteLength(text)});
function archivedModule(representation,{forged=false}={}){
 const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'archived-esm-input-')));
 try{
  const repository=path.join(directory,'work'),transport=path.join(directory,'transport');
  fs.mkdirSync(repository);fs.mkdirSync(transport);
  const filename=path.join(repository,'entry.mjs'),url=pathToFileURL(filename).href;
  const source='process.stdout.write("37");\n',foreign='throw Error("LIVE_SOURCE_MUST_NOT_RUN");\n';
  fs.writeFileSync(filename,foreign);
  const archive={schema:'formal-ai-authenticated-runtime-archive-v1',repository,
   modules:{[url]:{format:'module',...descriptor(source)}},
   reads:representation?{[url+'\0repository-read-'+representation]:descriptor(forged?foreign:source)}:{},
   commonjs:{},esm:{[JSON.stringify([null,url])]:url}};
  const config=path.join(directory,'config.json');
  fs.writeFileSync(config,JSON.stringify({repository,transportDirectory:transport,sourceArchive:archive}),{mode:0o600});
  const preload=path.join(directory,'archive-preload.cjs'),bootstrap=path.join(directory,'bootstrap.cjs');
  const loader=path.join(directory,'archive-loader.mjs');
  fs.copyFileSync(path.join(root,'js/provenance/archive-preload.js'),preload);
  fs.copyFileSync(path.join(root,'js/provenance/runtime-journal.js'),bootstrap);
  fs.copyFileSync(path.join(root,'js/provenance/archive-loader.mjs'),loader);
  fs.unlinkSync(filename);
  const child=spawnSync(process.execPath,['--require',preload,'--require',bootstrap,'--experimental-loader',loader,filename],{
   encoding:'utf8',timeout:30000,maxBuffer:512*1024,
   env:{...process.env,NODE_OPTIONS:'',FORMAL_AI_RUNTIME_CONFIG:config,FORMAL_AI_RUNTIME_BOOTSTRAP:bootstrap}});
  const rows=fs.readdirSync(transport).filter(name=>/^cjs-[0-9]+\.jsonl$/u.test(name))
   .flatMap(name=>fs.readFileSync(path.join(transport,name),'utf8').trimEnd().split('\n').filter(Boolean).map(JSON.parse));
  return{status:child.status,stdout:child.stdout,stderr:child.stderr,sources:rows.filter(row=>row.kind==='source'&&row.url===url)};
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
}
for(const representation of ['string','buffer'])test('archived ESM consumes actual authenticated '+representation+' compiler input',()=>{
 const result=archivedModule(representation);assert.equal(result.status,0,result.stderr);assert.equal(result.stdout,'37');
 assert.ok(result.sources.some(source=>source.format==='module'));
 assert.ok(result.sources.some(source=>source.format==='repository-read-'+representation));
});
test('ESM refuses conflicting read bytes before execution',()=>{
 const result=archivedModule('buffer',{forged:true});assert.notEqual(result.status,0);
 assert.match(result.stderr,/ArchivedRuntimeCompilerReadMismatch/u);assert.equal(result.stdout,'');
});
test('ESM without observed read does not synthesize a consumption receipt',()=>{
 const result=archivedModule(null);assert.equal(result.status,0,result.stderr);assert.equal(result.stdout,'37');
 assert.equal(result.sources.filter(source=>source.format.startsWith('repository-read-')).length,0);
});
