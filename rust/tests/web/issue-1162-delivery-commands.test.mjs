// Issue #1162: the delivery commands of one Hello World work item.
//
// The fixture rust/tests/fixtures/issue-1162/delivery-commands.json is written
// by experiments/issue-1162/regenerate-delivery-fixture.mjs and asserted by
// the Rust in-crate test tests/unit/agentic-coding/issue_1162_delivery_commands.rs,
// so both runtimes emit the same retirement step, pull request completion
// commands, feedback filter and exact-stdout read. The scratch-repository
// cases replay the run.yml + Main.yml tree the 2026-11 Kotlin and Scala
// deliveries ended with, and the notice cases replay the real orchestrator
// comments of the three test pull requests.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync, appendFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { attach, requestedIn } from '../../../js/agentic/ci_workflow.mjs';
import { recipeCommitCommand, supersededWorkflowStep } from '../../../js/agentic/git_commit.mjs';
import { prCompletionCommands } from '../../../js/agentic/command_reroute.mjs';
import { feedbackNeedsChanges, prFeedbackFilter } from '../../../js/agentic/restart_feedback.mjs';
import { exactSuccessfulStdout } from '../../../js/agentic/tool_result.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const fixture = name => JSON.parse(readText(`rust/tests/fixtures/issue-1162/${name}`));

function recipeFor(prompt) {
  const recipe = programContractAnswer(prompt)?.execution_recipe;
  assert.ok(recipe, 'the Hello World prompt has a program recipe');
  if (requestedIn(prompt)) attach(recipe, prompt);
  return recipe;
}

test('delivery commands match the fixture the Rust runtime asserts', () => {
  const delivery = fixture('delivery-commands.json');
  const kotlinIssue = readText('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md');
  assert.equal(delivery.workflows.length, 3);
  for (const entry of delivery.workflows) {
    const recipe = recipeFor(entry.prompt ?? kotlinIssue);
    assert.equal(recipe.path, entry.program, entry.id);
    assert.equal(recipe.supporting_files.find(file => /\.ya?ml$/u.test(file.path))?.path ?? null, entry.workflow, entry.id);
    assert.equal(supersededWorkflowStep(recipe), entry.retire_step, entry.id);
    if (entry.id === 'kotlin-issue-body') {
      const target = { branch: null, reference: delivery.pull_request.target, bindingRefused: false };
      assert.deepEqual(prCompletionCommands(recipe, target, delivery.pull_request.issue), delivery.pull_request.with_issue);
      assert.deepEqual(prCompletionCommands(recipe, target, delivery.pull_request.target), delivery.pull_request.without_issue);
    }
  }
  assert.equal(prFeedbackFilter(), delivery.feedback_filter);
  for (const entry of delivery.stdout) assert.equal(exactSuccessfulStdout(entry.raw), entry.exact, entry.id);
});

test('the workflow takes the file name the request suggests, never over a literal path', () => {
  for (const entry of fixture('workflow-names.json').cases) {
    const recipe = recipeFor(entry.prompt);
    assert.equal(recipe.supporting_files.find(file => /\.ya?ml$/u.test(file.path))?.path ?? null, entry.workflow, entry.id);
  }
});

test('the issue-titled completion edits the description once and closes the issue', () => {
  const { pull_request: pull } = fixture('delivery-commands.json');
  const [read, edit, ready] = pull.with_issue;
  assert.match(read, /gh pr view 'https:\/\/github\.com\/o\/r\/pull\/2' --json comments,reviews --jq /u);
  assert.match(edit, /gh issue view 'https:\/\/github\.com\/o\/r\/issues\/1' --json title --jq \.title/u);
  assert.match(edit, /Resolves https:\/\/github\.com\/o\/r\/issues\/1/u);
  assert.match(edit, /if test .* != "\$pr_title" \|\| test .* != "\$pr_body"; then gh pr edit/u);
  assert.match(ready, /gh pr ready/u);
  assert.doesNotMatch(pull.without_issue[1], /--title/u);
});

test('every observed orchestrator notice is bookkeeping and every negative is feedback', () => {
  const notices = fixture('own-status-notices.json');
  assert.equal(notices.own_notices.length, 14);
  assert.equal(notices.observed.unmatched, 0);
  const single = item => JSON.stringify({ comments: [item], reviews: [] });
  for (const notice of notices.own_notices) {
    const item = { author: { login: 'konard' }, viewerDidAuthor: true, includesCreatedEdit: false, body: notice.body };
    assert.equal(feedbackNeedsChanges(single(item)), false, notice.key);
  }
  for (const negative of notices.negatives) assert.equal(feedbackNeedsChanges(single(negative.item)), true, negative.id);
  assert.equal(feedbackNeedsChanges(JSON.stringify({ comments: [], reviews: [{ state: 'CHANGES_REQUESTED', body: '' }] })), true);
});

function scratch(callback) {
  const root = mkdtempSync(join(tmpdir(), 'issue-1162-retire-'));
  const remote = join(root, 'remote.git');
  const work = join(root, 'work');
  const sh = (command, cwd = work) => execFileSync('sh', ['-c', command], { cwd, encoding: 'utf8' });
  try {
    sh(`git init -q --bare '${remote}' && git init -q -b issue-1 '${work}'`, root);
    sh(`git config user.email t@example.invalid && git config user.name t && git remote add origin '${remote}'`);
    return callback({ work, remote, sh });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function kotlinRecipe() {
  const recipe = { path: 'Main.kt', language: 'kotlin', source: 'fun main() = println("Hello, World!")\n', commands: ['sh tests/verify-output.sh'], supporting_files: [] };
  attach(recipe, 'Create a GitHub Actions workflow that runs the program on every push and pull request.');
  const workflow = recipe.supporting_files.find(file => file.path.startsWith('.github/workflows/'));
  assert.ok(workflow, 'the workflow is attached');
  return { recipe, workflow };
}

test('a resumed delivery retires the earlier session workflow for the same program and nothing else', { timeout: 60_000 }, () => {
  scratch(({ work, remote, sh }) => {
    const { recipe, workflow } = kotlinRecipe();
    const earlier = workflow.source.replace(/\n {6}- run: \|/u, '\n      - run: |\n          kotlinc Main.kt\n      - run: |');
    mkdirSync(join(work, '.github/workflows'), { recursive: true });
    writeFileSync(join(work, '.github/workflows/run.yml'), earlier);
    writeFileSync(join(work, '.github/workflows/edited.yml'), earlier);
    writeFileSync(join(work, '.github/workflows/lint.yml'), 'name: Lint\non: push\njobs: {}\n');
    writeFileSync(join(work, '.github/workflows/other.yml'), earlier.replaceAll('Main.kt', 'Other.kt'));
    writeFileSync(join(work, 'Main.kt'), recipe.source);
    sh('git add -A && git commit -q -m earlier && git push -q origin issue-1');
    // A tracked copy with uncommitted user edits, and an untracked and a
    // symlinked copy, are never removed.
    appendFileSync(join(work, '.github/workflows/edited.yml'), '# kept by the user\n');
    writeFileSync(join(work, '.github/workflows/untracked.yml'), earlier);
    symlinkSync('run.yml', join(work, '.github/workflows/linked.yml'));

    writeFileSync(join(work, workflow.path), workflow.source);
    const command = recipeCommitCommand(recipe, { branch: null, reference: 'https://github.com/o/r/issues/1' });
    sh(command);
    const tracked = sh('git ls-files .github/workflows').trim().split('\n');
    assert.ok(!tracked.includes('.github/workflows/run.yml'));
    assert.ok(!existsSync(join(work, '.github/workflows/run.yml')));
    for (const kept of [workflow.path, '.github/workflows/edited.yml', '.github/workflows/lint.yml', '.github/workflows/other.yml']) {
      assert.ok(tracked.includes(kept), kept);
    }
    assert.ok(existsSync(join(work, '.github/workflows/untracked.yml')));
    assert.equal(sh('git rev-parse HEAD'), sh(`git --git-dir='${remote}' rev-parse issue-1`));
    assert.match(sh('git show --stat --format=%s HEAD'), /run\.yml/u);

    // The second run of the same delivery is a no-op.
    const head = sh('git rev-parse HEAD');
    assert.match(sh(command), /__formal_ai_commit_unchanged/u);
    assert.equal(sh('git rev-parse HEAD'), head);
  });
});

test('a first delivery with no earlier workflow commits only its own files', { timeout: 60_000 }, () => {
  scratch(({ work, sh }) => {
    const { recipe, workflow } = kotlinRecipe();
    writeFileSync(join(work, 'README.md'), '# test\n');
    sh('git add -A && git commit -q -m init && git push -q origin issue-1');
    writeFileSync(join(work, recipe.path), recipe.source);
    mkdirSync(join(work, '.github/workflows'), { recursive: true });
    writeFileSync(join(work, workflow.path), workflow.source);
    sh(recipeCommitCommand(recipe, { branch: null, reference: 'https://github.com/o/r/issues/1' }));
    assert.deepEqual(sh('git show --name-only --format= HEAD').trim().split('\n').sort(), [workflow.path, recipe.path].sort());
    assert.equal(sh('git status --porcelain'), '');
  });
});
