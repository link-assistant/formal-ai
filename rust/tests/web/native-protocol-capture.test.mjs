import {composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {installNativeCaptureObservationHost} from '../../../scripts/lib/native-capture-observation.mjs';
import {completeOwnedSourceReadFrame} from '../../../js/agentic/tool_result.mjs';
import {ownedCreationReadbackCommand} from '../../../js/agentic/progress_command_readback.mjs';
before(installNativeCaptureObservationHost);
import {before,test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {bytesDigest,checkedCapture,captureVerificationStdout} from '../../../scripts/capture-native-agentic-protocol.mjs';
const contract=JSON.parse(readFileSync('docs/case-studies/pull-request-1188/native-protocol-capture-provenance.json','utf8'));
test('immutable legacy session bytes retain their recorded provenance',()=>{
 for(const record of contract.captures)assert.equal(bytesDigest(readFileSync(record.legacy)),record['legacy-sha256']);
});
test('versioned sessions preserve complete actual native comparison operands',()=>{
 assert.equal(contract['pull-request-head'],'4a560cdb5a47dd3d175ca16adf6d521619437a4e');
 assert.equal(contract['source-commit'],'08295d4b7a8e57766c235a7ed908f5cd05f71a45');
 assert.equal(contract['source-tree'],'2ddcb2797fe522f4c9335e445d8d3c37e5406ba8');
 for(const record of contract.captures.filter(record=>record.observed)){
  const original=JSON.parse(readFileSync(record.legacy,'utf8')),bytes=readFileSync(record.observed);
  assert.equal(record.run,37870721512);
  assert.equal(record.job,record.id==='self-coding'?113640815027:113640815107);
  assert.equal(bytesDigest(bytes),record['observed-sha256']);
  const observed=checkedCapture(bytes,original,{historicalRawComparison:true});
  assert.deepEqual(observed.tools_advertised,[...original.tools_advertised,'read_file']);
  assert.equal(observed.steps.length,5);assert.equal(observed.turns,6);
  assert.deepEqual(observed.steps.map(step=>step.tool),['read_file','write_file','read_file','write_file','run_command']);
  assert.equal(observed.steps.at(-1).result,captureVerificationStdout(original.steps.at(-1)));
  assert.equal(bytes.at(-1)===10,record.id!=='self-coding');
 }
});
test('missing template capture remains explicit rather than synthesizing an observation',()=>{
 const record=contract.captures.find(record=>record.id==='ci-template');
 assert.equal(record.observed,undefined);assert.match(record.status,/Missing actual/);
});
test('capture validation refuses a foreign task or turn-cap outcome',()=>{
 const record=contract.captures.find(record=>record.observed),legacy=JSON.parse(readFileSync(record.legacy,'utf8')),actual=JSON.parse(readFileSync(record.observed,'utf8'));
 for(const changed of [{...actual,task:'foreign task'},{...actual,hit_turn_cap:true},{...actual,tools_advertised:['duplicate','duplicate']},
  {...actual,steps:actual.steps.filter(step=>step.tool!=='write_file')},
  {...actual,steps:actual.steps.filter(step=>step.tool!=='run_command')},
  {...actual,steps:actual.steps.map(step=>step.tool==='write_file'?{...step,arguments:{...step.arguments,content:'wrong observed bytes'}}:step)}])assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(changed)),legacy,{historicalRawComparison:true}));
});

test('actual CI sessions retain complete process receipts and immutable failed producer provenance',()=>{
 const selection=JSON.parse(readFileSync('docs/case-studies/pull-request-1188/native-protocol-captures/8abb066db/posthoc-validation.json','utf8'));
 const provenanceBytes=readFileSync(selection.provenance.path);
 assert.equal(bytesDigest(provenanceBytes),selection.provenance.sha256);
 const provenance=JSON.parse(provenanceBytes);
 assert.equal(provenance.run,'37957603176');
 assert.equal(provenance.identity['source-commit'],'96e579831186bb93c657e807056cd7e41ea4bdd4');
 assert.equal(provenance.identity['source-tree'],'21e50375d2de386a9932cdee34c9450ca4d09828');
 assert.ok(provenance.captures.every(record=>record.status==='failed'&&record.exit===0&&record.signal===null));
 for(const record of selection.captures){
  const legacy=JSON.parse(readFileSync(contract.captures.find(item=>item.id===record.id).legacy,'utf8'));
  const bytes=readFileSync(record.path);assert.equal(bytesDigest(bytes),record.sha256);
  const actual=checkedCapture(bytes,legacy);
  assert.equal(actual.steps.length,5);assert.equal(actual.turns,6);
  const final=actual.steps.at(-1),receipt=JSON.parse(final.result);
  assert.equal(receipt.stdout,captureVerificationStdout(legacy.steps.at(-1)));
  for(const change of [{schema:'unknown'},{command:'cat foreign.txt'},{exit_code:1},{exit_code:null},
   {complete:false},{timed_out:true},{truncated:true},{stdout:'wrong bytes'},{stderr:null}]){
   const rejected=structuredClone(actual);rejected.steps.at(-1).result=JSON.stringify({...receipt,...change});
   assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(rejected)),legacy));
  }
  const missing=structuredClone(actual);const partial={...receipt};delete partial.complete;
  missing.steps.at(-1).result=JSON.stringify(partial);
  assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(missing)),legacy));
  const bare=structuredClone(actual);bare.steps.at(-1).result=receipt.stdout;
  assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(bare)),legacy));
  const foreign=structuredClone(actual);foreign.steps.at(-1).arguments.command='cat foreign.txt';
  assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(foreign)),legacy));
 }
});

function literalFixture(){
 const task='Create file safe.txt containing owned bytes',write={tool:'write_file',arguments:{path:'safe.txt',content:'owned bytes'},result:'wrote 11 byte(s) to safe.txt'};
 const verification={tool:'run_command',arguments:{command:'cat safe.txt'},result:'owned bytes'};
 const legacy={task,steps:[write,verification]};
 const read={tool:'read_file',arguments:{path:'safe.txt'},result:completeOwnedSourceReadFrame('owned bytes')};
 const actual={task,tools_advertised:['write_file','read_file','run_command'],steps:[write,read],turns:3,hit_turn_cap:false,final_answer:'Observed owned bytes'};
 return{legacy,actual,write,read};
}
test('file-only original creation qualifies exact fresh provider Read without requiring the legacy chosen cat',()=>{
 const {legacy,actual}=literalFixture();assert.equal(checkedCapture(Buffer.from(JSON.stringify(actual)),legacy).steps.length,2);
});
test('literal capture refuses failed writes, pre-write or stale reads, unadvertised reads and foreign or opaque bytes',()=>{
 const {legacy,actual,write,read}=literalFixture();
 const rejected=[{...actual,steps:[{...write,result:'Error: write failed'},read]},
  {...actual,steps:[read,write]},
  {...actual,steps:[write,read,{...write,result:'Error: later write failed'}]},
  {...actual,tools_advertised:['write_file']},
  {...actual,steps:[write,{...read,arguments:{path:'foreign.txt'}}]},
  {...actual,steps:[write,{...read,result:completeOwnedSourceReadFrame('foreign bytes')}]},
  {...actual,steps:[write,{...read,result:'owned bytes'}]},
  {...actual,steps:[write,{...read,result:completeOwnedSourceReadFrame('owned bytes'),source_read:{path:'safe.txt',complete:false,format:'opaque'}}]}];
 for(const session of rejected)assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(session)),legacy));
});
test('owned framed Run remains an independent literal readback proof with exact command and bytes',()=>{
 const {legacy,actual,write}=literalFixture(),command=ownedCreationReadbackCommand('safe.txt');
 const receipt={schema:'command-execution-receipt/v1',command,exit_code:0,complete:true,truncated:false,timed_out:false,stdout:'(owned bytes)',stderr:''};
 const session={...actual,steps:[write,{tool:'run_command',arguments:{command},result:JSON.stringify(receipt)}]};
 assert.equal(checkedCapture(Buffer.from(JSON.stringify(session)),legacy).steps.length,2);
 assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify({...session,steps:[write,{...session.steps[1],result:JSON.stringify({...receipt,exit_code:1})}]})),legacy));
});
test('a task requesting program execution cannot replace its complete real command result with a Read',()=>{
 const {legacy,actual,write,read}=literalFixture();
 const task='Create file safe.txt containing owned bytes. Then run the command wc -c safe.txt.';
 const programLegacy={...legacy,task,steps:[write,{tool:'run_command',arguments:{command:'wc -c safe.txt'},result:'11 safe.txt\n'}]};
 assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify({...actual,task,steps:[write,read]})),programLegacy));
 const receipt={schema:'command-execution-receipt/v1',command:'wc -c safe.txt',exit_code:0,complete:true,truncated:false,timed_out:false,stdout:'11 safe.txt\n',stderr:''};
 const session={...actual,task,steps:[write,{tool:'run_command',arguments:{command:receipt.command},result:JSON.stringify(receipt)}]};
 assert.equal(checkedCapture(Buffer.from(JSON.stringify(session)),programLegacy).steps.length,2);
 assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify({...session,steps:[write,{...session.steps[1],result:JSON.stringify({...receipt,stdout:'wrong'})}]})),programLegacy));
});

test('all three immutable file-only originals retain their exact literal command and byte contract',()=>{
 for(const record of contract.captures){
  const legacy=JSON.parse(readFileSync(record.legacy,'utf8')),plan=composeGeneralChangePlan(legacy.task);
  const target=legacy.steps.filter(step=>step.tool==='write_file').at(-1),verification=legacy.steps.filter(step=>step.tool==='run_command').at(-1);
  assert.equal(plan.mode,'literal_file');assert.equal(plan.target,target.arguments.path);
  assert.equal(plan.content,target.arguments.content);assert.equal(plan.verification_command,verification.arguments.command);
  assert.equal(captureVerificationStdout(verification),plan.content);
 }
});
test('a complete literal Create cannot replace a legacy program command with a Read even when its stdout equals the source bytes',()=>{
 const {legacy,actual,write}=literalFixture();
 const programLegacy={...legacy,steps:[write,{tool:'run_command',arguments:{command:'node safe.txt'},result:'owned bytes'}]};
 assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(actual)),programLegacy));
});
