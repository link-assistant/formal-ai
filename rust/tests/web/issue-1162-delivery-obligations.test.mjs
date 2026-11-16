// Issue #1162 acceptance items 3, 4 and 6: every obligation of one Hello World
// delivery is graded on its own. The three 2026-11-10 pull requests
// (docs/case-studies/issue-1162/raw-data/test-repositories/) are historical
// replays: each must fail with exactly the defects the issue comment lists.
// The delivery the current recipe produces passes every offline obligation,
// and the exact-output obligation stays 'unobserved' until an independent run.
import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { attach, requestedIn } from '../../../js/agentic/ci_workflow.mjs';
import { exactOutputObligation, gradeHelloWorldDelivery } from '../../../scripts/lib/hello-world-delivery-grade.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const replay = name => JSON.parse(fs.readFileSync(`docs/case-studies/issue-1162/raw-data/test-repositories/${name}-pr-2.json`, 'utf8'));
const task = (language, extra = {}) => ({ language, issueNumber: 1, issueTitle: `Implement Hello World in ${language}`, suggestedWorkflowName: 'test-hello-world.yml', branchPattern: '^issue-1-', ...extra });

test('the 2026-11-10 Kotlin/claude delivery fails only on its listed defects', () => {
  const grade = gradeHelloWorldDelivery(replay('kotlin-claude'), task('Kotlin'));
  assert.deepEqual(grade.failed, ['workflow-count', 'workflow-path', 'no-session-evidence', 'title', 'description']);
  assert.deepEqual(grade.unobserved, ['exact-output']);
  assert.equal(grade.passed, false);
});

test('the 2026-11-10 Scala/agent delivery fails on the second workflow, failing checks, the stream and the draft', () => {
  const grade = gradeHelloWorldDelivery(replay('scala-agent'), task('Scala'));
  assert.deepEqual(grade.failed, ['workflow-count', 'workflow-path', 'workflow-runs', 'no-session-evidence', 'title', 'description', 'ready']);
});

test('the 2026-11-10 Rust/codex delivery fails on the missing workflow and the committed binary', () => {
  const grade = gradeHelloWorldDelivery(replay('rust-codex'), task('Rust'));
  assert.deepEqual(grade.failed, ['workflow-count', 'workflow-path', 'workflow-runs', 'no-build-artifacts', 'no-session-evidence', 'description']);
  assert.match(grade.obligations.find(row => row.id === 'no-build-artifacts').detail, /committed: main$/u);
});

function currentDelivery(language) {
  const issue = readText('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md').replaceAll('Kotlin', language);
  const recipe = programContractAnswer(issue)?.execution_recipe;
  assert.ok(recipe, language);
  if (requestedIn(issue)) attach(recipe, issue);
  const files = [recipe.path, ...recipe.supporting_files.map(file => file.path)]
    .map(filename => ({ filename, status: 'added' }));
  return {
    pull_request: {
      title: `Implement Hello World in ${language}`,
      body: `Verified artifact: ${recipe.path}\n\nResolves https://github.com/o/r/issues/1`,
      isDraft: false,
      state: 'OPEN',
      headRefName: 'issue-1-0123456789ab',
      checks: [{ name: 'test', conclusion: 'SUCCESS' }],
    },
    files,
  };
}

test('the delivery the current recipe produces meets every offline obligation for all three languages', () => {
  for (const language of ['Kotlin', 'Scala', 'Rust']) {
    const delivery = currentDelivery(language);
    const grade = gradeHelloWorldDelivery(delivery, task(language));
    assert.deepEqual(grade.failed, [], `${language}: ${JSON.stringify(grade.obligations)}`);
    assert.deepEqual(grade.unobserved, ['exact-output'], language);
    assert.equal(gradeHelloWorldDelivery(delivery, task(language, { stdout: 'Hello, World!\n' })).passed, true, language);
  }
});

test('the exact-output rule rejects a doubled line even when the generated test expects it', () => {
  assert.equal(exactOutputObligation('Hello, World!\n').status, 'pass');
  for (const [stdout, detail] of [
    ['Hello, World!\nHello, World!\n', '2 lines'],
    ['Hello, World!', 'missing final newline'],
    ['Hello, World!\r\n', 'carriage return in output'],
    ['Hello, World! \n', 'different text "Hello, World! \\n"'],
    ['hello, world!\n', 'different text "hello, world!\\n"'],
  ]) {
    assert.deepEqual(exactOutputObligation(stdout), { id: 'exact-output', status: 'fail', detail }, JSON.stringify(stdout));
  }
  assert.equal(exactOutputObligation(undefined).status, 'unobserved');
});

test('the literal ci/test-hello-world.yml requirement (#1450) is not met by a native workflow, and the reverse', () => {
  const native = currentDelivery('Kotlin');
  const literal = { ...native, files: native.files.map(file => file.filename.startsWith('.github/') ? { ...file, filename: 'ci/test-hello-world.yml' } : file) };
  const required = task('Kotlin', { workflowPath: 'ci/test-hello-world.yml' });
  assert.ok(gradeHelloWorldDelivery(native, required).failed.includes('workflow-path'));
  assert.ok(!gradeHelloWorldDelivery(literal, required).failed.includes('workflow-path'));
  assert.ok(gradeHelloWorldDelivery(literal, task('Kotlin')).failed.includes('workflow-path'));
});

test('a program in another language and a description without the issue fail on their own obligations', () => {
  const delivery = currentDelivery('Kotlin');
  delivery.files.push({ filename: 'Main.java', status: 'added' });
  delivery.pull_request.body = 'Verified artifact: Main.kt\n\nSee #12';
  assert.deepEqual(gradeHelloWorldDelivery(delivery, task('Kotlin')).failed, ['program-language', 'description']);
  delivery.pull_request.body = 'Fixes #1';
  delivery.files.push({ filename: 'Main.jar', status: 'added' });
  assert.deepEqual(gradeHelloWorldDelivery(delivery, task('Kotlin')).failed, ['program-language', 'no-build-artifacts']);
});
