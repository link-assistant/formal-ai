// Verified native artifacts reject incompatible inputs before running the original testcase.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {writeReceipt,verifyReceipt,runVerifiedTest} from '../../../scripts/native-test-artifact.mjs';
import YAML from 'yaml';
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
test('the selected unit transfer preserves the full original receipt while omitting unused executables',()=>{
 const f=fixture();try{
  for(const target of ['integration','source'])rmSync(join(f.directory,target));
  assert.throws(()=>verifyReceipt(f.directory,identity,f.binary),'full consumers still require all artifacts');
  assert.equal(runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root),0);
  assert.equal(readFileSync(join(f.root,'ran.txt'),'utf8'),'actual testcase ran\n');
 }finally{f.dispose();}
});
test('selected transfer refuses an empty selection, unknown target or malformed unused checksum',()=>{
 const f=fixture();try{
  for(const targets of [[],['unknown'],['unit','unit']]){
   assert.throws(()=>verifyReceipt(f.directory,identity,f.binary,{targets}));
  }
  const file=join(f.directory,'native-build-receipt.json'),receipt=JSON.parse(readFileSync(file,'utf8'));
  receipt.files.integration='invalid';writeFileSync(file,JSON.stringify(receipt));
  assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root));
  assert.equal(existsSync(join(f.root,'ran.txt')),false);
 }finally{f.dispose();}
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
test('self-coding transfer carries only its executed member and the original source-bound receipt',()=>{
 const release=YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8'));
 const steps=release.jobs['build-artifacts'].steps;
 const selected=steps.find(step=>step.with?.name==='formal-ai-self-coding-test-executable');
 assert(selected);
 assert.deepEqual(selected.with.path.trim().split('\n'),['dist/tests/unit','dist/tests/native-build-receipt.json']);
 assert(steps.indexOf(selected)>steps.findIndex(step=>step.name==='Bind native test artifacts to their actual build inputs'));
 assert.equal(steps.find(step=>step.with?.name==='formal-ai-test-executables').with.path,'dist/tests');
 const consumer=YAML.parse(readFileSync(new URL('../../../.github/workflows/agent-cli-e2e.yml',import.meta.url),'utf8'));
 const download=consumer.jobs['agent-cli-e2e'].steps.find(step=>step.name==='Download the current native self-coding test executable');
 assert.equal(download.with.name,selected.with.name);assert.equal(download.with.path,'dist/self-coding-tests');
 assert.equal(download['timeout-minutes'],3);
 assert(!download.if.includes('inputs.full-replay'),'ordinary pull requests must exercise the selected transfer');
 const replay=consumer.jobs['agent-cli-e2e'].steps.find(step=>step.name?.includes('Replay Hive Mind self-coding'));
 assert(!replay.if.includes('inputs.full-replay'));
 assert(replay.run.includes('native-test-artifact.mjs run dist/self-coding-tests release all agentic_coding::self_coding::self_coding_session_replays'));
});
