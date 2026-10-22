import assert from 'node:assert/strict';
import test from 'node:test';
import { publishedChangedPaths } from '../../../scripts/lib/hive-delivery-release.mjs';
import { applicableWorkflows } from '../../../scripts/lib/hive-delivery-admission.mjs';

const repository = 'owner/repository';
const before = 'a'.repeat(40), after = 'b'.repeat(40);
const oldTree = 'c'.repeat(40), newTree = 'd'.repeat(40);
const blob = 'e'.repeat(40), changedBlob = 'f'.repeat(40);
const leaf = (path, sha = blob, mode = '100644', type = 'blob') => ({ path, sha, mode, type, size: 0 });
const tree = (sha, entries) => ({ sha, truncated: false, tree: entries });
function fixture(oldEntries, newEntries, mutate = () => {}) {
  const responses = {
    ['commits/' + before]: { sha: before, tree: { sha: oldTree } },
    ['commits/' + after]: { sha: after, tree: { sha: newTree } },
    ['trees/' + oldTree + '']: tree(oldTree, oldEntries),
    ['trees/' + newTree + '']: tree(newTree, newEntries)
  };
  mutate(responses);
  const calls = [];
  return {
    calls,
    reader: { async read(endpoint) {
      assert.ok(endpoint.startsWith('repos/' + repository + '/git/'));
      calls.push(endpoint);
      const value = responses[endpoint.slice(('repos/' + repository + '/git/').length)];
      assert.ok(value, 'only immutable commit/tree reads');
      return value;
    } }
  };
}

test('751 changed paths retain the workflow trigger beyond the compare 300-file boundary', async () => {
  const oldEntries = [], newEntries = [];
  for (let index = 0; index < 750; index++) {
    const name = 'changed-' + String(index).padStart(4, '0') + '.txt';
    oldEntries.push(leaf(name)); newEntries.push(leaf(name, changedBlob));
  }
  newEntries.push(leaf('zz-required.txt'));
  const { reader, calls } = fixture(oldEntries, newEntries);
  const result = await publishedChangedPaths(reader, repository, before, after);
  assert.equal(result.files.length, 751);
  assert.equal(result.files[750], 'zz-required.txt');
  assert.equal(result.before.commit, before); assert.equal(result.after.tree, newTree);
  assert.equal(result.trees.find(value => value.sha === oldTree).entries, 750);
  assert.equal(result.trees.find(value => value.sha === newTree).entries, 751);
  assert.match(result.trees[1].metadataSha256, /^[a-f0-9]{64}$/u);
  const workflows = [
    { path: '.github/workflows/all.yml', document: { on: { push: {} } } },
    { path: '.github/workflows/required.yml', document: { on: { push: { paths: ['zz-required.txt'] } } } }
  ];
  assert.deepEqual(applicableWorkflows(workflows, { event: 'push', baseBranch: 'main', files: result.files }),
    ['.github/workflows/all.yml', '.github/workflows/required.yml']);
  assert.deepEqual(applicableWorkflows(workflows, { event: 'push', baseBranch: 'main', files: result.files.slice(0, 300) }),
    ['.github/workflows/all.yml']);
  assert.deepEqual(calls, [
    'repos/' + repository + '/git/commits/' + before,
    'repos/' + repository + '/git/commits/' + after,
    'repos/' + repository + '/git/trees/' + oldTree + '',
    'repos/' + repository + '/git/trees/' + newTree + ''
  ]);
});

test('removed and renamed paths, executable modes, symlinks and submodule identity all participate', async () => {
  const original = [leaf('old-name'), leaf('removed'), leaf('mode'), leaf('type'), leaf('unchanged')];
  const updated = [leaf('new-name'), leaf('mode', blob, '100755'), leaf('type', blob, '160000', 'commit'), leaf('unchanged')];
  const result = await publishedChangedPaths(fixture(original, updated).reader, repository, before, after);
  assert.deepEqual(result.files, ['mode', 'new-name', 'old-name', 'removed', 'type']);
  const symlink = await publishedChangedPaths(fixture([leaf('link')], [leaf('link', blob, '120000')]).reader, repository, before, after);
  assert.deepEqual(symlink.files, ['link']);
});

test('unchanged subtree hashes are skipped and file-directory replacement retains both operands', async () => {
  const directory = sha => ({ path: 'dir', sha, mode: '040000', type: 'tree' });
  const unchangedFixture = fixture([directory(blob)], [directory(blob)]);
  const unchanged = await publishedChangedPaths(unchangedFixture.reader, repository, before, after);
  assert.deepEqual(unchanged.files, []);
  assert.equal(unchangedFixture.calls.length, 4, 'unchanged subtree requires no fetch');
  const replacementFixture = fixture([leaf('dir')], [directory(changedBlob)], values => {
    values['trees/' + changedBlob] = tree(changedBlob, [leaf('file')]);
  });
  const replacement = await publishedChangedPaths(replacementFixture.reader, repository, before, after);
  assert.deepEqual(replacement.files, ['dir', 'dir/file']);
  assert.equal(replacementFixture.calls.length, 5);
});

test('truncated, malformed, duplicate, oversized or identity-mismatched trees refuse complete admission', async () => {
  const key = 'trees/' + newTree + '';
  const mutations = [
    values => { values[key].truncated = true; },
    values => { delete values[key].truncated; },
    values => { values[key].sha = blob; },
    values => { values['commits/' + before].sha = after; },
    values => { values[key].tree = null; },
    values => { values[key].tree = Array(100001).fill(leaf('file')); },
    values => { values[key].tree = [leaf('file'), leaf('file')]; },
    values => { values[key].tree = [leaf('../file')]; },
    values => { values[key].tree = [leaf('a//file')]; },
    values => { values[key].tree = [leaf('x'.repeat(1025))]; },
    values => { values[key].tree = [leaf('file', 'invalid')]; },
    values => { values[key].tree = [leaf('file', blob, '040000')]; },
    values => { values[key].tree = [leaf('file', blob, 'unknown')]; },
    values => { values[key].tree = [{ ...leaf('file'), size: -1 }]; }
  ];
  for (const mutate of mutations) {
    await assert.rejects(publishedChangedPaths(fixture([], [leaf('file')], mutate).reader, repository, before, after));
  }
  await assert.rejects(publishedChangedPaths(fixture([], []).reader, '../foreign', before, after));
  await assert.rejects(publishedChangedPaths(fixture([], []).reader, repository, 'main', after));
});

test('the existing 3000 changed-path limit remains a refusal rather than an incomplete inventory', async () => {
  const entries = Array.from({ length: 3001 }, (_, index) => leaf('file-' + index));
  await assert.rejects(publishedChangedPaths(fixture([], entries).reader, repository, before, after), /bounded complete changed-path inventory/u);
});

test('tree reads are cached across aliases and run with at most four concurrent nodes', async () => {
  const oldDirectories = [], newDirectories = [], children = {};
  for (let index = 0; index < 6; index++) {
    const oldSha = (100 + index).toString(16).padStart(40, '0');
    const newSha = (200 + index).toString(16).padStart(40, '0');
    oldDirectories.push({ path: 'dir-' + index, mode: '040000', type: 'tree', sha: oldSha });
    newDirectories.push({ path: 'dir-' + index, mode: '040000', type: 'tree', sha: newSha });
    children['trees/' + oldSha] = tree(oldSha, [leaf('file')]);
    children['trees/' + newSha] = tree(newSha, [leaf('file', changedBlob)]);
  }
  oldDirectories.push({ ...oldDirectories[0], path: 'alias' });
  newDirectories.push({ ...newDirectories[0], path: 'alias' });
  const original = fixture(oldDirectories, newDirectories, values => Object.assign(values, children));
  let active = 0, maximum = 0;
  const reader = { async read(endpoint) {
    active++; maximum = Math.max(maximum, active);
    try {
      await new Promise(resolve => setTimeout(resolve, 1));
      return await original.reader.read(endpoint);
    } finally { active--; }
  } };
  const result = await publishedChangedPaths(reader, repository, before, after);
  assert.equal(result.files.length, 7);
  assert.ok(maximum >= 2 && maximum <= 4);
  assert.equal(original.calls.length, 16, 'two commits plus fourteen unique trees, despite alias');
  assert.equal(new Set(original.calls).size, original.calls.length);
});

test('cyclic or exponentially aliased malformed metadata cannot bypass finite work through cached trees', async () => {
  const cyclic = fixture([], [{ path: 'cycle', mode: '040000', type: 'tree', sha: newTree }]);
  await assert.rejects(publishedChangedPaths(cyclic.reader, repository, before, after), /acyclic new tree/u);
  assert.equal(cyclic.calls.length, 4, 'cycle refusal does not repeat cached API reads');
  const children = {};
  const sha = index => (500 + index).toString(16).padStart(40, '0');
  for (let index = 0; index < 18; index++) {
    children['trees/' + sha(index)] = tree(sha(index), index === 17 ? [] : [
      { path: 'left', mode: '040000', type: 'tree', sha: sha(index + 1) },
      { path: 'right', mode: '040000', type: 'tree', sha: sha(index + 1) }
    ]);
  }
  const aliases = fixture([], [{ path: 'root', mode: '040000', type: 'tree', sha: sha(0) }],
    values => Object.assign(values, children));
  await assert.rejects(publishedChangedPaths(aliases.reader, repository, before, after), /bounded changed-path traversal/u);
  assert.ok(aliases.calls.length < 25, 'finite logical work is independent of deduplicated requests');
});
