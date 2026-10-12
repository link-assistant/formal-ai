import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {deflateRawSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
import {deriveFailureOutputContract,parseCanonicalOutputs,validateProducerOutputReceipt,
  writeProducerOutputReceipt,LOADED_WRITER_SHA256,preflightProducerOutputArchive} from '../../../scripts/release-producer-output-receipt.mjs';
import {productionProducerProfile,validateProducerObservation,joinProductionProducerEvidence}
  from '../../../scripts/lib/release-completion-producer.mjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const workflowPath='.github/workflows/desktop-release.yml';
const workflowBytes=fs.readFileSync(path.join(root,workflowPath));
const job=YAML.parse(workflowBytes.toString()).jobs.finalize;
const contract=deriveFailureOutputContract(job);
const guard=contract.guards[0];
const binding={repository:'example/project',runId:11,runAttempt:2,runHead:'a'.repeat(40),
  protocolCommit:'a'.repeat(40),protocolTree:'b'.repeat(40),workflowPath,
  workflowSha256:digest(workflowBytes),writerSha256:LOADED_WRITER_SHA256,jobId:'finalize'};
const makeRecord=()=>({schema:'formal-ai-producer-outputs/v1',...binding,stepId:'manifest',outputs:{incomplete:''}});
const receiptBytes=record=>Buffer.from(JSON.stringify(record,null,2)+'\n');
test('current source guard derives output witness obligations without delivery authority',()=>{
  assert.equal(contract.guards.length,1);assert.equal(guard.output,'incomplete');
  assert.equal(guard.producerId,'manifest');assert.equal(contract.requiredReceiptSteps.length,2);
  const result=validateProducerOutputReceipt(receiptBytes(makeRecord()),binding,contract);
  assert.equal(result.authenticated,false);assert.equal(result.skippedAdmission,false);
  assert.equal(result.publicationComplete,false);
});
test('command substitutions escaped quotes and unknown expressions remain regular obligations',()=>{
  for(const value of ['echo "$(touch /tmp/forbidden)"\nexit 1',
    'echo "`touch /tmp/forbidden`"\nexit 1',
    'echo "\\\"; touch /tmp/forbidden; echo \\\""\nexit 1',
    'echo "${{ steps.foreign.outputs.incomplete }}"\nexit 1',
    'echo "${UNTRUSTED}"\nexit 1','echo "closed"\nexit 0','echo "closed"\nexit 1\ntouch /tmp/forbidden']){
    const changed=structuredClone(job);changed.steps.find(step=>step.name===guard.name).run=value;
    assert.equal(deriveFailureOutputContract(changed),null,value);
  }
});
test('unknown predicates cannot become a failure-output obligation',()=>{
  for(const value of ["${{ always() }}","${{ steps.manifest.outputs.incomplete == '' }}",
    "${{ steps.manifest.outputs.incomplete != '' || true }}"]){
    const changed=structuredClone(job);changed.steps.find(step=>step.name===guard.name).if=value;
    assert.equal(deriveFailureOutputContract(changed),null);
  }
});
test('changed or omitted receipt producer upload and attestation refuse',()=>{
  for(const change of [value=>value.steps.find(step=>step.id==='manifest').env.RELEASE_PRODUCER_OUTPUT_STEP='foreign',
    value=>value.steps.find(step=>step.id==='manifest').env.RELEASE_PRODUCER_OUTPUT_KEYS='other',
    value=>value.steps.find(step=>step.name===contract.requiredReceiptSteps[0]).if='false',
    value=>value.steps.find(step=>step.name===contract.requiredReceiptSteps[1]).with.name='foreign',
    value=>value.steps.splice(value.steps.findIndex(step=>step.name===contract.requiredReceiptSteps[0]),1)]){
    const changed=structuredClone(job);change(changed);assert.throws(()=>deriveFailureOutputContract(changed));
  }
});
test('explicit canonical empty output differs from missing duplicate partial unsafe or multiline output',()=>{
  assert.deepEqual(parseCanonicalOutputs(Buffer.from('incomplete=\n'),['incomplete']),{incomplete:''});
  for(const value of ['', 'other=\n','incomplete=', 'incomplete=\nincomplete=\n',
    'incomplete<<EOF\n\nEOF\n','incomplete=$(touch /tmp/forbidden)\n','incomplete=`false`\n',
    'incomplete=""\n','incomplete=\r\n'])assert.throws(()=>parseCanonicalOutputs(Buffer.from(value),['incomplete']));
});
test('nonempty incomplete output is observed truthfully without delivery credit',()=>{
  const record=makeRecord();record.outputs.incomplete='native:linux-x64 signing:macos-arm64';
  const checked=validateProducerOutputReceipt(receiptBytes(record),binding,contract);
  assert.equal(checked.outputs.incomplete,record.outputs.incomplete);assert.equal(checked.skippedAdmission,false);
});
test('foreign source run attempt workflow writer job or producer receipt refuses',()=>{
  for(const [key,value] of Object.entries({repository:'foreign/project',runId:12,runAttempt:1,
    runHead:'c'.repeat(40),protocolCommit:'c'.repeat(40),protocolTree:'c'.repeat(40),
    workflowPath:'.github/workflows/foreign.yml',workflowSha256:'c'.repeat(64),
    writerSha256:'c'.repeat(64),jobId:'foreign',stepId:'foreign'})){
    const record=makeRecord();record[key]=value;assert.throws(()=>validateProducerOutputReceipt(receiptBytes(record),binding,contract),key);
  }
});
test('missing extra or duplicate JSON fields cannot invent canonical producer output',()=>{
  const record=makeRecord();delete record.outputs.incomplete;
  assert.throws(()=>validateProducerOutputReceipt(receiptBytes(record),binding,contract));
  const extra=makeRecord();extra.authenticated=true;
  assert.throws(()=>validateProducerOutputReceipt(receiptBytes(extra),binding,contract));
  const duplicate=receiptBytes(makeRecord()).toString().replace('"outputs": {','"runId": 11,\n  "outputs": {');
  assert.throws(()=>validateProducerOutputReceipt(Buffer.from(duplicate),binding,contract));
});
test('physical writer reads exact outputs and refuses symlink omitted key and existing destination',()=>{
  const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'release-output-writer-')));
  try{
    const outputs=path.join(directory,'outputs'),workflow=path.join(directory,'workflow.yml'),selection=path.join(directory,'selection.json');
    fs.writeFileSync(outputs,'incomplete=\n');fs.writeFileSync(workflow,workflowBytes);
    fs.writeFileSync(selection,JSON.stringify({commit:binding.protocolCommit,tree:binding.protocolTree}));
    const environment={GITHUB_WORKFLOW_REF:'example/project/'+workflowPath+'@refs/heads/main',
      GITHUB_REPOSITORY:'example/project',GITHUB_JOB:'finalize',GITHUB_RUN_ID:'11',GITHUB_RUN_ATTEMPT:'2',
      GITHUB_SHA:binding.runHead,RELEASE_PRODUCER_PROTOCOL_COMMIT:binding.protocolCommit,
      RELEASE_PRODUCER_OUTPUT_STEP:'manifest',RELEASE_PRODUCER_OUTPUT_KEYS:'incomplete'};
    const destination=path.join(directory,'receipt.json');
    const result=writeProducerOutputReceipt(outputs,workflow,selection,destination,environment);
    assert.equal(result.authenticated,false);assert.equal(result.publicationComplete,false);
    assert.equal(validateProducerOutputReceipt(fs.readFileSync(destination),binding,contract).outputs.incomplete,'');
    assert.throws(()=>writeProducerOutputReceipt(outputs,workflow,selection,destination,environment));
    fs.writeFileSync(outputs,'other=\n');
    assert.throws(()=>writeProducerOutputReceipt(outputs,workflow,selection,path.join(directory,'missing.json'),environment));
    const link=path.join(directory,'linked');fs.symlinkSync(outputs,link);
    assert.throws(()=>writeProducerOutputReceipt(link,workflow,selection,path.join(directory,'symlink.json'),environment));
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
test('source-registered skipped guard still refuses every public fake output frame',()=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'release-guard-source-'));
  const names=['scripts/lib/release-completion-producer.mjs','scripts/release-producer-output-receipt.mjs',
    'scripts/github-artifact-by-identifier.mjs','scripts/qualified-data-artifact-profiles.mjs',
    'scripts/lib/native-session-projections.mjs','scripts/lib/native-response-capture.mjs',workflowPath];
  try{
    for(const name of names){const destination=path.join(directory,name);fs.mkdirSync(path.dirname(destination),{recursive:true});fs.writeFileSync(destination,fs.readFileSync(path.join(root,name)));}
    const git=args=>execFileSync('git',args,{cwd:directory,encoding:'utf8'}).trim();
    git(['init','-q']);git(['config','user.name','Source fixture']);git(['config','user.email','fixture@example.invalid']);
    git(['remote','add','origin','https://github.com/example/project.git']);git(['add','.']);git(['commit','-qm','Guard source fixture']);
    const head=git(['rev-parse','HEAD']);const profile=productionProducerProfile(directory,head,workflowPath,'finalize');
    const run={id:11,run_attempt:2,status:'completed',event:'push',head_branch:'main',head_sha:head,
      workflow_id:3,path:workflowPath,repository:{full_name:'example/project'},head_repository:{full_name:'example/project'}};
    const input={defaultBranch:'main',run,attempt:structuredClone(run),workflow:{id:3,path:workflowPath},
      job:{id:4,run_id:11,head_sha:head,name:profile.jobName,status:'completed',conclusion:'success',
        steps:profile.requiredSteps.map(step=>({name:step.name,status:'completed',conclusion:'success'}))}};
    input.job.steps.find(step=>step.name===guard.name).conclusion='skipped';
    for(const fake of [{authenticated:true,outputs:{incomplete:''}},validateProducerOutputReceipt(receiptBytes(makeRecord()),binding,contract)]){
      assert.throws(()=>validateProducerObservation(profile,{...input,outputProof:fake,guardOutputProof:fake,canonicalOutputs:fake}),/authenticated canonical guard output/);
      assert.throws(()=>validateProducerObservation(profile,input,fake),/authenticated canonical guard output/);
      assert.throws(()=>joinProductionProducerEvidence([profile],[fake]));
    }
    input.job.steps.find(step=>step.name===guard.name).conclusion='success';
    assert.throws(()=>validateProducerObservation(profile,input),/failure-only guard must be skipped/);
    input.job.steps[0].conclusion='skipped';assert.throws(()=>validateProducerObservation(profile,input));
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('tiny archive bound applies to actual inflated bytes before maintained CRC decoding',()=>{
  const zip=(bytes,declared=bytes.length)=>{
    const compressed=deflateRawSync(bytes),start=31+compressed.length;
    const archive=Buffer.alloc(start+47+22);
    archive.writeUInt32LE(0x04034b50,0);archive.writeUInt16LE(8,8);archive.writeUInt16LE(1,26);
    archive[30]=97;compressed.copy(archive,31);
    archive.writeUInt32LE(0x02014b50,start);archive.writeUInt16LE(8,start+10);
    archive.writeUInt32LE(compressed.length,start+20);archive.writeUInt32LE(declared,start+24);
    archive.writeUInt16LE(1,start+28);archive[start+46]=97;
    const end=start+47;archive.writeUInt32LE(0x06054b50,end);
    archive.writeUInt16LE(1,end+8);archive.writeUInt16LE(1,end+10);
    archive.writeUInt32LE(47,end+12);archive.writeUInt32LE(start,end+16);
    return archive;
  };
  assert.doesNotThrow(()=>preflightProducerOutputArchive(zip(Buffer.from('{}\n'))));
  assert.throws(()=>preflightProducerOutputArchive(zip(Buffer.alloc(1024*1024,97),3)));
  assert.throws(()=>preflightProducerOutputArchive(zip(Buffer.alloc(65537,97))));
  assert.throws(()=>preflightProducerOutputArchive(Buffer.alloc(256*1024+1)));
});
