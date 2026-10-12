// Typed provenance fixtures and real Node process probes; these do not claim native Mac package execution.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFileSync,writeFileSync,mkdirSync,mkdtempSync,rmSync,chmodSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {createEvidenceFixture} from '../fixtures/native-release-evidence/observations.mjs';
import {collectReleaseEvidence,observeMacSigning,MAC_PACKAGE_TARGETS,observePackagedExecutable,validateSigningEvidence,sha256} from '../../../scripts/native-release-evidence.mjs';
const encode=value=>Buffer.from(JSON.stringify(value,null,2)+'\n');
function amend(f,change){const entry=f.signing[0],record=JSON.parse(entry.bytes);change(record);
 record.package_executable_sha256=record.package_executable_receipt?sha256(Buffer.from(JSON.stringify(record.package_executable_receipt)+'\n')):'0'.repeat(64);
 return {...f,signing:[{...entry,bytes:encode(record)},f.signing[1]]};}
test('post-sign executable bytes remain distinct from a source-bound original build receipt',()=>{
 const f=createEvidenceFixture(),record=JSON.parse(f.signing[0].bytes),receipt=record.package_executable_receipt;
 assert.notEqual(receipt.executable.sha256,receipt.parent.executable.sha256);
 assert.equal(receipt.executable.sha256,record.native_component.sha256);
 assert.equal(receipt.startup.stdout,'formal-ai 2.7.11\n');
 assert.deepEqual(receipt.identity.root_default_features,['portable']);
 assert.equal(collectReleaseEvidence(f).complete,true);
});
test('independent parent bytes cannot be replaced by an internally consistent signing copy',()=>{
 const f=createEvidenceFixture(),target=JSON.parse(f.signing[0].bytes).package_executable_receipt.identity.target;
 const altered=f.receipts.map(entry=>{if(entry.target!==target)return entry;const record=JSON.parse(entry.bytes);record.producer.job='other_native';return {...entry,bytes:encode(record)};});
 const result=collectReleaseEvidence({...f,receipts:altered});
 assert.equal(result.complete,false);assert.ok(result.failures.some(entry=>entry.kind==='signing'&&entry.target==='macos-arm64'));
});
test('startup source target package byte and signing cross-bindings reject independently',()=>{
 const changes=[r=>delete r.package_executable_receipt,
  r=>r.package_executable_receipt.startup.status=1,r=>r.package_executable_receipt.startup.signal='SIGTERM',
  r=>r.package_executable_receipt.startup.complete=false,r=>r.package_executable_receipt.startup.stdout='formal-ai 9.0.0\n',
  r=>r.package_executable_receipt.startup.arguments=['--help'],r=>r.package_executable_receipt.identity.target='x86_64-apple-darwin',
  r=>r.package_executable_receipt.identity.source_commit='0'.repeat(40),r=>r.package_executable_receipt.parent.receipt_sha256='0'.repeat(64),
  r=>r.package_executable_receipt.parent.executable.sha256='0'.repeat(64),r=>r.package_executable_receipt.executable.sha256='0'.repeat(64),
  r=>r.package_executable_receipt.package.sha256='0'.repeat(64),r=>r.package_executable_receipt.signing_observations_sha256='0'.repeat(64)];
 for(const change of changes){
  const f=createEvidenceFixture();
  const result=collectReleaseEvidence(amend(f,change));
  assert.equal(result.complete,false,String(change));assert.ok(result.failures.some(entry=>entry.kind==='signing'&&entry.target==='macos-arm64'));
 }
});
test('the canonical nested receipt digest refuses an unrecorded mutation',()=>{
 const f=createEvidenceFixture(),entry=f.signing[0],record=JSON.parse(entry.bytes);
 record.package_executable_receipt.startup.stderr='different actual command observation';
 assert.throws(()=>validateSigningEvidence(record,f.source,entry.label,f.bindings.sourceSha256));
});
test('actual process observation retains exact complete stdout stderr and executable bytes',()=>{
 const result=observePackagedExecutable(process.execPath);
 assert.deepEqual(result.executable,{bytes:readFileSync(process.execPath).length,sha256:sha256(readFileSync(process.execPath))});
 assert.deepEqual(result.startup,{arguments:['--version'],status:0,signal:null,complete:true,stdout:process.version+'\n',stderr:''});
 const f=createEvidenceFixture(),record=JSON.parse(f.signing[0].bytes);
 record.native_component=result.executable;record.package_executable_receipt.executable={name:'formal-ai',...result.executable};
 record.package_executable_receipt.startup=result.startup;
 record.package_executable_sha256=sha256(Buffer.from(JSON.stringify(record.package_executable_receipt)+'\n'));
 assert.throws(()=>validateSigningEvidence(record,f.source,record.label,f.bindings.sourceSha256),/startup version differs/u);
});
test('real nonzero exit and timeout never certify complete successful package execution',()=>{
 assert.throws(()=>observePackagedExecutable(process.execPath,(binary,_args,options)=>spawnSync(binary,['-e','process.stdout.write("partial");process.exit(7)'],options)));
 assert.throws(()=>observePackagedExecutable(process.execPath,(binary,_args,options)=>spawnSync(binary,['-e','process.stdout.write("partial");setInterval(()=>{},1000)'],{...options,timeout:40})),/ETIMEDOUT/u);
});
test('missing unknown and signalled process statuses are rejected without fabricated zero',()=>{
 for(const result of [{status:null,signal:null,stdout:'',stderr:''},{status:0,signal:'SIGKILL',stdout:'',stderr:''},{status:0,signal:null,stdout:undefined,stderr:''}]){
  assert.throws(()=>observePackagedExecutable(process.execPath,()=>result));
 }
});

test('the Mac signing producer invokes the actual packaged sidecar and binds its parent',context=>{
 if(process.platform!=='darwin') {
  assert.throws(()=>observeMacSigning('absent','absent',{source:{},sourceSha256:'0'.repeat(64),label:'macos-arm64',parentBytes:Buffer.from('{}')}));return;
 }
 const f=createEvidenceFixture(),label=process.arch==='arm64'?'macos-arm64':'macos-x64';
 const parent=f.receipts.find(entry=>entry.target===MAC_PACKAGE_TARGETS[label]);
 const root=mkdtempSync(join(tmpdir(),'mac-packaged-sidecar-'));context.after(()=>rmSync(root,{recursive:true,force:true}));
 const application=join(root,'actual fixture.app'),binary=join(application,'Contents/Resources/bin/formal-ai');
 mkdirSync(join(application,'Contents/Resources/bin'),{recursive:true});
 writeFileSync(binary,"#!/bin/sh\nprintf 'formal-ai 2.7.11\n'\n");chmodSync(binary,0o755);
 const packageFile=join(root,'formal-ai-desktop-'+label+'-2.7.11.dmg');writeFileSync(packageFile,'explicit DMG fixture bytes');
 const requests=[];
 const record=observeMacSigning(application,packageFile,{source:f.source,sourceSha256:f.bindings.sourceSha256,label,parentBytes:parent.bytes,
  runCommand:(command,args,options)=>{requests.push([command,args]);
   if(command===binary)return spawnSync(command,args,options);
   assert.equal(command,'codesign');return {status:0,signal:null,stdout:'',stderr:args.includes('--display')?'Signature=adhoc\n':''};}});
 assert.ok(requests.some(([command,args])=>command===binary&&args[0]==='--version'));
 assert.equal(record.package_executable_receipt.startup.stdout,'formal-ai 2.7.11\n');
 assert.equal(record.native_component.sha256,sha256(readFileSync(binary)));
 assert.equal(record.package_executable_receipt.parent.receipt_sha256,sha256(parent.bytes));
 assert.equal(validateSigningEvidence(record,f.source,label,f.bindings.sourceSha256,parent.bytes),'adhoc');
});
test('bytes that change during observed startup cannot become a certified receipt',context=>{
 const root=mkdtempSync(join(tmpdir(),'package-byte-race-'));context.after(()=>rmSync(root,{recursive:true,force:true}));
 const binary=join(root,'formal-ai');writeFileSync(binary,'before');
 assert.throws(()=>observePackagedExecutable(binary,()=>{writeFileSync(binary,'after');return {status:0,signal:null,stdout:'formal-ai 2.7.11\n',stderr:''};}),/changed during startup/u);
});
