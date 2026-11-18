#!/usr/bin/env node
// One actual native producer, with checked identities before any consumer execution.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync,chmodSync} from 'node:fs';
import {execFileSync,spawnSync} from 'node:child_process';
import {resolve,join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {augmentNativeTestIdentity,observeNativeTestPhase} from './lib/native-response-identity.mjs';
const RECEIPT='native-build-receipt.json';
const TARGETS=['unit','integration','source'];
const hash=path=>createHash('sha256').update(readFileSync(path)).digest('hex');
export function buildIdentity(cwd,profile,features,observe=null){
 return observeNativeTestPhase('build-identity',()=>{
 const git=args=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
 return augmentNativeTestIdentity(cwd,{'source-commit':git(['rev-parse','HEAD']),'source-tree':git(['rev-parse','HEAD^{tree}']),'cargo-lock-sha256':hash(join(cwd,'rust/Cargo.lock')),platform:process.platform,architecture:process.arch,'rust-flags':process.env.RUSTFLAGS??'',compiler:execFileSync('rustc',['-vV'],{cwd,encoding:'utf8'}).trim(),profile,features},process.env,observe);
 },observe);
}
export function writeReceipt(directory,identity,binary=resolve(directory,'../formal-ai'),observe=null){
 return observeNativeTestPhase('write-receipt',()=>{
 const files=Object.fromEntries(TARGETS.map(target=>[target,hash(join(directory,target))]));
 files.binary=hash(binary);
 const receipt={version:1,identity,files};
 writeFileSync(join(directory,RECEIPT),JSON.stringify(receipt,null,2)+'\n');return receipt;
 },observe);
}
export function verifyReceipt(directory,identity,binary,{targets=TARGETS,observe=null}={}){
 return observeNativeTestPhase('verify-receipt',()=>{
 const receipt=JSON.parse(readFileSync(join(directory,RECEIPT),'utf8'));
 assert.equal(receipt.version,1);
 assert.deepEqual(receipt.identity,identity);
 assert.deepEqual(Object.keys(receipt.files).sort(),[...TARGETS,'binary'].sort());
 assert.ok(Array.isArray(targets)&&targets.length>0&&new Set(targets).size===targets.length);
 assert.ok(targets.every(target=>TARGETS.includes(target)));
 for(const value of Object.values(receipt.files))assert.match(value,/^[a-f0-9]{64}$/u);
 for(const target of targets)assert.equal(hash(join(directory,target)),receipt.files[target]);
 assert.equal(hash(binary),receipt.files.binary);
 return receipt;
 },observe);
}
export function runVerifiedTest(directory,identity,binary,testName,cwd=process.cwd(),target='unit',observe=null){
 // Only the requested, receipt-bound target executes; unrelated files need not transfer.
 verifyReceipt(directory,identity,binary,{targets:[target],observe});
 const executable=resolve(directory,target);chmodSync(executable,0o755);
 const listing=observeNativeTestPhase('list-test',()=>execFileSync(executable,['--list','--format','terse'],{cwd,encoding:'utf8',maxBuffer:10e6}),observe);
 assert.equal(listing.split(/\r?\n/u).filter(line=>line===testName+': test').length,1);
 return observeNativeTestPhase('run-test',()=>{
  const result=spawnSync(executable,['--exact',testName,'--nocapture'],{cwd,stdio:'inherit'});
  if(result.error)throw result.error;
  assert.equal(result.signal,null);assert.equal(result.status,0);return result.status;
 },observe);
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const [mode,directory,profile,features,testName,target='unit']=process.argv.slice(2),cwd=process.cwd();
 assert.ok(directory&&profile&&features);
 const observe=event=>console.log(JSON.stringify({schema:'native-test-artifact-phase/v1',...event}));
 const identity=buildIdentity(cwd,profile,features,observe);
 if(mode==='write'||mode==='write-and-check'){
  writeReceipt(directory,identity,resolve(directory,'../formal-ai'),observe);
  if(mode==='write-and-check'){
   const cases=JSON.parse(readFileSync(testName,'utf8'));
   assert.ok(Array.isArray(cases)&&cases.length>0&&cases.length<=16);
   for(const entry of cases){
    assert.deepEqual(Object.keys(entry).sort(),['name','target']);
    assert.ok(typeof entry.name==='string'&&entry.name.length<=256&&/^[A-Za-z_][A-Za-z0-9_:]*$/u.test(entry.name),'invalid registered smoke name');
    const smokeObserver=event=>observe({...event,test:entry.name});
    runVerifiedTest(directory,identity,resolve('rust/target/release/formal-ai'),entry.name,cwd,entry.target,smokeObserver);
   }
  }
 }
 else if(mode==='run')runVerifiedTest(directory,identity,resolve('rust/target/release/formal-ai'),testName,cwd,target,observe);
 else assert.fail(mode);
}
