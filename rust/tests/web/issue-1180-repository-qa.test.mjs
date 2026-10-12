// Issue #1180 R10 and R11 in the JavaScript root: the status and definition
// questions (js/agentic/crate/history_repository_qa.mjs) claim and render as
// rust/tests/unit/issue_1180_repository_qa.rs pins for the native route. Git
// stays native, so the measured range and the commits are handed in.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const ROOT = new URL('../../../', import.meta.url);
let qa;
let rules;

before(async () => {
  await installNodeHost(new WorkerHost());
  const history = await import('../../../js/agentic/crate/history_context.mjs');
  qa = await import('../../../js/agentic/crate/history_repository_qa.mjs');
  rules = history.historyRules();
});

test('the seed names the status rule and the census', () => {
  assert.equal(rules.repositoryQa.statusScript, 'scripts/check-self-development-release.rs');
  assert.equal(rules.repositoryQa.statusLedger, 'data/meta/self-hosting-ledger.lino');
  assert.equal(rules.repositoryQa.statusTagMatch, 'v[0-9]*');
  assert.equal(rules.repositoryQa.censusDir, 'data/meta/self-ast/src');
  assert.equal(rules.repositoryQa.sourceRoot, 'rust');
  assert.deepEqual(rules.repositoryQa.statusCues.map(([language]) => language), ['en', 'ru', 'hi', 'zh', 'es']);
  assert.deepEqual(rules.repositoryQa.definitionCues.map(([language]) => language), ['en', 'ru', 'hi', 'zh', 'es']);
});

test('a cue alone never names a subject', () => {
  assert.equal(qa.statusSubject('Why does the self-development status fail?', rules), 'self-development status');
  assert.equal(qa.statusSubject('Why does the build fail?', rules), null);
  assert.deepEqual(qa.definitionSubjects('What does `evaluate_calculation` do?', rules), ['evaluate_calculation']);
  assert.deepEqual(qa.definitionSubjects('Что делает handleWordProblem?', rules), ['handleWordProblem']);
  assert.deepEqual(qa.definitionSubjects('What does parse do?', rules), []);
  assert.deepEqual(qa.definitionSubjects('Explain evaluate_calculation', rules), []);
});

test('the documentation comment above the item is quoted', () => {
  const source = 'fn before() {}\n\n/// Add two numbers.\n/// Overflow wraps.\n///\n/// Details nobody needs.\n#[must_use]\npub fn add(a: u8, b: u8) -> u8 {\n    a.wrapping_add(b)\n}\n';
  assert.equal(qa.symbolDoc(source, 8, rules), 'Add two numbers. Overflow wraps.');
  assert.equal(qa.symbolDoc(source, 1, rules), null);
});

test('the census locates evaluate_calculation and the answer quotes its documentation', () => {
  const census = readFileSync(new URL('data/meta/self-ast/src/calculation.lino', ROOT), 'utf8');
  const symbol = qa.censusDocumentSymbols(census).find((found) => found.name === 'evaluate_calculation');
  assert.equal(symbol.target, 'src/calculation.rs');
  assert.equal(symbol.kind, 'function');
  const source = readFileSync(new URL('rust/src/calculation.rs', ROOT), 'utf8');
  const commit = {
    id: 'commit:850e52409aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    conversationId: 'issue-1180',
    evidence: [],
  };
  assert.equal(
    qa.definitionAnswer(symbol, source, [commit], 'en', rules),
    `\`evaluate_calculation\` is a function in rust/src/calculation.rs, lines ${symbol.start} to ${symbol.end}. Its documentation comment reads: Evaluate an expression, delegating calculator-supported syntax to \`link-calculator\` and preserving the in-repo evaluator as a fallback for syntax the upstream crate does not support yet. The symbol was introduced by commit 850e52409 for issue #1180 (read from \`git log -S\`, with the issue from the commit's trailer).`,
  );
  assert.ok(qa.definitionAnswer(symbol, null, [], 'en', rules).includes('It carries no documentation comment'));
});

test('the status answer reads the floor and the target from the ledger', () => {
  const ledger = [
    'self_hosting_ledger',
    '  session_trailer "Formal-AI-Session"',
    '  release_cycle_floor "1"',
    '  release_cycle_unit "merged session-backed Formal AI pull request"',
    '  release_cycle_attribution "issue #1069"',
    '  target_policy "non-decreasing"',
    '  release',
    '    tag "v0.352.1"',
    '    target_percentage_basis_points "389"',
    '',
  ].join('\n');
  const answer = qa.statusAnswer(
    'self-development status',
    { tag: 'v0.352.1', commits: 202, sessions: 3 },
    ledger,
    [{ id: 'commit:abc', conversationId: 'issue-1014', evidence: [] }],
    'en',
    rules,
  );
  assert.equal(
    answer,
    'The self-development status is checked by scripts/check-self-development-release.rs, introduced for issue #1014. It measures the commit range v0.352.1..HEAD: 202 commits, 3 of them carrying the Formal-AI-Session trailer. The floor is 1 merged session-backed Formal AI pull request per release cycle (issue #1069), and the self-hosting share may not fall below the target of 3.89% recorded for v0.352.1 (non-decreasing). The status fails when the range holds no merged session-backed Formal AI pull request, or when its projected share falls below that target; this answer reads the range and the ledger data/meta/self-hosting-ledger.lino and does not run the measurement, so run scripts/check-self-development-release.rs to see which of the two applies.',
  );
});
