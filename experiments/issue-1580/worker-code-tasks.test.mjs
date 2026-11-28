// Issue #1580: code-task extraction retains behavior below the size warning.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {createWorkerContext, plain, REPO_ROOT} from '../../rust/tests/web/support/browser-runtime.mjs';

test('code-task worker sources preserve file-size headroom', () => {
  for (const file of [
    'js/worker/formal_ai_worker_code_tasks.js',
    'ts/worker/formal_ai_worker_code_tasks.ts',
    'js/worker/formal_ai_worker_code_refactoring.js',
    'ts/worker/formal_ai_worker_code_refactoring.ts',
  ]) {
    const source = readFileSync(REPO_ROOT + '/' + file, 'utf8');
    const lines = source.split('\n').length - Number(source.endsWith('\n'));
    assert.ok(lines <= 1400, file + ' has ' + lines + ' lines; warning threshold is 1400');
  }
});

test('extracted code-refactoring handlers preserve success, rejection and refusal in either load order', () => {
  for (const reverse of [false, true]) {
    const worker = createWorkerContext({}, reverse ? {reorderModules: modules => modules.reverse()} : {});
    const original = 'fetchData().then(data => process(data)).then(result => display(result)).catch(error => report(error));';
    const chain = worker.codeRefactoringParseChain(original);
    assert.equal(chain.thens.length, 2);
    const check = plain(worker.codeRefactoringExecutionCheck(original, chain));
    assert.equal(check.status, 'matched');
    assert.equal(check.scenarios, 4);
    assert.equal(check.calls, 3);
    assert.match(check.detail, /fetchData.*process.*display/);
    const rejected = plain(worker.codeRefactoringTraceRun(original, 1));
    assert.equal(rejected.length, 2);
    assert.match(rejected[1], /^report\(error</);
    assert.equal(worker.codeRefactoringParseChain('print(1)'), null);
    const loop = worker.codeRefactoringParseChain('fetchData().then(data => { while (true) {} });');
    assert.equal(worker.codeRefactoringExecutionCheck(original, loop).status, 'skipped');
    assert.equal(worker.tryCodeRefactoring('unrelated question'), null);
  }
});
