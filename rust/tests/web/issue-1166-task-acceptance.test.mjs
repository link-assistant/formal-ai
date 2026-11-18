// Issue #1166 independent acceptance: every clause of a task gets a verdict
// (discharged, contradicted, unresolved, inapplicable) with its justification
// and span, read from the task text and the candidate's observed files and
// output only. The cases are rust/tests/fixtures/issue-1166/acceptance.json,
// which rust/tests/unit/issue_1166_task_acceptance.rs reads too, so the
// JavaScript twin (js/agentic/task_acceptance.mjs, mirrored into ts/agentic)
// and the native module pin the same meanings and verdicts. The Python
// candidates are executed here, row by row, and only their stdout is observed.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { acceptanceReport, checkTask, specifyTask, workflowEvents } from '../../../js/agentic/task_acceptance.mjs';

const fixtures = new URL('../fixtures/issue-1166/', import.meta.url);
const read = (name) => readFileSync(new URL(name, fixtures), 'utf8');
const fixture = JSON.parse(read('acceptance.json'));
const taskOf = (id) => fixture.specificationCases.find((entry) => entry.id === id).task;
const verdictLines = (report) => report.verdicts.map((verdict) => `${verdict.span[0]}-${verdict.span[1]} ${verdict.verdict} ${verdict.justification}`);

before(async () => {
  await installNodeHost(new WorkerHost());
});

test('outputs_and_meanings_are_read_from_the_task_text', () => {
  assert.equal(fixture.specificationCases.length, 17);
  for (const { id, task, output, meaning } of fixture.specificationCases) {
    const specification = specifyTask(task);
    assert.deepEqual(specification.output, output, id);
    assert.equal(specification.meaning, meaning, id);
  }
});

test('equivalent_paraphrases_share_one_meaning_and_distinct_tasks_do_not', () => {
  for (const { id, cases } of fixture.paraphraseGroups) {
    assert.equal(new Set(cases.map((name) => specifyTask(taskOf(name)).meaning)).size, 1, id);
  }
  for (const { id, cases } of fixture.distinctGroups) {
    assert.equal(new Set(cases.map((name) => specifyTask(taskOf(name)).meaning)).size, cases.length, id);
  }
  assert.equal(new Set(fixture.multilingual.map((name) => specifyTask(read(name)).meaning)).size, 1);
});

test('every_clause_gets_a_verdict_with_a_justification_and_a_span', () => {
  const body = read('issue-1450-body.md');
  const specification = specifyTask(body);
  assert.equal(specification.task_identity, specifyTask(body).task_identity);
  assert.notEqual(specification.task_identity, specifyTask(`${body}\n`).task_identity);
  for (const { id, candidate, complete, verdicts } of fixture.candidateCases) {
    const report = checkTask(specification, { task_identity: specification.task_identity, ...candidate });
    assert.deepEqual(verdictLines(report), verdicts, id);
    assert.equal(report.complete, complete, id);
    for (const verdict of report.verdicts) {
      assert.ok(['discharged', 'contradicted', 'unresolved', 'inapplicable'].includes(verdict.verdict), id);
      assert.ok(verdict.justification !== '' && !/style/u.test(verdict.justification), id);
      assert.equal(Buffer.from(body).subarray(verdict.span[0], verdict.span[1]).toString().includes(verdict.clause.slice(0, 8)), true, id);
    }
  }
});

test('independent_execution_rejects_a_shared_misunderstanding', () => {
  const directory = mkdtempSync(join(tmpdir(), 'issue-1166-acceptance-'));
  try {
    for (const entry of fixture.executionCases) {
      const specification = specifyTask(taskOf(entry.task));
      const path = join(directory, 'main.py');
      writeFileSync(path, entry.program);
      if (entry.generated_test) {
        // The test generated from the same misunderstanding passes on its own.
        writeFileSync(join(directory, 'test_main.py'), entry.generated_test);
        execFileSync('python3', [join(directory, 'test_main.py'), path], { timeout: 30000 });
      }
      const rows = specification.output.table ? specification.output.table.rows.map((row) => row.args) : [[]];
      const observations = rows.map((args) => ({ args, stdout: execFileSync('python3', [path, ...args], { encoding: 'utf8', timeout: 30000 }) }));
      const report = checkTask(specification, {
        task_identity: specification.task_identity,
        program_path: 'main.py',
        files: [{ path: 'main.py', source: entry.program }],
        observations,
      });
      assert.deepEqual(verdictLines(report), entry.verdicts, entry.id);
      assert.equal(report.complete, entry.complete, entry.id);
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('workflow_triggers_are_read_in_every_yaml_form', () => {
  assert.deepEqual(workflowEvents('on: [push, pull_request]\njobs:\n'), ['push', 'pull_request']);
  assert.deepEqual(workflowEvents('"on": push\njobs:\n'), ['push']);
  assert.deepEqual(workflowEvents('on:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n'), ['push', 'pull_request']);
  assert.deepEqual(workflowEvents('on:\n  - push\n  - workflow_dispatch\njobs:\n'), ['push', 'workflow_dispatch']);
  assert.deepEqual(workflowEvents('jobs:\n'), []);
});

test('a_finished_recipe_reports_only_actionable_verdicts', () => {
  const objective = read('issue-1450-body.md');
  const workflow = fixture.candidateCases[0].candidate.files[1];
  const recipe = (source) => ({ language: 'kotlin', path: 'Main.kt', source, supporting_files: [workflow], commands: ['kotlinc Main.kt'] });
  const commented = fixture.candidateCases[0].candidate.files[0].source;
  const report = acceptanceReport(objective, recipe(commented), ['Hello, World!\n']);
  assert.match(report, /^Independent acceptance of the task: 16 discharged, 0 contradicted, 10 unresolved, 6 inapplicable\.$/mu);
  assert.match(report, /^- unresolved \(workflow_status_unobserved, bytes 817-856\): Show a green check mark when tests pass$/mu);
  assert.match(report, /The task is not complete/u);
  const bare = acceptanceReport(objective, recipe('fun main() {\n    println("Hello, World!")\n}\n'), ['Hello, World!\n']);
  assert.match(bare, /^- contradicted \(comments_missing, bytes 192-230\): Add clear comments explaining the code$/mu);
  assert.equal(acceptanceReport(taskOf('twice'), { language: 'python', path: 'main.py', source: 'print("Hi")\nprint("Hi")\n', supporting_files: [], commands: ['python3 main.py'] }, ['Hi\nHi\n']), '');
  assert.match(acceptanceReport(taskOf('twice'), { language: 'python', path: 'main.py', source: 'print("Hi")\n', supporting_files: [], commands: ['python3 main.py'] }, ['Hi\n']), /contradicted \(output_differs/u);
});
