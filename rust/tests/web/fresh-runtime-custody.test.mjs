import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {verifyDriveEvidence,retainedEvidenceBytes}from '../../../js/provenance/verify-drive.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
import {collectRuntimeObservation,captureFreshPlannerRun} from '../../../js/provenance/fresh-runtime.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
test('actual Node compile/load inputs survive source-file changes and reject archive tampering',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'runtime-custody-'));
 try{
  const transportDirectory=path.join(directory,'transport');fs.mkdirSync(transportDirectory);
  const config=path.join(directory,'config.json');fs.writeFileSync(config,JSON.stringify({transportDirectory}));
  const bootstrap=path.join(directory,'bootstrap.cjs');
  fs.copyFileSync(path.join(root,'js/provenance/runtime-journal.js'),bootstrap);
  const loader=path.join(directory,'loader.mjs');
  fs.copyFileSync(path.join(root,'js/provenance/runtime-loader.mjs'),loader);
  const common=path.join(directory,'example.cjs'),json=path.join(directory,'example.json');
  const commonBytes='module.exports = 17;\n',jsonBytes='{"observed":23}\n';
  fs.writeFileSync(common,commonBytes);fs.writeFileSync(json,jsonBytes);
  const main=path.join(directory,'main.mjs');
  fs.writeFileSync(main,"import {createRequire} from 'node:module';\n"+
   "import fs from 'node:fs';const require=createRequire(import.meta.url);\n"+
   "if(require('./example.cjs')!==17||require('./example.json').observed!==23)throw Error('ActualFixture');\n"+
   "fs.writeFileSync(new URL('./example.cjs',import.meta.url),'changed after compilation');\n");
  const child=spawnSync(process.execPath,['--require',bootstrap,'--experimental-loader',loader,main],{
   encoding:'utf8',timeout:20000,env:{...process.env,NODE_OPTIONS:'',NODE_TEST_CONTEXT:undefined,
    FORMAL_AI_RUNTIME_CONFIG:config,FORMAL_AI_RUNTIME_BOOTSTRAP:bootstrap}});
  assert.equal(child.status,0,child.stderr);assert.equal(child.signal,null);
  const observation=collectRuntimeObservation(transportDirectory);
  const compiled=observation.sources.find(row=>row.url.endsWith('/example.cjs'));
  const parsed=observation.sources.find(row=>row.url.endsWith('/example.json'));
  assert.equal(Buffer.from(compiled.bytes,'base64').toString(),commonBytes);
  assert.equal(Buffer.from(parsed.bytes,'base64').toString(),jsonBytes);
  assert(observation.sources.some(row=>row.url.endsWith('/main.mjs')&&row.boundary==='actual-esm-load-return'));
  assert.equal(observation.fullReplay.verified,false);
  const object=fs.readdirSync(path.join(transportDirectory,'objects'))[0];
  const filename=path.join(transportDirectory,'objects',object),bytes=fs.readFileSync(filename);
  const altered=Buffer.from(bytes);altered[0]^=1;fs.writeFileSync(filename,altered);
  assert.throws(()=>collectRuntimeObservation(transportDirectory),/RuntimeSourceDigest/);
  fs.writeFileSync(filename,bytes);
  const journal=fs.readdirSync(transportDirectory).find(name=>name.endsWith('.jsonl'));
  const journalPath=path.join(transportDirectory,journal),text=fs.readFileSync(journalPath,'utf8');
  fs.writeFileSync(journalPath,text.slice(0,-1));
  assert.throws(()=>collectRuntimeObservation(transportDirectory),/RuntimeJournalIncomplete/);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
test('missing journals and symlink aliases into the workspace fail closed',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'runtime-refusal-'));
 try{
  assert.throws(()=>collectRuntimeObservation(directory),/RuntimeJournalMissing/);
  const workspace=path.join(directory,'workspace');fs.mkdirSync(workspace);
  const alias=path.join(directory,'alias');fs.symlinkSync(workspace,alias,'dir');
  assert.throws(()=>captureFreshPlannerRun({directory:alias,workspace,prompt:'ignored'}),
   /FreshRuntimeEvidenceInsideWorkspace/);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('fresh real planner writes observed bytes and binds its actual loaded source archives',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'fresh-planner-custody-'));
 try{
  const workspace=path.join(directory,'workspace');fs.mkdirSync(workspace);
  const observed=captureFreshPlannerRun({directory:path.join(directory,'evidence'),workspace,
   prompt:'Create file heldout.mjs containing export const measure = 37;',steps:12});
  assert.equal(fs.readFileSync(path.join(workspace,'heldout.mjs'),'utf8'),'export const measure = 37;');
  assert.equal(observed.custody.processReceipt.status,0);
  assert.equal(observed.result.stop,'final');
  assert.equal(observed.fullReplay.verified,false);
  const proof=observed.result.provenance;
  verifyDriveEvidence(proof.directory,workspace,proof.anchor);
  const records=fs.readFileSync(path.join(proof.directory,'chain.lino'),'utf8').trimEnd().split('\n').map(decodeTyped);
  const runtime=records.filter(row=>row.kind==='runtime_sources');assert.equal(runtime.length,2);
  const final=JSON.parse(retainedEvidenceBytes(proof.directory,runtime[1].payload.evidence));
  assert(final.sources.some(row=>row.url.endsWith('/js/agentic/planner.mjs')));
  assert(final.sources.some(row=>row.boundary==='actual-commonjs-compile-input'));
  const retained=final.sources[0].evidence;
  const leaf=retained.chunks?.[0]??retained;
  const filename=path.join(proof.directory,leaf.file),bytes=fs.readFileSync(filename);
  fs.rmSync(filename);
  assert.throws(()=>verifyDriveEvidence(proof.directory,workspace,proof.anchor));
  fs.writeFileSync(filename,bytes);
  verifyDriveEvidence(proof.directory,workspace,proof.anchor);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('refused absent workspace children and symlink aliases create nothing',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'fresh-preflight-'));
 try{
  const workspace=path.join(directory,'workspace');fs.mkdirSync(workspace);
  const child=path.join(workspace,'must-not-exist','nested');
  assert.throws(()=>captureFreshPlannerRun({directory:child,workspace,prompt:'ignored'}),/FreshRuntimeEvidenceInsideWorkspace/);
  assert.deepEqual(fs.readdirSync(workspace),[]);
  const alias=path.join(directory,'alias');fs.symlinkSync(workspace,alias,'dir');
  assert.throws(()=>captureFreshPlannerRun({directory:path.join(alias,'also-absent'),workspace,prompt:'ignored'}),
   /FreshRuntimeEvidenceInsideWorkspace/);
  assert.deepEqual(fs.readdirSync(workspace),[]);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
test('archive symlinks and sparse oversize files refuse before content allocation/read',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'runtime-archive-'));
 try{
  const transport=path.join(directory,'transport');fs.mkdirSync(transport);
  const outside=path.join(directory,'outside');fs.writeFileSync(outside,'[]\n');
  const journal=path.join(transport,'cjs-0.jsonl');fs.symlinkSync(outside,journal);
  assert.throws(()=>collectRuntimeObservation(transport),/RuntimeArchiveSymlink/);fs.unlinkSync(journal);
  const descriptor=fs.openSync(journal,'w');fs.ftruncateSync(descriptor,2*1024*1024+1);fs.closeSync(descriptor);
  assert.throws(()=>collectRuntimeObservation(transport),/RuntimeArchiveSizeOrType/);fs.unlinkSync(journal);
  const objects=path.join(transport,'objects');fs.mkdirSync(objects);
  const name='a'.repeat(64)+'.src',object=path.join(objects,name);
  fs.writeFileSync(journal,JSON.stringify({sequence:0,kind:'source',file:'objects/'+name,size:3})+'\n');
  fs.symlinkSync(outside,object);
  assert.throws(()=>collectRuntimeObservation(transport),/RuntimeArchiveSymlink/);fs.unlinkSync(object);
  const sparse=fs.openSync(object,'w');fs.ftruncateSync(sparse,4*1024*1024+1);fs.closeSync(sparse);
  assert.throws(()=>collectRuntimeObservation(transport),/RuntimeArchiveSizeOrType/);
  fs.rmSync(objects,{recursive:true});fs.symlinkSync(directory,objects,'dir');
  assert.throws(()=>collectRuntimeObservation(transport),/RuntimeArchiveSymlink/);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
