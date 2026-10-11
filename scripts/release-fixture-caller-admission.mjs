import assert from 'node:assert/strict';

const descendantName = 'Active reusable fixture / Fixture lease holder active';

// Admission enables a contender to queue. It is never executing-lease authority.
export function validateCallerAdmission(run, jobs, group, context) {
  assert.match(context.repository, /^[\w.-]+\/[\w.-]+$/);
  assert.match(context.run, /^[1-9][0-9]*$/);
  assert.match(context.attempt, /^[1-9][0-9]*$/);
  assert.match(context.eventHead, /^[a-f0-9]{40}$/);
  const base = '/repos/' + context.repository + '/actions';
  const name = 'formal-ai-release-fixture-' + context.run + '-' + context.attempt;
  for (const snapshot of [run, jobs, group]) assert.equal(snapshot.status, 200);
  assert.equal(run.route, base + '/runs/' + context.run);
  assert.equal(jobs.route, base + '/runs/' + context.run + '/jobs?filter=latest&per_page=100');
  assert.equal(group.route, base + '/concurrency_groups/' + encodeURIComponent(name));
  assert.equal(String(run.body.id), context.run);
  assert.equal(String(run.body.run_attempt), context.attempt);
  assert.equal(run.body.head_sha, context.eventHead);
  assert.equal(run.body.path, '.github/workflows/release-runtime-fixture.yml');
  assert.equal(run.body.status, 'in_progress');
  assert.ok(Array.isArray(jobs.body.jobs));
  assert.equal(jobs.body.total_count, jobs.body.jobs.length, 'complete bounded job listing required');
  const matches = jobs.body.jobs.filter(job => job.name === descendantName);
  assert.equal(matches.length, 1, 'one source-declared admission descendant required');
  const child = matches[0];
  assert.ok(Number.isSafeInteger(child.id) && child.id > 0);
  assert.equal(String(child.run_id), context.run);
  assert.equal(String(child.run_attempt), context.attempt);
  assert.equal(child.head_sha, context.eventHead);
  assert.ok(['queued', 'in_progress'].includes(child.status));
  assert.equal(child.completed_at, null);
  assert.equal(child.url, 'https://api.github.com' + base + '/jobs/' + child.id);
  assert.equal(group.body.group_name, name);
  assert.ok(Array.isArray(group.body.group_members));
  assert.equal(group.body.total_count, group.body.group_members.length);
  const active = group.body.group_members.filter(member => member.status === 'in_progress');
  assert.equal(active.length, 1, 'one admitted private caller required');
  const member = active[0];
  assert.equal(member.job_id, child.id);
  assert.equal(member.job_name, child.name);
  assert.equal(String(member.run_id), context.run);
  assert.equal(member.job_url, child.url);
  assert.equal(member.run_url, 'https://api.github.com' + base + '/runs/' + context.run);
  assert.ok(!group.body.group_members.some(value => value.job_name === 'Fixture private queued contender'));
  const times = [run, jobs, group].map(snapshot => Date.parse(snapshot.observedAt));
  assert.ok(times.every(Number.isFinite));
  assert.ok(times[0] <= times[1] && times[1] <= times[2] && times[2] - times[0] <= 5000);
  return Object.freeze({
    observation: 'ObservedCallerAdmissionForContender',
    descendantJobId: child.id,
    descendantStatus: child.status,
    executingLeaseProof: false,
    productionAuthority: false
  });
}

// The caller supplies only its maintained physical GET provider, never witness metadata.
export async function observeCallerAdmission(get, context, deadline) {
  assert.ok(Number.isFinite(deadline) && deadline > Date.now());
  const base = '/repos/' + context.repository + '/actions';
  const name = 'formal-ai-release-fixture-' + context.run + '-' + context.attempt;
  const snapshots = [];
  const observe = async route => {
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0, 'admission deadline exhausted');
    const result = await get(route, Math.min(5000, remaining));
    snapshots.push(result);
    return result;
  };
  const run = await observe(base + '/runs/' + context.run);
  const jobs = await observe(base + '/runs/' + context.run + '/jobs?filter=latest&per_page=100');
  const group = await observe(base + '/concurrency_groups/' + encodeURIComponent(name));
  return { ...validateCallerAdmission(run, jobs, group, context), snapshots };
}
