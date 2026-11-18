import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {tryTaskDecompositionWithDepth, looksLikeTaskDecomposition}
  from '../../../js/agentic/crate/solver_handlers_task_decomposition.mjs';
import {statedTask} from '../../../js/agentic/crate/task_decomposition_stated_task.mjs';
import {normalizePrompt} from '../../../js/agentic/crate/engine.mjs';

let planChatStep;
before(async () => {
  await installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false});
  ({planChatStep} = await import('../../../js/agentic/planner.mjs'));
});

const ORIGINAL = 'Decompose this development task into the smallest independently verifiable steps: add a lightweight CI reminder for duplicate code across repository languages without running Rust or scanning source bodies locally. Separate evidence gathering, implementation and validation; identify what requires trusted public research.';
const SMALL = 'Decompose the task «Read a file and report its SHA-256 digest» into independently verifiable steps.';
const asks = text => looksLikeTaskDecomposition(normalizePrompt(text));

test('the unchanged natural requests reach the existing decomposition contract', async () => {
  for (const prompt of [ORIGINAL, SMALL]) {
    const expected = tryTaskDecompositionWithDepth(prompt);
    assert(expected);
    const actual = await planChatStep([{role: 'user', content: prompt}], ['read', 'edit', 'write']);
    assert.equal(actual?.kind, 'final');
    assert.equal(actual.answer, expected.answer);
    assert.equal(actual.answer.trim().length > 0, true);
  }
  // Routing parity does not validate the existing lexical atomicity oracle:
  // the original broad goal still needs a separate method-witness review.
});

test('a held-out missing software contract uses the approved four-stage tree', () => {
  const answer = tryTaskDecompositionWithDepth('Split this task into steps: implement the queue rewrite.');
  assert(answer);
  assert.equal(answer.decomposition.root.atomic, false);
  assert.equal(answer.decomposition.root.children.length, 4);
  assert.deepEqual(answer.decomposition.root.children.map(child => child.completion_criterion), [
    'requirements_are_independently_checkable',
    'regression_test_reproduces_failure',
    'requested_behavior_passes',
    'acceptance_checks_pass',
  ]);
  for (const child of answer.decomposition.root.children) {
    assert(answer.answer.includes(child.text));
    assert(answer.answer.includes('[' + child.completion_criterion + ']'));
  }
});

test('task-span recovery preserves quoted values and excludes unrelated framing', () => {
  const prompt = 'Split this task into steps: «Write "A: B" into notes.txt».\n\nDeadline: tomorrow.';
  assert.equal(statedTask(prompt, asks), 'Write "A: B" into notes.txt');
  assert.equal(statedTask('Break this task into steps: create a.txt. Deadline: tomorrow.', asks),
    'create a.txt. Deadline: tomorrow');
});

test('an unresolved leaf and an unsplit depth guard never announce an empty list', () => {
  const unknown = tryTaskDecompositionWithDepth('Split this task: understand everything.');
  assert.equal(unknown.decomposition.root.atomic, false);
  assert.equal(unknown.decomposition.root.children.length, 0);
  assert(!unknown.answer.includes('\n1.'));
  const bounded = tryTaskDecompositionWithDepth('Split this task: implement the queue rewrite.', 0);
  assert.equal(bounded.decomposition.root.reason, 'depth_bound');
  assert(!bounded.answer.includes('\n1.'));
});

test('ordinary execution requests do not become decomposition answers', () => {
  assert.equal(tryTaskDecompositionWithDepth('Create notes.txt containing exactly «hello».'), null);
});
