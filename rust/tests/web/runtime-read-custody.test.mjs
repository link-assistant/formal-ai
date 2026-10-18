import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync}from 'node:child_process';
import {fileURLToPath}from 'node:url';
import {collectRuntimeObservation}from '../../../js/provenance/fresh-runtime.mjs';
const repository=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
test('actual ESM filesystem read returns and original failed-read codes remain distinct from compiled sources',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'runtime-read-custody-'));
 try{
  const transportDirectory=path.join(directory,'transport');fileSystem.mkdirSync(transportDirectory);
  const configuration=path.join(directory,'configuration.json');
  fileSystem.writeFileSync(configuration,JSON.stringify({transportDirectory,repository:directory}));
  const bootstrap=path.join(directory,'bootstrap.cjs'),loader=path.join(directory,'loader.mjs');
  fileSystem.copyFileSync(path.join(repository,'js/provenance/runtime-journal.js'),bootstrap);
  fileSystem.copyFileSync(path.join(repository,'js/provenance/runtime-loader.mjs'),loader);
  const filename=path.join(directory,'seed.lino'),expected='(held-out "β 😀")\r\n';fileSystem.writeFileSync(filename,expected);
  const main=path.join(directory,'main.mjs');
  fileSystem.writeFileSync(main,"import {readFileSync,writeFileSync}from'node:fs';\n"+
   "const seed=new URL('./seed.lino',import.meta.url);readFileSync(seed,'utf8');readFileSync(seed);\n"+
   "try{readFileSync(new URL('./absent.lino',import.meta.url),'utf8');throw Error('ExpectedMissing');}catch(error){if(error.code!=='ENOENT')throw error;}\n"+
   "writeFileSync(seed,'changed after actual read');\n");
  const child=spawnSync(process.execPath,['--require',bootstrap,'--experimental-loader',loader,main],{
   encoding:'utf8',timeout:20000,env:{...process.env,NODE_OPTIONS:'',NODE_TEST_CONTEXT:undefined,
    FORMAL_AI_RUNTIME_CONFIG:configuration,FORMAL_AI_RUNTIME_BOOTSTRAP:bootstrap}});
  assert.equal(child.status,0,child.stderr);assert.equal(child.signal,null);
  const observed=collectRuntimeObservation(transportDirectory);
  const reads=observed.sources.filter(row=>row.url.endsWith('/seed.lino'));
  assert.equal(reads.length,2);assert(reads.every(row=>row.boundary==='actual-repository-file-read-return'));
  for(const row of reads)assert.equal(Buffer.from(row.bytes,'base64').toString('utf8'),expected);
  assert.deepEqual(new Set(reads.map(row=>row.format)),new Set(['repository-read-string','repository-read-buffer']));
  assert(observed.events.some(row=>row.kind==='file-read-failure'&&row.url.endsWith('/absent.lino')&&row.code==='ENOENT'));
  assert(!reads.some(row=>row.boundary==='actual-commonjs-compile-input'));
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
