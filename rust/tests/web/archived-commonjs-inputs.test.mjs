import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const descriptor=text=>({bytes:Buffer.from(text).toString('base64'),sha256:createHash('sha256').update(text).digest('hex'),size:Buffer.byteLength(text)});
function archivedCompile(format,representation,{forged=false}={}){
 const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'archived-compiler-input-')));
 try{
  const repository=path.join(directory,'work'),transport=path.join(directory,'transport');fs.mkdirSync(repository);fs.mkdirSync(transport);
  const filename=path.join(repository,format==='json'?'value.json':'value.cjs'),url=pathToFileURL(filename).href;
  const source=format==='json'?'37\n':'module.exports = 37;\n';
  const foreign=format==='json'?'999\n':'module.exports = 999;\n';
  fs.writeFileSync(filename,foreign);
  const entry=path.join(directory,'entry.cjs');fs.writeFileSync(entry,'process.stdout.write(String(require('+JSON.stringify(filename)+')));\n');
  const archive={schema:'formal-ai-authenticated-runtime-archive-v1',repository,modules:{[url]:{format,...descriptor(source)}},
   reads:representation?{[url+'\0repository-read-'+representation]:descriptor(forged?foreign:source)}:{},
   commonjs:{[JSON.stringify([entry,filename])]:filename},esm:{}};
  const config=path.join(directory,'config.json');fs.writeFileSync(config,JSON.stringify({repository,transportDirectory:transport,sourceArchive:archive}),{mode:0o600});
  const preload=path.join(directory,'archive-preload.cjs'),bootstrap=path.join(directory,'bootstrap.cjs');
  fs.copyFileSync(path.join(root,'js/provenance/archive-preload.js'),preload);fs.copyFileSync(path.join(root,'js/provenance/runtime-journal.js'),bootstrap);
  const child=spawnSync(process.execPath,['--require',preload,'--require',bootstrap,entry],{encoding:'utf8',timeout:30000,maxBuffer:512*1024,
   env:{...process.env,NODE_OPTIONS:'',FORMAL_AI_RUNTIME_CONFIG:config,FORMAL_AI_RUNTIME_BOOTSTRAP:bootstrap}});
  const rows=fs.readdirSync(transport).filter(name=>/^cjs-[0-9]+\.jsonl$/u.test(name)).flatMap(name=>fs.readFileSync(path.join(transport,name),'utf8').trimEnd().split('\n').filter(Boolean).map(JSON.parse));
  return{status:child.status,stdout:child.stdout,stderr:child.stderr,sources:rows.filter(row=>row.kind==='source'&&row.url===url)};
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
}
for(const format of ['commonjs','json'])for(const representation of ['string','buffer'])test('archived '+format+' consumes authenticated '+representation+' compiler input',()=>{
 const result=archivedCompile(format,representation);assert.equal(result.status,0,result.stderr);assert.equal(result.stdout,'37');
 assert.ok(result.sources.some(source=>source.format===format),'actual compile/JSON input observed');
 assert.ok(result.sources.some(source=>source.format==='repository-read-'+representation),'actual authenticated compiler read observed');
});
test('a conflicting authenticated read cannot be substituted for the compile input',()=>{
 const result=archivedCompile('commonjs','string',{forged:true});assert.notEqual(result.status,0);assert.match(result.stderr,/ArchivedRuntimeCompilerReadMismatch/u);assert.equal(result.stdout,'');
});
test('a module without a recorded read does not invent a consumption event',()=>{
 const result=archivedCompile('commonjs',null);assert.equal(result.status,0,result.stderr);assert.equal(result.stdout,'37');
 assert.equal(result.sources.filter(source=>source.format.startsWith('repository-read-')).length,0);
});
