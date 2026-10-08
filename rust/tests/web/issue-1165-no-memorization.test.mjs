// Issue #1165 R1165-8: the no-memorization gate, in the JavaScript root. It
// walks every file under data/ and counts Hello World output literals written
// as a code string, in every quote spelling a stored program uses, exactly as
// `data_stores_no_more_verbatim_hello_world_programs_than_the_ratchet` in
// rust/tests/unit/coding_discovery/no_memorization.rs does; both ratchets fall
// together. The one stored program left is Laravel's (no documentation page
// shows a Laravel Hello World command), so the gate is a ratchet at 1 and
// becomes a hard gate at 0. Source captures (the documentation captures seed,
// page content re-derived from SHA-pinned fixtures, and the issue bodies of
// the requirement-extraction corpus) are counted each on its own ratchet.

import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
/** Mirrors `HELLO_WORLD_PROGRAM_LITERALS_MAX`. */
const HELLO_WORLD_PROGRAM_LITERALS_MAX = 1;
/**
 * Mirrors `SOURCE_CAPTURES`: data captured from a source, each with the Hello
 * World literals it may hold. A path ending in `/` names a directory.
 */
const SOURCE_CAPTURES = [
  ['data/seed/coding-documentation-captures.lino', 23],
  ['data/benchmarks/issue-requirements/', 5],
];

/** Mirrors `source_capture`: the capture a repository-relative path belongs to. */
function sourceCapture(relative) {
  return SOURCE_CAPTURES.find(([capture]) => (capture.endsWith('/') ? relative.startsWith(capture) : relative === capture));
}
/** Mirrors `PROGRAM_QUOTES`: escaped double, single, `\x27`, plain double. */
const PROGRAM_QUOTES = ['\\"', "'", '\\x27', '"'];
const LINE_FEED = '\\\\n';

/** Mirrors `is_expectation_value`. */
function isExpectationValue(before) {
  return ['', '[', '('].includes(before) || before.endsWith(':') || before.endsWith(',')
    || [...before].every((character) => /[\p{L}\p{N}_-]/u.test(character));
}

/** Mirrors `hello_world_program_literals`. */
function helloWorldProgramLiterals(text) {
  let count = 0;
  for (const line of text.toLowerCase().split('\n')) {
    for (const quote of PROGRAM_QUOTES) {
      for (let start = line.indexOf(quote); start !== -1; start = line.indexOf(quote, start + quote.length)) {
        if (quote === '"' && line.slice(0, start).endsWith('\\')) continue;
        let rest = line.slice(start + quote.length);
        if (!rest.startsWith('hello')) continue;
        rest = rest.slice('hello'.length);
        if (rest.startsWith(',')) rest = rest.slice(1);
        if (!rest.startsWith(' world')) continue;
        rest = rest.slice(' world'.length);
        if (rest.startsWith('!')) rest = rest.slice(1);
        if (rest.startsWith(LINE_FEED)) rest = rest.slice(LINE_FEED.length);
        if (!rest.startsWith(quote)) continue;
        if (quote === '"' && isExpectationValue(line.slice(0, start).trim())) continue;
        count += 1;
      }
    }
  }
  return count;
}

function walk(directory, files = []) {
  for (const name of readdirSync(directory)) {
    const full = path.join(directory, name);
    if (statSync(full).isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

test('R1165-8: the counter reads every program quote spelling and leaves an expectation alone', () => {
  const programs = [
    '  code `fn main() {\\n    println!(\\"Hello, world!\\");\\n}`',
    '  code \'print("Hello, world!")\'',
    '  code \'puts "Hello, world!"\'',
    '  code "$this->line(\\x27Hello, world!\\x27);"',
    '  code "echo \'Hello, world!\';"',
  ].join('\n');
  assert.equal(helloWorldProgramLiterals(programs), 5);
  const expectations = ['  output "Hello, world!"', '  {"expectedOutput": "Hello, world!"}', '  ["Hello, world!", "Hello"]'].join('\n');
  assert.equal(helloWorldProgramLiterals(expectations), 0);
});

test('R1165-8: data/ stores no more verbatim Hello World programs than the ratchet', () => {
  let total = 0;
  const captured = new Map(SOURCE_CAPTURES.map(([capture]) => [capture, 0]));
  const offenders = [];
  for (const file of walk(path.join(ROOT, 'data'))) {
    const relative = path.relative(ROOT, file).split(path.sep).join('/');
    const found = helloWorldProgramLiterals(readFileSync(file, 'utf8'));
    const capture = sourceCapture(relative);
    if (capture) captured.set(capture[0], captured.get(capture[0]) + found);
    else if (found > 0) {
      total += found;
      offenders.push(`${relative} (${found})`);
    }
  }
  assert.ok(total <= HELLO_WORLD_PROGRAM_LITERALS_MAX, `data/ stores ${total}: ${offenders.join(', ')}`);
  for (const [capture, maximum] of SOURCE_CAPTURES) {
    assert.ok(captured.get(capture) <= maximum, `${capture} holds ${captured.get(capture)}, above its ratchet of ${maximum}`);
  }
  // The ratchet only goes down: a count below the ceiling means the ceiling
  // must fall with it, in both roots.
  assert.equal(total, HELLO_WORLD_PROGRAM_LITERALS_MAX, `lower the ratchet to ${total}: ${offenders.join(', ')}`);
  assert.deepEqual(offenders, ['data/seed/hello-world-programs.lino (1)']);
});
