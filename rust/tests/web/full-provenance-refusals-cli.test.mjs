import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import {verifyFullRepositoryProof}from '../../../js/provenance/full-verifier.mjs';
import {repositoryGit,repositoryText}from '../../../js/provenance/git-evidence.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
const git=(workspace,args)=>repositoryText(repositoryGit(workspace,args)).trimEnd();
import {withFullCliFixture} from './helpers/full-provenance-cli-fixture.mjs';

test("full CLI refuses dirty, foreign, truncated and incorrectly pinned proof", {skip:process.env.FAST_PR_PROFILE==='true'},()=>withFullCliFixture(({directory,workspace,captured,request,requestFile,invoke,delivered,head,base,before})=>{
  assert.throws(()=>verifyFullRepositoryProof({...request,sourceMode:'untrusted-source'}),/FullProofSourceModeUnsupported/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,head:base}),/FullProofCheckoutIdentity/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,expectedRuntimeSha256:'0'.repeat(64)}),/PinnedReplayRuntimeMismatch/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,expectedSourceIdentity:'0'.repeat(64)}),/PinnedReplaySourceIdentityMismatch/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,expectedTaskURL:'https://example.invalid/wrong'}),/PinnedReplayTaskMismatch/u);
  const dirty=path.join(workspace,'untracked.txt');fileSystem.writeFileSync(dirty,'not committed');
  assert.throws(()=>verifyFullRepositoryProof(request),/FullProofCheckoutNotClean/u);fileSystem.unlinkSync(dirty);
  const internal=path.join(request.directory,'not-independent.json');fileSystem.writeFileSync(internal,JSON.stringify(request),{mode:0o600});
  const self=invoke(internal);assert.notEqual(self.status,0);assert.match(self.stderr,/FullProofIndependentRequestRequired/u);fileSystem.unlinkSync(internal);
  const chainFile=path.join(request.directory,'chain.lino'),chain=fileSystem.readFileSync(chainFile);
  const rows=chain.toString('utf8').trimEnd().split('\n');fileSystem.writeFileSync(chainFile,rows.slice(0,-1).join('\n')+'\n');
  try{const truncated=invoke(requestFile);assert.equal(truncated.status,2);const unknown=decodeTyped(truncated.stdout.trimEnd());
   assert.equal(unknown.verification,'Unknown');assert.equal(unknown.trustedVerification,false);assert.equal(unknown.fullReplay.verified,false);
  }finally{fileSystem.writeFileSync(chainFile,chain);}
  fileSystem.appendFileSync(delivered,'\n// actual foreign delta\n');git(workspace,['add','retained.mjs']);git(workspace,['commit','-m','foreign']);
  assert.throws(()=>verifyFullRepositoryProof({...request,head:git(workspace,['rev-parse','HEAD'])}),/FullProofRepositoryCoverageIncomplete/u);
}));
