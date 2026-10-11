// Verified native artifacts reject incompatible inputs before running the original testcase.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {writeReceipt,verifyReceipt,runVerifiedTest} from '../../../scripts/native-test-artifact.mjs';
const identity={'source-commit':'a'.repeat(40),'source-tree':'b'.repeat(40),'cargo-lock-sha256':'c'.repeat(64),platform:'linux',architecture:'x64','rust-flags':'-Dwarnings',compiler:'actual producer compiler',profile:'release',features:'all'};
function fixture(){
 const root=mkdtempSync(join(tmpdir(),'native-test-artifact-')),directory=join(root,'tests'),binary=join(root,'formal-ai');mkdirSync(directory);
 const script='#!/bin/sh\nif [ "$1" = --list ]; then printf "suite::wanted: test\\n"; exit 0; fi\n[ "$1" = --exact ] && [ "$2" = suite::wanted ] || exit 9\nprintf "actual testcase ran\\n" > ran.txt\n';
 writeFileSync(join(directory,'unit'),script);for(const target of ['integration','source'])writeFileSync(join(directory,target),target+' executable bytes');writeFileSync(binary,'actual binary bytes');
 writeReceipt(directory,identity,binary);return {root,directory,binary,dispose:()=>rmSync(root,{recursive:true,force:true})};
}
test('same build identity and bytes execute the exact registered testcase',()=>{
 const f=fixture();try{assert.equal(runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root),0);assert.equal(readFileSync(join(f.root,'ran.txt'),'utf8'),'actual testcase ran\n');}finally{f.dispose();}
});
test('foreign source tree and build variants refuse before executing artifact bytes',()=>{
 const f=fixture();try{
  for(const [field,value] of [['source-commit','d'.repeat(40)],['source-tree','d'.repeat(40)],['cargo-lock-sha256','d'.repeat(64)],['platform','darwin'],['architecture','arm64'],['compiler','another compiler'],['rust-flags','--cfg incompatible'],['profile','debug'],['features','default']]){
   assert.throws(()=>runVerifiedTest(f.directory,{...identity,[field]:value},f.binary,'suite::wanted',f.root));assert.equal(existsSync(join(f.root,'ran.txt')),false);
  }
 }finally{f.dispose();}
});
test('tampered unit or runtime binary bytes are rejected before execution',()=>{
 for(const target of ['unit','binary']){const f=fixture();try{
  writeFileSync(target==='unit'?join(f.directory,'unit'):f.binary,'tampered');assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root));assert.equal(existsSync(join(f.root,'ran.txt')),false);
 }finally{f.dispose();}}
});
test('missing testcase and noncanonical receipt cannot silently run a different test',()=>{
 const f=fixture();try{
  assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::absent',f.root));assert.equal(existsSync(join(f.root,'ran.txt')),false);
  const path=join(f.directory,'native-build-receipt.json'),receipt=JSON.parse(readFileSync(path,'utf8'));receipt.files['../outside']='0'.repeat(64);writeFileSync(path,JSON.stringify(receipt));assert.throws(()=>verifyReceipt(f.directory,identity,f.binary));
 }finally{f.dispose();}
});
