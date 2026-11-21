// Read and write the measured CI durations the speed gate checks (PR #1188).
//
// Two records, both regenerated from real CI runs rather than written by hand:
//
// - `data/meta/ci-durations.lino`: per workflow job, the median and maximum
//   minutes over recent runs (`experiments/formal_ai_subagent/ci-durations.mjs`);
// - `data/meta/test-durations.lino`: per test, the seconds one run measured,
//   which the shard planner reads to start the longest tests first.
//
// Both are indented Links Notation: a record name, then indented `key value`
// pairs, then nested records. Quoted values hold names with spaces.

import { readFileSync, existsSync } from 'node:fs';

/** `key value` of one indented Links Notation line, value unquoted. */
function pair(line) {
  const match = line.trim().match(/^([A-Za-z0-9_-]+)\s+(.*)$/);
  if (!match) return null;
  const quoted = match[2].match(/^"((?:[^"\\]|\\.)*)"$/);
  return { key: match[1], value: quoted ? quoted[1].replace(/\\"/g, '"') : match[2] };
}

const indentation = (line) => line.length - line.trimStart().length;

/**
 * `data/meta/ci-durations.lino` as `{ header, workflows: [{ path, name,
 * wallMedian, wallMaximum, runs, jobs: [{ display, key, median, maximum,
 * samples }] }] }`.
 */
export function parseCiDurations(text) {
  const result = { header: {}, workflows: [] };
  let workflow = null, job = null, cohort = null, layout = [];
  for (const line of text.split('\n')) {
    if (!line.trim() || line.trimStart().startsWith('#')) continue;
    const entry = pair(line), depth = indentation(line);
    if (!entry) continue;
    if (depth === 2 && entry.key === 'workflow') {
      workflow = { path: entry.value, jobs: [] };
      result.workflows.push(workflow);
      job = null; cohort = null; layout = [];
    } else if (depth === 2) {
      result.header[entry.key] = entry.value;
    } else if (depth === 4 && entry.key === 'cohort' && workflow) {
      workflow.cohorts ??= [];
      workflow.layout = layout;
      cohort = { name: entry.value, shared: { executionCohort: entry.value }, jobs: [] };
      layout.push({ cohort: workflow.cohorts.length });
      workflow.cohorts.push(cohort);
      job = null;
    } else if (depth === 4 && entry.key === 'job' && workflow) {
      cohort = null;
      job = { display: entry.value };
      layout.push({ job: workflow.jobs.length });
      workflow.jobs.push(job);
    } else if (depth === 4 && workflow) {
      cohort = null; job = null;
      workflow[camel(entry.key)] = fieldValue(entry);
    } else if (depth === 6 && cohort && entry.key === 'job') {
      job = { display: entry.value };
      cohort.jobs.push(job);
      workflow.jobs.push(job);
    } else if (depth === 6 && cohort) {
      job = null;
      cohort.shared[camel(entry.key)] = fieldValue(entry);
    } else if (depth === 6 && job && !cohort) {
      job[camel(entry.key)] = fieldValue(entry);
    } else if (depth === 8 && job && cohort) {
      job[camel(entry.key)] = fieldValue(entry);
    }
  }
  // Resolve only explicitly nested members after all shared fields are known.
  for (const scope of result.workflows) {
    for (const group of scope.cohorts ?? []) {
      for (const member of group.jobs) {
        for (const [key, value] of Object.entries(group.shared)) {
          if (!(key in member)) member[key] = value;
        }
      }
    }
  }
  return result;
}

const camel = (key) => key.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
const numberOr = (value) => (/^-?\d+(?:\.\d+)?$/.test(value) ? Number(value) : value);
const fieldValue = (entry) => entry.key === 'source-head' ? entry.value : numberOr(entry.value);
const quote = (value) => `"${String(value).replace(/"/g, '\\"')}"`;

/** The text of `data/meta/ci-durations.lino` for measured workflows. */
const spelling = (key) => key.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
const scalar = (value) => typeof value === 'number' ? value : quote(value);

function renderJob(lines, job, depth, shared = {}) {
  const pad = ' '.repeat(depth);
  lines.push(`${pad}job ${quote(job.display)}`);
  for (const [key, value] of Object.entries(job)) {
    if (key === 'display' || (Object.hasOwn(shared, key) && shared[key] === value)) continue;
    lines.push(`${pad}  ${spelling(key)} ${scalar(value)}`);
  }
}

/** Preserve cohort scopes, observed provenance and arbitrary job fields. */
export function renderCiDurations({ comment = [], header, workflows }) {
  const lines = comment.map((line) => (line ? `# ${line}` : '#'));
  lines.push('ci-durations');
  for (const [key, value] of Object.entries(header)) lines.push(`  ${key} ${scalar(value)}`);
  for (const workflow of workflows) {
    lines.push(`  workflow ${quote(workflow.path)}`);
    for (const [key, value] of Object.entries(workflow)) {
      if (['path', 'jobs', 'cohorts', 'layout'].includes(key)) continue;
      lines.push(`    ${spelling(key)} ${scalar(value)}`);
    }
    const renderCohort = (index) => {
      const group = workflow.cohorts[index];
      lines.push(`    cohort ${quote(group.name)}`);
      for (const [key, value] of Object.entries(group.shared)) {
        if (key === 'executionCohort' && value === group.name) continue;
        lines.push(`      ${spelling(key)} ${scalar(value)}`);
      }
      for (const job of group.jobs) renderJob(lines, job, 6, group.shared);
    };
    if (workflow.layout) {
      for (const row of workflow.layout) {
        if (row.cohort !== undefined) renderCohort(row.cohort);
        else renderJob(lines, workflow.jobs[row.job], 4);
      }
    } else {
      for (const job of workflow.jobs) renderJob(lines, job, 4);
    }
  }
  return `${lines.join('\n')}\n`;
}

/** `data/meta/test-durations.lino` as a Map from test name to seconds. */
export function parseTestDurations(text) {
  const seconds = new Map();
  let pending = null;
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    const test = trimmed.match(/^test "((?:[^"\\]|\\.)*)"$/) ?? trimmed.match(/^test (\S+)$/);
    if (test) {
      pending = test[1];
      continue;
    }
    const value = trimmed.match(/^seconds (\d+(?:\.\d+)?)$/);
    if (value && pending) {
      seconds.set(pending, Number(value[1]));
      pending = null;
    }
  }
  return seconds;
}

/** The recorded test durations, or none when the file is absent. */
export function readTestDurations(path) {
  return existsSync(path) ? parseTestDurations(readFileSync(path, 'utf8')) : new Map();
}

/** Median of a non-empty list of numbers. */
export function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

/** Minutes rounded to one decimal. */
export const roundMinutes = (minutes) => Math.round(minutes * 10) / 10;

/**
 * Seconds per JavaScript suite file from the `[test-file]` events that
 * `scripts/run-ordered-javascript-tests.mjs` prints in a Layered CI js-tier job
 * log (issue #1571). A `completed` (or `failed`) event carries the file's own
 * `duration_ms`.
 *
 * When the job's budget kills the run, some files started and never reported.
 * Node's `run()` reports file summaries in queue order, so a fast file queued
 * after a slow one stays unreported until the slow one finishes: a missing
 * summary does not mean a file was still running. Only the first unreported
 * file in queue order is certainly still running when the log ends; the time
 * from its `started` line to the last timestamped line is a lower bound for it,
 * marked `complete: false`, taken only when `run-with-budget-warning.sh`
 * reported the kill. The other unreported files are left out. GitHub
 * prefixes every job-log line with an ISO timestamp, which makes the bound
 * computable.
 */
export function parseJavaScriptFileEvents(log) {
  const files = new Map();
  const started = new Map();
  const queued = [];
  let last = null;
  for (const line of log.split('\n')) {
    const stamp = line.match(/^﻿?(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z) /u);
    if (stamp) last = Date.parse(stamp[1]);
    const event = line.match(/\[test-file\] (queued|started|completed|failed) (\S+)(?: (\{.*\}))?\s*$/u);
    if (!event) continue;
    const [, state, file, payload] = event;
    if (state === 'queued') {
      queued.push(file);
      continue;
    }
    if (state === 'started') {
      if (stamp) started.set(file, Date.parse(stamp[1]));
      continue;
    }
    let milliseconds = null;
    try {
      milliseconds = JSON.parse(payload ?? 'null')?.duration_ms ?? null;
    } catch {
      milliseconds = null;
    }
    started.delete(file);
    if (!Number.isFinite(milliseconds) || milliseconds < 0) continue;
    const seconds = milliseconds / 1000;
    const previous = files.get(file);
    if (!previous || seconds > previous.seconds) files.set(file, { seconds, complete: true });
  }
  // A file can pass without a summary event of its own (run 43730269695 group
  // 2 printed `✔ rust/tests/web/server-dialog-lino.test.mjs` and no summary),
  // so a bound is taken only from a log whose budget actually killed the run.
  const killed = /was terminated after \d+s, its full \d+s execution budget/u.test(log);
  const blocking = queued.find((file) => !files.has(file));
  if (killed && last !== null && blocking && started.has(blocking)) {
    files.set(blocking, { seconds: Math.max(0, (last - started.get(blocking)) / 1000), complete: false });
  }
  return files;
}

/**
 * Fold observed seconds into a `javascript-test-durations` record (issue #1571).
 *
 * The js-tier plan used to weigh every unrecorded suite file at 0.1 s, and only
 * about a third of the files were recorded. A shard collecting several
 * unrecorded files that take 40 to 150 s each ran far past its 360 s budget
 * (run 43730269695, suite group 7 of 8: 29 files never reported). This function:
 *
 * - replaces a recorded file's seconds with what CI observed for it, since the
 *   plan runs under those same conditions, and drops the comment and receipt
 *   lines that described the older measurement. Two observations only ever
 *   raise a value: a file the budget killed (its seconds are a lower bound),
 *   and a file whose record carries a receipt (a receipt pins the exact
 *   seconds it measured, and only its own importer may restate them);
 * - adds every unrecorded file observed at or above `floor` seconds;
 * - writes `default-seconds`, the mean of the observed files left unrecorded,
 *   which is the weight the planner gives a file it has no record for.
 *
 * `raiseOnly` preserves existing weights and the fallback when importing a
 * faster cohort, such as PR tests that skip full integration work on main.
 * The result is sorted longest first and is a pure function of its inputs.
 */
export function mergeJavaScriptDurations(text, observed, { source, floor = 2, recordedDate, raiseOnly = false } = {}) {
  const lines = text.replace(/\n$/u, '').split('\n');
  const firstTest = lines.findIndex((line) => /^ {2}test /u.test(line));
  let headerEnd = firstTest < 0 ? lines.length : firstTest;
  while (headerEnd > 0 && /^\s*#/u.test(lines[headerEnd - 1])) headerEnd -= 1;
  const note = '  # observed-run: Layered CI runs whose js-tier `[test-file]` seconds were folded in (issue #1571).';
  const header = lines.slice(0, headerEnd)
    .filter((line) => line !== note && !/^ {2}(default-seconds|observed-run) /u.test(line));
  const entries = [];
  let comments = [];
  for (let index = headerEnd; index < lines.length; index += 1) {
    const line = lines[index];
    const test = line.match(/^ {2}test "((?:[^"\\]|\\.)*)"$/u);
    if (test) {
      const value = lines[index + 1]?.match(/^ {4}seconds (\d+(?:\.\d+)?)$/u);
      if (!value) throw new Error(`test "${test[1]}" has no seconds line`);
      const extra = [];
      index += 1;
      while (/^ {4}\S/u.test(lines[index + 1] ?? '')) extra.push(lines[(index += 1)]);
      entries.push({ file: test[1], seconds: Number(value[1]), text: value[1], comments, extra });
      comments = [];
    } else if (line.trim()) {
      comments.push(line);
    }
  }
  const byFile = new Map(entries.map((entry) => [entry.file, entry]));
  const round = (value) => Math.round(value * 10) / 10;
  const unrecorded = [];
  for (const [file, { seconds, complete = true }] of [...observed].sort(([left], [right]) => left.localeCompare(right))) {
    const entry = byFile.get(file);
    if (entry) {
      const replace = round(seconds) > entry.seconds
        || (complete && !raiseOnly && entry.extra.length === 0 && round(seconds) !== entry.seconds);
      if (replace) {
        Object.assign(entry, { seconds: round(seconds), text: String(round(seconds)), comments: [], extra: [] });
      }
    } else if (seconds >= floor) {
      const added = { file, seconds: round(seconds), text: String(round(seconds)), comments: [], extra: [] };
      entries.push(added);
      byFile.set(file, added);
    } else {
      unrecorded.push(seconds);
    }
  }
  entries.sort((left, right) => right.seconds - left.seconds || left.file.localeCompare(right.file));
  const out = header.map((line) => (recordedDate && /^ {2}recorded /u.test(line) ? `  recorded ${recordedDate}` : line));
  if (source) out.push(note, `  observed-run ${source}`);
  if (unrecorded.length) {
    const mean = unrecorded.reduce((sum, value) => sum + value, 0) / unrecorded.length;
    const previousDefault = raiseOnly ? Number(text.match(/^ {2}default-seconds (\S+)$/mu)?.[1] ?? 0) : 0;
    out.push(`  default-seconds ${Math.max(previousDefault, 0.1, Math.round(mean * 100) / 100)}`);
  }
  for (const entry of entries) out.push(...entry.comments, `  test "${entry.file}"`, `    seconds ${entry.text}`, ...entry.extra);
  return `${out.join('\n')}\n`;
}
