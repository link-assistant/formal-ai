import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {verifyTaskImageOperationalEvidence} from '../../../scripts/build-hive-original-delivery-image.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function fixture() {
  const hive = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-image-operational-control-'));
  fs.mkdirSync(path.join(hive, 'src'));
  const helperPath = 'src/held-out-operational-helper.mjs';
  const helper = 'export const retained = true;\n';
  fs.writeFileSync(path.join(hive, helperPath), helper);
  const owners = ['src/development-log.lib.mjs', 'src/formal-ai-attribution.lib.mjs'];
  const sources = owners.map(owner => {
    const bytes = "import './held-out-operational-helper.mjs';\n";
    fs.writeFileSync(path.join(hive, owner), bytes);
    return {path: owner, adaptedSha256: sha(bytes)};
  });
  const sourceIdentity = {commit: 'a'.repeat(40), tree: 'b'.repeat(40)};
  const adapter = {upstreamCommit: 'c'.repeat(40), formalAiSource: sourceIdentity,
    helperPath, helperSha256: sha(helper), sources, authorshipCredit: 0};
  const calls = [];
  const write = () => fs.writeFileSync(path.join(hive, '.formal-ai-operational-adapter.json'), JSON.stringify(adapter));
  write();
  return {hive, helperPath, owners, adapter, sourceIdentity, calls, write,
    input: {hive, sourceIdentity, hiveCommit: adapter.upstreamCommit,
      read: (command, args) => {calls.push({command, args}); return adapter.helperSha256 + '  /observed/helper';},
      run: (command, args) => {calls.push({command, args});}},
    cleanup: () => fs.rmSync(hive, {recursive: true, force: true})};
}
test('hosted preflight hashes the actual image helper and loads every adapted owner before readiness', () => {
  const f = fixture();
  try {
    const result = verifyTaskImageOperationalEvidence(f.input);
    assert.equal(result.actualImageHelperSha256, f.adapter.helperSha256);
    assert.equal(result.adaptedOwnerImports, 'passed');
    assert.equal(result.authorshipCredit, 0);
    assert.deepEqual(f.calls[0], {command: 'docker', args: ['run', '--rm', '--entrypoint', 'sha256sum',
      'hive-e2e:candidate', '/opt/hive-e2e/' + f.helperPath]});
    assert.deepEqual(f.calls[1].args.slice(0, 7), ['run', '--rm', '--entrypoint', 'node',
      'hive-e2e:candidate', '--input-type=module', '--eval']);
    for (const owner of f.owners) assert.ok(f.calls[1].args[7].includes('file:///opt/hive-e2e/' + owner));
    assert.equal(f.calls.length, 2);
  } finally { f.cleanup(); }
});
for (const mutation of ['missing helper', 'changed helper', 'changed owner', 'stale source', 'unsafe path', 'duplicate owner']) {
  test('image preflight refuses ' + mutation + ' before any Docker observation', () => {
    const f = fixture();
    try {
      if (mutation === 'missing helper') fs.unlinkSync(path.join(f.hive, f.helperPath));
      if (mutation === 'changed helper') fs.appendFileSync(path.join(f.hive, f.helperPath), '// altered\n');
      if (mutation === 'changed owner') fs.appendFileSync(path.join(f.hive, f.owners[0]), '// altered\n');
      if (mutation === 'stale source') f.adapter.formalAiSource = {commit: 'd'.repeat(40), tree: f.sourceIdentity.tree};
      if (mutation === 'unsafe path') f.adapter.helperPath = 'src/../outside.mjs';
      if (mutation === 'duplicate owner') f.adapter.sources.push(f.adapter.sources[0]);
      f.write();
      assert.throws(() => verifyTaskImageOperationalEvidence(f.input));
      assert.equal(f.calls.length, 0);
    } finally { f.cleanup(); }
  });
}
test('different image helper bytes refuse before importing an owner', () => {
  const f = fixture();
  try {
    f.input.read = (command, args) => { f.calls.push({command, args}); return 'e'.repeat(64) + '  /helper'; };
    assert.throws(() => verifyTaskImageOperationalEvidence(f.input), /actual task image operational helper digest/);
    assert.equal(f.calls.length, 1);
  } finally { f.cleanup(); }
});
test('an actual image import failure propagates without a successful readiness receipt', () => {
  const f = fixture();
  try {
    const error = new Error('ERR_MODULE_NOT_FOUND: image dependency');
    f.input.run = () => { throw error; };
    assert.throws(() => verifyTaskImageOperationalEvidence(f.input), thrown => thrown === error);
  } finally { f.cleanup(); }
});
