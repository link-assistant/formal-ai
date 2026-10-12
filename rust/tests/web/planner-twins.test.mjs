// R1188-U16: the agentic planner is JavaScript first, and a planner function
// cannot land in JavaScript alone. scripts/check-planner-twins.mjs counts the
// functions js/agentic/ exports with no Rust twin (by snake_case name or by a
// `Mirrors` citation, or a declared Rust built-in or dependency item) and holds
// the count to data/meta/planner-twin-ratchet.lino.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  exportedFunctions,
  functionsWithoutTwin,
  leadingComment,
  mirroredNames,
  ratchetFrom,
  rustFunctions,
  snakeCase,
  standsForBuiltIn,
} from '../../../scripts/check-planner-twins.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

test('a camelCase name meets its snake_case Rust twin', () => {
  assert.equal(snakeCase('extractRequirements'), 'extract_requirements');
  assert.equal(snakeCase('parseJSONLine'), 'parse_json_line');
  assert.equal(snakeCase('plan'), 'plan');
});

test('a Mirrors citation names the twin under another name', () => {
  const source = [
    '/** Mirrors `GeneralChangePlan::links_notation`. */',
    'export function planLinksNotation(plan) {}',
    '// Mirrors `seed::surface_present()`.',
    'export function surfacePresent(text, word) {}',
    'export function uncited() {}',
  ].join('\n');
  assert.deepEqual(exportedFunctions(source), [
    { name: 'planLinksNotation', mirrors: ['links_notation'], builtIn: false },
    { name: 'surfacePresent', mirrors: ['surface_present'], builtIn: false },
    { name: 'uncited', mirrors: [], builtIn: false },
  ]);
  assert.equal(standsForBuiltIn('/** Rust built-in `str::char_indices`. */'), true);
  assert.equal(standsForBuiltIn('/** Mirrors `x`. */'), false);
  assert.equal(leadingComment('const x = 1;\nexport function f() {}', 13), '');
  assert.deepEqual(mirroredNames('Mirrors `a::b.c(x)`'), ['c']);
});

test('a citation reads as check-twin-citations reads it: item keywords and wrapped lines', () => {
  assert.deepEqual(mirroredNames('/** Mirrors `fn answer` in rust/src/x.rs. */'), ['answer']);
  assert.deepEqual(mirroredNames('/** Mirrors `pub const fn slug`. */'), ['slug']);
  assert.deepEqual(mirroredNames('/** Mirrors `struct AppliedRule`. */'), ['AppliedRule']);
  assert.deepEqual(mirroredNames('/**\n * The file a pair saves as. Mirrors\n * `program_spec(..).language.save_as`.\n */'), ['program_spec']);
  assert.equal(standsForBuiltIn('/**\n * Rust built-in\n * `str::lines`.\n */'), true);
  assert.equal(standsForBuiltIn('/** Rust dependency `serde_json::from_str`. */'), true);
});

test('a const, static, struct or enum twins a function only through a citation', () => {
  const rust = rustFunctions([
    'pub const fn is_framework(x: u8) -> bool { true }',
    'pub const PROGRAM_TASKS: &[&str] = &[];',
    'pub struct AppliedRule { kind: String }',
    'pub enum AgenticPlan { Final(String) }',
    'static REPORTS: [u8; 0] = [];',
  ].join('\n'));
  assert.deepEqual([...rust].sort(), ['AgenticPlan', 'AppliedRule', 'PROGRAM_TASKS', 'REPORTS', 'is_framework']);
  const modules = [{
    path: 'js/agentic/a.mjs',
    source: [
      'export function isFramework() {}',
      '/** Mirrors `const PROGRAM_TASKS`. */',
      'export function programTasks() {}',
      '/** Mirrors `struct AppliedRule`. */',
      'export function appliedRule() {}',
      'export function agenticPlan() {}',
    ].join('\n'),
  }];
  assert.deepEqual(functionsWithoutTwin(modules, rust, new Set()), ['js/agentic/a.mjs:agenticPlan']);
});

test('only functions with no twin and outside exempt modules count', () => {
  const modules = [
    { path: 'js/agentic/a.mjs', source: 'export function twinned() {}\nexport function lonely() {}' },
    { path: 'js/agentic/host.mjs', source: 'export function readText() {}' },
  ];
  const missing = functionsWithoutTwin(modules, new Set(['twinned']), new Set(['js/agentic/host.mjs']));
  assert.deepEqual(missing, ['js/agentic/a.mjs:lonely']);
});

test('the ratchet records a ceiling and gives every exempt module a reason', () => {
  const lino = readFileSync(`${REPO_ROOT}/data/meta/planner-twin-ratchet.lino`, 'utf8');
  const { ceiling, exempt } = ratchetFrom(lino);
  assert.ok(Number.isInteger(ceiling));
  const reasons = lino.match(/^\s+reason "/gm) ?? [];
  assert.equal(reasons.length, exempt.size, 'each exempt module states its reason');
});

test('the repository holds the planner twin ratchet', () => {
  const output = execFileSync('node', ['scripts/check-planner-twins.mjs'], { cwd: REPO_ROOT, encoding: 'utf8' });
  assert.match(output, /planner functions without a Rust twin: (\d+) \(ceiling \1\)/);
});
