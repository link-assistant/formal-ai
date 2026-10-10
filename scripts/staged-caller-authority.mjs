import assert from 'node:assert/strict';
import { releaseContext } from './release-source-freshness.mjs';
import { createHash } from 'node:crypto';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const blob = bytes => createHash('sha1').update(Buffer.concat([Buffer.from('blob ' + Buffer.byteLength(bytes) + '\0'), Buffer.from(bytes)])).digest('hex');
const compiledContracts = new WeakSet();
// Transport is installed by the provider; caller/env strings never grant receipt authority.
export function createStagedAuthority(contract, read) {
  assert.ok(compiledContracts.has(contract), 'checked source compiler contract required');
  const receipts = new WeakMap();
  async function observe(expected) {
    const route = contract.routes.find(item => item.caller === expected.caller && item.ordinal === expected.ordinal);
    assert.ok(route, 'unregistered original operation');
    assert.equal(expected.stageJob, route.stageJob);
    assert.equal(expected.operationSha256, route.operationSha256);
    assert.match(expected.repository, /^[\w.-]+\/[\w.-]+$/u);
    assert.match(expected.head, /^[a-f0-9]{40}$/u);
    assert.match(expected.run, /^[1-9][0-9]*$/u);
    assert.match(expected.attempt, /^[1-9][0-9]*$/u);
    const base = '/repos/' + expected.repository + '/actions';
    const get = async path => {
      const snapshot = await read(path);
      assert.equal(snapshot.status, 200, path);
      return snapshot.body;
    };
    const run = await get(base + '/runs/' + expected.run + '/attempts/' + expected.attempt);
    assert.equal(String(run.id), expected.run);
    assert.equal(String(run.run_attempt), expected.attempt);
    assert.equal(run.repository.full_name, expected.repository);
    assert.equal(run.head_repository.full_name, expected.repository);
    assert.equal(run.head_sha, expected.head);
    assert.equal(run.head_branch, 'main');
    assert.equal(run.status, 'in_progress');
    assert.equal(run.path, '.github/workflows/release.yml');
    assert.equal(run.event, route.event);
    const workflow = await get(base + '/workflows/' + run.workflow_id);
    assert.equal(workflow.path, '.github/workflows/release.yml');
    assert.equal(workflow.state, 'active');
    const tree = await get('/repos/' + expected.repository + '/git/trees/' + expected.head + '?recursive=1');
    assert.equal(tree.truncated, false);
    for (const source of contract.sources) {
      const entry = tree.tree.find(entry => entry.path === source.path);
      assert.ok(entry && entry.type === 'blob' && entry.mode === '100644', source.path);
      const actual = await get('/repos/' + expected.repository + '/git/blobs/' + entry.sha);
      assert.equal(actual.encoding, 'base64');
      const bytes = Buffer.from(actual.content.replace(/\s/gu, ''), 'base64');
      assert.equal(blob(bytes), entry.sha);
      assert.equal(sha(bytes), source.sha256, source.path);
    }
    const jobs = await get(base + '/runs/' + expected.run + '/attempts/' + expected.attempt + '/jobs?per_page=100');
    assert.equal(jobs.total_count, jobs.jobs.length, 'truncated job identities refused');
    const matches = jobs.jobs.filter(job => job.name === route.descendantName && job.runner_name === expected.runnerName && job.status === 'in_progress');
    assert.equal(matches.length, 1, 'unique actual executing descendant required');
    const child = matches[0];
    assert.equal(String(child.run_id), expected.run);
    assert.equal(String(child.run_attempt), expected.attempt);
    assert.equal(child.head_sha, expected.head);
    assert.ok(Number.isSafeInteger(child.id));
    const group = await get(base + '/concurrency_groups/' + encodeURIComponent(contract.writerGroup) + '?ahead_of_job=' + child.id);
    assert.equal(group.group_name, contract.writerGroup);
    assert.ok(Array.isArray(group.group_members));
    // Require source-derived caller lease, not any same-run active job.
    const lease = group.group_members.find(member => String(member.run_id) === expected.run && member.status === 'in_progress' && member.job_name === route.descendantName && member.job_id === child.id);
    assert.ok(lease && Number.isSafeInteger(lease.job_id), 'actual canonical caller ancestor lease required');
    assert.equal(group.group_members.at(-1), lease, 'ancestor result must terminate at selected lease');
    const receipt = Object.freeze({});
    receipts.set(receipt, {
      context: Object.freeze({
        repository: expected.repository,
        head: expected.head,
        run: expected.run,
        attempt: expected.attempt,
        job: route.caller
      }),
      route,
      expected: Object.freeze({
        ...expected
      })
    });
    return receipt;
  }
  function context(receipt, expected) {
    const found = receipts.get(receipt);
    assert.ok(found, 'provider-issued receipt required');
    assert.deepEqual(expected, found.expected, 'receipt operation identity differs');
    return found.context;
  }
  function canonicalReleaseContext(receipt, environment, event) {
    const found = receipts.get(receipt);
    assert.ok(found, 'provider-issued receipt required');
    const expected = found.expected;
    for (const [field, key] of [['GITHUB_REPOSITORY', 'repository'], ['GITHUB_SHA', 'head'], ['GITHUB_RUN_ID', 'run'], ['GITHUB_RUN_ATTEMPT', 'attempt'], ['GITHUB_JOB', 'stageJob'], ['RUNNER_NAME', 'runnerName']]) assert.equal(environment[field], expected[key], field);
    return releaseContext({
      ...environment,
      GITHUB_JOB: found.route.caller
    }, event);
  }
  // Each phase must call observe again: old receipt is not a live-lease renewal.
  return Object.freeze({
    observe,
    context,
    canonicalReleaseContext
  });
}
export function compileContract({
  canonical,
  callerDraft,
  staged,
  packet,
  YAML,
  retainedSourceProof
}) {
  assert.equal(sha(canonical), packet.workflowSha256);
  const before = YAML.parse(canonical),
    caller = YAML.parse(callerDraft),
    callee = YAML.parse(staged);
  assert.equal(caller.concurrency,undefined,'caller workflow must not duplicate its job writer lease');
  assert.equal(callee.concurrency,undefined,'callee workflow must not acquire a writer lease');
  assert.ok(Object.values(callee.jobs).every(job=>job.concurrency===undefined),'callee jobs must not acquire writer leases');
  const routes = [];
  let bindings = 0;
  const groups = new Set();
  for (const declaration of packet.callers) {
    const original = before.jobs[declaration.caller],
      replacement = caller.jobs[declaration.caller];
    assert.equal(original.steps.length, declaration.steps.length);
    assert.equal(replacement.uses, './.github/workflows/release-staged.yml');
    assert.deepEqual(replacement.concurrency, original.concurrency);
    groups.add(original.concurrency.group);
    assert.equal(replacement.secrets, 'inherit');
    const mode = declaration.caller.replace('-release', '');
    for (const operation of declaration.steps) {
      assert.equal(sha(operation.raw), operation.sha256);
      assert.deepEqual(YAML.parse('steps:\n' + operation.raw).steps[0], original.steps[operation.ordinal], 'operation must equal canonical ordinal');
      const stageJob = mode + '_' + operation.stage,
        stage = callee.jobs[stageJob];
      assert.ok(stage);
      assert.ok(stage['timeout-minutes'] <= 30);
      assert.equal(stage.concurrency, undefined, 'callee must not reacquire caller writer lease');
      assert.equal(operation.stepId??null,original.steps[operation.ordinal].id??null,'source operation identifier differs');
      routes.push(Object.freeze({
        caller: declaration.caller,
        stepIdentifier: operation.stepId??null,
        ordinal: operation.ordinal,
        operationSha256: operation.sha256,
        stageJob,
        callerName: original.name,
        descendantName: original.name + ' / ' + stage.name,
        event: mode === 'auto' ? 'push' : 'workflow_dispatch'
      }));
    }
    bindings += declaration.crossStageOutputTransfers.length;
  }
  assert.equal(routes.length, 52);
  assert.equal(bindings, 104);
  assert.equal(groups.size, 1);
  const retained=[];
  if (retainedSourceProof !== undefined) {
    assert.deepEqual(Object.keys(retainedSourceProof).sort(), ['originalSource', 'packetSource']);
    assert.equal(retainedSourceProof.originalSource, canonical);
    assert.deepEqual(JSON.parse(retainedSourceProof.packetSource), packet);
    const prefix = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
    retained.push(
      {path: prefix + 'original-release-workflow.yml', sha256: sha(canonical)},
      {path: prefix + 'stage-source-coverage.json', sha256: sha(retainedSourceProof.packetSource)}
    );
  }
  const contract = Object.freeze({
    routes: Object.freeze(routes),
    writerGroup: [...groups][0],
    bindings,
    sources: Object.freeze([{
      path: '.github/workflows/release.yml',
      sha256: sha(callerDraft)
    }, {
      path: '.github/workflows/release-staged.yml',
      sha256: sha(staged)
    },...retained].map(Object.freeze))
  });
  compiledContracts.add(contract);
  return contract;
}

// This production-provider entry deliberately exposes no callback, mock transport or executor.
export async function observeCommandStagedContext({
  contract,
  expected,
  environment,
  event
}) {
  assert.ok(compiledContracts.has(contract), 'checked source compiler contract required');
  const route = contract.routes.find(route => route.caller === expected.caller && route.ordinal === expected.ordinal);
  assert.ok(route);
  assert.equal(expected.stageJob, route.stageJob);
  assert.equal(expected.operationSha256, route.operationSha256);
  for (const [field, key] of [['GITHUB_REPOSITORY', 'repository'], ['GITHUB_SHA', 'head'], ['GITHUB_RUN_ID', 'run'], ['GITHUB_RUN_ATTEMPT', 'attempt'], ['GITHUB_JOB', 'stageJob'], ['RUNNER_NAME', 'runnerName']]) assert.equal(environment[field], expected[key], field);
  const {
    createGovernedGithubCommandProvider
  } = await import('./governed-github-command-provider.mjs');
  const provider = createGovernedGithubCommandProvider({
    expected: {
      ...expected,
      workflowPath: '.github/workflows/release.yml',
      descendantName: route.descendantName
    },
    sourcePaths: contract.sources.map(source => source.path),
    writerGroup: contract.writerGroup
  });
  const read = path => {
    const snapshot = provider.read(path);
    provider.issued(snapshot, path);
    assert.equal(snapshot.status, 200, 'unknown or unavailable authenticatedGET');
    return snapshot;
  };
  const run = read(provider.runPath).body;
  assert.equal(run.status, 'in_progress');
  assert.equal(run.head_branch, 'main');
  assert.equal(run.event, route.event);
  const workflow = read(provider.registered().workflow).body;
  assert.equal(workflow.path, '.github/workflows/release.yml');
  assert.equal(workflow.state, 'active');
  read(provider.commitPath);
  const visited=new Set();const expectedTrees=contract.sources.length===4?8:3;
  for(let index=0;index<expectedTrees;index++){const path=provider.registered().trees.find(path=>!visited.has(path));assert.ok(path,'complete source tree path required');visited.add(path);read(path);}
  assert.equal(provider.registered().trees.length,expectedTrees);assert.equal(provider.registered().sources.length,contract.sources.length,'all immutable source declarations must bind');
  for (const source of provider.registered().sources) {
    const body = read(source.route).body;
    const bytes = Buffer.from(body.content.replace(/\s/gu, ''), 'base64');
    const expectedSource = contract.sources.find(expected => expected.path === source.path);
    assert.ok(expectedSource);
    assert.equal(sha(bytes), expectedSource.sha256, 'immutable source bytes differ');
  }
  const jobs = read(provider.registered().jobs).body;
  const actual = jobs.jobs.filter(job => job.name === route.descendantName && job.runner_name === expected.runnerName && job.status === 'in_progress');
  assert.equal(actual.length, 1);
  const ancestor = read(provider.registered().ancestor).body;
  assert.equal(ancestor.group_name, contract.writerGroup);
  const lease = ancestor.group_members.at(-1);
  assert.ok(lease);
  assert.equal(lease.job_id, actual[0].id);
  assert.equal(lease.job_name, route.descendantName);
  assert.equal(String(lease.run_id), expected.run);
  assert.equal(lease.status, 'in_progress');
  return releaseContext({
    ...environment,
    GITHUB_JOB: route.caller
  }, event);
}


// Structural proof helper returns no release context; only the governed route grants one.
export function requireCurrentGroupMembership(contract,expected,route,group,jobs) {
 assert.ok(compiledContracts.has(contract),'checked source compiler contract required');
 assert.ok(contract.routes.includes(route),'source-declared route required');
 return requireExactCurrentGroupMember(expected,route.descendantName,contract.writerGroup,group,jobs);
}

// This shared identity evaluator is pure and returns no release context.
export function requireExactCurrentGroupMember(expected,descendantName,writerGroup,group,jobs) {
 assert.equal(group.status,200);assert.equal(group.body.group_name,writerGroup);
 const activeJobs=jobs.body.jobs.filter(job=>job.name===descendantName&&job.runner_name===expected.runnerName&&job.status==='in_progress');
 assert.equal(activeJobs.length,1);const job=activeJobs[0];
 assert.ok(Number.isSafeInteger(job.id)&&job.id>0);assert.equal(String(job.run_id),expected.run);assert.equal(String(job.run_attempt),expected.attempt);assert.equal(job.head_sha,expected.head);
 assert.ok(Number.isSafeInteger(job.runner_id)&&job.runner_id>0);assert.equal(job.completed_at,null);assert.equal(job.url,'https://api.github.com/repos/'+expected.repository+'/actions/jobs/'+job.id);
 const members=group.body.group_members.filter(member=>member.status==='in_progress');assert.equal(members.length,1,'one exact active caller-group descendant required');const member=members[0];
 assert.equal(member.job_id,job.id);assert.equal(member.job_name,descendantName);assert.equal(String(member.run_id),expected.run);
 const groupTime=Date.parse(group.observedAt),jobTime=Date.parse(jobs.observedAt);assert.ok(Number.isFinite(groupTime)&&Number.isFinite(jobTime)&&groupTime>=jobTime&&groupTime-jobTime<=5000,'bounded matching current observations required');assert.ok(Date.parse(job.started_at)<=jobTime);
}

export async function observeCommandCurrentGroupStagedContext({
  contract,
  expected,
  environment,
  event
}) {
  assert.ok(compiledContracts.has(contract), 'checked source compiler contract required');
  const route = contract.routes.find(route => route.caller === expected.caller && route.ordinal === expected.ordinal);
  assert.ok(route);
  assert.equal(expected.stageJob, route.stageJob);
  assert.equal(expected.operationSha256, route.operationSha256);
  for (const [field, key] of [['GITHUB_REPOSITORY', 'repository'], ['GITHUB_SHA', 'head'], ['GITHUB_RUN_ID', 'run'], ['GITHUB_RUN_ATTEMPT', 'attempt'], ['GITHUB_JOB', 'stageJob'], ['RUNNER_NAME', 'runnerName']]) assert.equal(environment[field], expected[key], field);
  const {
    createGovernedGithubCommandProvider
  } = await import('./governed-github-command-provider.mjs');
  const provider = createGovernedGithubCommandProvider({
    expected: {
      ...expected,
      workflowPath: '.github/workflows/release.yml',
      descendantName: route.descendantName
    },
    sourcePaths: contract.sources.map(source => source.path),
    writerGroup: contract.writerGroup
  });
  const read = path => {
    const snapshot = provider.read(path);
    provider.issued(snapshot, path);
    assert.equal(snapshot.status, 200, 'unknown or unavailable authenticatedGET');
    return snapshot;
  };
  const run = read(provider.runPath).body;
  assert.equal(run.status, 'in_progress');
  assert.equal(run.head_branch, 'main');
  assert.equal(run.event, route.event);
  const workflow = read(provider.registered().workflow).body;
  assert.equal(workflow.path, '.github/workflows/release.yml');
  assert.equal(workflow.state, 'active');
  read(provider.commitPath);
  const visited=new Set();const expectedTrees=contract.sources.length===4?8:3;
  for(let index=0;index<expectedTrees;index++){const path=provider.registered().trees.find(path=>!visited.has(path));assert.ok(path,'complete source tree path required');visited.add(path);read(path);}
  assert.equal(provider.registered().trees.length,expectedTrees);assert.equal(provider.registered().sources.length,contract.sources.length,'all immutable source declarations must bind');
  for (const source of provider.registered().sources) {
    const body = read(source.route).body;
    const bytes = Buffer.from(body.content.replace(/\s/gu, ''), 'base64');
    const expectedSource = contract.sources.find(expected => expected.path === source.path);
    assert.ok(expectedSource);
    assert.equal(sha(bytes), expectedSource.sha256, 'immutable source bytes differ');
  }
  const jobSnapshot=read(provider.registered().jobs);
  const jobs=jobSnapshot.body;const jobObservedAt=jobSnapshot.observedAt;
  const actual = jobs.jobs.filter(job => job.name === route.descendantName && job.runner_name === expected.runnerName && job.status === 'in_progress');
  assert.equal(actual.length, 1);
  const groupSnapshot=read(provider.registered().group);
  requireCurrentGroupMembership(contract,expected,route,groupSnapshot,{body:jobs,observedAt:jobObservedAt});
  const lease = groupSnapshot.body.group_members.find(member=>member.status==='in_progress');
  assert.equal(lease.job_id, actual[0].id);
  assert.equal(lease.job_name, route.descendantName);
  assert.equal(String(lease.run_id), expected.run);
  assert.equal(lease.status, 'in_progress');
  return releaseContext({
    ...environment,
    GITHUB_JOB: route.caller
  }, event);
}
