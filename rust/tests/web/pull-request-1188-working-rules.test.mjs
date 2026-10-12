// R1188-U15, U22, U24, U25 and U26: how the work on PR #1188 is done is
// recorded where every subagent reads it (experiments/formal_ai_subagent/
// preamble.md) and is carried by the tools the agents run. Each test pins a
// rule to its record and to the mechanism that keeps it.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const read = (path) => readFileSync(`${REPO_ROOT}/${path}`, 'utf8');
const PREAMBLE = read('experiments/formal_ai_subagent/preamble.md');

test('U15: every owner message is collected verbatim and mapped to its rows', () => {
  const messages = read('docs/case-studies/pull-request-1188/user-messages.md');
  const count = Number(/^Messages: (\d+)\.$/mu.exec(messages)?.[1]);
  const entries = messages.match(/^## \d+\. \d{4}-\d{2}-\d{2}T/gmu) ?? [];
  assert.ok(count > 0, 'the collection states its message count');
  assert.equal(entries.length, count, 'one entry per collected message');
  const mapping = read('experiments/formal_ai_subagent/requirement-coverage.lino');
  const rows = read('docs/requirements/issue-1188-user-requirements.md').match(/^\| (R1188-U\d+) \|/gmu) ?? [];
  for (const row of rows) {
    const id = row.slice(2, -2);
    assert.match(mapping, new RegExp(`\\b${id}\\b`, 'u'), `${id} is mapped to an owner requirement`);
  }
  const output = execFileSync('node', ['experiments/formal_ai_subagent/requirement-coverage.mjs', '--check'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
  });
  assert.match(output, /up to date/u);
});

test('U22: current twelve-subagent capacity retains dated limits and completed work', () => {
  assert.match(PREAMBLE, /At most three subagents at once/u);
  assert.match(PREAMBLE, /Historical limit: At most three subagents at once/u);
  assert.match(PREAMBLE, /up to 12 GPT-6\.1-sol subagents, 13 active agents including the coordinator/u);
  assert.match(PREAMBLE, /reuse completed slots without extra agents/u);
  assert.match(PREAMBLE, /Never stop a running agent early; every started task is delivered in full/u);
  // The task stopped on 2026-10-08 (SPANISH) was resumed and delivered: the
  // response-language debt it was started for is zero.
  assert.match(read('data/meta/response-language-parity-debt.lino'), /^ {2}ceiling 0$/mu);
});

test('U24: local checks are minimal and leave every Rust compile to CI', () => {
  assert.match(PREAMBLE, /Local tests are minimal/u);
  const runner = read('experiments/formal_ai_subagent/local-gates.mjs');
  assert.match(runner, /compiles a Rust script with no JS twin \(CI only\)/u);
  assert.match(runner, /const jsOnly = !process\.argv\.includes\('--with-rust-scripts'\)/u);
});

test('U25: no Rust builds and no repository copies on the workstation', () => {
  assert.match(PREAMBLE, /never copy the whole repository/u);
  assert.match(PREAMBLE, /delete every copy before you report/u);
  const runner = read('experiments/formal_ai_subagent/local-gates.mjs');
  assert.match(runner, /const NEEDS_BUILD = /u, 'a gate that needs cargo or a built binary is skipped locally');
  assert.match(read('data/meta/ci-gates/check-disk-usage-policy.lino'), /disk policy/u);
});

test('U26: the CI fixer continuously monitors and uses the coordinated batch barrier', () => {
  const task = read('experiments/formal_ai_subagent/tasks/cifix-loop.md');
  assert.match(task, /continuously monitor CI\/CD/u);
  assert.match(task, /R1188-U26/u);
  assert.match(task, /never push independently/u);
  assert.match(task, /every fix in the current CI failure batch is fully committed/u);
  assert.match(task, /before reading HEAD/u);
  const arrangement = read('docs/case-studies/pull-request-1188/multi-agent-workflow.md');
  assert.match(arrangement, /four active agents including the coordinator/u);
  assert.match(arrangement, /Historical capacity \(2026-10-09\)/u);
  assert.match(arrangement, /up to 12 GPT-6\.1-sol subagents, with 13 active agents including the coordinator/u);
  assert.match(arrangement, /at least one CI watcher and one whole-ask Formal AI delivery agent active throughout/u);
  assert.match(arrangement, /coordinator is the sole repository writer/u);
  assert.match(arrangement, /exclude every incomplete next cohort from staging and pushing/u);
  assert.match(arrangement, /zero autonomous authorship or amplification credit/u);
  assert.match(arrangement, /unsupported native ports are not automated translation proof/u);
  assert.match(arrangement, /must never implement the requirement itself manually/u);
  assert.match(arrangement, /retry the original request unchanged/u);
});
