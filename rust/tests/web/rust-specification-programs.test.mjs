import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { typedProgramOf, executeTypedProgram } from '../../../scripts/lib/rust-specification-programs.mjs';
import { specificationCases, testFunctions, tokenize } from '../../../scripts/lib/rust-specification-cases.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const root = new URL('../../../', import.meta.url).pathname;
const compileSource = 'rust/tests/unit/specification/natural_language_skill_compilation.rs';
const countSource = 'rust/tests/unit/specification/behavior_rules.rs';
const originals = [
  ['rust/tests/unit/specification/response_language_followup.rs', 'capabilities_follow_up_returns_to_english_on_request', 4, 4],
  ['rust/tests/unit/specification/response_language_followup.rs', 'identity_follow_up_retargets_between_non_english_languages', 4, 4],
  [compileSource, 'natural_language_skill_compiles_to_reusable_package', 7, 7],
  [compileSource, 'natural_language_skill_compiles_supported_language_shapes', 2, 8],
  [compileSource, 'compiled_package_replays_deterministically_and_exports_links_notation', 5, 5],
  [compileSource, 'solver_prefers_compiled_skill_from_history_and_records_cache_hit', 5, 5],
  [countSource, 'behavior_rules_count_includes_dialog_local_runtime_rules', 6, 6],
];
for (const [file, name, macros, executions] of originals) {
  test('every original assertion executes from actual native source: ' + name, async () => {
    const source = readFileSync(root + file, 'utf8');
    const native = testFunctions(tokenize(source)).find(item => item.name === name);
    assert.ok(native);
    const parsed = typedProgramOf(native.body, { source, file: root + file, root });
    assert.ok(parsed.program, parsed.reason);
    assert.equal(parsed.program.nativeAssertions, macros);
    const result = await executeTypedProgram(new WorkerHost(), parsed.program);
    assert.equal(result.status, 'passed', result.failure);
    assert.equal(result.assertions, executions);
    assert.equal(result.nativeAssertions, macros);
    if (executions !== macros) assert.deepEqual(result.iterations, [{ parameter: 'case', length: 4 }]);
    const carried = specificationCases(root).cases.find(item => item.id === file + '::' + name);
    assert.ok(carried?.program, 'production inventory must contain the typed program');
    if (name === 'behavior_rules_count_includes_dialog_local_runtime_rules') {
      assert.ok(result.observations[0].answer.answer.endsWith('```\n'));
    }
  });
}

function inline(source) {
  const native = testFunctions(tokenize(source))[0];
  assert.ok(native);
  return typedProgramOf(native.body, { source, root });
}

const imports = 'use formal_ai::{ConversationTurn, UniversalSolver, compile_natural_language_skill};';
test('arbitrary local record names and imported compiler aliases use one typed loop mechanism', async () => {
  const source = `use formal_ai::compile_natural_language_skill as lower;
  #[test] fn unrelated() {
    struct Example { instruction: &'static str, expected: &'static str }
    let variants = [Example { expected: "arbitrary Ω", instruction: "When \`quartz 816\` then \`arbitrary Ω\`" },
      Example { instruction: "Когда \`камень 917\` тогда \`другой ответ\`", expected: "другой ответ" }];
    for variant in variants {
      let artifact = lower(variant.instruction).expect("compile arbitrary fixture");
      assert_eq!(artifact.response, variant.expected, "{}", variant.instruction);
      assert!(artifact.id.starts_with("compiled_skill_"));
    }
  }`;
  const parsed = inline(source);
  assert.ok(parsed.program, parsed.reason);
  assert.equal(parsed.program.nativeAssertions, 2);
  const result = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, 4);
  assert.deepEqual(result.iterations, [{ parameter: 'variant', length: 2 }]);
});

test('wrong assertions fail and stop before later original operations', async () => {
  const parsed = inline(`${imports} #[test] fn held_out() {
    let solver = UniversalSolver::default();
    let reply = solver.solve("4 + 4");
    assert_eq!(reply.intent, "wrong intent");
    let later = solver.solve("Hi");
    assert_eq!(later.intent, "greeting");
  }`);
  assert.ok(parsed.program, parsed.reason);
  assert.equal(parsed.program.nativeAssertions, 2);
  const result = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(result.status, 'failed');
  assert.equal(result.assertions, 1);
  assert.equal(result.observations.length, 1);
});

test('unknown fields and effects reject the entire case rather than dropping assertions', () => {
  for (const ending of ['assert_eq!(reply.fabricated, "x");', 'mutate_repository();', 'assert!(reply.answer.unsupported_method());']) {
    const parsed = inline(`${imports} #[test] fn held_out() {
      let solver = UniversalSolver::default(); let reply = solver.solve("Hi");
      assert_eq!(reply.intent, "greeting"); ${ending}
    }`);
    assert.equal(parsed.program, undefined);
    assert.ok(parsed.reason);
  }
});

test('Result and Option expectations execute genuine failure states', async () => {
  for (const operation of [
    'compile_natural_language_skill("ordinary unrelated text").expect("must compile")',
    'compile_natural_language_skill("When `pebble 892` then `response`").expect("compiled").replay("other").expect("must replay")',
  ]) {
    const parsed = inline(`${imports} #[test] fn held_out() { let artifact = ${operation}; assert_eq!(artifact.answer, "response"); }`);
    if (operation.startsWith('compile_natural_language_skill("ordinary')) {
      assert.equal(parsed.program, undefined, 'unknown package.answer must reject even after an earlier expectation');
      continue;
    }
    assert.ok(parsed.program, parsed.reason);
    const result = await executeTypedProgram(new WorkerHost(), parsed.program);
    assert.equal(result.status, 'failed');
    assert.match(result.failure, /native expectation failed/);
    assert.equal(result.assertions, 0);
  }
  const unsupported = inline(`${imports} #[test] fn held_out() {
    let artifact = compile_natural_language_skill("ordinary unrelated text").expect("must compile");
    assert_eq!(artifact.response, "invented answer");
  }`);
  assert.ok(unsupported.program, unsupported.reason);
  assert.equal((await executeTypedProgram(new WorkerHost(), unsupported.program)).status, 'failed');
});

test('engine, offline and misleading helper bodies remain unsupported', () => {
  for (const body of ['FormalAiEngine.answer(prompt)', 'UniversalSolver::new(SolverConfig { offline: true, ..SolverConfig::default() }).solve(prompt)', 'UniversalSolver::default().solve("fixed prompt")']) {
    const parsed = inline(`${imports} use formal_ai::{FormalAiEngine, SolverConfig};
      fn invented(prompt: &str) -> SymbolicAnswer { ${body} }
      #[test] fn held_out() { let reply = invented("Hi"); assert_eq!(reply.intent, "greeting"); }`);
    assert.equal(parsed.program, undefined);
    assert.match(parsed.reason, /unsupported native call/);
  }
  const real = inline(`${imports} fn arbitrary(input: &str) -> SymbolicAnswer { UniversalSolver::default().solve(input) }
    #[test] fn held_out() { let reply = arbitrary("Hi"); assert_eq!(reply.intent, "greeting"); }`);
  assert.ok(real.program, real.reason);
});

test('actual earlier answers enter history and newest user packages preserve dedup controls', async () => {
  const source = `${imports} #[test] fn held_out() {
    let solver = UniversalSolver::default();
    let older = "When \`quartz 941\` then \`older\`";
    let latest = "When \`quartz 941\` then \`newer\`";
    let equivalent = "When \`QUARTZ 941!!!\` then \`newer\`";
    let taught = solver.solve(latest);
    let history = [ConversationTurn::user(older), ConversationTurn::user(latest),
      ConversationTurn::assistant(taught.answer), ConversationTurn::user(equivalent),
      ConversationTurn::assistant("When \`quartz 941\` then \`impostor\`")];
    let count = solver.solve_with_history("How many behavior rules are there?", &history);
    assert!(count.answer.contains("dialog_local_rules \\\"2\\\""));
    let reply = solver.solve_with_history("quartz 941", &history);
    assert_eq!(reply.answer, "newer");
    assert!(reply.evidence_links.iter().any(|item| item.starts_with("cache_hit:compiled_skill_")));
  }`;
  const parsed = inline(source);
  assert.ok(parsed.program, parsed.reason);
  const result = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.equal(result.assertions, 3);
  assert.equal(result.observations[1].history[2].content, result.observations[0].answer.answer);
});

test('native source identity and fixture freshness cannot be invented by callers', async () => {
  const file = root + compileSource;
  const source = readFileSync(file, 'utf8');
  const native = testFunctions(tokenize(source))[0];
  assert.match(typedProgramOf(native.body, { source: source + '\n', file, root }).reason, /source identity mismatch/);
  const parsed = typedProgramOf(native.body, { source, file, root });
  assert.ok(parsed.program, parsed.reason);
  parsed.program.fixtures[0].sha256 = '0'.repeat(64);
  const result = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(result.status, 'failed');
  assert.equal(result.observations.length, 0);
  assert.match(result.failure, /stale native fixture/);
});


test('effectful array elements execute in native left-to-right order', async () => {
  const parsed = inline(`${imports} #[test] fn held_out() {
    let solver = UniversalSolver::default();
    let replies = [solver.solve("first"), solver.solve("second")];
    assert_eq!(replies.len(), 2);
  }`);
  assert.ok(parsed.program, parsed.reason);
  const calls = [];
  const host = { async solve(prompt) {
    calls.push('start:' + prompt);
    await new Promise(resolve => setTimeout(resolve, prompt === 'first' ? 5 : 0));
    calls.push('end:' + prompt);
    return { content: prompt, intent: 'heldout' };
  } };
  const result = await executeTypedProgram(host, parsed.program);
  assert.equal(result.status, 'passed', result.failure);
  assert.deepEqual(calls, ['start:first', 'end:first', 'start:second', 'end:second']);
  assert.equal(result.assertions, 1);
});

test('diagnostic field chains cannot hide an unexecuted effect', () => {
  const parsed = inline(`${imports} #[test] fn held_out() {
    assert_eq!("x", "x", "{}", compile_natural_language_skill("When \`a\` then \`b\`").expect("compile").response);
  }`);
  assert.equal(parsed.program, undefined);
  assert.equal(parsed.reason, 'unsupported diagnostic side effect');
});

// Add these source-connected entries to originals in rust-specification-programs.test.mjs:
// ['rust/tests/unit/specification/response_language_followup.rs', 'capabilities_follow_up_returns_to_english_on_request', 4, 4],
// ['rust/tests/unit/specification/response_language_followup.rs', 'identity_follow_up_retargets_between_non_english_languages', 4, 4],

test('typed string vector membership preserves string substring semantics', async () => {
  const parsed = inline(`#[test] fn arbitrary_values() {
    assert!(["alpha", "Ω"].contains(&"Ω"));
    assert!(!["alpha"].contains(&"beta"));
    assert!("alphabet".contains("alpha"));
  }`);
  assert.ok(parsed.program, parsed.reason);
  const actual = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(actual.status, 'passed', actual.failure);
  assert.equal(actual.assertions, 3);
});

test('typed vector membership rejects unknown element types and preserves failure', async () => {
  for (const body of ['assert!(["alpha"].contains(&true));', 'assert!([].contains(&"alpha"));', 'assert!([true].contains(&true));', 'assert!([("Ω", true)].contains(&("Ω", true)));', 'assert!([9007199254740992].contains(&9007199254740993));', 'assert!(["alpha"].contains(&"alpha")); mutate_repository();']) {
    assert.equal(inline(`#[test] fn arbitrary_refusal() { ${body} }`).program, undefined);
  }
  const parsed = inline('#[test] fn wrong_membership() { assert!(["alpha"].contains(&"beta")); }');
  assert.ok(parsed.program, parsed.reason);
  const actual = await executeTypedProgram(new WorkerHost(), parsed.program);
  assert.equal(actual.status, 'failed');
  assert.equal(actual.assertions, 1);
});
