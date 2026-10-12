import test from 'node:test';
import assert from 'node:assert/strict';
import { fileReadFinalAnswer } from '../../../js/agentic/file_read/audit.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
test('unavailable line slices refuse unrelated content while valid slices retain exact bounds', async () => {
  await installNodeHost(new WorkerHost());
  let passed = 0;
  for (const [path, content, slice] of [['n.txt', 'SECRET1\nSECRET2\nSECRET3\n', {
    from: 3,
    to: 2,
    fromEnd: false
  }], ['other.txt', 'SECRET1\nSECRET2\n', {
    from: 20,
    to: 30,
    fromEnd: false
  }], ['empty.txt', '', {
    from: 1,
    to: 1,
    fromEnd: false
  }], ['nested/Case.txt', 'SECRET1\n', {
    from: 9,
    to: 8,
    fromEnd: true
  }]]) {
    const answer = fileReadFinalAnswer({
      kind: 'line_slice',
      ...slice
    }, [[path, content]], '');
    assert.equal(answer, `Cannot read the requested line slice from \`${path}\`: no selected lines are available.`);
    assert.ok(!answer.includes('SECRET'));
    passed++;
  }
  for (const [slice, expected] of [[{
    from: 1,
    to: 2,
    fromEnd: false
  }, 'Lines 1-2 of `n.txt`:\n\n```text\none\ntwo\n```'], [{
    from: 2,
    to: 1,
    fromEnd: true
  }, 'Lines 2-3 of `n.txt`:\n\n```text\ntwo\nthree\n```'], [{
    from: 2,
    to: 99,
    fromEnd: false
  }, 'Lines 2-3 of `n.txt`:\n\n```text\ntwo\nthree\n```']]) {
    assert.equal(fileReadFinalAnswer({
      kind: 'line_slice',
      ...slice
    }, [['n.txt', 'one\ntwo\nthree\n']], ''), expected);
    passed++;
  }
  assert.equal(fileReadFinalAnswer({
    kind: 'full'
  }, [['n.txt', 'one\ntwo\n']], ''), 'Contents of `n.txt`:\n\n```text\none\ntwo\n```');
  passed++;
  console.log(JSON.stringify({
    passed,
    native: false,
    projection: 'scratch one-key observer only; root maintained projection required'
  }));
  assert.equal(passed, 8);
});
