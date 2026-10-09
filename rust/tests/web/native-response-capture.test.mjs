import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawn} from 'node:child_process';
import {collectNativeResponseRecords} from '../../../scripts/lib/native-response-capture.mjs';
const caller='source_fixture::original';
const identity={'source-commit':'a'.repeat(40),'source-tree':'b'.repeat(40),'cargo-lock-sha256':'c'.repeat(64),'native-inputs-sha256':'d'.repeat(64),'test-inputs-sha256':'e'.repeat(64),compiler:'actual-process-fixture only, not Rust',target:'fixture-node-host',profile:'original-fixture',features:'fixture','rust-flags':'','encoded-rust-flags':'','cargo-profile-overrides':{}};
const longAnswer='失败 is authored data\r\n😀'.repeat(2400)+'\n';
const record=(processId,sequence,thread=caller)=>({schema:'native-response-observation/v1',entrypoint:sequence===1?'UniversalSolver::solve_with_history_probability_store_and_intent_cache':'FormalAiEngine::answer_with_memory','process-id':processId,sequence,'caller-thread':thread,prompt:'actual fixture request\nsecond line',history:[{role:'user',content:'first exact request'},{role:'assistant',content:'prior exact answer'}],'config-debug':'FixtureConfig { source: true }',context:{'fixture-only':true},response:{intent:'fixture',answer:longAnswer,confidence:1,evidence_links:[],links_notation:'(answer exact)\n',derivation_id:'fixture-source'}});
async function processObservation(mode='success') {
 const directory=mkdtempSync(join(tmpdir(),'response-process-fixture-'));
 const source=`const fs=require('node:fs');const dir=process.argv[1];const make=${record.toString()};const caller=${JSON.stringify(caller)};const longAnswer=${JSON.stringify(longAnswer)};fs.writeFileSync(dir+'/native-'+process.pid+'.jsonl',[make(process.pid,1),make(process.pid,2),make(process.pid,3,null)].map(JSON.stringify).join('\\n')+'\\n');${mode==='signal'?"process.kill(process.pid,'SIGTERM');":`process.exitCode=${mode==='exit1'?1:0};`}`;
 const child=spawn(process.execPath,['-e',source,directory],{stdio:['ignore','pipe','pipe']});
 const streams=[];child.stdout.on('data',b=>streams.push(b));child.stderr.on('data',b=>streams.push(b));
 const observed=await new Promise((resolve,reject)=>{child.on('error',reject);child.on('close',(exitCode,signal)=>resolve({processId:child.pid,exitCode,signal}));});
 const bytes=readFileSync(join(directory,'native-'+child.pid+'.jsonl'));rmSync(directory,{recursive:true,force:true});
 return {bytes,processId:child.pid,execution:{...observed,succeeded:observed.exitCode===0&&observed.signal===null,problems:[],completed:1,summary:{passed:1,failed:0,ignored:0}},identity,expectedIdentity:structuredClone(identity),caller};
}
function collect(fixture){return collectNativeResponseRecords(fixture.bytes,fixture);}
test('actual successful Node fixture process retains whole UTF8 answer, ordered history, both boundaries and unbound caller',async()=>{
 const fixture=await processObservation();const result=collect(fixture);
 assert.equal(result.bound.length,2);assert.equal(result.unbound.length,1);
 assert.deepEqual(result.bound.map(r=>r.entrypoint),['UniversalSolver::solve_with_history_probability_store_and_intent_cache','FormalAiEngine::answer_with_memory']);
 assert.equal(result.bound[1].response.answer,longAnswer);assert.equal(result.bound[1]['answer-utf8-bytes'],Buffer.byteLength(longAnswer));
 assert.ok(result.bound[1]['answer-utf8-bytes']>30000);assert.deepEqual(result.bound[1].history,record(1,1).history);
 assert.equal(result.unbound[0]['caller-thread'],null);assert.equal(result.bound[0]['config-debug'],'FixtureConfig { source: true }');
});
for(const mode of ['exit1','signal'])test('actual '+mode+' process cannot certify response despite a forged passing summary',async()=>{const awaitFixture=await processObservation(mode);assert.throws(()=>collect(awaitFixture));});
for(const [name,mutate] of [
 ['invalid confidence type',f=>{const records=f.bytes.toString().trimEnd().split('\n').map(JSON.parse);records[0].response.confidence=true;f.bytes=Buffer.from(records.map(JSON.stringify).join('\n')+'\n');}],
 ['array context',f=>{const records=f.bytes.toString().trimEnd().split('\n').map(JSON.parse);records[0].context=[];f.bytes=Buffer.from(records.map(JSON.stringify).join('\n')+'\n');}],
 ['missing terminal LF',f=>f.bytes=f.bytes.subarray(0,-1)],
 ['invalid UTF8',f=>f.bytes=Buffer.concat([Buffer.from([0xff]),f.bytes])],
 ['malformed JSON',f=>f.bytes=Buffer.from('{not-json}\n')],
 ['foreign process',f=>{const records=f.bytes.toString().trimEnd().split('\n').map(JSON.parse);records[0]['process-id']++;f.bytes=Buffer.from(records.map(JSON.stringify).join('\n')+'\n');}],
 ['duplicate sequence',f=>{const records=f.bytes.toString().trimEnd().split('\n').map(JSON.parse);records[1].sequence=records[0].sequence;f.bytes=Buffer.from(records.map(JSON.stringify).join('\n')+'\n');}],
 ['no matching actual caller',f=>f.caller='unrelated::test'],
 ['missing actual process',f=>f.execution.processId=null],
 ['missing actual completion',f=>f.execution.completed=0],
 ['compiler mismatch',f=>f.expectedIdentity.compiler='different compiler'],
 ['target mismatch',f=>f.expectedIdentity.target='different target'],
 ['encoded flag mismatch',f=>f.expectedIdentity['encoded-rust-flags']='different flags'],
 ['source mismatch',f=>f.expectedIdentity['native-inputs-sha256']='f'.repeat(64)],
 ['test input mismatch',f=>f.expectedIdentity['test-inputs-sha256']='f'.repeat(64)],
 ['missing source digest on both sides',f=>{delete f.identity['native-inputs-sha256'];delete f.expectedIdentity['native-inputs-sha256'];}]
])test(name+' refuses certification',async()=>{const f=await processObservation();f.identity=structuredClone(identity);mutate(f);assert.throws(()=>collect(f));});
