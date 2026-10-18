import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash}from 'node:crypto';
import {spawnSync}from 'node:child_process';
import {fileURLToPath,pathToFileURL}from 'node:url';
test('archive transport executes retained ESM CJS and read inputs after physical replacement and refuses unknown or corrupted inputs',()=>{
 const owned=fileSystem.realpathSync(fileSystem.mkdtempSync(path.join(os.tmpdir(),'archived-transport-')));
 try{
  const workspace=path.join(owned,'engine');fileSystem.mkdirSync(workspace);
  const entry=path.join(workspace,'entry.mjs'),common=path.join(workspace,'module.cjs'),data=path.join(workspace,'input.dat');
  const original="import{createRequire}from'node:module';import fs from'node:fs';const require=createRequire(import.meta.url);console.log(require('./module.cjs')+Number(fs.readFileSync(new URL('./input.dat',import.meta.url),'utf8')));";
  const descriptor=text=>{const bytes=Buffer.from(text);return{bytes:bytes.toString('base64'),size:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')};};
  const entryURL=pathToFileURL(entry).href,commonURL=pathToFileURL(common).href,dataURL=pathToFileURL(data).href;
  // Synthetic archive metadata isolates the transport; signed production proof is exercised by full-provenance-cli.
  const archive={schema:'formal-ai-authenticated-runtime-archive-v1',repository:workspace,
   modules:{[entryURL]:{format:'module',...descriptor(original)},[commonURL]:{format:'commonjs',...descriptor('module.exports=70;')}},
   reads:{[dataURL+'\0repository-read-string']:descriptor('7')},esm:{[JSON.stringify([null,entryURL])]:entryURL},commonjs:{[JSON.stringify([entry,'./module.cjs'])]:common}};
  fileSystem.writeFileSync(entry,'throw Error("INSTALLED_ENGINE_MUST_NOT_EXECUTE");');
  fileSystem.writeFileSync(common,'module.exports=999;');fileSystem.writeFileSync(data,'1000');
  const preload=path.join(owned,'preload.cjs'),loader=path.join(owned,'loader.mjs'),configuration=path.join(owned,'config.json');
  for(const[source,target]of [['archive-preload.js',preload],['archive-loader.mjs',loader]]){
   fileSystem.copyFileSync(fileURLToPath(new URL('../../../js/provenance/'+source,import.meta.url)),target);
  }
  const run=()=>{fileSystem.writeFileSync(configuration,JSON.stringify({sourceArchive:archive}));
   return spawnSync(process.execPath,['--max-old-space-size=256','--require',preload,'--experimental-loader',loader,entry],{
    encoding:'utf8',timeout:15000,maxBuffer:1024*1024,
    env:{...process.env,NODE_OPTIONS:'',NODE_TEST_CONTEXT:undefined,FORMAL_AI_RUNTIME_CONFIG:configuration,FORMAL_AI_RUNTIME_BOOTSTRAP:undefined}});};
  const positive=run();assert.equal(positive.status,0,positive.stderr);assert.equal(positive.stdout,'77\n');
  assert.equal(fileSystem.readFileSync(entry,'utf8'),'throw Error("INSTALLED_ENGINE_MUST_NOT_EXECUTE");');
  fileSystem.unlinkSync(common);fileSystem.unlinkSync(data);const deleted=run();assert.equal(deleted.status,0,deleted.stderr);assert.equal(deleted.stdout,'77\n');
  archive.modules[entryURL].sha256='0'.repeat(64);const corrupted=run();assert.notEqual(corrupted.status,0);assert.match(corrupted.stderr,/ArchivedRuntimeTransportBytes/u);
  archive.modules[entryURL]= {format:'module',...descriptor(original)};
  delete archive.esm[JSON.stringify([null,entryURL])];const unobservedEntry=run();assert.notEqual(unobservedEntry.status,0);
  assert.match(unobservedEntry.stderr,/ArchivedRuntimeUnobservedResolution/u);archive.esm[JSON.stringify([null,entryURL])]=entryURL;
  delete archive.commonjs[JSON.stringify([entry,'./module.cjs'])];const unknownImport=run();assert.notEqual(unknownImport.status,0);assert.match(unknownImport.stderr,/ArchivedRuntimeUnobservedResolution/u);
  archive.commonjs[JSON.stringify([entry,'./module.cjs'])]=common;
  delete archive.reads[dataURL+'\0repository-read-string'];const unknownRead=run();assert.notEqual(unknownRead.status,0);assert.match(unknownRead.stderr,/ArchivedRuntimeUnobservedRead/u);
 }finally{fileSystem.rmSync(owned,{recursive:true,force:true});}
});
