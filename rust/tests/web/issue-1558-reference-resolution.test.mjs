// Issue #1558: file names in work-item text are resolved against the
// repository before they are read, a failed read changes the next action, and
// a read that already answered (successfully or not) is never planned again.
// Twin of rust/tests/unit/agentic-coding/issue_1558_reference_resolution.rs.
//
// The sessions replay the recorded Agent CLI request of PR #1493 (system
// prompts, solve prompt, issue fetch) through the server's agentic entry with
// the recorded text of each issue, and answer read, glob and `find` calls from
// a workspace the way the Agent CLI tools do (rust/tests/web/helpers/issue-1558-session.mjs).

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, realpathSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../../js/server/node-host-install.mjs';
import { runSession } from './helpers/issue-1558-session.mjs';
import { parsesAsJsonObject } from '../../../js/agentic/work_item_report.mjs';

const FIXTURES = 'rust/tests/fixtures/issue-1558';
const ROOT = process.cwd();
const worker = new WorkerHost();
let entry;
let references;
let repeatedActionStop;

before(async () => {
  await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
  const { agenticOutcome, chatCompletionFromPlan } = await import('../../../js/server/agentic.mjs');
  const { parseChatRequest } = await import('../../../js/server/chat-request.mjs');
  entry = { worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest };
  references = await import('../../../js/agentic/work_item_references.mjs');
  ({ repeatedActionStop } = await import('../../../js/agentic/repetition_guard.mjs'));
});

const issueText = issue => readFileSync(`${FIXTURES}/issue-${issue}.txt`, 'utf8');

/** A throwaway git workspace holding `files`; removed after `body` returns. */
async function inWorkspace(files, body) {
  const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'issue-1558-')));
  try {
    for (const [file, content] of Object.entries(files)) {
      mkdirSync(path.join(root, path.dirname(file)), { recursive: true });
      writeFileSync(path.join(root, file), content);
    }
    execFileSync('git', ['init', '-q'], { cwd: root });
    if (Object.keys(files).length) execFileSync('git', ['add', '.'], { cwd: root });
    return await body(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

/** The invariants every session of this issue keeps, and its evidence report. */
function settled(session) {
  assert.equal(session.stop, 'final', `the session must end with an answer: ${session.stop}`);
  assert.doesNotMatch(session.final, /^Repeated action stopped/u, 'the repetition guard must not end the session');
  const seen = new Set();
  for (const call of session.calls) {
    const key = call.tool === 'read' ? `read ${call.args.filePath}` : `${call.tool} ${call.arguments}`;
    assert.ok(!seen.has(key), `an answered action was planned again: ${key}`);
    seen.add(key);
  }
  assert.equal(repeatedActionStop(session.messages), null, 'the transcript holds no repeated action');
  // Issue #1560: the answer is a localized report; the record stays on the planned result.
  assert.equal(parsesAsJsonObject(session.final), false, session.final);
  const report = session.record;
  assert.ok(report, 'a work-item final keeps its record');
  assert.equal(report.schema, 'repository-work-item-evidence/v1');
  assert.equal(report.authored, false);
  assert.equal(report.verified, false);
  return report;
}

const reads = (session, root) => session.calls.filter(call => call.tool === 'read').map(call => path.relative(root, call.args.filePath));

describe('issue #1558 references in the recorded issue texts', () => {
  test('1485: the linked `hive-delivery-attempt.mjs` is read at scripts/', async () => {
    const session = await runSession(entry, 1485, ROOT);
    const report = settled(session);
    assert.deepEqual(reads(session, ROOT), ['scripts/hive-delivery-attempt.mjs']);
    assert.equal(report.reason, 'EvidenceCollected');
    assert.equal(report.observations[0].reference, 'hive-delivery-attempt.mjs');
  });

  test('1487: `text_formalization.mjs` is read at js/agentic/crate/', async () => {
    const session = await runSession(entry, 1487, ROOT);
    settled(session);
    const read = reads(session, ROOT);
    assert.equal(read[0], 'js/agentic/crate/text_formalization.mjs');
    assert.ok(!read.includes('text_formalization.mjs'), 'the bare name is not read at the workspace root');
  });

  test('1507: `Node.js` is a term, never read as a file; the other files are still read', async () => {
    const session = await runSession(entry, 1507, ROOT);
    const report = settled(session);
    const read = reads(session, ROOT);
    assert.ok(!read.some(file => path.basename(file) === 'Node.js'), read.join(', '));
    assert.ok(read.includes('js/agentic/node-host.mjs'));
    assert.deepEqual(report.terms, ['Node.js']);
    assert.deepEqual(report.unresolved, []);
  });

  for (const [issue, names] of [[1484, ['formal-ai-draft.mjs', 'formal-ai-draft-image.lib.mjs']], [1488, ['formal-ai-draft.lib.mjs', 'formal-ai-draft.mjs']]]) {
    test(`${issue}: files of the Hive Mind checkout are reported once as unresolved, never read`, async () => {
      const session = await runSession(entry, issue, ROOT);
      const report = settled(session);
      assert.deepEqual(reads(session, ROOT), []);
      assert.equal(report.reason, 'UnresolvedReferences');
      for (const name of names) {
        const rows = report.unresolved.filter(row => row.reference === name);
        assert.equal(rows.length, 1, `${name} is reported exactly once`);
        assert.equal(rows[0].reason, 'OutsideRepository');
        assert.equal(rows[0].repository, 'link-assistant/hive-mind');
      }
    });
  }

  test('reference extraction resolves link text to the linked path', () => {
    const of = (issue, repository = 'link-assistant/formal-ai') =>
      references.workItemReferences(issueText(issue), `https://github.com/${repository}/issues/${issue}`);
    const r1485 = of(1485).find(row => row.name === 'hive-delivery-attempt.mjs');
    assert.equal(r1485.path, 'scripts/hive-delivery-attempt.mjs');
    assert.equal(r1485.external, false);
    const r1487 = of(1487).find(row => row.name === 'text_formalization.mjs');
    assert.equal(r1487.path, 'js/agentic/crate/text_formalization.mjs');
    assert.equal(of(1487).filter(row => row.name === 'text_formalization.mjs').length, 1, 'link text and link are one reference');
    for (const row of of(1484)) assert.equal(row.external, true, row.name);
    assert.equal(references.productName('Node.js'), true);
    assert.equal(references.productName('js/agentic/node-host.mjs'), false);
    assert.equal(references.productName('package.json'), false);
  });
});

describe('issue #1558 a failed read changes the next action', () => {
  test('a bare name missing at the root is searched and the candidate nearest the context is read', async () => {
    await inWorkspace({
      'scripts/runner/helper.mjs': 'export const runner = 1;\n',
      'docs/examples/helper.mjs': 'export const example = 1;\n',
    }, async root => {
      const issueTextValue = 'Fix the runner\n\nThe runner script `helper.mjs` exits early; see the scripts directory.\n\n__formal_ai_exit=0\n';
      const session = await runSession(entry, 7, root, { issueText: issueTextValue, repository: 'example/project' });
      const report = settled(session);
      assert.deepEqual(session.calls.map(call => call.tool), ['glob', 'read']);
      assert.deepEqual(reads(session, root), ['scripts/runner/helper.mjs']);
      assert.equal(report.observations[0].resolvedBy, 'ranked-name-match');
      assert.equal(report.reason, 'EvidenceCollected');
    });
  });

  test('a path that fails to read is searched once, then reported unresolved; the remaining evidence is still read', async () => {
    await inWorkspace({ 'src/present.mjs': 'export const present = 1;\n' }, async root => {
      const issueTextValue = 'Two files\n\nCompare docs/missing-notes.md with src/present.mjs.\n\n__formal_ai_exit=0\n';
      const session = await runSession(entry, 8, root, { issueText: issueTextValue, repository: 'example/project' });
      const report = settled(session);
      assert.deepEqual(session.calls.map(call => [call.tool, call.answer.failed]), [['read', true], ['glob', false], ['read', false]]);
      assert.deepEqual(reads(session, root), ['docs/missing-notes.md', 'src/present.mjs']);
      assert.equal(report.reason, 'UnresolvedReferences');
      assert.deepEqual(report.unresolved, [{ reference: 'docs/missing-notes.md', target: 'docs/missing-notes.md', reason: 'NotFound' }]);
      assert.equal(report.observations.find(row => row.path === 'docs/missing-notes.md').readFailed, true);
    });
  });

  test('without a glob tool the search runs through the shell', async () => {
    await inWorkspace({ 'lib/tool/config.json': '{"a":1}\n' }, async root => {
      const issueTextValue = 'Config\n\nThe `config.json` of the tool is wrong.\n\n__formal_ai_exit=0\n';
      const session = await runSession(entry, 9, root, { issueText: issueTextValue, repository: 'example/project', tools: ['bash', 'read'] });
      settled(session);
      assert.deepEqual(session.calls.map(call => call.tool), ['read', 'bash', 'read']);
      assert.deepEqual(reads(session, root), ['config.json', 'lib/tool/config.json']);
    });
  });

  test('an external reference does not stop the session: the repository file is still read', async () => {
    await inWorkspace({ 'src/app.mjs': 'export default 1;\n' }, async root => {
      const issueTextValue = 'Port\n\nPort [draft.mjs](https://github.com/other/upstream/blob/abc123/scripts/draft.mjs) into src/app.mjs.\n\n__formal_ai_exit=0\n';
      const session = await runSession(entry, 10, root, { issueText: issueTextValue, repository: 'example/project' });
      const report = settled(session);
      assert.deepEqual(reads(session, root), ['src/app.mjs']);
      assert.deepEqual(report.unresolved, [{ reference: 'draft.mjs', target: 'scripts/draft.mjs', reason: 'OutsideRepository', repository: 'other/upstream' }]);
    });
  });
});

describe('issue #1558 a successful read is not planned again (hive-mind PR 4040)', () => {
  test('hive-mind 4033: `src/solve.results.lib.mjs` is read once under its absolute binding', async () => {
    await inWorkspace({
      'src/solve.results.lib.mjs': '// results\nexport const placeholder = true;\n',
      'src/pull-request-changes.lib.mjs': '// changes\nexport const changes = true;\n',
    }, async root => {
      const session = await runSession(entry, 4033, root, { issueText: readFileSync(`${FIXTURES}/hive-mind-4033.txt`, 'utf8'), repository: 'link-assistant/hive-mind' });
      const report = settled(session);
      const read = reads(session, root);
      assert.equal(read.filter(file => file === 'src/solve.results.lib.mjs').length, 1);
      assert.ok(read.includes('src/pull-request-changes.lib.mjs'));
      assert.ok(report.observations.some(row => row.path === 'src/solve.results.lib.mjs' && row.complete === true));
    });
  });

  test('a work item naming an existing file and asking for a change in it reads the file once', async () => {
    await inWorkspace({ 'src/greeting.mjs': 'const greeting = "hello";\nexport default greeting;\n' }, async root => {
      const issueTextValue = 'Rename the greeting\n\nIn src/greeting.mjs, replace «const greeting = "hello";» with «const greeting = "hello, world";». Preserve all other content.\n\n__formal_ai_exit=0\n';
      const session = await runSession(entry, 11, root, { issueText: issueTextValue, repository: 'example/project' });
      const report = settled(session);
      assert.deepEqual(reads(session, root), ['src/greeting.mjs']);
      assert.equal(report.observations[0].complete, true);
    });
  });

  test('the repetition guard stays in place', () => {
    const call = (id, filePath) => ({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: 'read', arguments: JSON.stringify({ filePath }) } }] });
    const failure = id => [call(id, '/ws/missing.mjs'), { role: 'tool', tool_call_id: id, content: 'Error: File not found: /ws/missing.mjs' }];
    const messages = [{ role: 'user', content: 'Read missing.mjs' }, ...failure('a'), ...failure('b'), ...failure('c')];
    assert.match(repeatedActionStop(messages), /^Repeated action stopped after \d attempts/u);
  });
});
