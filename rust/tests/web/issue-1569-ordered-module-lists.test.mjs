// Issue #1569: PR #1563 declared `mod work_item_file_io;` after
// `mod work_item_references;` in rust/src/agentic_coding/modules.rs. The PR
// passed, because the gate that keeps union-merged lists canonical
// (scripts/normalize-ordered-lists.rs, check-ordered-lists-are-canonical) is a
// rust-script gate that runs only on main, and every main push after it
// failed the lint job.
//
// This is the pull-request-time slice of that gate, run without compiling
// Rust: in a registered `rust_declarations` list, every maximal run of bare
// `mod name;` declarations and every maximal run of `pub use module::...;`
// blocks must already be in the gate's order (by name, by module key) with no
// repeats, so the gate would leave the file byte-equal. The registry, the run
// boundaries and both grammars follow `parse_registry`,
// `normalize_rust_declarations`, `parse_mod_declaration` and
// `parse_reexport_module` in the Rust script. The directory mirror and the
// JS module lists stay with the full gate on main.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';

const POLICY = 'data/meta/merge-conflict-policy.lino';

const unquote = (value) => {
  const trimmed = value.trim();
  return trimmed.length >= 2 && trimmed.startsWith('"') && trimmed.endsWith('"') ? trimmed.slice(1, -1) : trimmed;
};

/** Mirrors `parse_registry`: the `{path, kind}` of every registered list file. */
export function registeredLists(source) {
  const files = [];
  let kind = null;
  let current = null;
  const close = () => {
    if (current) files.push(current);
    current = null;
  };
  for (const line of source.split('\n')) {
    if (line.trim() === '') continue;
    const indent = line.length - line.trimStart().length;
    const trimmed = line.trim();
    const space = trimmed.indexOf(' ');
    const [key, value] = space < 0 ? [trimmed, ''] : [trimmed.slice(0, space), trimmed.slice(space + 1).trim()];
    if (indent === 2) {
      close();
      kind = null;
    }
    if (indent === 4 && key === 'list_kind') kind = unquote(value);
    else if (indent === 4 && key === 'file') {
      close();
      if (kind !== null) current = { path: '', kind };
    } else if (indent === 6 && key === 'path' && current) current.path = unquote(value);
  }
  close();
  return files.filter((file) => file.path !== '');
}

/** Mirrors `parse_mod_declaration`: `[visibility, name]` or null. */
export function modDeclaration(line) {
  if (!line.endsWith(';')) return null;
  const rest = line.slice(0, -1);
  const at = rest.lastIndexOf('mod ');
  if (at < 0) return null;
  const visibility = rest.slice(0, at).trimEnd();
  if (!['', 'pub', 'pub(crate)', 'pub(super)'].includes(visibility)) return null;
  const name = rest.slice(at + 4).trim();
  if (!/^[a-z0-9_]+$/u.test(name)) return null;
  return [visibility, name];
}

/** Mirrors `parse_reexport_module`: `[visibility, key]` or null. */
export function reexportModule(line) {
  const visibility = ['pub', 'pub(crate)', 'pub(super)'].find((candidate) => line.startsWith(`${candidate} use `));
  if (visibility === undefined) return null;
  const rest = line.slice(visibility.length + 5);
  const brace = rest.indexOf('{');
  const key = brace >= 0 ? `${rest.slice(0, brace)}{` : rest.trim().replace(/;+$/u, '').trim();
  const separator = key.indexOf('::');
  if (separator < 0) return null;
  const module = key.slice(0, separator);
  if (module === '' || !/^[a-z0-9_]+$/u.test(module)) return null;
  return [visibility, key];
}

// The gate sorts both runs by Rust byte order (`String::cmp`, a BTreeMap),
// which for these ASCII names is JavaScript's default string order.
function runProblem(kind, lineOf, entries) {
  const canonical = [...new Set(entries)].sort();
  const repeated = [...new Set(entries.filter((entry, position) => entries.indexOf(entry) !== position))];
  if (repeated.length > 0) return [`line ${lineOf[0]}: ${kind} run repeats ${repeated.join(', ')}`];
  const differing = entries.map((entry, position) => position).filter((position) => entries[position] !== canonical[position]);
  if (differing.length === 0) return [];
  const [first] = differing;
  return [`line ${lineOf[first]}: ${kind} run has ${entries[first]} where ${canonical[first]} belongs `
    + `(${differing.length} of ${entries.length} entries out of place)`];
}

/** The problem of each maximal `mod` run and re-export run that is out of order or repeats an entry. */
export function modRunProblems(source) {
  const problems = [];
  const lines = source.split('\n');
  for (let index = 0; index < lines.length;) {
    const entries = [];
    const lineOf = [];
    if (modDeclaration(lines[index])) {
      for (; index < lines.length && modDeclaration(lines[index]); index += 1) {
        entries.push(modDeclaration(lines[index])[1]);
        lineOf.push(index + 1);
      }
      problems.push(...runProblem('mod', lineOf, entries));
    } else if (reexportModule(lines[index])) {
      while (index < lines.length && reexportModule(lines[index])) {
        entries.push(reexportModule(lines[index])[1]);
        lineOf.push(index + 1);
        while (index < lines.length && !lines[index].trimEnd().endsWith(';')) index += 1;
        if (index < lines.length) index += 1;
      }
      problems.push(...runProblem('re-export', lineOf, entries));
    } else index += 1;
  }
  return problems;
}

test('the registry names the Rust declaration lists the gate keeps canonical', () => {
  const lists = registeredLists(fs.readFileSync(POLICY, 'utf8'));
  const declarations = lists.filter((file) => file.kind === 'rust_declarations').map((file) => file.path);
  assert.ok(declarations.includes('rust/src/agentic_coding/modules.rs'), declarations.join(', '));
  assert.ok(declarations.includes('rust/src/lib.rs'), declarations.join(', '));
});

test('the declaration runs PR #1563 committed are refused, and its canonical order passes', () => {
  // PR #1563 inserted work_item_file_io after work_item_references, ahead of
  // work_item_page: two names out of place in one run.
  const committed = 'pub use x::{a};\nmod work_item_evidence;\nmod work_item_references;\nmod work_item_file_io;\n'
    + 'pub mod work_item_page;\npub mod work_item_report;\nfn tail() {}\n';
  assert.deepEqual(modRunProblems(committed),
    ['line 3: mod run has work_item_references where work_item_file_io belongs (3 of 5 entries out of place)']);
  const canonical = 'mod work_item_evidence;\nmod work_item_file_io;\npub mod work_item_page;\nmod work_item_references;\n';
  assert.deepEqual(modRunProblems(canonical), []);
  assert.deepEqual(modRunProblems('pub mod a;\nmod a;\n'), ['line 1: mod run repeats a']);
  // A blank line or any other item ends a run, as in normalize_rust_declarations.
  assert.deepEqual(modRunProblems('mod b;\n\nmod a;\n'), []);
  assert.equal(modDeclaration('pub(in crate) mod a;'), null);
  assert.equal(modDeclaration('mod Upper;'), null);
  const reexports = 'pub use beta::{\n    b,\n};\npub use alpha::a;\npub(crate) use gamma::{c};\npub use gamma::{d};\n';
  assert.deepEqual(modRunProblems(reexports), ['line 1: re-export run repeats gamma::{']);
  assert.deepEqual(modRunProblems('pub use beta::b;\npub use alpha::a;\n'),
    ['line 1: re-export run has beta::b where alpha::a belongs (2 of 2 entries out of place)']);
  assert.equal(reexportModule('pub use crate::x;')?.[1], 'crate::x');
  assert.equal(reexportModule('use alpha::a;'), null);
});

test('every registered Rust declaration list holds its mod runs in canonical order', () => {
  const lists = registeredLists(fs.readFileSync(POLICY, 'utf8')).filter((file) => file.kind === 'rust_declarations');
  const stale = [];
  for (const file of lists) {
    for (const problem of modRunProblems(fs.readFileSync(file.path, 'utf8'))) stale.push(`${file.path} ${problem}`);
  }
  assert.deepEqual(stale, [], 'run `rust-script scripts/normalize-ordered-lists.rs --write` in CI or reorder by hand');
});
