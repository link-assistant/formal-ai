import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { symbolicFromWorker } from '../../../js/server/solve.mjs';
import { testFunctions, tokenize } from '../../../scripts/lib/rust-specification-cases.mjs';
import { typedProgramOf, executeTypedProgram } from '../../../scripts/lib/rust-specification-programs.mjs';

const root = new URL('../../../', import.meta.url).pathname;
const file = 'rust/tests/unit/specification/conversation_history.rs';
const source = readFileSync(root + file, 'utf8');
const originals = [
  ['solve_with_history_searches_english_dialog_history_by_term', 7],
  ['solve_with_history_reports_no_dialog_history_matches', 3],
  ['solve_with_history_searches_dialog_history_in_russian', 4],
  ['solve_with_history_searches_dialog_history_in_hindi', 4],
  ['solve_with_history_searches_dialog_history_in_chinese', 4],
  ['solve_with_history_accepts_other_conversation_query_forms', 3],
];
for (const [name, assertions] of originals) {
  test('whole original native history assertions: ' + name, async () => {
    const native = testFunctions(tokenize(source)).find(item => item.name === name);
    assert.ok(native);
    const parsed = typedProgramOf(native.body, { source, file: root + file, root });
    assert.ok(parsed.program, parsed.reason);
    assert.equal(parsed.program.nativeAssertions, assertions);
    const result = await executeTypedProgram(new WorkerHost(), parsed.program);
    assert.equal(result.status, 'passed', result.failure);
    assert.equal(result.assertions, assertions);
    const final = result.observations.at(-1).answer;
    assert.ok(final.solver_events.some(event => event.kind === 'filter:memory_query'));
    assert.ok(final.solver_events.some(event => event.kind === 'filter:memory_scope'));
  });
}

test('search result has exact native English bytes and only actual turn matches', async () => {
  const host = new WorkerHost();
  const history = [
    { role: 'user', content: 'Rust library notes' },
    { role: 'system', content: 'Rust unavailable privileged notes' },
    { role: 'assistant', content: 'RUST library details' },
    { role: 'user', content: 'Unrelated Python note' },
  ];
  const response = symbolicFromWorker(await host.solve('Recall Rust!', history), history);
  assert.equal(response.answer, 'Found 2 mention(s) of "rust" in the conversation history.\n- turn 1 user: Rust library notes\n- turn 3 assistant: RUST library details');
  const matches = response.solver_events.filter(event => event.kind === 'memory_match');
  assert.deepEqual(matches.map(event => event.payload), [
    'turn=1 role=user content=Rust library notes',
    'turn=3 role=assistant content=RUST library details',
  ]);
  assert.ok(!response.answer.includes('privileged'));
  assert.ok(!response.answer.includes('Python'));
});

test('empty query and unrelated syntax do not invent history or persisted memory', async () => {
  const host = new WorkerHost();
  for (const prompt of ['Recall', 'When did I mention?', 'Someone mentioned Rust yesterday']) {
    const actual = await host.run('historyRecallQuery(normalizePrompt(__prompt))', { __prompt: prompt });
    assert.equal(actual, null, prompt);
  }
  const result = await host.solve('When did I ask about Haskell?', []);
  assert.equal(result.content, 'No mentions of "haskell" found in the conversation history.');
  assert.ok(result.solverEvents.some(event => event.kind === 'filter:memory_matches' && event.payload === '0'));
  assert.ok(!result.solverEvents.some(event => event.kind === 'memory_match'));
});


test('seeded Spanish slots and inserted template-like message bytes remain genuine history', async () => {
  const host = new WorkerHost();
  const history = [{ role: 'user', content: 'Rust {turn} {role} {content} {term}' }];
  const answer = await host.solve('Recall Rust', history);
  assert.equal(answer.content, 'Found 1 mention(s) of "rust" in the conversation history.\n- turn 1 user: Rust {turn} {role} {content} {term}');
  assert.ok(answer.solverEvents.some(event => event.kind === 'memory_match' && event.payload.endsWith(history[0].content)));
  const ascii = await host.solve('Busca en mis conversaciones Rust', history);
  assert.equal(ascii.intent, 'conversation_recall');
  assert.ok(ascii.content.includes(history[0].content));
  const spanish = await host.solve('¿Cuándo mencioné Rust?', history);
  assert.equal(spanish.intent, 'conversation_recall');
  assert.ok(spanish.content.startsWith('Se encontraron 1 menciones'));
  assert.ok(spanish.content.includes(history[0].content));
});
