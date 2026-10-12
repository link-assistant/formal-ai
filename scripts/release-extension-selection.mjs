import assert from 'node:assert/strict';
import { verifyArtifactArchive } from './github-artifact-by-identifier.mjs';
/** Authenticated artifact identifies a candidate; signed release verification grants authority later. */
export function selectCompletedSource(expected,
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
