import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { typedProgramOf, executeTypedProgram } from '../../../scripts/lib/rust-specification-programs.mjs';
import { testFunctions, tokenize } from '../../../scripts/lib/rust-specification-cases.mjs';

const root = new URL('../../../', import.meta.url).pathname;
function parse(body) {
  const source = '#[test] fn renamed_contract() { ' + body + ' }';
  return typedProgramOf(testFunctions(tokenize(source))[0].body, { source, root });
}
async function passes(body, assertions) {
  const parsed = parse(body);
  assert.ok(parsed.program, parsed.reason);
  const result = await executeTypedProgram({}, parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, assertions);
  return { parsed, result };
}

test('mixed tuple elements retain types, arity and every native assertion', async () => {
  const unsupported = parse(`let rows = [("quartz", 7, true), ("granite", 9, false)];
    for (word, amount, enabled) in rows { assert!(word.len_unavailable_if_guessed()); }`);
  assert.equal(unsupported.program, undefined);
  assert.match(unsupported.reason, /unsupported native method/);
  const actual = await passes(`let rows = [("quartz", 7, true), ("granite", 9, false)];
    for (word, amount, enabled) in rows {
      assert!(word.contains("a")); assert_ne!(amount, 0); assert_eq!(enabled, enabled);
    }`, 6);
  assert.equal(actual.parsed.program.nativeAssertions, 3);
  assert.deepEqual(actual.parsed.program.steps[1].bindings.map(v => v.type), ['string', 'number', 'boolean']);
  assert.deepEqual(actual.result.iterations, [{ parameter: '(word,amount,enabled)', length: 2 }]);
});

test('grouping and a one-element tuple have distinct exact native types', async () => {
  await passes('assert_eq!((17), 17); assert_eq!((17,), (17,)); for (element,) in [(17,), (21,)] { assert_ne!(element, 0); }', 4);
  const parsed = parse('assert_eq!((17,), 17);');
  assert.equal(parsed.program, undefined);
  assert.match(parsed.reason, /type mismatch/);
});

test('renamed nested patterns and nested tuple values use the same mechanism', async () => {
  await passes(`let rows = [(("maple", true), (7,)), (("willow", false), (9,))];
    for ((arboreal, flag), (number,)) in rows {
      assert!(!arboreal.contains("z")); assert_eq!(flag, flag); assert_ne!(number, 0);
    }`, 6);
});

test('explicit immutable reference patterns require a borrowed producer', async () => {
  await passes(`let rows = [("maple", 7), ("willow", 9)];
    for &(name, amount) in &rows { assert!(!name.contains("z")); assert_ne!(amount, 0); }`, 4);
  const parsed = parse('for &(name, amount) in [("maple", 7)] { assert_ne!(amount, 0); }');
  assert.equal(parsed.program, undefined);
  assert.match(parsed.reason, /borrowed iteration/);
});

test('iterator and match-ergonomic tuple bindings preserve iteration order', async () => {
  await passes('let rows = [("maple", 7), ("willow", 9)]; for (tree, amount) in rows.iter() { assert!(!tree.contains("z")); assert_ne!(amount, 0); }', 4);
});

test('tuple element effects execute exactly once in left-to-right native order', async () => {
  const parsed = parse(`let solver = formal_ai::UniversalSolver::default();
    let rows = [(solver.solve("first"), solver.solve("second")), (solver.solve("third"), solver.solve("fourth"))];
    for (left, right) in rows { assert_eq!(left.intent, "heldout"); assert_eq!(right.intent, "heldout"); }`);
  assert.ok(parsed.program, parsed.reason);
  const calls = [];
  const host = { async solve(prompt) {
    calls.push('start:' + prompt);
    await new Promise(resolve => setTimeout(resolve, prompt === 'first' ? 4 : 0));
    calls.push('end:' + prompt);
    return { content: prompt, intent: 'heldout' };
  } };
  const result = await executeTypedProgram(host, parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, 4);
  assert.deepEqual(calls, ['start:first', 'end:first', 'start:second', 'end:second', 'start:third', 'end:third', 'start:fourth', 'end:fourth']);
});

test('a failing original assertion stops before later tuple iteration effects', async () => {
  const parsed = parse(`let solver = formal_ai::UniversalSolver::default();
    for (prompt, expected) in [("first", "wrong"), ("later", "heldout")] {
      let response = solver.solve(prompt); assert_eq!(response.intent, expected);
    }`);
  assert.ok(parsed.program, parsed.reason);
  const calls = [];
  const result = await executeTypedProgram({ async solve(prompt) { calls.push(prompt); return { content: prompt, intent: 'heldout' }; } }, parsed.program);
  assert.equal(result.status, 'failed');
  assert.equal(result.assertions, 1);
  assert.deepEqual(calls, ['first']);
});

test('wrong tuple arity, heterogeneous row types and duplicate binding reject the complete case', () => {
  for (const body of [
    'for (x, y) in [(1,)] { assert_eq!(x, 1); }',
    'for (x,) in [(1, 2)] { assert_eq!(x, 1); }',
    'for ((x, y), z) in [(1, 2)] { assert_eq!(x, 1); }',
    'let rows = [(1, "x"), ("x", 1)]; assert_eq!(rows.len(), 2);',
    'for (x, x) in [(1, 2)] { assert_eq!(x, 1); }',
    'for (x, y) in [(1, "text")] { assert_eq!(x, y); }',
  ]) {
    const parsed = parse(body);
    assert.equal(parsed.program, undefined, body);
    assert.ok(parsed.reason);
  }
});

test('unsupported tails, effects, rest patterns and malformed commas never discard a case fragment', () => {
  for (const body of [
    'assert_eq!((1,,2), (1,2));',
    'assert_eq!((1,,), (1,));',
    'assert_eq!((,), ());',
    'assert_eq!((1,), (1,)); mutate_repository();',
    'assert_eq!((1,), (1,)); assert_eq!((1,).unknown, 1);',
    'for (x, ..) in [(1, 2)] { assert_eq!(x, 1); }',
    'for (mut x, y) in [(1, 2)] { assert_eq!(x, 1); }',
    'for (&x, y) in [(1, 2)] { assert_eq!(x, 1); }',
  ]) {
    const parsed = parse(body);
    assert.equal(parsed.program, undefined, body);
    assert.ok(parsed.reason);
  }
});

test('wildcards and unit tuples conserve the original assertion count', async () => {
  const actual = await passes('for (_, amount) in [("maple", 7), ("willow", 9)] { assert_ne!(amount, 0); } for () in [(), ()] { assert_eq!((), ()); }', 4);
  assert.equal(actual.parsed.program.nativeAssertions, 2);
});

test('the complete unchanged native multilingual tuple case executes all original operations', async () => {
  const file = root + 'rust/tests/unit/specification/conversational_reverification.rs';
  const source = readFileSync(file, 'utf8');
  const native = testFunctions(tokenize(source)).find(value => value.name === 'assistant_name_can_be_set_and_recalled_in_every_language');
  assert.ok(native);
  const parsed = typedProgramOf(native.body, { source, file, root });
  assert.ok(parsed.program, parsed.reason);
  assert.equal(parsed.program.nativeAssertions, 4);
  const result = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, 16);
  assert.equal(result.observations.length, 8);
  assert.deepEqual(result.iterations, [{ parameter: '(language,assignment,question,expected_name)', length: 4 }]);
});
