#!/usr/bin/env node
// Issue #1191 (requirement 9): every GitHub Actions runner label is pinned.
//
// `ubuntu-latest`, `windows-latest` and `macos-latest` are aliases GitHub
// moves to a new image on its own schedule, so one commit can build on two
// different operating systems in two runs and a green run proves nothing about
// the next one. This check reads every workflow and composite action and fails
// on any runner label that ends in `-latest`.
//
// A label is only checked if it can be read, so the check fails closed: a
// `runs-on` it cannot resolve to literal labels is an error too, unless it is
// listed in UNRESOLVABLE_ALLOWED with its reason. It resolves
//
//   runs-on: ubuntu-24.04                      a literal label
//   runs-on: [self-hosted, linux]              every label of the list
//   runs-on: {group: g, labels: [a, b]}        the labels of a runner group
//   runs-on: ${{ matrix.os }}                  every value the job's matrix
//                                              gives `os`: its own list and
//                                              every `include` row
//
// and nothing else: `fromJSON(...)`, `inputs.*`, a computed matrix, a label
// spliced from parts, or a matrix key no row defines cannot be read here.
//
// Usage:
//   node scripts/check-runner-labels.mjs           check this repository
//   node scripts/check-runner-labels.mjs ROOT      check the repository at ROOT

import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import YAML from 'yaml';

const REPOSITORY = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/** The directories whose YAML can choose or describe a runner. */
export const SCANNED_DIRECTORIES = ['.github/workflows', '.github/actions'];

/**
 * `runs-on` expressions this check may not resolve, keyed `file#job`, each
 * with the reason it is safe. Empty: every runner in this repository is a
 * literal or a matrix value, so any new expression must be read by a person
 * before it is let through.
 */
export const UNRESOLVABLE_ALLOWED = new Map();

/** A label GitHub may move to another image without a change here. */
export function isMovingLabel(label) {
  return /-latest$/iu.test(label.trim());
}

const MATRIX_REFERENCE = /^\$\{\{\s*matrix\.([A-Za-z_][A-Za-z0-9_-]*)\s*\}\}$/u;

/** Every value a job's matrix can give `key`, or a reason it cannot be read. */
function matrixValues(job, key) {
  const matrix = job.strategy?.matrix;
  if (matrix === undefined || matrix === null) return { error: 'the job has no strategy.matrix' };
  if (typeof matrix !== 'object' || Array.isArray(matrix)) {
    return { error: 'strategy.matrix is computed (' + JSON.stringify(matrix) + ')' };
  }
  const values = [];
  const dimension = matrix[key];
  if (dimension !== undefined) {
    if (!Array.isArray(dimension)) {
      return { error: 'matrix.' + key + ' is computed (' + JSON.stringify(dimension) + ')' };
    }
    dimension.forEach((value, index) => values.push({ value, via: 'matrix.' + key + '[' + index + ']' }));
  }
  const include = matrix.include;
  if (include !== undefined) {
    if (!Array.isArray(include)) {
      return { error: 'strategy.matrix.include is computed (' + JSON.stringify(include) + ')' };
    }
    include.forEach((row, index) => {
      if (row !== null && typeof row === 'object' && Object.hasOwn(row, key)) {
        values.push({ value: row[key], via: 'matrix.include[' + index + '].' + key });
      }
    });
  }
  if (values.length === 0) return { error: 'no matrix row defines `' + key + '`' };
  return { values };
}

/** Resolves one `runs-on` value to literal labels, or to the reasons it cannot. */
export function resolveRunsOn(runsOn, job) {
  const labels = [];
  const errors = [];
  const literal = (value, via) => {
    if (typeof value !== 'string' && typeof value !== 'number') {
      errors.push(via + ' is not a label (' + JSON.stringify(value) + ')');
    } else if (String(value).includes('${{')) {
      errors.push(via + ' is an expression this check cannot resolve (' + JSON.stringify(value) + ')');
    } else {
      labels.push({ label: String(value), via });
    }
  };
  const one = (value, via) => {
    const reference = typeof value === 'string' ? value.trim().match(MATRIX_REFERENCE) : null;
    if (!reference) return literal(value, via);
    const found = matrixValues(job, reference[1]);
    if (found.error) {
      errors.push(via + ' ' + JSON.stringify(value) + ' cannot be resolved: ' + found.error);
      return undefined;
    }
    // A matrix value is itself a label; it may not defer to another expression.
    for (const row of found.values) literal(row.value, via + ' <- ' + row.via);
    return undefined;
  };
  if (Array.isArray(runsOn)) {
    runsOn.forEach((value, index) => one(value, 'runs-on[' + index + ']'));
  } else if (runsOn !== null && typeof runsOn === 'object') {
    if (runsOn.labels === undefined && runsOn.group === undefined) {
      errors.push('runs-on is an object with neither `group` nor `labels`');
    } else if (Array.isArray(runsOn.labels)) {
      runsOn.labels.forEach((value, index) => one(value, 'runs-on.labels[' + index + ']'));
    } else if (runsOn.labels !== undefined) {
      one(runsOn.labels, 'runs-on.labels');
    }
  } else {
    one(runsOn, 'runs-on');
  }
  return { labels, errors };
}

/** Every `runs-on` found anywhere under `value`, with its path. */
function nestedRunsOn(value, path, found) {
  if (value === null || typeof value !== 'object') return found;
  for (const [key, child] of Object.entries(value)) {
    const childPath = path + (Array.isArray(value) ? '[' + key + ']' : '.' + key);
    if (key === 'runs-on') found.push({ path: childPath, runsOn: child });
    nestedRunsOn(child, childPath, found);
  }
  return found;
}

/**
 * Checks one workflow or composite action. Returns the labels it reads and
 * the problems it finds, each `{file, job, message}`.
 */
export function checkSource(file, source, allowed = UNRESOLVABLE_ALLOWED) {
  const labels = [];
  const problems = [];
  const problem = (job, message) => problems.push({ file, job, message });
  const document = YAML.parseDocument(source);
  if (document.errors.length > 0) {
    problem(null, 'is not valid YAML: ' + document.errors[0].message);
    return { labels, problems };
  }
  const value = document.toJSON();
  const isWorkflow = value !== null && typeof value === 'object' && Object.hasOwn(value, 'jobs');
  if (!isWorkflow) {
    // A composite action cannot choose a runner, so it must not name one.
    for (const nested of nestedRunsOn(value, '', [])) {
      problem(null, nested.path.slice(1) + ' names a runner in a file that is not a workflow');
    }
    return { labels, problems };
  }
  if (value.jobs === null || typeof value.jobs !== 'object' || Array.isArray(value.jobs)) {
    problem(null, '`jobs` is not a mapping');
    return { labels, problems };
  }
  for (const [id, job] of Object.entries(value.jobs)) {
    if (job === null || typeof job !== 'object') {
      problem(id, 'is not a mapping');
      continue;
    }
    if (!Object.hasOwn(job, 'runs-on')) {
      // A reusable-workflow call runs on the runners the called workflow names.
      if (typeof job.uses !== 'string') problem(id, 'has neither `runs-on` nor `uses`');
      continue;
    }
    const resolved = resolveRunsOn(job['runs-on'], job);
    if (resolved.errors.length > 0) {
      const reason = allowed.get(file + '#' + id);
      if (reason === undefined) {
        for (const error of resolved.errors) problem(id, error + ' (fail closed: pin a literal label)');
      }
    }
    for (const entry of resolved.labels) {
      labels.push({ file, job: id, ...entry });
      if (isMovingLabel(entry.label)) {
        problem(id, entry.via + ' is "' + entry.label + '", a moving label; pin the image (for example ubuntu-24.04)');
      }
    }
  }
  return { labels, problems };
}

/** The workflow and action YAML files under `root`, repository-relative. */
export function listSources(root) {
  const files = [];
  const walk = directory => {
    for (const name of readdirSync(directory).sort()) {
      const path = join(directory, name);
      if (statSync(path).isDirectory()) walk(path);
      else if (/\.ya?ml$/u.test(name)) files.push(relative(root, path).split('\\').join('/'));
    }
  };
  for (const directory of SCANNED_DIRECTORIES) {
    if (existsSync(join(root, directory))) walk(join(root, directory));
  }
  return files;
}

/** Checks every workflow and action of the repository at `root`. */
export function checkRepository(root = REPOSITORY, allowed = UNRESOLVABLE_ALLOWED) {
  const files = listSources(root);
  const labels = [];
  const problems = [];
  for (const file of files) {
    const result = checkSource(file, readFileSync(join(root, file), 'utf8'), allowed);
    labels.push(...result.labels);
    problems.push(...result.problems);
  }
  for (const key of allowed.keys()) {
    const [file, job] = key.split('#');
    if (!files.includes(file)) problems.push({ file, job, message: 'is allowed unresolvable, but no such file is checked' });
  }
  return { files, labels, problems };
}

function main(arguments_) {
  if (arguments_.length > 1) {
    console.error('usage: node scripts/check-runner-labels.mjs [ROOT]');
    return 2;
  }
  const root = arguments_.length === 1 ? resolve(arguments_[0]) : REPOSITORY;
  const { files, labels, problems } = checkRepository(root);
  if (files.length === 0) {
    console.error('check-runner-labels: no workflow or action under ' + SCANNED_DIRECTORIES.join(' or ') + ' in ' + root);
    return 1;
  }
  if (problems.length > 0) {
    for (const { file, job, message } of problems) {
      console.error(file + (job === null ? '' : ': job `' + job + '`') + ': ' + message);
    }
    console.error('check-runner-labels: ' + problems.length + ' problem(s); every runner label must be pinned (issue #1191).');
    return 1;
  }
  const distinct = [...new Set(labels.map(entry => entry.label))].sort();
  console.log('check-runner-labels: ' + files.length + ' files, ' + labels.length + ' runner labels, all pinned: ' + distinct.join(', '));
  return 0;
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  process.exitCode = main(process.argv.slice(2));
}
