import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import {bytesDigest,checkedCapture} from '../../../scripts/capture-native-agentic-protocol.mjs';
const directory='docs/case-studies/issue-982/self-hosting/contract/';
const provenance=JSON.parse(readFileSync(directory+'session-v3-provenance.json'));
const legacy=JSON.parse(readFileSync(directory+'session.json'));
const observedBytes=readFileSync(directory+provenance.observed);
test('actual failed CI operand retains the complete immutable issue982 contract session',()=>{
 assert.equal(bytesDigest(observedBytes),provenance.observedSha256);
 const log=gunzipSync(readFileSync(directory+provenance.rawLog));
 assert.equal(bytesDigest(log),provenance.rawLogSha256);
 const operand=log.toString('utf8').split('\n').find(line=>line.includes(' right: ')&&line.includes('Create file memory-upgrade-contract.md'));
 assert.equal(JSON.parse(operand.split(' right: ')[1]),observedBytes.toString('utf8'));
 const job=JSON.parse(readFileSync(directory+provenance.jobReceipt));
 const run=JSON.parse(readFileSync(directory+provenance.runReceipt));
 assert.equal(job.id,113928931943);assert.equal(job.run_id,37957603076);
 assert.equal(run.head_sha,'8abb066dbe9895ba843ca690e0d9e9f21c12e571');
 assert.equal(job.conclusion,'failure');assert.equal(run.event,'pull_request');
 assert.ok(log.includes(Buffer.from('96e579831186bb93c657e807056cd7e41ea4bdd4')));
 const actual=checkedCapture(observedBytes,legacy),normalized=structuredClone(actual);
 assert.equal(actual.turns,legacy.turns+2);
 for(const index of [0,2]){
  assert.equal(actual.steps[index].tool,'read_file');
  for(const key of ['path','filePath','file_path'])assert.equal(actual.steps[index].arguments[key],'.formal-ai/general-change-plan.lino');
 }
 assert.ok(actual.steps[2].result.includes(legacy.task));
 assert.deepEqual(actual.tools_advertised,[...legacy.tools_advertised,'read_file']);
 normalized.steps.at(-1).result=JSON.parse(normalized.steps.at(-1).result).stdout;
 normalized.steps.splice(2,1);normalized.steps.splice(0,1);
 normalized.tools_advertised.pop();normalized.turns=legacy.turns;
 assert.deepEqual(normalized,legacy,'every original whole-session field and target byte stays exact');
});
test('actual native command observations refuse false completion or changed original target bytes',()=>{
 const actual=JSON.parse(observedBytes),receipt=JSON.parse(actual.steps.at(-1).result);
 for(const changes of [{schema:'unknown'},{command:'cat foreign.txt'},{exit_code:1},{exit_code:null},
  {complete:false},{timed_out:true},{truncated:true},{stdout:'wrong bytes'},{stderr:null}]){
  const rejected=structuredClone(actual);rejected.steps.at(-1).result=JSON.stringify({...receipt,...changes});
  assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(rejected)),legacy));
 }
 const changed=structuredClone(actual);changed.steps[3].arguments.content='changed original leaf';
 assert.throws(()=>checkedCapture(Buffer.from(JSON.stringify(changed)),legacy));
});
test('rollback remains explicitly uncaptured after the actual first assertion failure',()=>{
 assert.deepEqual(provenance.capturedCases,['contract']);
 assert.equal(provenance.rollback,'Not executed after the original first whole-session assertion failed; original legacy fixture and live native comparison remain required.');
 const source=readFileSync('rust/tests/integration/issue_982_self_hosting.rs','utf8');
 assert.ok(source.includes('assert_literal_capture_transition(committed_session, &rendered)'));
 assert.ok(source.includes('assert_eq!(committed_leaf, expected_leaf)'));
 assert.ok(source.includes('assert_eq!(arguments["content"], expected_leaf)'));
});
