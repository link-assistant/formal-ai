import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, realpathSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const freeze = value => {
  if (value && typeof value === 'object') {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
};
export function requireRegisteredTreeEntry(body, treeSha, name, type, mode) {
  assert.equal(body.sha, treeSha);
  assert.equal(body.truncated, false);
  assert.ok(!name.includes('/') && name !== '.' && name !== '..', 'exact single tree component required');
  const matches = body.tree.filter(entry => entry.path === name);
  assert.equal(matches.length, 1, 'unique exact source tree entry required');
  const entry = matches[0];
  assert.equal(entry.type, type);
  assert.equal(entry.mode, mode);
  assert.match(entry.sha, /^[a-f0-9]{40}$/u);
  return entry;
}
export function registeredSourceProfile(sourcePaths) {
  const retained='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  assert.ok(Array.isArray(sourcePaths));assert.equal(new Set(sourcePaths).size,sourcePaths.length);
  assert.ok(sourcePaths.length===2 || sourcePaths.length===4);
  const workflows=sourcePaths.filter(file=>/^\.github\/workflows\/[a-z0-9-]+\.yml$/u.test(file));assert.equal(workflows.length,2);
  if(sourcePaths.length===4)assert.deepEqual(sourcePaths.filter(file=>!workflows.includes(file)).sort(),[retained+'original-release-workflow.yml',retained+'stage-source-coverage.json'].sort());
  const root={children:new Map()};
  for (const file of sourcePaths) {
    const parts = file.split('/');
    let node = root;
    for (const [index, name] of parts.entries()) {
      assert.ok(name && name !== '.' && name !== '..');
      let next = node.children.get(name);
      if (!next) {
        next = {children: new Map()};
        node.children.set(name, next);
      }
      if (index === parts.length - 1) {
        assert.equal(next.path, undefined);
        next.path = file;
      }
      node = next;
    }
  }
  return {root,maximumRequests:sourcePaths.length===4?17:12,requiredRequests:sourcePaths.length===4?17:10,expectedSourceCount:sourcePaths.length};
}
export function createGovernedGithubCommandProvider({
  expected,
  sourcePaths,
  writerGroup
}) {
  const scope = freeze({
    ...expected
  });
  assert.match(scope.repository, /^[\w.-]+\/[\w.-]+$/u);
  assert.match(scope.head, /^[a-f0-9]{40}$/u);
  assert.match(scope.run, /^[1-9][0-9]*$/u);
  assert.match(scope.attempt, /^[1-9][0-9]*$/u);
  assert.match(scope.workflowPath, /^\.github\/workflows\/[a-z0-9-]+\.yml$/u);
  const profile=registeredSourceProfile(sourcePaths);
  if(process.env.GITHUB_ACTIONS==='true')assert.ok(typeof process.env.GH_TOKEN==='string' && process.env.GH_TOKEN.trim().length>0,'explicit hosted GitHub read credential required');
  const installed = ['/usr/bin/gh', '/opt/homebrew/bin/gh'].find(file => existsSync(file));
  assert.ok(installed, 'governed GitHubCLI installation absent');
  const executable = realpathSync(installed);
  assert.ok(statSync(executable).isFile());
  const executableSha256 = hash(readFileSync(executable));
  const receipts = new WeakMap();
  const base = '/repos/' + scope.repository;
  const runPath = base + '/actions/runs/' + scope.run + '/attempts/' + scope.attempt;
  const commitPath = base + '/git/commits/' + scope.head;
  const jobsPath = base + '/actions/runs/' + scope.run + '/attempts/' + scope.attempt + '/jobs?per_page=100';
  const groupPath = base + '/actions/concurrency_groups/' + encodeURIComponent(writerGroup);
  const trees = new Map(),
    blobs = new Map();
  let workflowId = null,
    childId = null;
  const deadline = Date.now() + 30000;
  let count = 0;
  function read(route) {
    const allowed = route === runPath || route === commitPath || route === jobsPath || workflowId !== null && route === base + '/actions/workflows/' + workflowId || trees.has(route) || blobs.has(route) || childId !== null && (route === groupPath || route === groupPath + '?ahead_of_job=' + childId);
    assert.ok(allowed, 'unregistered governedGET route');
    assert.ok(++count <= profile.maximumRequests, 'bounded source/lease request inventory exceeded');
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0, 'aggregate observation deadline expired');
    assert.equal(realpathSync(installed), executable);
    assert.equal(hash(readFileSync(executable)), executableSha256, 'GitHubCLI executable changed');
    const arguments_ = ['api', '--hostname', 'github.com', '--method', 'GET', '-H', 'Accept: application/vnd.github+json', '-H', 'X-GitHub-Api-Version: 2026-03-10', route];
    const process = spawnSync(executable, arguments_, {
      shell: false,
      encoding: null,
      timeout: Math.min(5000, remaining),
      maxBuffer: 4 * 1024 * 1024
    });
    const raw = process.stdout ?? Buffer.alloc(0);
    let body = null;
    try {
      body = JSON.parse(raw.toString('utf8'));
    } catch {}
    const status = process.status === 0 ? 200 : Number(body?.status) || 0;
    const snapshot = freeze({
      status,
      route,
      body,
      responseSha256: hash(raw),
      observedAt: new Date().toISOString(),
      provider: {
        executableSha256,
        method: 'GET',
        version: '2026-03-10',
        processStatus: process.status,
        signal: process.signal,
        error: process.error?.code,
        bytes: raw.length,
        stderrSha256: hash(process.stderr ?? Buffer.alloc(0))
      }
    });
    receipts.set(snapshot, {
      route,
      raw: Buffer.from(raw),
      stderr: Buffer.from(process.stderr ?? Buffer.alloc(0))
    });
    if (status !== 200) return snapshot;
    if (route === runPath) {
      assert.equal(String(body.id), scope.run);
      assert.equal(String(body.run_attempt), scope.attempt);
      assert.equal(body.repository.full_name, scope.repository);
      assert.equal(body.head_repository.full_name, scope.repository);
      assert.equal(body.head_sha, scope.head);
      assert.equal(body.path, scope.workflowPath);
      assert.ok(Number.isSafeInteger(body.workflow_id));
      workflowId = body.workflow_id;
    }
    if (route === commitPath) {
      assert.equal(body.sha, scope.head);
      assert.match(body.tree.sha, /^[a-f0-9]{40}$/u);
      trees.set(base + '/git/trees/' + body.tree.sha, {
        node: profile.root,
        sha: body.tree.sha
      });
    }
    if(trees.has(route)) {
      const registered=trees.get(route);assert.equal(body.sha,registered.sha);assert.equal(body.truncated,false);
      for(const [name,node] of registered.node.children) {
        const source=requireRegisteredTreeEntry(body,registered.sha,name,node.path?'blob':'tree',node.path?'100644':'040000');
        if(node.path){const blobRoute=base+'/git/blobs/'+source.sha;const entries=blobs.get(blobRoute)??[];if(!entries.some(entry=>entry.path===node.path))entries.push({path:node.path,sha:source.sha});blobs.set(blobRoute,entries);}
        else {const treeRoute=base+'/git/trees/'+source.sha;assert.ok(!trees.has(treeRoute),'unqualified duplicate subtree authority refused');trees.set(treeRoute,{sha:source.sha,node});}
      }
    }
    if (blobs.has(route)) {
      const source = blobs.get(route)[0];
      assert.equal(body.sha, source.sha);
      assert.equal(body.encoding, 'base64');
      const bytes = Buffer.from(body.content.replace(/\s/gu, ''), 'base64');
      assert.equal(bytes.length, body.size);
      const gitHash = createHash('sha1').update(Buffer.concat([Buffer.from('blob ' + bytes.length + '\0'), bytes])).digest('hex');
      assert.equal(gitHash, source.sha);
    }
    if (route === jobsPath) {
      assert.equal(body.total_count, body.jobs.length);
      const job = body.jobs.filter(job => job.name === scope.descendantName && job.runner_name === scope.runnerName);
      assert.equal(job.length, 1);
      assert.equal(String(job[0].run_id), scope.run);
      assert.equal(String(job[0].run_attempt), scope.attempt);
      assert.equal(job[0].head_sha, scope.head);
      childId = job[0].id;
      assert.ok(Number.isSafeInteger(childId));
    }
    return snapshot;
  }
  function issued(snapshot, route) {
    const record = receipts.get(snapshot);
    assert.ok(record && record.route === route, 'provider-issued exactGET observation required');
    return snapshot;
  }
  function processStreams(snapshot) {
    const record = receipts.get(snapshot);
    assert.ok(record, 'provider-issued observation required');
    return {
      stdout: Buffer.from(record.raw),
      stderr: Buffer.from(record.stderr)
    };
  }
  function registered() {
    return freeze({
      workflow: workflowId === null ? null : base + '/actions/workflows/' + workflowId,
      trees: [...trees.keys()],
      blobs: [...blobs.keys()],
      sources: [...blobs].flatMap(([route,sources])=>sources.map(source=>({route,...source}))),
      jobs: jobsPath,
      ancestor: childId === null ? null : groupPath + '?ahead_of_job=' + childId,
      group: childId === null ? null : groupPath
    });
  }
  return Object.freeze({
    read,
    issued,
    registered,
    processStreams,
    runPath,
    commitPath
  });
}
