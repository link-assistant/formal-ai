// Read the jobs of GitHub Actions workflows for the CI speed gate (PR #1188).
//
// The gate runs in the lint lanes, where the repository's node_modules are not
// installed, so this reads the workflow text by indentation instead of loading
// a YAML library. It understands the shapes this repository writes: jobs at
// two spaces under a column-zero `jobs:`, job keys at four spaces, `needs:` as
// a scalar, an inline list (possibly over several lines) or a block list, and
// `timeout-minutes:` as a number or a `${{ matrix.<key> }}` expression whose
// values the job's matrix lists.

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const JOB_KEY = /^ {2}([A-Za-z0-9_-]+):\s*(?:#.*)?$/;
const JOB_FIELD = /^ {4}([A-Za-z0-9_-]+):\s*(.*)$/;
const MATRIX_TIMEOUT = /^\$\{\{\s*matrix\.([A-Za-z0-9_-]+)\s*\}\}$/;

/** A YAML scalar without its quotes and trailing comment. */
export function unquote(value) {
  const quoted = value.trim().match(/^(['"])(.*?)\1(?:\s+#.*)?$/);
  return quoted ? quoted[2] : value.replace(/\s+#.*$/, '').trim();
}

/** The names an inline or scalar `needs:` value lists. */
function needsList(value) {
  return unquote(value)
    .replace(/^\[|\]$/g, '')
    .split(',')
    .map((name) => unquote(name))
    .filter(Boolean);
}

/** The column-zero block of a top-level key, as lines. */
function topLevelBlock(lines, key) {
  const start = lines.findIndex((line) => line.startsWith(`${key}:`));
  if (start < 0) return [];
  const end = lines.findIndex((line, index) => index > start && /^[^\s#]/.test(line));
  return lines.slice(start, end < 0 ? lines.length : end);
}

/** The events a workflow runs on. */
export function workflowTriggers(text) {
  const lines = text.replace(/\r\n/g, '\n').split('\n');
  const block = topLevelBlock(lines, 'on');
  if (block.length === 0) return [];
  const inline = block[0].replace(/^on:\s*/, '').replace(/(^|\s)#.*$/, '').trim();
  if (inline) return needsList(inline);
  return block
    .slice(1)
    .map((line) => line.match(/^ {2}([a-z_]+):/)?.[1])
    .filter(Boolean);
}

/** The timeout values a `timeout-minutes:` value can take within its job. */
function timeoutValues(value, body) {
  const text = unquote(value);
  if (/^\d+$/.test(text)) return [Number(text)];
  const key = text.match(MATRIX_TIMEOUT)?.[1];
  if (!key) return [];
  const pattern = new RegExp(`(?:^|[\\s{,])${key}:\\s*(\\d+)`, 'g');
  return [...body.join('\n').matchAll(pattern)].map((match) => Number(match[1]));
}

/** Every job of one workflow text, keyed by job id, in file order. */
export function workflowJobs(text) {
  const lines = text.replace(/\r\n/g, '\n').split('\n');
  const block = topLevelBlock(lines, 'jobs');
  const jobs = [];
  let current = null;
  for (const line of block.slice(1)) {
    const key = line.match(JOB_KEY)?.[1];
    if (key) {
      current = { id: key, body: [] };
      jobs.push(current);
    } else if (current) {
      current.body.push(line);
    }
  }
  return jobs.map(({ id, body }) => readJob(id, body));
}

function readJob(id, body) {
  const job = {
    id,
    name: null,
    uses: null,
    timeout: null,
    timeoutValues: [],
    needs: [],
    stepTimeouts: [],
    body: body.join('\n'),
  };
  for (let index = 0; index < body.length; index += 1) {
    const field = body[index].match(JOB_FIELD);
    if (field) {
      const [, key, value] = field;
      if (key === 'name') job.name = unquote(value);
      if (key === 'uses') job.uses = unquote(value);
      if (key === 'timeout-minutes') {
        job.timeout = unquote(value);
        job.timeoutValues = timeoutValues(value, body);
      }
      if (key === 'needs') job.needs = readNeeds(value, body, index);
    }
    const step = body[index].match(/^ {8}timeout-minutes:\s*(\S+)/);
    if (step && /^\d+$/.test(step[1])) job.stepTimeouts.push(Number(step[1]));
  }
  return job;
}

function readNeeds(value, body, index) {
  const text = value.replace(/\s+#.*$/, '').trim();
  if (text.startsWith('[') && !text.includes(']')) {
    let joined = text;
    for (let next = index + 1; next < body.length && !joined.includes(']'); next += 1) {
      joined += ` ${body[next].replace(/\s+#.*$/, '').trim()}`;
    }
    return needsList(joined);
  }
  if (text) return needsList(text);
  const listed = [];
  for (let next = index + 1; next < body.length; next += 1) {
    const item = body[next].match(/^ {6}-\s*(\S+)/);
    if (!item) break;
    listed.push(unquote(item[1]));
  }
  return listed;
}

/** Every workflow under a directory: `{ path, name, triggers, jobs }`. */
export function readWorkflows(root, directory = '.github/workflows') {
  return readdirSync(join(root, directory))
    .filter((file) => /\.ya?ml$/.test(file))
    .sort()
    .map((file) => {
      const text = readFileSync(join(root, directory, file), 'utf8');
      const name = unquote(text.match(/^name:\s*(.+)$/m)?.[1] ?? file);
      return { path: `${directory}/${file}`, name, triggers: workflowTriggers(text), jobs: workflowJobs(text) };
    });
}

/**
 * A pattern matching the display names GitHub gives a job: its `name:` with
 * every `${{ … }}` expression free, or its id when it has no name.
 */
export function displayNamePattern(job) {
  const template = job.name ?? job.id;
  const source = template
    .split(/\$\{\{[\s\S]*?\}\}/)
    .map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
    .join('.*');
  return new RegExp(`^${source}$`);
}

/**
 * The job id a measured display name belongs to. A job reached through a
 * reusable workflow reports `<caller> / <inner>`; it is attributed to the
 * caller, whose `uses:` names the workflow holding the inner job.
 */
export function jobIdForDisplayName(workflow, displayName) {
  const [caller] = displayName.split(' / ');
  const exact = workflow.jobs.filter((job) => displayNamePattern(job).test(displayName));
  const candidates = exact.length ? exact : workflow.jobs.filter((job) => job.uses && displayNamePattern(job).test(caller));
  // The most specific template wins: fewer free characters, more literal text.
  candidates.sort((left, right) => (right.name ?? right.id).length - (left.name ?? left.id).length);
  return candidates[0]?.id ?? null;
}
