import test from 'node:test';
import assert from 'node:assert/strict';
import {Readable} from 'node:stream';
import {createHash} from 'node:crypto';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {execFileSync} from 'node:child_process';
import {hashArchiveStream,hashArchiveFile,hashGitArchive} from '../../../scripts/release-stage-transfer.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
test('archive hashing passes the old 256MiB output ceiling without retaining the archive', async () => {
  const chunk = Buffer.alloc(65536, 0x61);
  const chunks = 4097;
  const expected = createHash('sha256');
  for (let index = 0; index < chunks; index++) expected.update(chunk);
  const observed = await hashArchiveStream(Readable.from((function* () {
    for (let index = 0; index < chunks; index++) yield chunk;
  })()));
  assert.equal(observed.bytes, chunks * chunk.length);
  assert.ok(observed.bytes > 256 * 1024 * 1024);
  assert.equal(observed.sha256, expected.digest('hex'));
});
test('the complete physical source file and actual independent Git archive agree exactly', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'release-archive-stream-'));
  try {
    execFileSync('git', ['init', '-q', directory]);
    writeFileSync(join(directory, 'source.txt'), 'small source fixture with Unicode α and binary-safe bytes');
    execFileSync('git', ['-C', directory, 'add', 'source.txt']);
    execFileSync('git', ['-C', directory, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'source']);
    const bytes = execFileSync('git', ['-C', directory, 'archive', '--format=tar', 'HEAD']);
    const archive = join(directory, 'source.tar');
    writeFileSync(archive, bytes);
    const file = await hashArchiveFile(archive);
    const independent = await hashGitArchive(directory);
    assert.deepEqual(file, {sha256: sha(bytes), bytes: bytes.length});
    assert.deepEqual(independent, file);
    writeFileSync(archive, bytes.subarray(0, bytes.length - 1));
    assert.notDeepEqual(await hashArchiveFile(archive), independent);
    await assert.rejects(hashArchiveFile(join(directory, 'missing.tar')));
    const empty = mkdtempSync(join(tmpdir(), 'release-archive-no-head-'));
    try {
      execFileSync('git', ['init', '-q', empty]);
      await assert.rejects(hashGitArchive(empty));
    } finally { rmSync(empty, {recursive: true, force: true}); }
  } finally { rmSync(directory, {recursive: true, force: true}); }
});
test('empty, textual and interrupted archive streams never return an accepted digest', async () => {
  await assert.rejects(hashArchiveStream(Readable.from([])));
  await assert.rejects(hashArchiveStream(Readable.from(['unowned textual chunk'])));
  await assert.rejects(hashArchiveStream(Readable.from((async function* () {
    yield Buffer.from('partial bytes');
    throw new Error('controlled stream interruption');
  })())), /controlled stream interruption/);
});
