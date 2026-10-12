// Issue #1180 R11 in the JavaScript root: the subprocess and storage half of
// repository history (js/agentic/crate/history_store.mjs) runs git, appends
// to the events store and advances the cursor as
// rust/src/history_context/cursor.rs does, so an incremental import is
// idempotent and resumes from the stored watermark.

import assert from 'node:assert/strict';
import childProcess from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { before, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

let history;
let store;
let rules;
let io;

const git = (cwd, ...args) =>
  childProcess.execFileSync('git', args, {
    cwd,
    encoding: 'utf8',
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: 'Tester',
      GIT_AUTHOR_EMAIL: 'tester@example.com',
      GIT_COMMITTER_NAME: 'Tester',
      GIT_COMMITTER_EMAIL: 'tester@example.com',
    },
  });

function scratchRepository() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-1180-store-'));
  git(root, 'init', '-q', '-b', 'main');
  git(root, 'config', 'commit.gpgsign', 'false');
  git(root, 'remote', 'add', 'origin', 'https://github.com/example-owner/example-repo.git');
  fs.writeFileSync(path.join(root, 'lib.rs'), 'pub fn first() {}\n');
  git(root, 'add', '.');
  git(root, 'commit', '-q', '-m', 'add first', '-m', 'Fixes #12');
  return root;
}

before(async () => {
  await installNodeHost(new WorkerHost());
  history = await import('../../../js/agentic/crate/history_context.mjs');
  store = await import('../../../js/agentic/crate/history_store.mjs');
  rules = history.historyRules();
  io = store.nodeHistoryIo({ fs, path, childProcess });
});

test('the slug comes from the origin remote, and paths sit under it', () => {
  const root = scratchRepository();
  assert.equal(store.repositorySlug(io, root), 'example-owner-example-repo');
  const { storePath, cursorPath } = store.storePaths(io, '/memory', root);
  assert.equal(storePath, path.join('/memory', 'repository-history', 'example-owner-example-repo', 'events.lino'));
  assert.equal(cursorPath, path.join('/memory', 'repository-history', 'example-owner-example-repo', 'cursor.lino'));
  fs.rmSync(root, { recursive: true, force: true });
});

test('an incremental import is idempotent and resumes from the cursor', () => {
  const root = scratchRepository();
  const memory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-1180-memory-'));
  assert.equal(store.importIncremental(io, root, null, memory, rules), 1);
  assert.equal(store.importIncremental(io, root, null, memory, rules), 0, 'a second pass appends nothing');

  fs.writeFileSync(path.join(root, 'lib.rs'), 'pub fn first() {}\npub fn second() {}\n');
  git(root, 'commit', '-q', '-am', 'add second');
  const head = git(root, 'rev-parse', 'HEAD').trim();
  assert.equal(store.importIncremental(io, root, null, memory, rules), 1, 'only the new commit is appended');

  const { storePath, cursorPath } = store.storePaths(io, memory, root);
  const cursor = store.readCursor(io, cursorPath);
  assert.equal(cursor.lastCommitSha, head);
  assert.equal(store.cursorText(cursor), `${store.CURSOR_ROOT}\n  last_commit_sha "${head}"\n`);

  const text = fs.readFileSync(storePath, 'utf8');
  assert.equal((text.match(/^ {2}event "/gmu) || []).length, 2);
  assert.ok(text.includes(`${rules.pathEvidencePrefix}lib.rs`), text);
  assert.ok(text.includes(`${rules.symbolEvidencePrefix}lib.rs#`), 'the named-item diff is evidence');
  fs.rmSync(root, { recursive: true, force: true });
  fs.rmSync(memory, { recursive: true, force: true });
});

test('a missing cursor is a fresh start and run ids take the maximum', () => {
  const fresh = store.readCursor(io, path.join(os.tmpdir(), 'formal-ai-no-such-cursor.lino'));
  assert.deepEqual(fresh, { lastCommitSha: null, lastIssuePrUpdatedAt: null, lastCiRunDatabaseId: null });
  const ci = rules.records.find((record) => record.kind === 'ci_run');
  const events = [9, 41, 17].map((number) => ({ id: `${ci.idPrefix}${number}`, kind: ci.kind }));
  assert.equal(store.advanceCursor(fresh, events, rules).lastCiRunDatabaseId, 41);
});
