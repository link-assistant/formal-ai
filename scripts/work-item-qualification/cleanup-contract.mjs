// Narrow handler for the existing custody/claim finalizer. Labels alone never authorize effects.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {taskNames,ownedTaskBody} from './fixtures.mjs';
import {prepareFixture} from './fixtures.mjs';
import {workItemIdentity} from './policy.mjs';
export const workItemWorkflow='.github/workflows/work-item-qualification.yml';
export const workItemMode='work-item-1483';
const sha=value=>createHash('sha256').update(value).digest('hex');
export function requireWorkItemOwner({owner,run,policyHash,inputHash}){
  assert.equal(owner.schema,'formal-ai-hive-effect-owner/v1');assert.equal(owner.taskMode,workItemMode);
  assert.equal(owner.repository,'link-assistant/formal-ai');assert.equal(owner.run,run.id);
  assert.equal(owner.attempt,run.run_attempt);assert.equal(owner.protocolCommit,run.head_sha);
  assert.equal(owner.selectedSource,run.head_sha);assert.equal(owner.scope,'pr-1495');
  assert.equal(owner.policyHash,policyHash);assert.equal(owner.inputHash,inputHash);
  assert.equal(run.repository.full_name,owner.repository);assert.equal(run.head_repository.full_name,owner.repository);
  assert.equal(run.path.split('@')[0],workItemWorkflow);assert.equal(run.event,'workflow_dispatch');
  assert.equal(run.status,'completed');assert.equal(owner.attempt,1);
  assert.deepEqual(owner.tools,['agent','claude','codex']);
  return owner;
}
export function requireWorkItemReceipt({receipt,owner,run,jobs}){
  assert.equal(receipt.schema,'formal-ai-work-item-owned-task/v1');assert.deepEqual(receipt.owner,owner);
  assert.equal(receipt.repository,owner.repository);assert.equal(receipt.source,owner.selectedSource);
  assert(owner.tools.includes(receipt.tool)&&taskNames.includes(receipt.task));
  assert.equal(receipt.baseBranch,'e2e/work-item/'+owner.run+'/'+receipt.tool+'/'+receipt.task);
  if(receipt.baseCommit!==null)assert.match(receipt.baseCommit,/^[a-f0-9]{40}$/u);
  const job=jobs.find(row=>row.id===receipt.job);assert(job);
  assert.equal(job.name,'Actual Hive '+receipt.tool+' work-item '+receipt.task);assert.equal(job.status,'completed');
  if(receipt.issueNumber!==null){assert(Number.isSafeInteger(receipt.issueNumber)&&receipt.issueNumber>0);assert.match(receipt.issueBodySha256,/^[a-f0-9]{64}$/u);}
  return job;
}
export async function cleanupWorkItemReceipt({receipt,owner,run,jobs,api,expectedFixture,save}){
  const job=requireWorkItemReceipt({receipt,owner,run,jobs}),root='repos/'+owner.repository;
  if(receipt.issueNumber===null){
    assert(receipt.pullRequestNumber===null&&receipt.head===null);
    if(receipt.baseCommit===null)return {state:'not-started',outcome:'unresolved'};
    if(receipt.issueBodySha256){
      const body=ownedTaskBody(expectedFixture,owner,receipt.tool,receipt.task);
      assert.equal(sha(body),receipt.issueBodySha256);
      const candidates=await api(root+'/issues?state=all&labels=e2e-task&since='+encodeURIComponent(job.started_at)+'&per_page=100');
      assert(Array.isArray(candidates)&&candidates.length<100,'complete bounded uncertain-issue census');
      const matching=candidates.filter(issue=>!issue.pull_request&&issue.user.login==='github-actions[bot]'
        &&sha(issue.body??'')===receipt.issueBodySha256
        &&Date.parse(issue.created_at)>=Date.parse(job.started_at)&&Date.parse(issue.created_at)<=Date.parse(job.completed_at));
      if(matching.length!==1)return {state:'retained-uncertain',outcome:'unresolved',reason:'uncertain-issue-creation'};
      receipt.issueNumber=matching[0].number;save(receipt);
      return cleanupWorkItemReceipt({receipt,owner,run,jobs,api,expectedFixture,save});
    }
    // A recorded orphan snapshot is owned even when issue creation failed; uncertain unrecorded effects remain retained.
    const reference=await api(root+'/git/ref/heads/'+encodeURIComponent(receipt.baseBranch));
    assert.equal(reference.object.sha,receipt.baseCommit);
    const rules=await api(root+'/rules/branches/'+encodeURIComponent(receipt.baseBranch));
    assert(Array.isArray(rules)&&rules.length<=100);
    if(rules.some(rule=>rule.type==='deletion'))return {state:'protected-residue',outcome:'unresolved',retained:[receipt.baseBranch]};
    await api(root+'/git/refs/heads/'+encodeURIComponent(receipt.baseBranch),{method:'DELETE'});
    receipt.cleanup={state:'complete',outcome:'unresolved',issueCreation:'not-started'};save(receipt);return receipt.cleanup;
  }
  // Reconstruct from trusted frozen source/task recipe, never an arbitrary artifact's desired title/body.
  assert.equal(sha(ownedTaskBody(expectedFixture,owner,receipt.tool,receipt.task)),receipt.issueBodySha256);
  const issue=await api(root+'/issues/'+receipt.issueNumber);
  assert.equal(issue.user.login,'github-actions[bot]');assert.equal(issue.title,expectedFixture.title);
  assert.equal(sha(issue.body),receipt.issueBodySha256);
  assert(Date.parse(issue.created_at)>=Date.parse(job.started_at)&&Date.parse(issue.created_at)<=Date.parse(job.completed_at));
  const labels=issue.labels.map(row=>typeof row==='string'?row:row.name);
  assert(labels.includes('e2e-task')&&labels.includes('no-formal-ai-draft'));
  const comments=await api(root+'/issues/'+receipt.issueNumber+'/comments?per_page=100');
  assert.equal(comments.length,issue.comments);assert(comments.length<=100&&comments.every(row=>row.user.login==='github-actions[bot]'));
  const pulls=await api(root+'/pulls?state=all&base='+encodeURIComponent(receipt.baseBranch)+'&per_page=100');
  assert(pulls.length<=1);
  const branches=[receipt.baseBranch];
  if(pulls.length){
    const pr=await api(root+'/pulls/'+pulls[0].number);
    assert.equal(pr.base.ref,receipt.baseBranch);assert.equal(pr.base.repo.full_name,owner.repository);
    assert.equal(pr.head.repo.full_name,owner.repository);assert.equal(pr.merged,false);assert.equal(pr.merged_at,null);
    assert(new RegExp('^issue-'+receipt.issueNumber+'-[A-Za-z0-9-]+$').test(pr.head.ref));
    if(receipt.head)assert.equal(pr.head.sha,receipt.head,'candidate changed after recorded verification');
    if(receipt.pullRequestNumber)assert.equal(pr.number,receipt.pullRequestNumber);
    branches.push(pr.head.ref);receipt.pullRequestNumber=pr.number;receipt.head=pr.head.sha;
  }
  const outcome=receipt.taskDisposition==='unresolved'?'unresolved':receipt.complete===true?'passed':receipt.failure?'failed':'unresolved';
  receipt.cleanup={state:'claimed',outcome,branches};save(receipt);
  // Caller must publish/revalidate its existing immutable cleanup claim before invoking this handler.
  await api(root+'/issues/'+receipt.issueNumber+'/comments',{method:'POST',body:{body:
    '<!-- formal-ai-work-item-cleanup run='+owner.run+' tool='+receipt.tool+' task='+receipt.task+' -->\n'
    +'Owned test fixture outcome: '+outcome+'. This status is not whole-issue completion. Evidence: '
    +'https://github.com/'+owner.repository+'/actions/runs/'+owner.run}});
  if(receipt.pullRequestNumber)await api(root+'/pulls/'+receipt.pullRequestNumber,{method:'PATCH',body:{state:'closed'}});
  await api(root+'/issues/'+receipt.issueNumber,{method:'PATCH',body:{state:'closed',state_reason:outcome==='passed'?'completed':'not_planned'}});
  const retained=[];
  for(const branch of branches){
    const rules=await api(root+'/rules/branches/'+encodeURIComponent(branch));assert(Array.isArray(rules)&&rules.length<=100);
    const reference=await api(root+'/git/ref/heads/'+encodeURIComponent(branch));assert.match(reference.object.sha,/^[a-f0-9]{40}$/u);
    const expected=branch===receipt.baseBranch?receipt.baseCommit:receipt.head;assert.equal(reference.object.sha,expected);
    if(rules.some(rule=>rule.type==='deletion')){retained.push({branch,head:expected,reason:'protected-branch'});continue;}
    await api(root+'/git/refs/heads/'+encodeURIComponent(branch),{method:'DELETE'});
  }
  const observedIssue=await api(root+'/issues/'+receipt.issueNumber);
  assert.equal(observedIssue.number,receipt.issueNumber);assert.equal(observedIssue.state,'closed');
  let observedPull=null;
  if(receipt.pullRequestNumber){
    observedPull=await api(root+'/pulls/'+receipt.pullRequestNumber);
    assert.equal(observedPull.number,receipt.pullRequestNumber);assert.equal(observedPull.state,'closed');
  }
  const closure={schema:'work-item-cleanup-observed-closure/v1',
    issue:{number:observedIssue.number,state:observedIssue.state},
    pullRequest:observedPull?{number:observedPull.number,state:observedPull.state}:null,
    branches:branches.map(branch=>({branch,head:branch===receipt.baseBranch?receipt.baseCommit:receipt.head}))};
  receipt.cleanup={state:retained.length?'protected-residue':'complete',outcome,retained,closure};save(receipt);
  return receipt.cleanup;
}
/** Called by applyCleanupClaim only AFTER existing immutable claim publication/revalidation. */
export async function finalizeWorkItemOwner({reader,owner,run,jobs,identity,api,readArtifact,fixtureFor,save=()=>{}}){
  requireWorkItemOwner({owner,run,...identity});
  assert(Array.isArray(jobs)&&jobs.length<=100&&jobs.every(row=>row.status==='completed'));
  const listing=await reader.pages('repos/'+owner.repository+'/actions/runs/'+owner.run+'/artifacts','artifacts');
  assert(listing.length<=100);
  const outcomes=[];
  for(const tool of owner.tools)for(const task of taskNames){
    const name='work-item-resource-'+tool+'-'+task;
    const found=listing.filter(row=>row.name===name);
    const job=jobs.find(row=>row.name==='Actual Hive '+tool+' work-item '+task);
    if(found.length===0&&job&&job.steps.some(step=>step.name==='Run ordinary Hive prompts through isolated actual current clients'&&step.conclusion==='skipped')){
      outcomes.push({tool,task,state:'not-started'});continue;
    }
    if(found.length!==1){outcomes.push({tool,task,state:'retained-uncertain',reason:'missing-or-ambiguous-resource-receipt'});continue;}
    const metadata=found[0];
    assert.equal(metadata.workflow_run.id,owner.run);assert.equal(metadata.workflow_run.head_sha,owner.protocolCommit);
    const receipt=await readArtifact(reader,owner.repository,metadata,'resource.json');
    assert.equal(receipt.tool,tool);assert.equal(receipt.task,task);
    const fixture=await fixtureFor({task,source:owner.selectedSource});
    const outcome=await cleanupWorkItemReceipt({receipt,owner,run,jobs,api,expectedFixture:fixture,save});
    outcomes.push({tool,task,...outcome});
  }
  const retained=outcomes.filter(row=>!['complete','not-started'].includes(row.state));
  return {schema:'formal-ai-work-item-cleanup-outcome/v1',run:owner.run,attempt:owner.attempt,
    complete:retained.length===0,outcomes,retained:retained.length,deliveryCredit:false};
}
/** Integration target for generated-task-cleanup-journal applyCleanupClaim. */
export async function finalizeWorkItemRun({runId,expectedRunAttempt,expectedProtocolHead,owner,apply,client}){
  assert.equal(apply,true);assert.equal(owner.run,runId);assert.equal(owner.attempt,expectedRunAttempt);
  assert.equal(owner.protocolCommit,expectedProtocolHead);
  // Maintained placement is scripts/work-item-qualification; every imported service is already used by the old finalizer.
  const {githubReader}=await import('../admit-hive-original-delivery.mjs');
  const {sourceBytes}=await import('../lib/hive-delivery-source.mjs');
  const {artifactJson}=await import('../lib/hive-delivery-journal.mjs');
  const reader=githubReader({repository:owner.repository});
  const run=await reader.read('repos/'+owner.repository+'/actions/runs/'+runId+'/attempts/'+expectedRunAttempt);
  const listing=await reader.read('repos/'+owner.repository+'/actions/runs/'+runId+'/jobs?per_page=100&filter=latest');
  assert.equal(listing.jobs.length,listing.total_count);assert(listing.total_count<=100);
  const identity=await workItemIdentity({read:name=>sourceBytes(reader,owner.repository,owner.protocolCommit,name)});
  let writes=0;
  const api=async(endpoint,{method='GET',body}={})=>{
    assert(endpoint.startsWith('repos/'+owner.repository+'/'));
    if(method==='GET')return client.json(endpoint);
    assert(['POST','PATCH','DELETE'].includes(method)&&++writes<=120,'finite 12-row cleanup mutations');
    const mutation=['--method',method];
    for(const [key,value] of Object.entries(body??{}))mutation.push('-f',key+'='+value);
    const result=await client.request(endpoint,{mutation});
    assert(result.ok&&!result.truncated&&!result.downloadTruncated&&!result.timedOut,'uncertain cleanup mutation; never retry');
    return result.text?.trim()?JSON.parse(result.text):null;
  };
  const result=await finalizeWorkItemOwner({reader,owner,run,jobs:listing.jobs,identity,api,readArtifact:artifactJson,
    fixtureFor:({task,source})=>prepareFixture({task,source,read:(commit,name)=>sourceBytes(reader,owner.repository,commit,name)})});
  return {cleanupReceipt:{schema:'generated-e2e-cleanup-proof/v1',sourceRun:runId,complete:result.complete,
    closed:result.outcomes.filter(row=>row.state==='complete').length,retained:result.retained,
    noEffects:result.outcomes.every(row=>row.state==='not-started'),workItem:result},
    applied:{results:result.outcomes.map(row=>({status:row.state==='retained-uncertain'?'mutation-uncertain':row.state}))}};
}

/** Only actual finalizer results enter here after authenticated owner/claim checks. */
export function knownProtectedWorkItemResidue(result, owner) {
  if (result?.schema !== 'formal-ai-work-item-cleanup-outcome/v1'
    || result.run !== owner.run || result.attempt !== owner.attempt
    || result.complete !== false || result.deliveryCredit !== false
    || !Array.isArray(result.outcomes) || !Array.isArray(owner.tools)
    || result.outcomes.length !== owner.tools.length * taskNames.length) return false;
  const expected = new Set(owner.tools.flatMap(tool => taskNames.map(task => tool + '/' + task)));
  let protectedRows = 0;
  for (const row of result.outcomes) {
    if (!expected.delete(row.tool + '/' + row.task)) return false;
    if (row.state === 'complete' || row.state === 'not-started') continue;
    if (row.state !== 'protected-residue' || !Array.isArray(row.retained)
      || row.retained.length === 0) return false;
    const closure = row.closure;
    if (closure?.schema !== 'work-item-cleanup-observed-closure/v1'
      || !Number.isSafeInteger(closure.issue?.number) || closure.issue.number <= 0
      || closure.issue.state !== 'closed' || !Array.isArray(closure.branches)
      || closure.branches.length < 1 || closure.branches.length > 2) return false;
    const hasPull = closure.pullRequest !== null;
    if (hasPull && (!Number.isSafeInteger(closure.pullRequest?.number)
      || closure.pullRequest.number <= 0 || closure.pullRequest.state !== 'closed')) return false;
    if (closure.branches.length !== (hasPull ? 2 : 1)) return false;
    const base = 'e2e/work-item/' + owner.run + '/' + row.tool + '/' + row.task;
    const branches = new Map();
    for (const entry of closure.branches) {
      if (!entry || typeof entry.branch !== 'string' || branches.has(entry.branch)
        || !/^[a-f0-9]{40}$/u.test(entry.head ?? '')) return false;
      if (entry.branch !== base && (!hasPull
        || !new RegExp('^issue-' + closure.issue.number + '-[A-Za-z0-9-]+$').test(entry.branch))) return false;
      branches.set(entry.branch, entry.head);
    }
    if (!branches.has(base)) return false;
    const retained = new Set();
    for (const entry of row.retained) {
      if (!entry || typeof entry !== 'object' || entry.reason !== 'protected-branch'
        || retained.has(entry.branch) || !branches.has(entry.branch)
        || branches.get(entry.branch) !== entry.head) return false;
      retained.add(entry.branch);
    }
    protectedRows++;
  }
  return expected.size === 0 && protectedRows > 0 && result.retained === protectedRows;
}
