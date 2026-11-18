import assert from 'node:assert/strict';
import { verifyArtifactArchive } from './github-artifact-by-identifier.mjs';
/** Authenticate the completed Desktop run and its complete job inventory. */
function completedProducer(expected,
 github) {
  assert.match(expected.repository,
 /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  for (const key of ['runId',
 'attempt']) assert.match(String(expected[key]),
 /^[1-9][0-9]*$/u);
  const base = 'repos/' + expected.repository;
  const run = github.json(base + '/actions/runs/' + expected.runId + '/attempts/' + expected.attempt);
  assert.equal(String(run.id),
 String(expected.runId));
  assert.equal(run.run_attempt,
 Number(expected.attempt));
  assert.equal(run.repository.full_name,
 expected.repository);
  assert.equal(run.head_repository.full_name,
 expected.repository);
  assert.equal(run.head_branch,
 'main');
  assert.equal(run.path,
 '.github/workflows/desktop-release.yml');
  assert.equal(run.status,
 'completed');
  assert.ok(['workflow_run',
 'release',
 'workflow_dispatch'].includes(run.event));
  assert.ok(!['cancelled',
 'skipped'].includes(run.conclusion));
  assert.match(run.head_sha,
 /^[a-f0-9]{40}$/u);
  const jobs = github.json(base + '/actions/runs/' + expected.runId + '/attempts/' + expected.attempt + '/jobs?per_page=100');
  assert.equal(jobs.total_count,
 jobs.jobs.length,
 'incomplete producer job selection');
  return {base, run, jobs};
}
/** A successful no-op Desktop workflow is not evidence of publication. */
export function completedDesktopPublication(expected, github) {
  const {run, jobs} = completedProducer(expected, github);
  assert.equal(run.conclusion, 'success', 'completed Desktop producer must have succeeded');
  const named = name => {
    const selected = jobs.jobs.filter(job => job.name === name);
    assert.equal(selected.length, 1, 'one exact producer job required: ' + name);
    return selected[0];
  };
  const identifiers = new Set();
  for (const job of jobs.jobs) {
    assert.ok(Number.isSafeInteger(job.id) && job.id > 0 && !identifiers.has(job.id));
    identifiers.add(job.id);
    assert.equal(String(job.run_id), String(expected.runId));
    assert.equal(job.run_attempt, Number(expected.attempt));
    assert.equal(job.status, 'completed');
  }
  const source = named('Select exact native release source');
  if (source.conclusion !== 'skipped') {
    assert.equal(source.conclusion, 'success');
    return {active: true, publicationAuthority: false};
  }
  assert.equal(named('Resolve target release').conclusion, 'success');
  for (const name of ['Package VS Code extension (.vsix)', 'Publish SHA256SUMS.txt + provenance']) {
    assert.equal(named(name).conclusion, 'skipped');
  }
  for (const job of jobs.jobs) {
    if (job.name === 'Resolve target release') continue;
    if (job.name === 'base / Pin base commit') {
      assert.ok(['success', 'skipped'].includes(job.conclusion));
    } else assert.equal(job.conclusion, 'skipped', 'inactive producer must have no product jobs');
  }
  return {active: false, reason: 'authenticated-desktop-publication-inactive', publicationAuthority: false};
}
/** Authenticated artifact identifies a candidate; signed release verification grants authority later. */
export function selectCompletedSource(expected, github) {
  const {base, run, jobs} = completedProducer(expected, github);
  for (const name of ['Package VS Code extension (.vsix)',
 'Publish SHA256SUMS.txt + provenance']) {
    const selected = jobs.jobs.filter(job => job.name === name);
    assert.equal(selected.length,
 1);
    assert.equal(String(selected[0].run_id),
 String(expected.runId));
    assert.equal(selected[0].run_attempt,
 Number(expected.attempt));
    assert.equal(selected[0].status,
 'completed');
    assert.equal(selected[0].conclusion,
 'success');
  }
  const list = github.json(base + '/actions/runs/' + expected.runId + '/artifacts?per_page=100');
  assert.equal(list.total_count,
 list.artifacts.length,
 'incomplete producer artifact selection');
  const selected = list.artifacts.filter(item => item.name === 'native-release-source');
  assert.equal(selected.length,
 1,
 'missing or ambiguous source artifact');
  const endpoint = base + '/actions/artifacts/' + selected[0].id;
  const metadata = github.json(endpoint),

    archive = github.bytes(endpoint + '/zip');
  const entries = verifyArtifactArchive({
    repository: expected.repository,

    id: selected[0].id,

    run: expected.runId,

    head: run.head_sha,

    name: 'native-release-source'
  },
 metadata,
 archive);
  assert.deepEqual(github.json(endpoint),
 metadata,
 'source artifact changed during transfer');
  const receipts = entries.filter(entry => entry.name === 'source-selection.json' &&
 !entry.directory);
  assert.equal(receipts.length,
 1,
 'exact source receipt required');
  const source = JSON.parse(new TextDecoder('utf-8',
 {
    fatal: true
  }).decode(receipts[0].bytes));
  assert.equal(source.version,
 1);
  assert.equal(source.producer_run,
 String(expected.runId));
  assert.match(source.release_tag,
 /^v[0-9]+\.[0-9]+\.[0-9]+$/u);
  for (const key of ['source_commit',
 'source_tree']) assert.match(source[key],
 /^[a-f0-9]{40}$/u);
  assert.equal(source.package_version,
 source.release_tag.slice(1));
  return {
    repository: expected.repository,

    tag: source.release_tag,

    sourceCommit: source.source_commit,

    sourceTree: source.source_tree,

    protocolCommit: run.head_sha,

    runId: Number(expected.runId),

    attempt: Number(expected.attempt),

    selectionOnly: true,

    publicationAuthorized: false
  };
}
