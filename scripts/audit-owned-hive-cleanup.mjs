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
/** Standard bot checks resolve the optional request through authenticated PR metadata. */
export async function resolveCleanupAuditOwner(environment,client=createEvidenceClient({maxRequests:2,bounds:{retries:0}})){
 const owner=environment.CLEANUP_AUDIT_OWNER??'',body=environment.CLEANUP_AUDIT_BODY??'';
 const pull=environment.CLEANUP_AUDIT_PR??'';
 if(!pull)return cleanupAuditOwner(owner,body);
 assert.equal(owner,'','one explicit owner source');assert.equal(body,'','dispatch uses fresh PR metadata');
 assert.equal(environment.CLEANUP_AUDIT_REPOSITORY,'link-assistant/formal-ai');
 assert.match(pull,/^[1-9][0-9]*$/u);assert(Number.isSafeInteger(Number(pull)));
 assert.match(environment.CLEANUP_AUDIT_SOURCE??'',/^[a-f0-9]{40}$/u);
 const pr=await client.json('repos/link-assistant/formal-ai/pulls/'+pull);
 assert.equal(pr.number,Number(pull));assert.equal(pr.head.repo.full_name,environment.CLEANUP_AUDIT_REPOSITORY);
 assert.equal(pr.head.sha,environment.CLEANUP_AUDIT_SOURCE,'checks source must remain the current PR head');
 const current=pr.body??'';
 return current.includes('Cleanup-audit-owner:')?cleanupAuditOwner('',current):null;
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
 const owner=process.argv[2]?cleanupAuditOwner(process.argv[2]):await resolveCleanupAuditOwner(process.env);
 if(owner===null)console.log(JSON.stringify({schema:'owned-hive-cleanup-audit/v1',status:'not-requested',apply:false,cleanupCredit:false,complete:false}));
 else{const result=await auditOwnedHiveCleanup(owner);console.log(JSON.stringify(result));if(!result.complete)process.exitCode=1;}
}
