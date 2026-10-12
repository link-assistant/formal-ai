import assert from 'node:assert/strict';
export function validateDiagnosticDescendant(snapshot, expected) {
  assert.equal(snapshot.status,200);
  const job=snapshot.body;
  assert.equal(job.id,expected.id);assert.equal(String(job.run_id),expected.run);
  assert.equal(String(job.run_attempt),expected.attempt);assert.equal(job.head_sha,expected.eventHead);
  assert.equal(job.name,expected.name);assert.equal(job.runner_name,expected.runnerName);
  assert.equal(job.status,'in_progress');assert.ok(Number.isSafeInteger(job.runner_id) && job.runner_id>0);
  assert.equal(job.url,'https://api.github.com/repos/'+expected.repository+'/actions/jobs/'+expected.id);
  return true;
}
export function validateRunAncestorDiagnostic(snapshot, expected) {
  assert.equal(snapshot.status,200);assert.equal(snapshot.body.group_name,expected.group);
  const members=snapshot.body.group_members;assert.ok(Array.isArray(members) && members.length>0);
  const selected=members.at(-1);assert.equal(String(selected.run_id),expected.run);
  assert.equal(selected.status,'in_progress');
  return {observation:'ObservedRunScopedGroup',descendantLeaseProof:false,productionAuthority:false};
}
export async function collectLeaseDiagnostics({context,child,group,runnerName,get,deadline}) {
  const snapshots=[];
  try {
    assert.ok(Number.isSafeInteger(child.id));assert.ok(runnerName);
    const base='/repos/'+context.repository+'/actions';
    const observe=async route=>{const remaining=deadline-Date.now();assert.ok(remaining>0);const value=await get(route,Math.min(5000,remaining));snapshots.push(value);return value;};
    const job=await observe(base+'/jobs/'+child.id);
    validateDiagnosticDescendant(job,{...context,id:child.id,name:child.name,runnerName});
    const ancestor=await observe(base+'/concurrency_groups/'+encodeURIComponent(group)+'?ahead_of_run='+context.run);
    const diagnostic=validateRunAncestorDiagnostic(ancestor,{...context,group});
    return {...diagnostic,snapshots};
  } catch(error) {
    return {observation:'Unknown',reason:error.message,descendantLeaseProof:false,productionAuthority:false,snapshots};
  }
}
