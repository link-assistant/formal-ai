import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,mkdirSync,readFileSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {augmentNativeTestIdentity,trackedTestInputIdentity} from '../../../scripts/lib/native-response-identity.mjs';
import {writeReceipt} from '../../../scripts/native-test-artifact.mjs';
import {captureOriginalTests} from '../../../scripts/capture-native-responses.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const originalSource='#[test]\nfn original_case() { /* fixture only, not compiled or native */ }\n';
function fixture(mode='success'){
 const cwd=mkdtempSync(join(tmpdir(),'native-capture-source-fixture-'));const directory=join(cwd,'executables');mkdirSync(directory);
 for(const [path,body] of [['rust/src/lib.rs','// fixture\n'],['rust/Cargo.toml','[package]\nname="fixture"\n[[test]]\nname="unit"\npath="tests/unit/mod.rs"\n'],['rust/tests/unit/mod.rs',mode==='unregistered'?'':mode==='opaque'?'include!("unknown.rs"); #[path="original.rs"] mod actual_source_module;':'#[path="original.rs"] mod actual_source_module;'],['rust/Cargo.lock','fixture lock\n'],['rust/tests/unit/original.rs',originalSource],['.gitignore','rust/src/ignored.rs\n'],['fixture-data.txt','whole tracked input\n']]){const full=join(cwd,path);mkdirSync(join(full,'..'),{recursive:true});writeFileSync(full,body);}
 const git=args=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();git(['init','-q']);git(['add','.']);git(['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-qm','actual fixture bytes']);
 const compiler='Node fixture metadata only; no Rust compilation\nhost: fixture-host';
 const identity=augmentNativeTestIdentity(cwd,{'source-commit':git(['rev-parse','HEAD']),'source-tree':git(['rev-parse','HEAD^{tree}']),'cargo-lock-sha256':sha(readFileSync(join(cwd,'rust/Cargo.lock'))),compiler,profile:'fixture-original',features:'fixture','rust-flags':process.env.RUSTFLAGS??''});
 const program=`#!/usr/bin/env node
const fs=require('node:fs');const args=process.argv.slice(2),caller='actual_source_module::original_case',mode=${JSON.stringify(mode)};
if(args.includes('--list')){console.log((mode==='wrong-namespace'?'different_module::original_case':caller)+': test');if(mode==='ambiguous')console.log(caller+': test');if(mode==='leaf-decoy')console.log('different_module::original_case: test');process.exit(0);}
if(!args.includes('--nocapture')||!args.includes(caller))process.exit(3);
const record={schema:'native-response-observation/v1',entrypoint:'FormalAiEngine::answer_with_memory','process-id':process.pid,sequence:1,'caller-thread':caller,prompt:'process fixture only',history:[],'config-debug':'Actual fixture config',context:{fixture:true},response:{intent:'fixture',answer:'😀\\r\\n'.repeat(10000)+'\\n',confidence:1,evidence_links:[],links_notation:'(fixture response)\\n',derivation_id:'fixture'}};
if(mode!=='missing')fs.writeFileSync(process.env.FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR+'/native-'+process.pid+'.jsonl',JSON.stringify(record)+'\\n');
if(mode==='observer-error')console.error('[native-response-observation] failed to retain actual response: fixture disk failure');
console.log('test '+caller+' ... ok');console.log('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;');
if(mode==='exit1')process.exitCode=1;if(mode==='signal')process.kill(process.pid,'SIGTERM');
`;
 for(const target of ['unit','integration','source'])writeFileSync(join(directory,target),program);
 const binary=join(directory,'formal-ai');writeFileSync(binary,program);writeReceipt(directory,identity,binary);
 return {cwd,directory,binary,identity,outputDirectory:join(cwd,'captures'),selections:['rust/tests/unit/original.rs::original_case'],close:()=>rmSync(cwd,{recursive:true,force:true})};
}
test('checked actual Node harness process retains original source, complete answer, actual PID and distinct channels',async()=>{
 const f=fixture();try{const report=await captureOriginalTests({...f,expectedIdentity:f.identity});assert.equal(report.selected,1);assert.equal(report.accepted,1);assert.equal(report.expectedAnswersUpdated,false);
  assert.equal(readFileSync(join(f.cwd,'rust/tests/unit/original.rs'),'utf8'),originalSource);
  const observed=report.observations[0];assert.ok(observed.execution.batches[0].processId>0);assert.equal(observed.captured.bound[0]['process-id'],observed.execution.batches[0].processId);
  assert.equal(observed.captured.bound[0].response.answer,'😀\r\n'.repeat(10000)+'\n');assert.ok(observed.captured.bound[0]['answer-utf8-bytes']>30000);
  assert.equal(readFileSync(join(f.outputDirectory,'000/stderr.bin')).length,0);assert.ok(readFileSync(join(f.outputDirectory,'000/stdout.bin')).length>0);
 }finally{f.close();}
});
for(const mode of ['missing','observer-error','exit1','signal'])test('actual '+mode+' outcome preserves failure instead of accepting printed success',async()=>{const f=fixture(mode);try{const report=await captureOriginalTests({...f,expectedIdentity:f.identity});assert.equal(report.accepted,0);assert.equal(report.observations[0].captured,null);assert.equal(typeof report.observations[0].error,'string');}finally{f.close();}});
test('ambiguous original function listing refuses execution',async()=>{const f=fixture('ambiguous');try{await assert.rejects(captureOriginalTests({...f,expectedIdentity:f.identity}),/uniquely listed/);}finally{f.close();}});
test('altered executable is refused before original process dispatch',async()=>{const f=fixture();try{writeFileSync(join(f.directory,'unit'),'tampered');await assert.rejects(captureOriginalTests({...f,expectedIdentity:f.identity}));}finally{f.close();}});
for(const path of ['rust/src/lib.rs','rust/tests/unit/original.rs','fixture-data.txt'])test('actual modified '+path+' cannot share original producer identity',async()=>{const f=fixture();try{writeFileSync(join(f.cwd,path),'different bytes\n');await assert.rejects(captureOriginalTests({...f,expectedIdentity:f.identity}),/uncommitted/);}finally{f.close();}});
test('ignored actual native input is refused',()=>{const f=fixture();try{writeFileSync(join(f.cwd,'rust/src/ignored.rs'),'hidden compiled source');assert.throws(()=>augmentNativeTestIdentity(f.cwd,f.identity),/untracked native/);}finally{f.close();}});
test('untracked test helper is refused',()=>{const f=fixture();try{writeFileSync(join(f.cwd,'rust/tests/unit/untracked.rs'),'new helper');assert.throws(()=>trackedTestInputIdentity(f.cwd),/untracked test/);}finally{f.close();}});
test('target, encoded flags and actual profile overrides are retained without feature/profile substitution',()=>{const f=fixture();try{const changed=augmentNativeTestIdentity(f.cwd,f.identity,{CARGO_BUILD_TARGET:'different-real-target',CARGO_ENCODED_RUSTFLAGS:'-C\x1fdebuginfo=0',CARGO_PROFILE_RELEASE_LTO:'thin'});assert.equal(changed.target,'different-real-target');assert.equal(changed['encoded-rust-flags'],'-C\x1fdebuginfo=0');assert.deepEqual(changed['cargo-profile-overrides'],{CARGO_PROFILE_RELEASE_LTO:'thin'});assert.equal(changed.profile,'fixture-original');assert.equal(changed.features,'fixture');}finally{f.close();}});

test('unrelated same-leaf listing cannot replace or make ambiguous the exact registered caller',async()=>{const f=fixture('leaf-decoy');try{const report=await captureOriginalTests({...f,expectedIdentity:f.identity});assert.equal(report.accepted,1);assert.equal(report.observations[0].caller,'actual_source_module::original_case');assert.equal(report.observations[0].trace[1].kind,'path');assert.equal(report.observations[0].callerListedByCheckedExecutable,true);}finally{f.close();}});
for(const mode of ['wrong-namespace','unregistered','opaque'])test('actual '+mode+' source/listing refuses original dispatch',async()=>{const f=fixture(mode);try{await assert.rejects(captureOriginalTests({...f,expectedIdentity:f.identity}),mode==='wrong-namespace'?/uniquely listed/:mode==='opaque'?/opaque item macro/:/no unique registration/);assert.equal(existsSync(f.outputDirectory),false);}finally{f.close();}});
