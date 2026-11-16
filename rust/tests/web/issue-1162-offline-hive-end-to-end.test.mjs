// Issue #1162 acceptance items 2 and 7 (sub-issues #1157, #1158, #1160),
// offline: one Hello World work item driven end to end by the JS planner with
// no LLM, against a real temporary git repository, a bare origin and a fake
// `gh` (experiments/issue-1162/offline-hive-end-to-end.mjs). The prompts are Hive
// Mind's own, verbatim, from rust/tests/fixtures/hive-mind/. No network, no
// credentials; the CI checks the fake `gh` reports are simulated, so this is
// local end-to-end evidence, not hosted CI evidence.
//
// Root cause pinned here: Hive Mind's formal-ai prompt says "Update the pull
// request at <url>." and that wording did not bind the delivery to the pull
// request, so the pull request was never read, retitled, described or marked
// ready and stayed a `[WIP]` draft.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { ISSUE, hivePrompt, outcome, session, workspace } from '../../../experiments/issue-1162/offline-hive-end-to-end.mjs';
import { executionRecipeFinalAnswer } from '../../../js/agentic/command_reroute.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';

const has = command => {
  try {
    execFileSync('sh', ['-c', `command -v ${command}`], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
};
const missing = (...commands) => {
  const absent = commands.filter(command => !has(command));
  return absent.length === 0 ? false : `needs ${absent.join(', ')} on PATH`;
};

// A tool call failed when its bash receipt ends with a nonzero exit code or
// the adapter marked it as an error (#1157: "the second run must end
// successfully without a failed tool call").
const failedCalls = run => run.transcript.filter(entry =>
  entry.is_error === true || /\nExit Code: [1-9]\d*\s*$/u.test(String(entry.result?.content ?? entry.result ?? '')));

function assertDelivered(env, language, program) {
  const result = outcome(env);
  assert.equal(result.pr.title, `Implement Hello World in ${language}`);
  assert.equal(result.pr.isDraft, false, 'the pull request is marked ready');
  assert.match(result.pr.body, new RegExp(`^Verified artifact: ${program.replace('.', '\\.')}\\n`, 'u'));
  assert.match(result.pr.body, /- `sh tests\/verify-output\.sh`/u);
  assert.ok(result.pr.body.endsWith(`Resolves ${ISSUE}`), result.pr.body);
  assert.equal(result.status, '', 'the working tree is clean');
  const delivered = ['.github/workflows/test-hello-world.yml', 'README.md', program, 'tests/verify-output.sh'];
  for (const file of delivered) assert.ok(result.pushed.includes(file), `${file} is pushed`);
  assert.ok(result.pushed.every(file => delivered.includes(file) || file === '.gitignore'), result.pushed.join(', '));
  return result;
}

test('Hive Mind\'s verbatim formal-ai prompt ("Update the pull request at") delivers a titled, described, ready pull request; a rerun is a no-op', {
  skip: missing('rustc', 'jq', 'python3'),
  timeout: 300_000,
}, async () => {
  const env = workspace('Rust');
  try {
    const first = await session(env, hivePrompt('exact'));
    assert.equal(first.stop, 'final');
    assert.match(String(first.answer), /the pull request is ready\.$/u);
    const delivered = assertDelivered(env, 'Rust', 'main.rs');
    assert.equal(delivered.commits, '2');
    const calls = delivered.ghCalls;
    for (const call of ['pr view', 'pr checks', 'pr edit', 'pr ready']) assert.ok(calls.includes(call), `${call} ran`);
    assert.ok(calls.indexOf('pr edit') < calls.lastIndexOf('pr ready'), 'the description is written before readiness');

    const second = await session(env, hivePrompt('exact'));
    assert.equal(second.stop, 'final');
    assert.match(String(second.answer), /the pull request is ready\.$/u);
    assert.deepEqual(failedCalls(second).map(entry => entry.arguments), [], 'the second run has no failed tool call');
    const again = assertDelivered(env, 'Rust', 'main.rs');
    assert.equal(again.commits, '2', 'a repeated run makes no new commit');
    const added = again.ghCalls.slice(calls.length);
    assert.ok(!added.includes('pr edit') && !added.includes('pr ready'), added.join(', '));
  } finally {
    fs.rmSync(env.root, { recursive: true, force: true });
  }
});

test('Hive Mind\'s regular prompt ("Your prepared Pull Request:") still binds the delivery to the pull request (#1160)', {
  skip: missing('rustc', 'jq', 'python3'),
  timeout: 300_000,
}, async () => {
  const env = workspace('Rust');
  try {
    const run = await session(env, hivePrompt('regular'));
    assert.equal(run.stop, 'final');
    assertDelivered(env, 'Rust', 'main.rs');
  } finally {
    fs.rmSync(env.root, { recursive: true, force: true });
  }
});

test('a stray Main.jar after a Kotlin delivery is ignored, committed, pushed and the pull request stays ready (#1160)', {
  skip: missing('kotlinc', 'java', 'jq', 'python3'),
  timeout: 900_000,
}, async () => {
  const env = workspace('Kotlin');
  try {
    await session(env, hivePrompt('exact'));
    assertDelivered(env, 'Kotlin', 'Main.kt');
    fs.writeFileSync(path.join(env.dir, 'Main.jar'), 'PK\u0003\u0004 stray build output');
    const run = await session(env, hivePrompt('restart'));
    assert.equal(run.stop, 'final');
    assert.deepEqual(failedCalls(run).map(entry => entry.arguments), [], 'the restart has no failed tool call');
    assert.match(String(run.answer), /the pull request is ready\.$/u);
    const result = assertDelivered(env, 'Kotlin', 'Main.kt');
    assert.ok(!result.tracked.includes('Main.jar') && !result.pushed.includes('Main.jar'));
    assert.ok(result.pushed.includes('.gitignore'));
    assert.match(fs.readFileSync(path.join(env.dir, '.gitignore'), 'utf8'), /^\/?Main\.jar$/mu);
  } finally {
    fs.rmSync(env.root, { recursive: true, force: true });
  }
});

test('an unchanged tree on a repeated commit is reported as success, not as a marker (#1157)', () => {
  const recipe = programContractAnswer('Write a Rust program that prints `Hello, World!`.').execution_recipe;
  const target = { branch: 'issue-1162', reference: 'https://github.com/example/repo/issues/1162', bindingRefused: false };
  const outputs = recipe.commands.map(() => '');
  const unchanged = executionRecipeFinalAnswer(recipe, [...outputs, '__formal_ai_commit_unchanged\n0123456789abcdef'], target);
  assert.match(unchanged, /Nothing to commit: the verified files already match the last commit on `issue-1162`/u);
  assert.match(unchanged, /is a success, not a failure/u);
  const committed = executionRecipeFinalAnswer(recipe, [...outputs, '0123456789abcdef'], target);
  assert.doesNotMatch(committed, /Nothing to commit/u);
});
