import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import YAML from 'yaml';
import {cleanupAuditOwner,auditOwnedHiveCleanup,resolveCleanupAuditOwner} from '../../../scripts/audit-owned-hive-cleanup.mjs';
test('audit requires one explicit bounded owner and never infers one from arbitrary text',()=>{
 assert.equal(cleanupAuditOwner('41'),41);assert.equal(cleanupAuditOwner('', 'Cleanup-audit-owner: 41\n'),41);
 for(const [value,body] of [['0',''],['1;write',''],['9007199254740992',''],['','owner 41'],
  ['41','Cleanup-audit-owner: 42'],['','Cleanup-audit-owner: 41\nCleanup-audit-owner: 42']])assert.throws(()=>cleanupAuditOwner(value,body));
});
const client=()=>({attempts:[],transferredBytes:()=>0,request:async()=>{throw Error('unexpected transport');}});
test('audit preserves refusal diagnostics without granting cleanup or executing a mutation',async()=>{
 const result=await auditOwnedHiveCleanup(41,{client:client(),finalize:async options=>{
  assert.equal(options.apply,false);await assert.rejects(async()=>options.client.request('owned',{mutation:['PATCH']}),/forbids mutations/u);
  return {discovery:[{job:42,status:'retained',reason:'incomplete-or-ambiguous-owned-log',evidence:{logOk:false,error:'HTTP 403'}}]};
 }});
 assert.equal(result.complete,false);assert.equal(result.cleanupCredit,false);assert.equal(result.discovery[0].evidence.error,'HTTP 403');
});
test('read-only eligible and no-effect owners remain distinct from actual successful cleanup',async()=>{
 const result=await auditOwnedHiveCleanup(41,{client:client(),finalize:async()=>({discovery:[
  {job:42,status:'would-close-owned-fixture'},{job:43,status:'not-started'}]})});
 assert.equal(result.complete,true);assert.equal(result.apply,false);assert.equal(result.cleanupCredit,false);
});
test('PR and manual audits have read-only scope and cannot enter the mutating cleanup jobs',()=>{
 const workflow=YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/hive-original-delivery-finalize.yml',import.meta.url),'utf8'));
 const audit=workflow.jobs.audit;
 assert(Object.values(audit.permissions).every(scope=>scope==='read'));
 assert.match(audit.if,/head.repo.full_name == github.repository/u);
 assert.match(audit.if,/Cleanup-audit-owner:/u);
 assert(audit.steps.filter(step=>step.uses?.startsWith('actions/checkout@')).every(step=>step.with['persist-credentials']===false));
 assert.equal(workflow.jobs.prepare.if.includes('workflow_dispatch'),false);
 assert.equal(workflow.jobs.prepare.if.includes('pull_request'),false);
 assert.equal(workflow.jobs.sweep.needs,'prepare');
 assert(audit.steps.filter(step=>step.run).every(step=>!step.run.includes('${{')),'owner input stays in environment data');
 assert.equal(audit.steps.find(step=>step.run).shell,'bash','pipefail must retain a failed audit before tee');
 assert.deepEqual(workflow.on.pull_request['branches-ignore'],['e2e/**']);
 assert.deepEqual(workflow.on.workflow_dispatch.inputs.mode.options,['checks']);
 assert(workflow.on.workflow_dispatch.inputs['pull-request']);
});
test('standard checks dispatch reads its owner from exact-head PR metadata without inventing an audit',async()=>{
 const environment={CLEANUP_AUDIT_PR:'42',CLEANUP_AUDIT_SOURCE:'a'.repeat(40),CLEANUP_AUDIT_REPOSITORY:'link-assistant/formal-ai'};
 const pr={number:42,head:{sha:environment.CLEANUP_AUDIT_SOURCE,repo:{full_name:environment.CLEANUP_AUDIT_REPOSITORY}},body:''};
 const client={json:async endpoint=>{assert.equal(endpoint,'repos/link-assistant/formal-ai/pulls/42');return pr;}};
 assert.equal(await resolveCleanupAuditOwner(environment,client),null);
 pr.body='Cleanup-audit-owner: 41';assert.equal(await resolveCleanupAuditOwner(environment,client),41);
 pr.head.sha='b'.repeat(40);await assert.rejects(resolveCleanupAuditOwner(environment,client),/current PR head/u);
 await assert.rejects(resolveCleanupAuditOwner({...environment,CLEANUP_AUDIT_OWNER:'41'},client),/one explicit/u);
});
