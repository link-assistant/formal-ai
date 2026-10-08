#!/usr/bin/env node
// The tally of the tasks delegated to Formal AI on PR #1188 (R1188-U13).
//
// docs/case-studies/pull-request-1188/formal-ai-dogfood.md keeps one ladder
// row per delegated task: `| <id> | <task> | <before> | <after> |`. This
// script counts them instead of a person keeping the count:
//
// - each row's outcome (the bold word that opens its Before cell): passed,
//   failed, partial;
// - each failure's resolution, read from its After cell: fixed (a pass after
//   the fix, `Fixed`, `Fixed in T<n>`), open (`Open`), not reproduced;
// - per subagent tag (`REQ-ROUTE:` opening the task cell), the same counts.
//
// The rule a failure answers to: it is fixed by a general mechanism with a
// regression test. A fixed failure whose row names no test, pin, gap or row
// that carries the fix, and whose id no test file under rust/tests/ cites, is
// counted as `fixed-without-test`, and that count is
// held to the ceiling in data/meta/formal-ai-tally-ratchet.lino, which only
// falls. A failure with no resolution at all always fails the check.
//
// Usage: node scripts/tally-formal-ai-dogfood.mjs [--write | --check]
//   --write renders docs/case-studies/pull-request-1188/formal-ai-tally.md;
//   --check compares it and checks the ratchet (gate check-formal-ai-tally).

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const LEDGER = 'docs/case-studies/pull-request-1188/formal-ai-dogfood.md';
const TALLY = 'docs/case-studies/pull-request-1188/formal-ai-tally.md';
const RATCHET = 'data/meta/formal-ai-tally-ratchet.lino';

/** What names the regression test or the fix that carries one. */
const TEST_REFERENCE = /\btests?\b|\.test\.mjs|\.rs\b|\bpins?\b|\bpinned\b|\bregression\b|\bG\d+\b|\bT\d+\b/u;

/**
 * The ladder rows of the ledger: `{id, task, before, after}`.
 * @param {string} text
 * @returns {Array<{id: string, task: string, before: string, after: string}>}
 */
export function ladderRows(text) {
  const rows = [];
  for (const line of text.split('\n')) {
    const match = /^\| (T?\d+[a-z]?) \| (.*) \|\s*$/u.exec(line);
    if (!match) {
      continue;
    }
    const cells = match[2].split(' | ');
    if (cells.length < 3) {
      continue;
    }
    const after = cells.pop();
    const before = cells.pop();
    rows.push({ id: match[1], task: cells.join(' | '), before, after });
  }
  return rows;
}

/**
 * The rows of the ledger's "Edits by hand" table (R1188-U13): per agent and
 * round, the edits Formal AI made and the files the agent wrote by hand.
 * @param {string} text
 * @returns {Array<{id: string, agent: string, round: string, delegated: number, byHand: number, why: string}>}
 */
export function handRows(text) {
  const rows = [];
  for (const line of text.split('\n')) {
    const match = /^\| (H\d+) \| ([^|]+?) \| ([^|]+?) \| (\d+) \| (\d+) \| (.*) \|\s*$/u.exec(line);
    if (match) {
      rows.push({
        id: match[1],
        agent: match[2],
        round: match[3],
        delegated: Number(match[4]),
        byHand: Number(match[5]),
        why: match[6],
      });
    }
  }
  return rows;
}

/**
 * The share of edits Formal AI made, per agent and over all, as text.
 * @param {ReturnType<typeof handRows>} rows
 * @returns {Array<string>}
 */
export function renderHandEdits(rows) {
  if (rows.length === 0) {
    return [];
  }
  const share = (delegated, byHand) =>
    delegated + byHand === 0 ? '-' : `${Math.round((100 * delegated) / (delegated + byHand))}%`;
  const sums = new Map();
  for (const row of rows) {
    const sum = sums.get(row.agent) ?? { delegated: 0, byHand: 0 };
    sum.delegated += row.delegated;
    sum.byHand += row.byHand;
    sums.set(row.agent, sum);
  }
  const delegated = rows.reduce((total, row) => total + row.delegated, 0);
  const byHand = rows.reduce((total, row) => total + row.byHand, 0);
  return [
    '## Formal AI edits beside edits by hand',
    '',
    'From the "Edits by hand" table of the ledger: per agent, the edits it delegated to Formal AI',
    'and the files it wrote itself. A tool run by rule counts as neither.',
    '',
    '| Who | By Formal AI | By hand | Delegated share |',
    '| --- | ---: | ---: | ---: |',
    `| **All** | ${delegated} | ${byHand} | ${share(delegated, byHand)} |`,
    ...[...sums]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([agent, sum]) => `| ${agent} | ${sum.delegated} | ${sum.byHand} | ${share(sum.delegated, sum.byHand)} |`),
    '',
  ];
}

/**
 * A row's outcome: `passed`, `failed`, `partial` or `unrecorded`.
 * @param {string} before
 * @returns {string}
 */
export function outcomeOf(before) {
  const lead = before.replace(/\*\*/gu, '').trim();
  if (/^Partial/iu.test(lead)) {
    return 'partial';
  }
  if (/^Fail/iu.test(lead)) {
    return 'failed';
  }
  if (/^Pass/iu.test(lead)) {
    return 'passed';
  }
  return 'unrecorded';
}

/**
 * A failure's resolution: `fixed`, `open`, `not-reproduced` or `unresolved`.
 * @param {string} after
 * @returns {string}
 */
export function resolutionOf(after) {
  const lead = after.replace(/\*\*/gu, '').trim();
  if (/^(?:Not reproduced|No change needed)/iu.test(lead)) {
    return 'not-reproduced';
  }
  if (/^Open\b/iu.test(lead) || /^Rephrased\b.*\bG\d+\b/iu.test(lead)) {
    return 'open';
  }
  if (/^(?:Pass|Fixed|Fix\b|Safe\b)/iu.test(lead) || /\bfixed\b/iu.test(lead)) {
    return 'fixed';
  }
  return 'unresolved';
}

/**
 * The subagent tag that opens a task cell (`REQ-ROUTE: …`), or `coordinator`.
 * @param {string} task
 * @returns {string}
 */
export function agentOf(task) {
  const tag = /^([A-Z][A-Z0-9-]{2,}):/u.exec(task.trim())?.[1];
  return tag && !/^[GT]\d+$/u.test(tag) ? tag : 'coordinator';
}

/**
 * The tally of the ladder rows.
 * @param {ReturnType<typeof ladderRows>} rows
 * @param {Set<string>} citedIds task ids a test file names (`T92`)
 */
export function tally(rows, citedIds = new Set()) {
  const empty = () => ({ tasks: 0, passed: 0, failed: 0, partial: 0, unrecorded: 0, fixed: 0, open: 0, 'not-reproduced': 0, unresolved: 0 });
  const total = empty();
  const agents = new Map();
  const fixedWithoutTest = [];
  const unresolved = [];
  for (const row of rows) {
    const agent = agents.get(agentOf(row.task)) ?? empty();
    agents.set(agentOf(row.task), agent);
    const outcome = outcomeOf(row.before);
    for (const counts of [total, agent]) {
      counts.tasks += 1;
      counts[outcome] += 1;
    }
    if (outcome !== 'failed' && outcome !== 'partial') {
      continue;
    }
    const resolution = resolutionOf(row.after);
    for (const counts of [total, agent]) {
      counts[resolution] += 1;
    }
    if (resolution === 'unresolved') {
      unresolved.push(row.id);
    } else if (resolution === 'fixed' && !TEST_REFERENCE.test(row.after) && !citedIds.has(row.id)) {
      fixedWithoutTest.push(row.id);
    }
  }
  return { total, agents, fixedWithoutTest, unresolved };
}

/**
 * The tally page.
 * @param {ReturnType<typeof tally>} result
 * @returns {string}
 */
export function renderTally({ total, agents, fixedWithoutTest, unresolved }, hand = []) {
  const header = '| Who | Tasks | Passed | Failed | Partial | Fixed | Open | Not reproduced | Unresolved |';
  const rule = '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |';
  const line = (name, counts) =>
    `| ${name} | ${counts.tasks} | ${counts.passed} | ${counts.failed} | ${counts.partial} | ${counts.fixed} | ` +
    `${counts.open} | ${counts['not-reproduced']} | ${counts.unresolved} |`;
  const byAgent = [...agents].sort(([left], [right]) => left.localeCompare(right)).map(([name, counts]) => line(name, counts));
  return [
    '# Formal AI delegation tally for pull request #1188',
    '',
    `Generated by \`node scripts/tally-formal-ai-dogfood.mjs --write\` from the ladder of`,
    '[formal-ai-dogfood.md](formal-ai-dogfood.md); `--check` regenerates and compares. Do not edit by hand.',
    '',
    'A task is one small step handed to Formal AI. Its outcome is the bold word that opens the',
    'Before cell; a failure is resolved by its After cell: fixed by a general mechanism (with the',
    'regression test, pin, gap or row that carries it), still open, or not reproduced.',
    '',
    header,
    rule,
    line('**All**', total),
    ...byAgent,
    '',
    `Fixed failures whose row names no regression test, pin, gap or carrying row: ${fixedWithoutTest.length}` +
      (fixedWithoutTest.length ? ` (${fixedWithoutTest.join(', ')}).` : '.'),
    `Failures with no resolution: ${unresolved.length}` + (unresolved.length ? ` (${unresolved.join(', ')}).` : '.'),
    '',
    ...renderHandEdits(hand),
  ].join('\n');
}

/** The task ids (`T92`) that the test files under rust/tests/ name. */
function citedTaskIds(root) {
  const files = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', 'rust/tests/web', 'rust/tests/unit'], {
    cwd: root,
    encoding: 'utf8',
  })
    .split('\n')
    .filter((path) => (path.endsWith('.mjs') || path.endsWith('.rs')) && existsSync(join(root, path)));
  const ids = new Set();
  for (const path of files) {
    for (const match of readFileSync(join(root, path), 'utf8').matchAll(/\bT\d+\b/gu)) {
      ids.add(match[0]);
    }
  }
  return ids;
}

function ceilingOf(text) {
  const match = /^\s*fixed[-_]without[-_]test[-_]ceiling\s+(\d+)\s*$/mu.exec(text);
  if (!match) {
    throw new Error(`${RATCHET} names no fixed-without-test-ceiling`);
  }
  return Number(match[1]);
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const read = (path) => readFileSync(join(root, path), 'utf8');
  const ledger = read(LEDGER);
  const result = tally(ladderRows(ledger), citedTaskIds(root));
  const page = renderTally(result, handRows(ledger));
  if (argv.includes('--write')) {
    writeFileSync(join(root, TALLY), page);
    console.log(`wrote ${TALLY}: ${result.total.tasks} tasks, ${result.total.failed} failed, ${result.total.fixed} fixed`);
    return 0;
  }
  const problems = [];
  let current = '';
  try {
    current = read(TALLY);
  } catch {
    current = '';
  }
  if (current !== page) {
    problems.push(`${TALLY} is stale; run node scripts/tally-formal-ai-dogfood.mjs --write`);
  }
  if (result.unresolved.length > 0) {
    problems.push(`failure rows with no resolution in their After cell: ${result.unresolved.join(', ')}`);
  }
  const ceiling = ceilingOf(read(RATCHET));
  const measured = result.fixedWithoutTest.length;
  if (measured > ceiling) {
    problems.push(`fixed failures naming no regression test: ${measured} above the ceiling ${ceiling} (${result.fixedWithoutTest.join(', ')})`);
  } else if (measured < ceiling) {
    problems.push(`fixed failures naming no regression test fell to ${measured}; lower fixed-without-test-ceiling in ${RATCHET}`);
  }
  for (const problem of problems) {
    console.error(`::error::${problem}`);
  }
  console.log(`formal-ai tally: ${result.total.tasks} tasks, ${result.total.failed} failed, ${result.total.fixed} fixed, ${measured} fixed without a named test (ceiling ${ceiling})`);
  return problems.length ? 1 : 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
