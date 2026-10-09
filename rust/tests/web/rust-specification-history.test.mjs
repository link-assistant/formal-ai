import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { tokenize, testFunctions } from '../../../scripts/lib/rust-specification-cases.mjs';
import { typedProgramOf, executeTypedProgram } from '../../../scripts/lib/rust-specification-programs.mjs';
import { historySourceContract } from '../../../scripts/lib/rust-specification-bindings.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const root = new URL('../../../', import.meta.url).pathname;
const files = ['conversation_history.rs', 'reasoning_paths.rs'];
function inline(source) {
  return typedProgramOf(testFunctions(tokenize(source))[0].body, { source, root });
}
const imports = 'use formal_ai::{solve_with_history, ConversationTurn, UniversalSolver, SymbolicAnswer};';

test('real free-history native functions retain every original assertion or reject the complete declaration', () => {
  let supported = 0;
  let macros = 0;
  let unsupported = 0;
  for (const name of files) {
    const file = root + 'rust/tests/unit/specification/' + name;
    const source = readFileSync(file, 'utf8');
    for (const native of testFunctions(tokenize(source))) {
      if (!native.body.some((token, index) => token.text === 'solve_with_history'
        && native.body[index + 1]?.text === '(' && native.body[index - 1]?.text !== '.')) continue;
      const parsed = typedProgramOf(native.body, { source, file, root });
      if (!parsed.program) {
        unsupported += 1;
        assert.equal(parsed.reason, 'unsupported native record declaration');
        continue;
      }
      const count = native.body.filter((token, index) => ['assert', 'assert_eq', 'assert_ne'].includes(token.text)
        && native.body[index + 1]?.text === '!').length;
      assert.equal(parsed.program.nativeAssertions, count);
      assert.ok(parsed.program.fixtures.some(fixture => fixture.file === root + 'rust/src/solver.rs'));
      assert.ok(parsed.program.fixtures.some(fixture => fixture.file === root + 'rust/src/lib.rs'));
      supported += 1;
      macros += count;
    }
  }
  assert.equal(supported, 18);
  assert.equal(macros, 51);
  assert.equal(unsupported, 1);
});

for (const name of ['solve_with_history_recalls_name_across_turns', 'solve_with_history_recalls_last_question',
  'solve_with_history_falls_through_for_unrelated_prompts']) {
  test('unchanged native history assertions execute: ' + name, async () => {
    const file = root + 'rust/tests/unit/specification/reasoning_paths.rs';
    const source = readFileSync(file, 'utf8');
    const native = testFunctions(tokenize(source)).find(value => value.name === name);
    assert.ok(native);
    const parsed = typedProgramOf(native.body, { source, file, root });
    assert.ok(parsed.program, parsed.reason);
    const result = await executeTypedProgram(new WorkerHost(), parsed.program);
    assert.equal(result.status, 'passed', result.failure);
    assert.equal(result.assertions, parsed.program.nativeAssertions);
    assert.ok(result.observations[0].history.length > 0);
  });
}

test('arbitrary imported aliases and pure helper parameter names bind the actual values', async () => {
  const parsed = inline(`use formal_ai::{solve_with_history as dispatch, ConversationTurn, SymbolicAnswer};
    fn scan(unused: &str, packet: &SymbolicAnswer, prefix: &str) -> bool {
      packet.evidence_links.iter().any(|element| element.starts_with(prefix))
    }
    #[test] fn held_out() {
      let turns = [ConversationTurn::user("arbitrary prior message 829")];
      let observed = dispatch("Hi", &turns);
      assert_eq!(observed.intent, "greeting");
      assert!(scan("ignored", &observed, "prior_turn:user"));
      assert!(!scan("prior_turn:user", &observed, "nonexistent 947"));
    }`);
  assert.ok(parsed.program, parsed.reason);
  const result = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, 3);
  assert.equal(result.observations.length, 1);
});

test('same-typed helper arguments follow body use and reversed calls remain false', async () => {
  const parsed = inline(`fn differently_named(left: &str, right: &str) -> bool { right.starts_with(left) }
    #[test] fn held_out() {
      assert!(differently_named("quartz", "quartz 951"));
      assert!(!differently_named("quartz 951", "quartz"));
    }`);
  assert.ok(parsed.program, parsed.reason);
  const result = await executeTypedProgram({ solve() { throw new Error('unexpected effect'); } }, parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, 2);
  assert.deepEqual(result.observations, []);
});

test('unknown, recursive, assertion-bearing and effectful helper bodies reject whole cases', () => {
  for (const body of [
    'UniversalSolver::default().solve(value).intent == "greeting"',
    'predicate(value)', 'assert!(true); true', 'value.unknown_predicate()',
  ]) {
    const parsed = inline(`${imports} fn predicate(value: &str) -> bool { ${body} }
      #[test] fn held_out() { assert!(predicate("Hi")); }`);
    assert.equal(parsed.program, undefined);
    assert.match(parsed.reason, /unsupported native call predicate/);
  }
  const wrong = inline(`fn predicate(value: &str) -> bool { value }
    #[test] fn held_out() { assert!(predicate("Hi")); }`);
  assert.equal(wrong.program, undefined);
});

test('a local unsupported declaration shadows an import while impl methods do not', () => {
  const bad = inline(`use formal_ai::solve_with_history as route;
    fn route(prompt: &str) -> SymbolicAnswer { FormalAiEngine.answer(prompt) }
    #[test] fn held_out() { let reply = route("Hi"); assert_eq!(reply.intent, "greeting"); }`);
  assert.equal(bad.program, undefined);
  const good = inline(`${imports} impl Unrelated {
    fn solve_with_history(prompt: &str) -> SymbolicAnswer { FormalAiEngine.answer(prompt) }
  } #[test] fn held_out() {
    let reply = solve_with_history("Hi", &[]); assert_eq!(reply.intent, "greeting");
  }`);
  assert.ok(good.program, good.reason);
});

test('free history call arguments evaluate in native order before the outer call', async () => {
  const parsed = inline(`${imports} #[test] fn held_out() {
    let solver = UniversalSolver::default();
    let reply = solve_with_history(solver.solve("first 731").answer,
      &[ConversationTurn::assistant(solver.solve("second 842").answer)]);
    assert_eq!(reply.answer, "first 731");
  }`);
  assert.ok(parsed.program, parsed.reason);
  const calls = [];
  const host = { async solve(prompt, history) {
    calls.push({ prompt, history });
    await new Promise(resolve => setTimeout(resolve, prompt === 'first 731' ? 5 : 0));
    return { content: prompt, intent: 'heldout' };
  } };
  const result = await executeTypedProgram(host, parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.deepEqual(calls.map(value => value.prompt), ['first 731', 'second 842', 'first 731']);
  assert.deepEqual(calls[2].history, [{ role: 'assistant', content: 'second 842' }]);
});

test('public source contract validates the forwarding body, not a function name marker', () => {
  const temporary = mkdtempSync(join(tmpdir(), 'formal-ai-history-contract-'));
  try {
    mkdirSync(join(temporary, 'rust/src'), { recursive: true });
    const entry = join(temporary, 'rust/src/lib.rs');
    const producer = join(temporary, 'rust/src/solver.rs');
    writeFileSync(entry, 'pub use solver::{solve_with_history};');
    const body = 'pub fn solve_with_history(query: &str, prior: &[ConversationTurn]) -> SymbolicAnswer { UniversalSolver::default().solve_with_history(query, prior) }';
    writeFileSync(producer, body);
    assert.equal(historySourceContract(temporary).length, 2);
    writeFileSync(producer, body.replace('query, prior)', '"substituted", prior)'));
    assert.throws(() => historySourceContract(temporary), /unsupported native history producer body/);
    writeFileSync(producer, body.replace('UniversalSolver::default()', 'UniversalSolver::new(configuration)'));
    assert.throws(() => historySourceContract(temporary), /unsupported native call/);
    writeFileSync(producer, body);
    writeFileSync(entry, 'mod solver;');
    assert.throws(() => historySourceContract(temporary), /not publicly exported/);
  } finally { rmSync(temporary, { recursive: true, force: true }); }
});

test('changed producer identity stops before any worker operation', async () => {
  const parsed = inline(`${imports} #[test] fn held_out() {
    let reply = solve_with_history("Hi", &[]); assert_eq!(reply.intent, "greeting");
  }`);
  assert.ok(parsed.program, parsed.reason);
  parsed.program.fixtures.find(fixture => fixture.file === root + 'rust/src/solver.rs').sha256 = '0'.repeat(64);
  const result = await executeTypedProgram({ solve() { throw new Error('must not run'); } }, parsed.program);
  assert.equal(result.status, 'failed');
  assert.equal(result.assertions, 0);
  assert.deepEqual(result.observations, []);
  assert.equal(result.failure, 'stale native fixture identity');
});
