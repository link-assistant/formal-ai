// Issue #1191: the tests-as-docs gate (scripts/check-tests-as-docs.rs) ran only
// in the main-branch Rust gate stage, so main "CI/CD Pipeline" run 43710854562
// failed at 686367d20 on a loose-only test that its green pull request never
// checked. Its JavaScript twin now runs in the fast pull-request lint. These
// cases are the Rust script's own inline tests, plus a replay of that failure.
import { test, before, after } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

import {
  diff, loadRustParser, parseAllowlist, renderAllowlist, scanFile,
} from '../../../scripts/check-tests-as-docs.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
let parser;
before(async () => { parser = await loadRustParser(root); });
after(() => parser.delete());

const scan = (file, source) => scanFile(file, source, parser);
const entry = (file, testName) => ({ file, test: testName });

const LOOSE_TEST = `
#[test]
fn reported_russian_name_question_is_answered() {
    let response = FormalAiEngine.answer("Как твое имя?");
    assert!(response.answer.contains("formal AI"));
    assert!(!response.answer.contains("Claude"));
}
`;

const EXACT_TEST = `
#[test]
fn assistant_name_answer_is_shown_verbatim() {
    let response = FormalAiEngine.answer("What is your name?");
    assert_eq!(response.answer, "I am formal AI.");
    assert!(response.answer.contains("formal AI"));
}
`;

const ONE_OF_TEST = `
#[test]
fn assistant_name_answer_is_one_of_the_documented_variations() {
    const EXPECTED: &[&str] = &["I am formal AI.", "My name is formal AI."];
    let response = FormalAiEngine.answer("What is your name?");
    assert!(
        EXPECTED.contains(&response.answer.as_str()),
        "got: {}",
        response.answer,
    );
}
`;

test('a loose-only test is flagged; exact and one-of answers pass', () => {
  assert.deepEqual(scan('tests/unit/assistant_name.rs', LOOSE_TEST), [
    entry('tests/unit/assistant_name.rs', 'reported_russian_name_question_is_answered'),
  ]);
  assert.deepEqual(scan('tests/unit/assistant_name.rs', EXACT_TEST), []);
  assert.deepEqual(scan('tests/unit/assistant_name.rs', ONE_OF_TEST), []);
});

test('tests without answer assertions, answer methods and diagnostics are ignored', () => {
  const ignored = [
    '#[test]\nfn roles_registry_is_sorted() {\n    assert!(is_sorted(&roles()));\n}\n',
    '#[test]\nfn route_only() {\n    let response = FormalAiEngine.answer("hello");\n    assert_eq!(response.intent, "greeting");\n}\n' +
      '#[test]\nfn measured_delta() {\n    assert!(delta.answer_changed());\n}\n',
    '#[test]\nfn route_with_answer_diagnostic() {\n    let response = engine.solve("Find y: 7 * y = 84");\n' +
      '    assert_eq!(\n        response.intent,\n        "verifiable_task",\n        "unexpected answer: {}",\n        response.answer,\n    );\n}\n',
    '#[test]\nfn records_an_answer_for_later_inspection() {\n    let answer = engine.solve("hello").answer;\n' +
      '    write_fixture(answer);\n    assert!(fixture_exists());\n}\n',
    '#[test]\nfn checks_ranked_sources() {\n    assert!(execution.answer.sources.iter().all(|source| source.rank > 0));\n}\n',
    '#[test]\nfn exact_alias() {\n    let answer = engine.solve("hello").answer;\n    assert_eq!(answer.trim(), "I am formal AI.");\n}\n',
    '#[test]\nfn json_answer_is_exact() {\n    let response = engine.answer("hi");\n    let shape = "{\\"answer\\": 1}";\n' +
      '    assert_eq!(response.answer, shape);\n}\n',
  ];
  for (const source of ignored) assert.deepEqual(scan('tests/unit/routes.rs', source), [], source);
});

test('diagnostics, dynamic parity and comments never fake an exact answer', () => {
  const flagged = {
    route_and_loose_answer: '#[test]\nfn route_and_loose_answer() {\n    let response = engine.solve("hello");\n' +
      '    assert!(response.answer.contains("formal"));\n    assert_eq!(response.intent, "greeting", "got {}", response.answer);\n}\n',
    parity_only: '#[test]\nfn parity_only() {\n    let first = engine.solve("hello").answer;\n' +
      '    let second = engine.solve("hello again").answer;\n    assert_eq!(first, second);\n}\n',
    name_is_answered: '#[test]\nfn name_is_answered() {\n    let response = engine.answer("hi");\n' +
      '    // assert_eq!(response.answer, "I am formal AI.");\n    assert!(response.answer.contains("formal"));\n}\n',
  };
  for (const [name, source] of Object.entries(flagged)) {
    assert.deepEqual(scan('tests/unit/x.rs', source), [entry('tests/unit/x.rs', name)]);
  }
});

test('every test function of a file is scanned, in source order', () => {
  const second = LOOSE_TEST.replace('reported_russian_name_question_is_answered', 'a_second_loose_answer');
  assert.deepEqual(scan('tests/unit/all.rs', `${LOOSE_TEST}${second}${EXACT_TEST}`), [
    entry('tests/unit/all.rs', 'reported_russian_name_question_is_answered'),
    entry('tests/unit/all.rs', 'a_second_loose_answer'),
  ]);
});

test('history projections keep metadata out of answer text', () => {
  const before = '#[test] fn history_projection() { let first = solver.solve("Rust"); let history = [ConversationTurn::assistant(first.answer)]; let reply = solver.solve_with_history("itself", &history); ';
  const metadata = 'assert_eq!(reply.intent, "unknown"); assert!(!reply.links_notation.contains("coreference:resolved"));';
  const file = 'tests/unit/history_projection.rs';
  assert.deepEqual(scan(file, `${before}${metadata}}`), []);
  assert.deepEqual(scan(file, `${before}${metadata} assert!(reply.answer.contains("unknown")); }`), [entry(file, 'history_projection')]);
  assert.deepEqual(scan(file, `${before}${metadata} assert_eq!(reply.answer, "Exact answer"); }`), []);
});

test('new debt and stale rows are reported separately, and the allowlist round-trips', () => {
  const difference = diff([entry('tests/unit/a.rs', 'loose')], [entry('tests/unit/b.rs', 'fixed')]);
  assert.deepEqual(difference, { added: [entry('tests/unit/a.rs', 'loose')], stale: [entry('tests/unit/b.rs', 'fixed')] });
  const entries = [entry('tests/unit/a.rs', 'loose')];
  assert.deepEqual(parseAllowlist(renderAllowlist(entries)), entries);
  // The gate compares sets (Rust BTreeSet), so a hand-merged row order is not drift.
  const committed = parseAllowlist(readFileSync(path.join(root, 'scripts/tests-as-docs-allowlist.txt'), 'utf8'));
  assert.deepEqual(parseAllowlist(renderAllowlist(committed)), committed);
});

test('only #[test] functions, also inside inline modules, are audited', () => {
  const source = `#[cfg(test)]\nmod tests {\n    /// documented\n    #[test]\n    #[should_panic]\n    fn nested() { assert!(r.answer.contains("x")); }\n` +
    `    #[tokio::test]\n    async fn other_runner() { assert!(r.answer.contains("x")); }\n}\nfn helper() { assert!(r.answer.contains("x")); }\n`;
  assert.deepEqual(scan('tests/unit/m.rs', source), [entry('tests/unit/m.rs', 'nested')]);
});

// The production-only failure this twin closes: the 686367d20 test asserted on
// its answer only with contains, and the allowlist did not list it.
test('replays main run 43710854562: a loose-only proof test fails before merge', () => {
  const replay = JSON.parse(readFileSync(path.join(root, 'rust/tests/fixtures/issue-1191/tests-as-documentation-686367d20.json'), 'utf8'));
  assert.equal(replay.schema, 'main-only-regression-replay/v1');
  const found = scan(replay.path, replay.source);
  assert.deepEqual(found, [replay.expectedEntry]);
  const allowlist = parseAllowlist(readFileSync(path.join(root, 'scripts/tests-as-docs-allowlist.txt'), 'utf8'));
  assert.deepEqual(diff(found, allowlist).added, [replay.expectedEntry], 'the allowlist must not absorb the replayed debt');
  const fixed = replay.source.replace(
    '    assert!(\n        response.answer.contains("Disproof")',
    '    assert_eq!(response.answer, "Disproof (method: proof by contradiction).");\n    assert!(\n        response.answer.contains("Disproof")',
  );
  assert.notEqual(fixed, replay.source);
  assert.deepEqual(scan(replay.path, fixed), [], 'an exact answer assertion is what made main green again (10edee05b)');
});
