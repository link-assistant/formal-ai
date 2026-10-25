// Read-only default-token diagnosis. This output never grants cleanup or delivery authority.
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {createEvidenceClient} from './collect-test-replays.mjs';
import {finalizeGeneratedRun} from './finalize-generated-test-issues.mjs';
import {redact} from './collect-ci-failures.mjs';
export function cleanupAuditOwner(value,body=''){
 const matches=[...body.matchAll(/^Cleanup-audit-owner:\s*([1-9][0-9]*)\s*$/gmu)];
 if(value){assert.equal(matches.length,0,'one audit owner source required');assert.match(value,/^[1-9][0-9]*$/u);}
 else assert.equal(matches.length,1,'one explicit Cleanup-audit-owner required');
 const owner=Number(value||matches[0][1]);assert(Number.isSafeInteger(owner)&&owner>0);return owner;
}
export async function auditOwnedHiveCleanup(runId,{client=createEvidenceClient({maxRequests:100,
 bounds:{retries:0,logBytes:1048576,logDownloadBytes:8388608}}),finalize=finalizeGeneratedRun}={}){
 assert(Number.isSafeInteger(runId)&&runId>0);
 const readOnly={...client,request:(endpoint,options={})=>{
  assert(!options.mutation,'cleanup audit forbids mutations');return client.request(endpoint,options);
 }};
 const outcome={schema:'owned-hive-cleanup-audit/v1',run:runId,apply:false,cleanupCredit:false,complete:false};
 try{
  const result=await finalize({runId,apply:false,client:readOnly});
  outcome.discovery=result.discovery.map(({job,status,reason,evidence})=>({job,status,reason,...(evidence?{evidence}:{})}));
  outcome.complete=outcome.discovery.length>0&&outcome.discovery.every(row=>['would-close-owned-fixture','not-started'].includes(row.status));
 }catch(error){outcome.error=redact(error.message).slice(0,1000);}
 outcome.requests=client.attempts;outcome.transferredBytes=client.transferredBytes();return outcome;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const owner=cleanupAuditOwner(process.argv[2]??process.env.CLEANUP_AUDIT_OWNER,process.env.CLEANUP_AUDIT_BODY);
 const result=await auditOwnedHiveCleanup(owner);console.log(JSON.stringify(result));if(!result.complete)process.exitCode=1;
}
