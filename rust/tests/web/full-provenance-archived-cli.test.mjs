import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import {repositoryGit,repositoryText}from '../../../js/provenance/git-evidence.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
const git=(workspace,args)=>repositoryText(repositoryGit(workspace,args)).trimEnd();
import {withFullCliFixture} from './helpers/full-provenance-cli-fixture.mjs';

test("full CLI verifies the same independent pins against authenticated archived source", {skip:process.env.FAST_PR_PROFILE==='true'},()=>withFullCliFixture(({directory,workspace,captured,request,requestFile,invoke,delivered,head,base,before})=>{
  const archivedRequest=path.join(directory,'independent-archived-request.json');
  fileSystem.writeFileSync(archivedRequest,JSON.stringify({...request,sourceMode:'archived'}),{mode:0o600});
  const archived=invoke(archivedRequest);assert.equal(archived.status,0,archived.stdout+'\n'+archived.stderr);
  const archivedReport=decodeTyped(archived.stdout.trimEnd());assert.equal(archivedReport.verification,'Verified');
  assert.equal(archivedReport.trustedVerification,true);assert.equal(archivedReport.fullReplay.verified,true);
  assert.equal(archivedReport.fullReplay.scope,'authenticated-retained-js-engine-logical-memory-and-recorded-tool-inputs');
  assert(archivedReport.fullReplay.consumedInputs.modules>100);assert(archivedReport.fullReplay.consumedInputs.reads>100);
  assert.equal(archivedReport.repository.share,1);assert.equal(archivedReport.execution.length,1);
  assert.equal(fileSystem.statSync(delivered,{bigint:true}).ctimeNs,before.ctimeNs);assert.equal(git(workspace,['rev-parse','HEAD']),head);
}));
