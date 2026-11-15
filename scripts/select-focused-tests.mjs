#!/usr/bin/env node
// Select the focused offline tests and policy checks for changed files or a
// captured failure family (issue #1486). They run before any Rust compilation,
// Docker image or Hive solve, so a planner, adapter or workflow regression is
// caught in seconds.
//
// Usage:
//   node scripts/select-focused-tests.mjs --changed <file>...     # print tests, one per line
//   node scripts/select-focused-tests.mjs --base origin/main      # changed files since the merge base
//   node scripts/select-focused-tests.mjs --since <commit>        # changed files since a fetched commit
//   node scripts/select-focused-tests.mjs --all                   # every family (no comparison base)
//   node scripts/select-focused-tests.mjs --family work-item-read-policy
//   node scripts/select-focused-tests.mjs --reason ReadPolicyRefused
//   node scripts/select-focused-tests.mjs ... --json              # {families, tests, checks}
//   node scripts/select-focused-tests.mjs ... --run               # run them sequentially
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export const FOCUSED_TEST_MAP_SCHEMA = 'focused-test-map/v1';

/** Fail closed on a map whose families, components or files do not exist. */
export function validateFocusedTestMap(map, exists = existsSync) {
  const problems = [];
  if (map?.schema !== FOCUSED_TEST_MAP_SCHEMA) problems.push('schema');
  const ids = new Set();
  const reasons = new Map();
  for (const family of map?.families ?? []) {
    if (ids.has(family.id)) problems.push('duplicate family ' + family.id);
    ids.add(family.id);
    if (!family.tests?.length) problems.push('family without tests ' + family.id);
    for (const test of family.tests ?? []) if (!exists(test)) problems.push('missing test ' + test);
    for (const check of family.checks ?? []) if (!Array.isArray(check) || check[0] !== 'node' || !exists(check[1])) problems.push('invalid check ' + JSON.stringify(check));
    for (const reason of family.reasons ?? []) {
      if (reasons.has(reason)) problems.push('reason ' + reason + ' in ' + reasons.get(reason) + ' and ' + family.id);
      reasons.set(reason, family.id);
    }
  }
  for (const component of map?.components ?? []) {
    if (!component.prefixes?.length) problems.push('component without prefixes');
    for (const family of component.families ?? []) if (!ids.has(family)) problems.push('unknown family ' + family);
  }
  return problems;
}

/** Families selected by changed paths, explicit families and captured reasons. */
export function selectFocused(map, { changed = [], families = [], reasons = [] } = {}) {
  const selected = new Set(families);
  for (const reason of reasons) {
    const family = map.families.find(entry => (entry.reasons ?? []).includes(reason));
    if (!family) throw Error('UnmappedFailureReason:' + reason);
    selected.add(family.id);
  }
  for (const path of changed) {
    for (const component of map.components) {
      if (component.prefixes.some(prefix => path === prefix || path.startsWith(prefix))) component.families.forEach(family => selected.add(family));
    }
  }
  const ordered = map.families.filter(family => selected.has(family.id));
  const unknown = [...selected].filter(id => !ordered.some(family => family.id === id));
  if (unknown.length) throw Error('UnknownFailureFamily:' + unknown.join(','));
  const tests = [...new Set(ordered.flatMap(family => family.tests))];
  const checks = [];
  for (const check of ordered.flatMap(family => family.checks ?? [])) {
    if (!checks.some(prior => JSON.stringify(prior) === JSON.stringify(check))) checks.push(check);
  }
  return { families: ordered.map(family => family.id), tests, checks };
}

function values(argv, name) {
  const found = [];
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] !== name) continue;
    for (let next = index + 1; next < argv.length && !argv[next].startsWith('--'); next += 1) found.push(argv[next]);
  }
  return found;
}

function main(argv) {
  const map = JSON.parse(readFileSync('data/benchmarks/focused-test-map.json', 'utf8'));
  const problems = validateFocusedTestMap(map);
  if (problems.length) {
    console.error('focused test map is invalid:\n' + problems.join('\n'));
    return 1;
  }
  const changed = values(argv, '--changed');
  for (const base of values(argv, '--base')) {
    changed.push(...execFileSync('git', ['diff', '--name-only', base + '...HEAD'], { encoding: 'utf8' }).split('\n').filter(Boolean));
  }
  for (const since of values(argv, '--since')) {
    changed.push(...execFileSync('git', ['diff', '--no-renames', '--name-only', since, 'HEAD'], { encoding: 'utf8' }).split('\n').filter(Boolean));
  }
  const families = argv.includes('--all') ? map.families.map(family => family.id) : values(argv, '--family');
  const selection = selectFocused(map, { changed, families, reasons: values(argv, '--reason') });
  console.error('focused families: ' + (selection.families.join(', ') || 'none'));
  if (argv.includes('--json')) console.log(JSON.stringify(selection));
  else if (!argv.includes('--run')) for (const test of selection.tests) console.log(test);
  if (!argv.includes('--run')) return 0;
  for (const command of [...selection.checks, ...selection.tests.map(test => ['node', '--test', '--test-timeout=120000', test])]) {
    console.log('$ ' + command.join(' '));
    const result = spawnSync(command[0], command.slice(1), { stdio: 'inherit', timeout: 300000 });
    if (result.status !== 0) return result.status ?? 1;
  }
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) process.exitCode = main(process.argv.slice(2));
