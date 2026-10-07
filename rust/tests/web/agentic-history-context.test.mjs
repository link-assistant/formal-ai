// Issue #1180 R11: the JavaScript twin of rust/src/history_context
// (js/agentic/crate/history_context.mjs) reads the same rules seed and maps
// a commit to the same memory event as the Rust tests in
// rust/tests/unit/issue_1180_history_context.rs expect.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let history;
let rules;

before(async () => {
  await installNodeHost(new WorkerHost());
  history = await import('../../../js/agentic/crate/history_context.mjs');
  rules = history.historyRules();
});

const RS = '\u001e';
const US = '\u001f';

function logRecord(sha, subject, body, paths) {
  return [RS, sha, US, 'Ada', US, '2026-10-01T00:00:00Z', US, subject, US, body, US, `\n${paths.join('\n')}\n`].join('');
}

describe('history context rules seed', () => {
  test('the rows under the root are read', () => {
    assert.ok(rules.records.some((record) => record.kind === 'commit'));
    assert.ok(rules.trailers.length > 0);
    assert.equal(rules.coauthorPrefix, 'Co-Authored-By:');
    assert.equal(rules.coauthorEvidencePrefix, 'coauthor:');
    assert.ok(rules.itemKeywords.get('ast_census').includes('fn'));
    assert.ok(rules.itemKeywords.get('es_meta_extract').includes('function'));
    assert.deepEqual(rules.itemSpanKeywords.get('ast_census'), ['impl']);
    assert.ok(rules.itemModifiers.includes('pub'));
  });
});

describe('git log parsing and commit formalization', () => {
  test('records come back chronological', () => {
    const text = logRecord('b2', 'second', '', ['b.rs']) + logRecord('a1', 'first', '', ['a.rs']);
    const commits = history.parseLogOutput(text);
    assert.deepEqual(commits.map((commit) => commit.sha), ['a1', 'b2']);
    assert.deepEqual(commits[0].changedPaths, ['a.rs']);
  });

  test('co-author trailers become evidence', () => {
    const body = 'Body.\n\nCo-Authored-By: Claude <noreply@example.com>\nco-authored-by: Ada <ada@example.com>';
    assert.deepEqual(history.coauthors(body, rules), ['Claude', 'Ada']);
    const [raw] = history.parseLogOutput(logRecord('c3', 'subject', body, ['src/x.rs']));
    const event = history.formalizeCommit(raw, [], null, rules);
    assert.ok(event.evidence.includes('coauthor:Claude'));
    assert.ok(event.evidence.includes('coauthor:Ada'));
    assert.ok(event.id.endsWith('c3'));
    assert.equal(event.kind, 'commit');
  });

  test('the number a pattern captures', () => {
    assert.equal(history.patternNumber('Merge pull request #42 from x', 'Merge pull request #%number%'), '42');
    assert.equal(history.patternNumber('no number here', 'Merge pull request #%number%'), null);
  });
});

describe('named top-level items', () => {
  test('rust items are named, diffed and attached as symbol evidence', () => {
    const before = 'pub fn kept() {}\n\nfn edited() {\n    1\n}\n\nstruct Gone;\n';
    const after = 'pub fn kept() {}\n\nfn edited() {\n    2\n}\n\npub(crate) const fn added() {}\n\nimpl Display for Answer {\n}\n';
    const changes = history.diffNamedItems('src/a.rs', 'ast_census', before, after, rules);
    const byItem = Object.fromEntries(changes.map((change) => [change.item, change.delta]));
    assert.deepEqual(byItem, {
      'src/a.rs#fn added': 1,
      'src/a.rs#fn edited': 0,
      'src/a.rs#impl Display for Answer': 1,
      'src/a.rs#struct Gone': -1,
    });
    const [raw] = history.parseLogOutput(logRecord('d4', 'subject', '', ['src/a.rs']));
    const event = history.formalizeCommit(raw, changes, null, rules);
    assert.ok(event.evidence.some((value) => value.endsWith('src/a.rs#fn added')));
  });

  test('ES items are named', () => {
    const source = 'export async function load() {}\nexport class Store {}\nconst limit = 3;\n';
    assert.deepEqual([...history.namedItems(source, 'es_meta_extract', rules).keys()], [
      'class Store',
      'const limit',
      'function load',
    ]);
  });
});
