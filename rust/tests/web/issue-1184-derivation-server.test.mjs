// Issue #1184 R1184-9: the JavaScript server persists every answer's
// derivation and serves the explain request as the Rust binary does —
// rust/src/derivation.rs `finalize_answer` (the `derivation:<id>` link, the
// appended Links Notation record, the file under
// `data/cache/derivations/<id>.lino` of the working directory) and
// rust/src/cli_explain.rs `run_explain` (`explain <answer-id>
// [--format text|links]`, the seed's `derivation_record_missing` miss).

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, realpathSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, before, test } from 'node:test';

import { RENDER_EMIT_KIND, answerDerivationId, explainText, fromLino } from '../../../js/agentic/crate/derivation.mjs';
import { finalizeAnswer, runExplain } from '../../../js/server/derivation-store.mjs';
import { solveSymbolic } from '../../../js/server/solve.mjs';
import { REPO_ROOT, WorkerHost } from '../../../js/server/worker-host.mjs';

let host;
let root;
before(async () => {
  host = new WorkerHost();
  await host.boot();
  root = realpathSync(mkdtempSync(path.join(tmpdir(), 'formal-ai-1184-server-')));
});
after(() => rmSync(root, { recursive: true, force: true }));

const MISS_TAIL = '(an answer id has the shape `answer_<16 hex digits>` and is created when the answer is returned)';

test('a served answer links, appends and persists its derivation record', async () => {
  const symbolic = await solveSymbolic({ worker: host, derivationRoot: root }, 'What is 2 + 2?', []);
  const id = answerDerivationId(symbolic.answer);
  const file = path.join(root, 'data/cache/derivations', `${id}.lino`);
  assert.deepEqual(symbolic.evidence_links.slice(-2), [`derivation:${id}`, `data/cache/derivations/${id}.lino`]);
  assert.ok(existsSync(file), 'the record is filed under the working directory');
  const bytes = readFileSync(file, 'utf8');
  assert.ok(symbolic.links_notation.endsWith(`\n${bytes}`), 'links_notation carries the same record');
  const record = fromLino(bytes);
  assert.equal(record.answer_id, id);
  assert.equal(record.rendering, `answer_id=${id};format=text`, 'the final render event names this answer');
});

test('the explain request prints the persisted record in either format', async () => {
  const symbolic = await solveSymbolic({ worker: host, derivationRoot: root }, 'What is 2 + 2?', []);
  const id = answerDerivationId(symbolic.answer);
  const bytes = readFileSync(path.join(root, 'data/cache/derivations', `${id}.lino`), 'utf8');
  const text = runExplain([id], { cwd: root });
  assert.deepEqual(text, { code: 0, stdout: explainText(fromLino(bytes)), stderr: '' });
  assert.ok(text.stdout.startsWith(`derivation ${id}\n`));
  assert.ok(text.stdout.includes('  stage search_queries\n    not recorded\n'));
  assert.deepEqual(runExplain([id, '--format', 'links'], { cwd: root }), { code: 0, stdout: bytes, stderr: '' });
  assert.deepEqual(runExplain(['--format=links', id], { cwd: root }).stdout, bytes);

  const cli = spawnSync(process.execPath, [path.join(REPO_ROOT, 'js/server/main.mjs'), 'explain', id], { cwd: root, encoding: 'utf8' });
  assert.equal(cli.status, 0, cli.stderr);
  assert.equal(cli.stdout, text.stdout, 'node js/server/main.mjs explain is the same request');
});

test('an unknown or unusable id is the seed miss with exit 1', () => {
  const directory = `${root}/data/cache/derivations`;
  assert.deepEqual(runExplain(['answer_ffffffffffffffff'], { cwd: root }), {
    code: 1,
    stdout: '',
    stderr: `Error: "no derivation record for \`answer_ffffffffffffffff\` under ${directory} ${MISS_TAIL}"\n`,
  });
  assert.equal(runExplain(['../secrets'], { cwd: root }).code, 1);
  assert.equal(runExplain([], { cwd: root }).code, 2);
  assert.equal(runExplain(['answer_ffffffffffffffff', '--format', 'yaml'], { cwd: root }).code, 2);
});

test('a refused write is reported, never silently dropped', () => {
  const symbolic = { answer: 'x', evidence_links: ['response:unknown'], links_notation: '' };
  const failing = { writeText: () => { throw new Error('read-only'); }, createDirAll: () => {} };
  const finalized = finalizeAnswer(symbolic, [{ kind: 'web_search:request', payload: 'q' }], root, failing);
  const id = answerDerivationId('x');
  assert.equal(finalized.evidence_links[1], `derivation:${id}`);
  assert.ok(finalized.evidence_links[2].startsWith('derivation:persistence_failed:'));
  const record = fromLino(finalized.links_notation.trimStart());
  assert.deepEqual(record.search_queries, ['q']);
  assert.equal(record.rendering, `answer_id=${id};format=text`);
  assert.equal(RENDER_EMIT_KIND, 'render:emit');
});
