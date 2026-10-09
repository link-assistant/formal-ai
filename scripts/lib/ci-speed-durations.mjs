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
