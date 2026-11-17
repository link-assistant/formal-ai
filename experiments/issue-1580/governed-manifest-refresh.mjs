// Review a complete governed-witness refresh against two exact manifest changes.
// This is source-fixture evidence, not execution of the native specification.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { deserialize } from 'node:v8';
import { execFileSync } from 'node:child_process';

const [previousFile, currentFile, previousRevision, reportFile] = process.argv.slice(2);
assert.ok(previousFile && currentFile && previousRevision && reportFile,
  'usage: node governed-manifest-refresh.mjs BEFORE AFTER PREVIOUS_REVISION REPORT');
const digest = value => createHash('sha256').update(value).digest('hex');
const previousBytes = readFileSync(previousFile);
const currentBytes = readFileSync(currentFile);
const previous = deserialize(previousBytes);
const current = deserialize(currentBytes);
const replacements = [];
for (const file of ['desktop/package.json', 'vscode/package.json']) {
  const before = execFileSync('git', ['show', previousRevision + ':' + file], { encoding: 'utf8' });
  const after = readFileSync(file, 'utf8');
  assert.notEqual(before, after, file + ' must contain the reviewed manifest change');
  replacements.push({ file, kind: 'source', before, after });
  replacements.push({ file, kind: 'sha256', before: digest(before), after: digest(after) });
}
assert.deepEqual(current.sourceManifest, previous.sourceManifest,
  'native source manifest must remain completely identical');
const differences = [];
let visitedPairs = 0;
const pairs = new WeakMap();
const walk = (before, after, path) => {
  if (Object.is(before, after)) return;
  assert.equal(typeof after, typeof before, path + ': value type changed');
  if (before === null || after === null || typeof before !== 'object') {
    const replacement = replacements.find(item => before === item.before && after === item.after);
    assert.ok(replacement, 'unreviewed scalar difference at ' + path);
    assert.ok(path.startsWith('/inventory/cases/'), 'only case fixture values may change: ' + path);
    differences.push({ path, file: replacement.file, kind: replacement.kind,
      previousSha256: digest(String(before)), currentSha256: digest(String(after)) });
    return;
  }
  const seen = pairs.get(before);
  if (seen?.has(after)) return;
  if (seen) seen.add(after);
  else pairs.set(before, new WeakSet([after]));
  visitedPairs++;
  assert.ok(visitedPairs <= 200000, 'bounded complete-object review exceeded pair limit');
  assert.equal(Object.getPrototypeOf(after), Object.getPrototypeOf(before), path + ': prototype changed');
  assert.deepEqual(Object.keys(after), Object.keys(before), path + ': keys or order changed');
  for (const key of Object.keys(before)) walk(before[key], after[key], path + '/' + key);
};
walk(previous, current, '');
for (const replacement of replacements) {
  assert.ok(differences.some(item => item.file === replacement.file && item.kind === replacement.kind),
    'expected both manifest text and hash refreshes for ' + replacement.file);
}
const report = {
  previousRevision,
  previousBaselineSha256: digest(previousBytes),
  currentBaselineSha256: digest(currentBytes),
  readerSha256: digest(readFileSync('scripts/lib/rust-specification-cases.mjs')),
  nativeSourceManifestUnchanged: true,
  caseCount: current.inventory.cases.length,
  unsupportedCount: current.inventory.unsupported.length,
  visitedPairs,
  differenceCount: differences.length,
  manifests: replacements.filter(item => item.kind === 'source').map(item => ({
    file: item.file, previousSha256: digest(item.before), currentSha256: digest(item.after)
  })),
  differences,
  limitation: 'Complete source-fixture delta review only; no native specification was executed.'
};
writeFileSync(reportFile, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
