// Issue #1166 R1166-4 browser twin: the worker's catalog `write_program` arm
// records the `obligation_gap` events the native `WriteProgram` branch of
// rust/src/solver.rs records (`record_obligation_gaps` behind
// `request_carries_work_obligations || request_demands(CiWorkflow)`), in the
// same order and shape, between `legacy_intent` and `procedure_cache`. The
// cases are rust/tests/fixtures/issue-1166/executor-gaps.json, which
// rust/tests/unit/issue_1166_executor_gaps.rs and
// rust/tests/web/issue-1166-executor-gaps.test.mjs read too. The worker port
// (js/worker/formal_ai_worker_obligations.js,
// js/worker/formal_ai_worker_obligation_plans.js) is also held line for line
// to the agentic root (js/agentic/crate/intent_formalization_obligations.mjs).

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { composeGeneralChangePlan } from '../../../js/agentic/general_planner.mjs';
import { obligationGapLines } from '../../../js/agentic/crate/intent_formalization_obligations.mjs';
import { expectationRules } from '../../../js/agentic/crate/obligation_ledger.mjs';
import {
  isCheckable, shippedLedger, splitOnceCheckable, strategies,
} from '../../../js/agentic/crate/task_decomposition.mjs';

const fixture = JSON.parse(readFileSync(new URL('../fixtures/issue-1166/executor-gaps.json', import.meta.url), 'utf8'));
const canonical = readFileSync(new URL('../fixtures/issue-1166/hello-world-kotlin-en.txt', import.meta.url), 'utf8');

// Clauses that reach every branch of the plan reader `derive_expectation`
// consults: literal-file and command-output writes, repository work items,
// checkable splits, a composed-document specification, non-Latin scripts.
const PLAN_PROMPTS = [
  'Create notes.txt with content: hello there',
  'Write "hello" to notes.txt',
  'Write the following into README.md: # Title',
  'Run `ls -la` and save the output to listing.txt',
  'Run "git status" and write its output to status.txt',
  'Implement https://github.com/link-assistant/formal-ai/issues/1166',
  'Write a hello world program in Python\nAlso, fix https://github.com/a/b/pull/12 and add a CI badge',
  'First, create a file named a.txt containing hi\nThen, run `echo x` and save the output to b.txt\nFinally, add tests',
  'Add a function to src/lib.rs, then run the tests, then update the README',
  'Fix the bug in parser.js and add a regression test',
  'Write a program. Also, write a sorting function, then add tests; finally document it.',
  'Compose a note with the following sections: intro, body, outro and save it to note.md',
  'Создай файл notes.txt с текстом привет',
  '创建文件 a.txt，内容是 你好',
];

let host;
let realm;
before(async () => {
  host = new WorkerHost();
  realm = await installNodeHost(host);
});

// Worker values live in the worker's own realm (other prototypes), so they are
// compared as plain JSON.
const plain = (value) => JSON.parse(JSON.stringify(value));
const kinds = (result) => (result.solverEvents || []).map((event) => event.kind);

test('the catalog write_program arm records the native obligation_gap events', async () => {
  assert.equal(fixture.catalogCases.length, 6);
  for (const { id, prompt, expected } of fixture.catalogCases) {
    const result = await host.solve(prompt, []);
    assert.equal(result.intent, 'write_program', id);
    const gaps = plain(result.solverEvents).filter((event) => event.kind === 'obligation_gap');
    assert.deepEqual(gaps, expected.map((payload) => ({ kind: 'obligation_gap', payload })), id);
    if (!expected.length) continue;
    const order = kinds(result);
    const first = order.indexOf('obligation_gap');
    assert.equal(first, order.indexOf('legacy_intent') + 1, id);
    const after = order[first + expected.length];
    assert.ok(after === undefined || after !== 'obligation_gap', id);
  }
});

test('a non-program answer records no obligation_gap event', async () => {
  const result = await host.solve('First, print exactly: "Hi"\nThen, harmonize the quantum flux\n', []);
  assert.notEqual(result.intent, 'write_program');
  assert.ok(!kinds(result).includes('obligation_gap'));
});

test('the worker reports the shared fixture gap lines', () => {
  for (const { id, prompt, expected } of fixture.gapCases) {
    assert.deepEqual(plain(realm.obligationGapLines(prompt)), expected, id);
  }
});

test('the worker gap lines and plan reader agree with the agentic root', () => {
  const prompts = [
    canonical,
    ...fixture.gapCases.map((entry) => entry.prompt),
    ...fixture.bindingCases.map((entry) => entry.prompt),
    ...fixture.catalogCases.map((entry) => entry.prompt),
    ...PLAN_PROMPTS,
  ];
  for (const prompt of prompts) {
    assert.deepEqual(plain(realm.obligationGapLines(prompt)), obligationGapLines(prompt), prompt);
    assert.equal(realm.obligationPlanMode(prompt), composeGeneralChangePlan(prompt)?.mode ?? null, prompt);
    assert.equal(realm.obligationIsCheckable(prompt), isCheckable(prompt), prompt);
    assert.deepEqual(plain(realm.obligationSplitOnceCheckable(prompt)), splitOnceCheckable(prompt), prompt);
  }
});

test('the mirrored data/meta decisions equal the files', () => {
  const rules = expectationRules().map(({ when, mode, expectation, otherwise, reason }) =>
    ({ when, mode, expectation, otherwise: otherwise ?? '', reason }));
  assert.deepEqual(plain(realm.obligationExpectationRules()), rules);
  const approved = strategies().some((strategy) =>
    strategy.activation === 'missing_operation_contract' && shippedLedger().has(strategy.id));
  assert.equal(realm.obligationMissingContractStrategyApproved(), approved);
});
