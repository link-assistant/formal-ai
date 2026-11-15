// Declared mock verification and controlled bytes only; this does not establish real publication authority.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {verifyAuthoritativeTransfer, signedSubjectAttempts} from '../../../scripts/release-extension-policy.mjs';
import {transferFromAuthenticatedGithub} from '../../../scripts/release-extension-github.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const vscodeJob = 'Package VS Code extension (.vsix)';
const sourceJob = 'Publish SHA256SUMS.txt + provenance';
function fixture(latest = 2) {
  const expected = {repository:'owner/repository',tag:'v1.2.3',sourceCommit:'a'.repeat(40),
    sourceTree:'b'.repeat(40),protocolCommit:'c'.repeat(40),runId:11,attempt:latest,
    consumerCheckJobId:44,consumerRunId:10,consumerAttempt:1};
  const name = 'formal-ai-vscode-1.2.3.vsix', sourceName = 'formal-ai-native-source-1.2.3.json';
  const artifactBytes = Buffer.from('CONTROLLED TEXT NOT A REAL VSIX\n');
  const sourceBytes = Buffer.from(JSON.stringify({version:1,release_tag:expected.tag,
    source_commit:expected.sourceCommit,source_tree:expected.sourceTree,producer_run:'11'}));
  const run = attempt => ({id:11,run_attempt:attempt,repository:{full_name:expected.repository},
    head_repository:{full_name:expected.repository},head_branch:'main',path:'.github/workflows/desktop-release.yml',
    event:'workflow_run',status:'completed',conclusion:attempt === 1 ? 'failure' : 'success',head_sha:expected.protocolCommit});
  const jobs = attempt => ({total_count:2,jobs:[vscodeJob,sourceJob].map((name,index)=>({
    name,id:100+attempt*10+index,run_id:11,run_attempt:attempt,status:'completed',conclusion:'success',head_sha:expected.protocolCommit}))});
  const evidence = {run:run(latest),jobs:jobs(latest).jobs,
    consumerChecks:{id:44,run_id:10,run_attempt:1,status:'completed',conclusion:'success'},
    release:{tag_name:expected.tag,draft:false,prerelease:false,assets:[
      {id:33,name,state:'uploaded',size:artifactBytes.length,digest:'sha256:'+sha(artifactBytes)},
      {id:34,name:sourceName},{id:35,name:'SHA256SUMS.txt'}]},
    tagCommit:{sha:expected.sourceCommit,commit:{tree:{sha:expected.sourceTree}}},
    artifactBytes,sourceBytes,checksumBytes:Buffer.from(sha(artifactBytes)+'  '+name+'\n'),
    producerProofs:[{name,attempt:1,run:run(1),jobs:jobs(1)},
      {name:sourceName,attempt:latest,run:run(latest),jobs:jobs(latest)}]};
  const state = {expected,evidence,name,sourceName,artifactAttempts:[1],sourceAttempts:[1,latest],verification:[],reads:[]};
  state.verify = request => {
    state.verification.push(request);
    return (request.name === name ? state.artifactAttempts : state.sourceAttempts).map(attempt => {
      const invocation = 'https://github.com/'+expected.repository+'/actions/runs/11/attempts/'+attempt;
      const record = {verificationResult:{signature:{certificate:{
        runInvocationURI:invocation,buildSignerURI:'https://github.com/'+expected.repository+'/.github/workflows/desktop-release.yml@refs/heads/main',
        buildSignerDigest:expected.protocolCommit,sourceRepositoryURI:'https://github.com/'+expected.repository,
        issuer:'https://token.actions.githubusercontent.com',runnerEnvironment:'github-hosted'}},
        statement:{predicateType:'https://slsa.dev/provenance/v1',subject:[{name:request.name,digest:{sha256:sha(request.bytes)}}],
          predicate:{runDetails:{metadata:{invocationId:invocation}}}}}};
      state.mutateSignature?.(record,request);
      return record;
    });
  };
  return state;
}
function transfer(state) {
  const {expected,evidence} = state, base = 'repos/'+expected.repository;
  const github = {
    json(endpoint) {
      state.reads.push(endpoint);
      if(endpoint === base+'/actions/runs/11/attempts/'+expected.attempt)return evidence.run;
      if(endpoint === base+'/actions/runs/11/attempts/'+expected.attempt+'/jobs?per_page=100')
        return {total_count:evidence.jobs.length,jobs:evidence.jobs};
      if(endpoint === base+'/releases/tags/v1.2.3')return evidence.release;
      if(endpoint === base+'/commits/v1.2.3')return evidence.tagCommit;
      if(endpoint === base+'/actions/jobs/44')return evidence.consumerChecks;
      const proof = evidence.producerProofs.find(row => endpoint === base+'/actions/runs/11/attempts/'+row.attempt
        || endpoint === base+'/actions/runs/11/attempts/'+row.attempt+'/jobs?per_page=100');
      assert(proof,'missing authenticated producer metadata');
      return endpoint.endsWith('/jobs?per_page=100') ? proof.jobs : proof.run;
    },
    bytes(endpoint) {
      state.reads.push(endpoint);
      const known = new Map([[base+'/releases/assets/33',evidence.artifactBytes],
        [base+'/releases/assets/34',evidence.sourceBytes],[base+'/releases/assets/35',evidence.checksumBytes]]);
      assert(known.has(endpoint),'unexpected asset read');
      return Buffer.from(known.get(endpoint));
    },
    verify:state.verify
  };
  return transferFromAuthenticatedGithub(expected,44,'/private/tmp',github);
}
test('partial rerun preserves actual VSIX attempt one while source prefers its valid latest signature',()=>{
  const f = fixture();
  const request = {name:f.sourceName,bytes:f.evidence.sourceBytes};
  assert.deepEqual(signedSubjectAttempts(f.expected,request,f.verify(request)),[1,2]);
  f.verification.length=0;
  const result = verifyAuthoritativeTransfer(f.expected,f.evidence,f.verify);
  assert.equal(result.producerAttempt,1);assert.equal(result.sourceProducerAttempt,2);
  assert.equal(result.latestWorkflowAttempt,2);assert.equal(result.producerRun,11);
  assert.equal(result.publicationCommandExecuted,false);assert(result.bytes.equals(f.evidence.artifactBytes));
  assert.equal(f.verification.length,2);
  assert.equal(f.evidence.producerProofs[0].run.conclusion,'failure','unrelated original Windows failure cannot fabricate VSIX failure');
  assert.deepEqual(result.subjectProducers.map(row=>[row.name,row.attempt]),
    [[f.name,1],[f.sourceName,2]]);
});
test('legacy callers without new proof evidence remain strictly single-attempt',()=>{
  const f = fixture();
  delete f.evidence.producerProofs;
  assert.throws(()=>verifyAuthoritativeTransfer(f.expected,f.evidence,f.verify));
  f.artifactAttempts=[2];f.sourceAttempts=[2];
  assert.equal(verifyAuthoritativeTransfer(f.expected,f.evidence,f.verify).producerAttempt,2);
});
test('signed prior attempt needs unique exact successful job, workflow and source evidence',()=>{
  const mutations = [
    f=>delete f.evidence.producerProofs,
    f=>f.evidence.producerProofs=[],
    f=>f.evidence.producerProofs.push(f.evidence.producerProofs[0]),
    f=>f.evidence.producerProofs[0].name=f.sourceName,
    f=>f.evidence.producerProofs[0].attempt=2,
    f=>f.evidence.producerProofs[0].run.run_attempt=2,
    f=>f.evidence.producerProofs[0].run.id=12,
    f=>f.evidence.producerProofs[0].run.head_sha='d'.repeat(40),
    f=>f.evidence.producerProofs[0].run.repository.full_name='fork/repository',
    f=>f.evidence.producerProofs[0].run.head_repository.full_name='fork/repository',
    f=>f.evidence.producerProofs[0].run.head_branch='feature',
    f=>f.evidence.producerProofs[0].run.path='.github/workflows/foreign.yml',
    f=>f.evidence.producerProofs[0].run.event='pull_request',
    f=>f.evidence.producerProofs[0].run.status='in_progress',
    f=>f.evidence.producerProofs[0].run.conclusion='cancelled',
    f=>f.evidence.producerProofs[0].jobs.total_count=3,
    f=>f.evidence.producerProofs[0].jobs.jobs[0].conclusion='failure',
    f=>f.evidence.producerProofs[0].jobs.jobs[0].status='queued',
    f=>f.evidence.producerProofs[0].jobs.jobs[0].run_attempt=2,
    f=>f.evidence.producerProofs[0].jobs.jobs[0].run_id=12,
    f=>f.evidence.producerProofs[0].jobs.jobs[0].head_sha='d'.repeat(40),
    f=>{const jobs=f.evidence.producerProofs[0].jobs;jobs.jobs.push(jobs.jobs[0]);jobs.total_count++;},
    f=>f.evidence.producerProofs[1].attempt=1,
    f=>f.evidence.release.tag_name='v9.9.9',
    f=>f.evidence.tagCommit.sha='d'.repeat(40),
    f=>f.evidence.release.assets[0].digest='sha256:'+'0'.repeat(64)
  ];
  for(const mutate of mutations){
    const f=fixture();mutate(f);
    assert.throws(()=>verifyAuthoritativeTransfer(f.expected,f.evidence,f.verify));
  }
});
test('multiple nonlatest signed attempts are ambiguous and future or mismatched signatures refuse',()=>{
  const f=fixture(3);f.artifactAttempts=[1,2];
  assert.throws(()=>verifyAuthoritativeTransfer(f.expected,f.evidence,f.verify),error=>{
    assert(error.message.includes('ambiguous-nonlatest'));
    assert(error.message.includes('"candidateAttempts":[1,2]'));
    assert(error.message.length<3000);assert(!error.message.includes('https://'));return true;
  });
  for(const mutate of [
    record=>record.verificationResult.signature.certificate.runInvocationURI='https://github.com/owner/repository/actions/runs/12/attempts/1',
    record=>record.verificationResult.signature.certificate.buildSignerDigest='d'.repeat(40),
    record=>record.verificationResult.signature.certificate.buildSignerURI='https://github.com/owner/repository/.github/workflows/desktop-release.yml@refs/heads/feature',
    record=>record.verificationResult.signature.certificate.sourceRepositoryURI='https://github.com/fork/repository',
    record=>record.verificationResult.signature.certificate.issuer='https://untrusted.invalid',
    record=>record.verificationResult.signature.certificate.runnerEnvironment='self-hosted',
    record=>record.verificationResult.statement.predicate.runDetails.metadata.invocationId='mismatched',
    record=>record.verificationResult.statement.subject[0].digest.sha256='0'.repeat(64)
  ]){
    const value=fixture();value.mutateSignature=mutate;
    assert.throws(()=>verifyAuthoritativeTransfer(value.expected,value.evidence,value.verify),error=>{
      assert(error.message.length<3000);assert(!error.message.includes('https://'));return true;
    });
  }
  const future=fixture();future.artifactAttempts=[3];
  assert.throws(()=>verifyAuthoritativeTransfer(future.expected,future.evidence,future.verify));
  const missing=fixture();missing.artifactAttempts=[];
  assert.throws(()=>verifyAuthoritativeTransfer(missing.expected,missing.evidence,missing.verify));
});
test('connected readonly adapter fetches signed original attempt proof and verifies each immutable subject once',()=>{
  const f=fixture();
  const result=transfer(f);
  assert.equal(result.producerAttempt,1);assert.equal(result.sourceProducerAttempt,2);
  assert.equal(result.latestWorkflowAttempt,2);assert(result.bytes.equals(f.evidence.artifactBytes));
  assert.equal(f.verification.length,2);
  assert(f.reads.includes('repos/owner/repository/actions/runs/11/attempts/1'));
  assert(f.reads.includes('repos/owner/repository/actions/runs/11/attempts/1/jobs?per_page=100'));
  assert.equal(f.reads.filter(value=>value.endsWith('/attempts/1/jobs?per_page=100')).length,1);
  assert.equal(result.publicationCommandExecuted,false);
});
test('connected adapter grants no bytes authority for failed, missing, incomplete or wrong-source producer metadata',()=>{
  for(const mutate of [
    f=>f.evidence.producerProofs.splice(0,1),
    f=>f.evidence.producerProofs[0].jobs.jobs[0].conclusion='failure',
    f=>f.evidence.producerProofs[0].jobs.total_count=3,
    f=>f.evidence.producerProofs[0].run.head_sha='d'.repeat(40)
  ]){
    const f=fixture();mutate(f);let result;
    assert.throws(()=>{result=transfer(f);});
    assert.equal(result,undefined);
  }
});
