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
  const observed=checkedCapture(bytes,original);
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
  {...actual,steps:actual.steps.map(step=>step.tool==='write_file'?{...step,arguments:{...step.arguments,content:'wrong observed bytes'}}:step)}])assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(changed)),legacy));
});
