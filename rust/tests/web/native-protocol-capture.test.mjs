import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {bytesDigest,checkedCapture} from '../../../scripts/capture-native-agentic-protocol.mjs';
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
  assert.equal(observed.steps.at(-1).result,original.steps.at(-1).result);
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
  assert.equal(receipt.stdout,legacy.steps.at(-1).result);
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
