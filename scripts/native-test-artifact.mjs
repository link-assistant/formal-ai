#!/usr/bin/env node
// One actual native producer, with checked identities before any consumer execution.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync,chmodSync} from 'node:fs';
import {execFileSync,spawnSync} from 'node:child_process';
import {resolve,join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {augmentNativeTestIdentity} from './lib/native-response-identity.mjs';
const RECEIPT='native-build-receipt.json';
const TARGETS=['unit','integration','source'];
const hash=path=>createHash('sha256').update(readFileSync(path)).digest('hex');
export function buildIdentity(cwd,profile,features){
 const git=args=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
 return augmentNativeTestIdentity(cwd,{'source-commit':git(['rev-parse','HEAD']),'source-tree':git(['rev-parse','HEAD^{tree}']),'cargo-lock-sha256':hash(join(cwd,'rust/Cargo.lock')),platform:process.platform,architecture:process.arch,'rust-flags':process.env.RUSTFLAGS??'',compiler:execFileSync('rustc',['-vV'],{cwd,encoding:'utf8'}).trim(),profile,features});
}
export function writeReceipt(directory,identity,binary=resolve(directory,'../formal-ai')){
 const files=Object.fromEntries(TARGETS.map(target=>[target,hash(join(directory,target))]));
 files.binary=hash(binary);
 const receipt={version:1,identity,files};
 writeFileSync(join(directory,RECEIPT),JSON.stringify(receipt,null,2)+'\n');return receipt;
}
export function verifyReceipt(directory,identity,binary){
 const receipt=JSON.parse(readFileSync(join(directory,RECEIPT),'utf8'));
 assert.equal(receipt.version,1);
 assert.deepEqual(receipt.identity,identity);
 assert.deepEqual(Object.keys(receipt.files).sort(),[...TARGETS,'binary'].sort());
 for(const target of TARGETS)assert.equal(hash(join(directory,target)),receipt.files[target]);
 assert.equal(hash(binary),receipt.files.binary);
 return receipt;
}
export function runVerifiedTest(directory,identity,binary,testName,cwd=process.cwd()){
 verifyReceipt(directory,identity,binary);
 const executable=resolve(directory,'unit');chmodSync(executable,0o755);
 const listing=execFileSync(executable,['--list','--format','terse'],{cwd,encoding:'utf8',maxBuffer:10e6});
 assert.equal(listing.split(/\r?\n/u).filter(line=>line===testName+': test').length,1);
 const result=spawnSync(executable,['--exact',testName,'--nocapture'],{cwd,stdio:'inherit'});
 if(result.error)throw result.error;
 assert.equal(result.signal,null);assert.equal(result.status,0);return result.status;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const [mode,directory,profile,features,testName]=process.argv.slice(2),cwd=process.cwd();
 assert.ok(directory&&profile&&features);
 const identity=buildIdentity(cwd,profile,features);
 if(mode==='write')writeReceipt(directory,identity);
 else if(mode==='run')runVerifiedTest(directory,identity,resolve('rust/target/release/formal-ai'),testName,cwd);
 else assert.fail(mode);
}
