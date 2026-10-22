// Published-source admission authenticates JSON/data only; hosted delivery later executes the immutable image.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { artifactJson } from './hive-delivery-journal.mjs';
import { sourceBytes, readWorkflowPolicy } from './hive-delivery-source.mjs';
import { applicableWorkflows, ciAdmissionDecision, admissionPolicy, deliveryWorkflow } from './hive-delivery-admission.mjs';
import { validateVersionWitness } from '../refresh-release-specification-witness.mjs';
import { verifyAnonymousImageManifest } from '../verify-anonymous-image-manifest.mjs';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const text = bytes => new TextDecoder('utf-8', {
  fatal: true
}).decode(bytes);
/** Allowed metadata paths come from the unchanged maintained producer's declarations. */
export function preparedMetadataPolicy(versionSource, witnessSource, metricSource) {
  const scalar = (source, name) => {
    const result = new RegExp('const ' + name + ': &str = "([^"\\n]+)";', 'u').exec(source);
    assert.ok(result, 'missing producer declaration ' + name);
    return result[1];
  };
  const status = /const STATUS_SURFACES: &\[&str\] = &\[([^\]]+)\];/u.exec(versionSource);
  assert.ok(status);
  const surfaces = [...status[1].matchAll(/"([^"\n]+)"/gu)].map(value => value[1]);
  assert.ok(surfaces.length > 0 && surfaces.every(value => value.endsWith('.md')));
  const witness = scalar(witnessSource, 'WITNESS'),
    ledger = scalar(metricSource, 'DEFAULT_LEDGER'),
    mapping = scalar(versionSource, 'FRAGMENT_RELEASE_MAP'),
    archive = scalar(versionSource, 'CHANGELOG_ARCHIVE_DIR');
  return {
    surfaces,
    witness,
    ledger,
    mapping,
    archive
  };
}
export function requirePreparedVersionDelta({
  merge,
  source,
  commit,
  files,
  beforeCargo,
  afterCargo,
  beforeLock,
  afterLock,
  beforeWitness,
  afterWitness,
  version,
  policy
}) {
  assert.notEqual(merge, source);
  assert.equal(commit.sha, source);
  assert.equal(commit.parents.length, 1);
  assert.equal(commit.parents[0].sha, merge, 'prepared version must directly follow checked source');
  assert.ok(files.length > 0 && files.length < 300, 'complete bounded version delta');
  const versionLine = /^(version\s*=\s*")[^"]+(".*)$/mu;
  assert.ok(versionLine.test(beforeCargo));
  assert.equal(beforeCargo.replace(versionLine, (_whole, prefix, suffix) => prefix + version + suffix), afterCargo, 'only package version may change');
  const lockVersion = /(\[\[package\]\]\s*\nname\s*=\s*"formal-ai"\s*\nversion\s*=\s*")[^"]+(")/u;
  assert.ok(lockVersion.test(beforeLock));
  assert.equal(beforeLock.replace(lockVersion, (_whole, prefix, suffix) => prefix + version + suffix), afterLock, 'dependency edges and checksums unchanged');
  const allowed = name => name === 'rust/Cargo.toml' || name === 'rust/Cargo.lock' || name === policy.witness || name === policy.ledger || name === policy.mapping || name === 'CHANGELOG.md' || policy.surfaces.includes(name) || name.startsWith(policy.archive + '/') && name.endsWith('.md');
  for (const file of files) {
    if (file.filename.startsWith('changelog.d/')) {
      assert.equal(file.status, 'removed', 'only consumed fragments');
      continue;
    }
    assert.ok(allowed(file.filename), 'unverified source delta ' + file.filename);
    if (file.previous_filename) assert.ok(allowed(file.previous_filename), 'unverified rename source');
  }
  if (files.some(value => value.filename === policy.witness)) validateVersionWitness(beforeWitness, afterWitness, beforeCargo, afterCargo);
  return {
    kind: 'guarded-direct-prepared-version',
    ciSource: merge,
    publishedSource: source,
    paths: files.map(value => value.filename)
  };
}
async function tinyAsset(reader, repository, release, name) {
  const matches = release.assets.filter(value => value.name === name);
  assert.equal(matches.length, 1, 'one published JSON asset ' + name);
  const asset = matches[0];
  assert.equal(asset.state, 'uploaded');
  assert.ok(asset.size > 0 && asset.size <= 131072);
  assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
  const bytes = await reader.bytes('repos/' + repository + '/releases/assets/' + asset.id, {
    accept: 'application/octet-stream'
  });
  assert.equal(bytes.length, asset.size);
  assert.equal('sha256:' + hash(bytes), asset.digest);
  return {
    bytes,
    value: JSON.parse(text(bytes))
  };
}
export async function boundedRegistryFetch(url, options) {
  assert.ok(url.startsWith('https://ghcr.io/token?') || /^https:\/\/ghcr\.io\/v2\/link-assistant\/formal-ai\/manifests\/sha256:[a-f0-9]{64}$/u.test(url));
  const response = await fetch(url, {
    ...options,
    redirect: 'error'
  });
  const reader = response.body?.getReader();
  assert.ok(reader);
  let count = 0;
  const chunks = [];
  try {
    for (let index = 0; index < 65536; index++) {
      const item = await reader.read();
      if (item.done) return new Response(Buffer.concat(chunks), {
        status: response.status,
        headers: response.headers
      });
      count += item.value.length;
      assert.ok(count <= 4 * 1024 * 1024, 'registry metadata budget');
      chunks.push(Buffer.from(item.value));
    }
    throw new Error('registry read action bound');
  } finally {
    void reader.cancel().catch(() => {});
  }
}
/** Diff immutable Merkle trees, skipping equal subtrees instead of a capped compare-file list.
 * Nonrecursive reads avoid GitHub's recursive 100,000-entry/7 MiB truncation.
 * https://docs.github.com/en/rest/git/trees#get-a-tree
 * Every read still uses the existing reader's smaller byte/request/deadline limits.
 */
export async function publishedChangedPaths(reader, repository, before, after) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  const objectId = value => assert.match(value ?? '', /^[a-f0-9]{40}$/u);
  objectId(before);
  objectId(after);
  const prefix = 'repos/' + repository + '/git/';
  const root = async commit => {
    const node = await reader.read(prefix + 'commits/' + commit);
    assert.equal(node.sha, commit, 'immutable changed-path commit');
    objectId(node.tree?.sha);
    return { commit, tree: node.tree.sha };
  };
  const left = await root(before), right = await root(after);
  const trees = new Map(), provenance = [], files = new Set();
  const modeType = {
    '040000': 'tree', '100644': 'blob', '100755': 'blob', '120000': 'blob', '160000': 'commit'
  };
  const readTree = sha => {
    if (trees.has(sha)) return trees.get(sha);
    const pending = (async () => {
      const response = typeof reader.readTree === 'function' ? await reader.readTree(sha) : await reader.read(prefix + 'trees/' + sha);
      assert.equal(response.sha, sha, 'immutable changed-path tree');
      assert.equal(response.truncated, false, 'complete tree required');
      assert.ok(Array.isArray(response.tree) && response.tree.length <= 100000, 'bounded tree entries');
      const entries = new Map();
      for (const entry of response.tree) {
        assert.equal(typeof entry.path, 'string');
        assert.ok(entry.path.length > 0 && entry.path.length <= 1024 && !/[\u0000-\u001f\u007f/]/u.test(entry.path), 'bounded tree component');
        assert.ok(entry.path !== '.' && entry.path !== '..', 'canonical tree component');
        assert.ok(!entries.has(entry.path), 'unique tree component');
        objectId(entry.sha);
        assert.ok(Object.hasOwn(modeType, entry.mode), 'known Git mode');
        assert.equal(entry.type, modeType[entry.mode], 'Git mode and type agree');
        // Canonical Git trees bind mode, name and OID; typed trees need no extra blob-size lookup.
        if (entry.type === 'blob' && (typeof reader.readTree !== 'function' || Object.hasOwn(entry, 'size'))) {
          assert.ok(Number.isSafeInteger(entry.size) && entry.size >= 0, 'bounded blob size metadata');
        }
        entries.set(entry.path, entry);
      }
      provenance.push({ sha, entries: entries.size, metadataSha256: hash(Buffer.from(JSON.stringify(response))) });
      return entries;
    })();
    trees.set(sha, pending);
    return pending;
  };
  let visited = 0;
  const visit = async (name, oldEntry, newEntry, oldAncestors = [], newAncestors = []) => {
    assert.ok(++visited <= 100000, 'bounded changed-path traversal');
    assert.ok(name.length <= 1024, 'bounded complete Git path');
    if (oldEntry && newEntry && oldEntry.sha === newEntry.sha && oldEntry.mode === newEntry.mode && oldEntry.type === newEntry.type) return [];
    if (oldEntry && oldEntry.type !== 'tree' || newEntry && newEntry.type !== 'tree') {
      files.add(name);
      assert.ok(files.size <= 3000, 'bounded complete changed-path inventory');
    }
    if (oldEntry?.type === 'tree') assert.ok(!oldAncestors.includes(oldEntry.sha), 'acyclic old tree');
    if (newEntry?.type === 'tree') assert.ok(!newAncestors.includes(newEntry.sha), 'acyclic new tree');
    const oldLineage = oldEntry?.type === 'tree' ? [...oldAncestors, oldEntry.sha] : oldAncestors;
    const newLineage = newEntry?.type === 'tree' ? [...newAncestors, newEntry.sha] : newAncestors;
    const childrenFor = entry => entry?.type === 'tree' ? readTree(entry.sha) : Promise.resolve(new Map());
    const [oldChildren, newChildren] = typeof reader.readTree === 'function'
      ? await Promise.all([childrenFor(oldEntry), childrenFor(newEntry)])
      : [await childrenFor(oldEntry), await childrenFor(newEntry)];
    return [...new Set([...oldChildren.keys(), ...newChildren.keys()])].map(component =>
      [name ? name + '/' + component : component, oldChildren.get(component), newChildren.get(component), oldLineage, newLineage]);
  };
  const pending = [['', { type: 'tree', mode: '040000', sha: left.tree }, { type: 'tree', mode: '040000', sha: right.tree }]];
  // Four active nodes bound concurrent API reads without multiplying the reader's shared limits.
  while (pending.length) {
    const children = await Promise.all(pending.splice(0, 4).map(args => visit(...args)));
    for (const batch of children) {
      assert.ok(visited + pending.length + batch.length <= 100000, 'bounded changed-path traversal');
      pending.push(...batch);
    }
  }
  return {
    kind: 'complete-immutable-tree-delta/v1',
    method: 'bounded-nonrecursive-merkle-diff',
    before: left,
    after: right,
    trees: provenance.sort((a, b) => a.sha < b.sha ? -1 : a.sha > b.sha ? 1 : 0),
    files: [...files].sort()
  };
}
/** Denial-only filter; a parent match never grants release authority. */
export function lateSourceCandidate(source, commit, head) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  assert.match(head, /^[a-f0-9]{40}$/u);
  assert.equal(commit.sha, source);
  assert.ok(Array.isArray(commit.parents));
  for (const parent of commit.parents) assert.match(parent.sha, /^[a-f0-9]{40}$/u);
  return head === source || (commit.parents.length === 1 && commit.parents[0].sha === head);
}
/** Cheap observed-state refusal only; the full applicable-workflow census still runs afterward. */
export async function precheckPublishedCi(reader, repository, source, requiredPath = null) {
  const runs = await reader.pages('repos/' + repository + '/actions/runs?head_sha=' + source, 'workflow_runs');
  for (const run of runs) {
    assert.equal(run.repository.full_name, repository);
    assert.equal(run.head_repository.full_name, repository);
    assert.equal(run.head_sha, source, 'precheck CI source');
  }
  assert.ok(Array.isArray(runs) && runs.length <= 2000, 'bounded complete CI inventory');
  if (requiredPath !== null && runs.length === 0) return {admitted: false, reason: 'missing applicable CI ' + requiredPath};
  const observed = [...new Set(runs.filter(run => run.path !== deliveryWorkflow
    && run.path !== '.github/workflows/hive-original-delivery-finalize.yml').map(run => run.path))];
  const expected = requiredPath === null ? observed : [requiredPath];
  return expected.length ? ciAdmissionDecision({source, expected, runs, complete: true}) : {admitted: true, ci: []};
}
/** No open PR is inferred from a default-branch workflow SHA; published source has its own authenticated chain. */
export async function observePublishedAdmission({
  reader,
  repository,
  verifiedRunId,
  lateCompletionHead = null
}) {
  assert.ok(Number.isSafeInteger(verifiedRunId) && verifiedRunId > 0);
  const prefix = 'repos/' + repository;
  const latest = await reader.read(prefix + '/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
  assert.ok(latest.workflow_runs.length === 1);
  const verified = latest.workflow_runs[0];
  assert.equal(verified.id, verifiedRunId, 'only the exact latest completed public-download producer may admit released effects');
  assert.equal(verified.repository.full_name, repository);
  assert.equal(verified.head_repository.full_name, repository);
  assert.equal(verified.path, '.github/workflows/verify-release-downloads.yml');
  assert.equal(verified.status, 'completed');
  assert.equal(verified.conclusion, 'success');
  const inventories = await reader.pages(prefix + '/actions/runs/' + verified.id + '/artifacts', 'artifacts');
  const selections = inventories.filter(value => value.name === 'public-release-selection');
  assert.equal(selections.length, 1);
  assert.equal(selections[0].workflow_run.id, verified.id);
  assert.equal(selections[0].workflow_run.head_sha, verified.head_sha);
  const selection = await artifactJson(reader, repository, selections[0], 'selection.json', {
    archiveLimit: 2 * 1024 * 1024,
    memberLimit: 1024 * 1024
  });
  assert.equal(selection.schema, 'public-release-download-selection/v1');
  assert.equal(selection.repository, repository);
  assert.match(selection.tag, /^v\d+\.\d+\.\d+$/u);
  const release = await reader.read(prefix + '/releases/tags/' + selection.tag);
  assert.equal(release.id, selection.releaseId);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  const prepared = await tinyAsset(reader, repository, release, 'prepared-release-source.json');
  const executable = await tinyAsset(reader, repository, release, 'prepared-release-executable.json');
  const full = await tinyAsset(reader, repository, release, 'prepared-release-full-image.json');
  assert.match(selection.sourceCommit, /^[a-f0-9]{40}$/u);
  assert.match(String(prepared.value.producer_run), /^[1-9]\d*$/u);
  const source = selection.sourceCommit,
    version = selection.tag.slice(1),
    sourceCommit = await reader.read(prefix + '/git/commits/' + source);
  assert.equal(sourceCommit.tree.sha, selection.sourceTree);
  if (lateCompletionHead !== null && !lateSourceCandidate(source, sourceCommit, lateCompletionHead)) {
    return {admitted: false, reason: 'late-ci-source-mismatch'};
  }
  const tag = await reader.read(prefix + '/git/ref/tags/' + selection.tag);
  let object = tag.object;
  const visited = new Set();
  for (let depth = 0; object.type === 'tag'; depth++) {
    assert.ok(depth < 4 && !visited.has(object.sha));
    visited.add(object.sha);
    object = (await reader.read(prefix + '/git/tags/' + object.sha)).object;
  }
  assert.equal(object.type, 'commit');
  assert.equal(object.sha, source);
  assert.equal(prepared.value.source_commit, source);
  assert.equal(prepared.value.source_tree, selection.sourceTree);
  assert.equal(prepared.value.package_version, version);
  assert.equal(executable.value.identity.source_commit, source);
  assert.equal(executable.value.identity.source_tree, selection.sourceTree);
  assert.equal(executable.value.identity.source_selection_sha256, hash(prepared.bytes));
  const image = full.value;
  assert.equal(image.expected.revision, source);
  assert.equal(image.expected.version, version);
  assert.equal(image.sourceSelectionSha256, hash(prepared.bytes));
  assert.equal(image.expected.binarySha256, executable.value.executable.sha256);
  assert.equal(image.receipt.source_commit, source);
  assert.equal(image.receipt.package_version, version);
  assert.equal(image.receipt.image, image.image);
  assert.equal(image.receipt.executable.sha256, image.expected.binarySha256);
  assert.equal(image.receipt.version_output.trim(), 'formal-ai ' + version);
  assert.equal(image.runtime.status, 0);
  assert.equal(image.runtime.signal, null);
  assert.equal(image.runtime.complete, true);
  assert.match(image.image, /^ghcr\.io\/link-assistant\/formal-ai@sha256:[a-f0-9]{64}$/u);
  await verifyAnonymousImageManifest(image.image, boundedRegistryFetch);
  const main = await reader.read(prefix + '/actions/runs/' + prepared.value.producer_run);
  assert.equal(main.repository.full_name, repository);
  assert.equal(main.head_repository.full_name, repository);
  assert.equal(main.path, '.github/workflows/release.yml');
  assert.equal(main.head_branch, 'main');
  assert.equal(main.event, 'push');
  assert.equal(main.status, 'completed');
  assert.equal(main.conclusion, 'success');
  const earlyChecked = await precheckPublishedCi(reader, repository, main.head_sha, main.path);
  if (!earlyChecked.admitted) return earlyChecked;
  if (source !== main.head_sha) {
    const earlyPublished = await precheckPublishedCi(reader, repository, source);
    if (!earlyPublished.admitted) return earlyPublished;
  }
  const artifacts = await reader.pages(prefix + '/actions/runs/' + main.id + '/artifacts', 'artifacts');
  const events = artifacts.filter(value => value.name === 'hive-source-event-' + main.run_attempt);
  assert.equal(events.length, 1);
  assert.equal(events[0].workflow_run.id, main.id);
  assert.equal(events[0].workflow_run.head_sha, main.head_sha);
  const event = await artifactJson(reader, repository, events[0], 'source-event.json');
  assert.equal(event.schema, 'formal-ai-hive-source-event/v1');
  assert.equal(event.repository, repository);
  assert.equal(event.run, main.id);
  assert.equal(event.attempt, main.run_attempt);
  assert.equal(event.event, 'push');
  assert.equal(event.ref, 'refs/heads/main');
  assert.equal(event.source, main.head_sha);
  assert.equal(event.after, main.head_sha);
  assert.match(event.before, /^[a-f0-9]{40}$/u);
  const comparison = await reader.read(prefix + '/compare/' + event.before + '...' + event.after);
  assert.equal(comparison.status, 'ahead');
  const changedPaths = await publishedChangedPaths(reader, repository, event.before, event.after);
  const workflows = await readWorkflowPolicy(reader, repository, main.head_sha);
  const expected = applicableWorkflows(workflows, {
    event: 'push',
    baseBranch: 'main',
    files: changedPaths.files
  });
  const runs = await reader.pages(prefix + '/actions/runs?head_sha=' + main.head_sha, 'workflow_runs');
  for (const run of runs) {
    assert.equal(run.repository.full_name, repository);
    assert.equal(run.head_repository.full_name, repository);
  }
  const checkedDecision = ciAdmissionDecision({source: main.head_sha, expected, runs, complete: true});
  if (!checkedDecision.admitted) return checkedDecision;
  const ci = checkedDecision.ci;
  let delta = {
    kind: 'exact-ci-source',
    ciSource: source,
    publishedSource: source
  };
  if (source !== main.head_sha) {
    const changed = await reader.read(prefix + '/compare/' + main.head_sha + '...' + source);
    assert.equal(changed.status, 'ahead');
    assert.equal(changed.total_commits, 1);
    const policy = preparedMetadataPolicy(text(await sourceBytes(reader,
 repository,
 main.head_sha,
 'scripts/version-and-commit.rs')),
 text(await sourceBytes(reader,
 repository,
 main.head_sha,
 'scripts/version-specification-witness.rs')),
 text(await sourceBytes(reader,
 repository,
 main.head_sha,
 'scripts/self-hosting-metric.rs')));
    const [beforeCargo,
 afterCargo,
 beforeLock,
 afterLock] = await Promise.all(['rust/Cargo.toml',
 'rust/Cargo.toml',
 'rust/Cargo.lock',
 'rust/Cargo.lock'].map((name,
 index) => sourceBytes(reader,
 repository,
 index % 2 === 0 ? main.head_sha : source,
 name)));
    let beforeWitness, afterWitness;
    if (changed.files.some(value => value.filename === policy.witness)) {
      beforeWitness = await sourceBytes(reader, repository, main.head_sha, policy.witness);
      afterWitness = await sourceBytes(reader, repository, source, policy.witness);
    }
    delta = requirePreparedVersionDelta({
      merge: main.head_sha,
      source,
      commit: sourceCommit,
      files: changed.files,
      beforeCargo: text(beforeCargo),
      afterCargo: text(afterCargo),
      beforeLock: text(beforeLock),
      afterLock: text(afterLock),
      beforeWitness,
      afterWitness,
      version,
      policy
    });
  }
  const publishedRuns = source === main.head_sha ? [] : await reader.pages(prefix + '/actions/runs?head_sha=' + source, 'workflow_runs');
  for (const run of publishedRuns) {
    assert.equal(run.repository.full_name, repository);
    assert.equal(run.head_repository.full_name, repository);
  }
  const publishedExpected = [...new Set(publishedRuns.filter(run => run.path !== deliveryWorkflow && run.path !== '.github/workflows/hive-original-delivery-finalize.yml').map(run => run.path))];
  const publishedDecision = publishedExpected.length ? ciAdmissionDecision({
    source, expected: publishedExpected, runs: publishedRuns, complete: true
  }) : {admitted: true, ci: []};
  if (!publishedDecision.admitted) return publishedDecision;
  const publishedObservedCi = publishedDecision.ci;
  const selected = {
    repository,
    scope: 'release-' + selection.tag,
    source,
    sourceTree: selection.sourceTree,
    event: 'published-release',
    tag: selection.tag,
    ciSource: main.head_sha
  };
  return {
    schema: admissionPolicy,
    admitted: true,
    selected,
    expected,
    ci,
    publishedObservedCi,
    changedPaths,
    delta,
    publicDownloadVerification: {
      run: verified.id,
      attempt: verified.run_attempt,
      release: release.id
    },
    publishedRuntime: {
      image: image.image,
      source,
      binarySha256: image.expected.binarySha256
    },
    authorshipCredit: 0
  };
}

/** Validate authenticated identity separately from normal late-event source mismatch. */
function requireLatePublishedIdentity(trigger, admission) {
  assert.equal(admission.schema, admissionPolicy);
  assert.equal(admission.admitted, true);
  const selected = admission.selected;
  assert.equal(selected.event, 'published-release');
  assert.equal(trigger.repository.full_name, selected.repository);
  assert.equal(trigger.head_repository.full_name, selected.repository);
  assert.equal(trigger.status, 'completed');
  assert.equal(trigger.conclusion, 'success');
  assert.equal(trigger.event, 'push');
  assert.equal(trigger.head_branch, 'main');
  assert.equal(admission.delta.ciSource, selected.ciSource);
  assert.equal(admission.delta.publishedSource, selected.source);
  assert.equal(admission.delta.kind, selected.source === selected.ciSource
    ? 'exact-ci-source' : 'guarded-direct-prepared-version');
}
function latePublishedDenial(trigger, admission) {
  const selected = admission.selected;
  if (trigger.head_sha !== selected.ciSource && trigger.head_sha !== selected.source)
    return 'late-ci-source-does-not-match-published-release';
  const records = [...admission.ci, ...admission.publishedObservedCi];
  if (!records.some(record => record.id === trigger.id && record.attempt === trigger.run_attempt
    && record.path === trigger.path && record.source === trigger.head_sha))
    return 'late-completion-is-not-current-authenticated-ci';
  return null;
}
/** A completed push may wake the exact published-source chain, never inherit unrelated old CI. */
export function latePublishedBindingDecision(trigger, admission) {
  requireLatePublishedIdentity(trigger, admission);
  const reason = latePublishedDenial(trigger, admission);
  return reason === null ? admission : {admitted: false, reason};
}
