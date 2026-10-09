// Literal file operands must be authorized in their own statement.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { composeGeneralChangePlan } from '../../../js/agentic/general_planner.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
test('the original repair instruction does not become destructive source contents', () => {
  assert.equal(composeGeneralChangePlan("Inspect the shared literal-file request parser, whole-file guard and general planner in js/agentic/general_planner.mjs, js/agentic/write_request.mjs and js/agentic/literal_write_guard.mjs, with their native counterparts. Repair the general routing regression that sends declarative new files and explicit literal-content creation requests to read or semantic commands instead of write. Preserve read-before-replacement for existing edits, exact payload bytes, same-statement target/content binding, client-owned workspaces, bounded failed-write retries and honest verification status. Reproduce the original requests new file: notes.txt, contents: hello and Create a file named hello.txt with the content hello world using the actual planner before changes. Read the relevant tests and source before authoring; run only closest JavaScript checks, no native compiler. Report any exact missing capability instead of replacing tests or gates."), null);
});
test('a later write does not license an earlier read with a content-like preposition', () => {
  const plan = composeGeneralChangePlan('Read file f.txt with care. Write report.txt containing done.');
  assert.notEqual(plan?.target, 'f.txt');
});
test('payload write vocabulary cannot turn an explicit read into a whole-file write', () => {
  assert.equal(composeGeneralChangePlan('Read file f.txt with instructions to write a new document.'), null);
});
test('declarative creation and a real write action retain their original bytes', () => {
  for (const prompt of ['new file: notes.txt, contents: hello', 'Create a file named notes.txt with the content hello']) {
    const plan = composeGeneralChangePlan(prompt);
    assert.equal(plan.target, 'notes.txt');
    assert.equal(plan.content, 'hello');
  }
});

test('a write-only client receives the actual target and a truthful auxiliary gap', async () => {
  const { planGeneralChangeStep } = await import('../../../js/agentic/general_execution.mjs');
  const { finalResult, FinalDisposition } = await import('../../../js/agentic/final_result.mjs');
  const prompt = 'Create a file named hello.txt with the content hello world';
  const plan = composeGeneralChangePlan(prompt);
  const messages = [{ role: 'user', content: prompt }];
  const next = planGeneralChangeStep(messages, ['write'], plan);
  assert.equal(next.kind, 'tool_calls');
  const call = next.calls[0];
  assert.equal(call.tool, 'write');
  const args = JSON.parse(call.arguments);
  assert.equal(args.path, 'hello.txt');
  assert.equal(args.content, 'hello world');
  messages.push({ role: 'assistant', tool_calls: [{ id: 'target', type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
  messages.push({ role: 'tool', tool_call_id: 'target', name: call.tool, content: '{"success":true}' });
  const final = planGeneralChangeStep(messages, ['write'], plan);
  assert.equal(final.kind, 'final');
  assert.equal(finalResult(final).disposition, FinalDisposition.Gap);
  assert.equal(finalResult(final).origin, 'auxiliary_event_unavailable');
  assert.doesNotMatch(final.answer, /Completed the general change request/);
});
test('write-only target transport failure remains an actual failure', async () => {
  const { planGeneralChangeStep } = await import('../../../js/agentic/general_execution.mjs');
  const prompt = 'Create a file named hello.txt with the content hello world';
  const plan = composeGeneralChangePlan(prompt);
  const messages = [{ role: 'user', content: prompt }];
  const call = planGeneralChangeStep(messages, ['write'], plan).calls[0];
  messages.push({ role: 'assistant', tool_calls: [{ id: 'target', type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
  messages.push({ role: 'tool', tool_call_id: 'target', name: call.tool, content: '{"is_error":true,"error":"Write transport unavailable"}' });
  const final = planGeneralChangeStep(messages, ['write'], plan);
  assert.equal(final.kind, 'final');
  assert.match(final.answer, /Write transport unavailable/);
  assert.doesNotMatch(final.answer, /Completed the general change request/);
});


test('closed punctuation operands retain their exact bytes', () => {
  for (const payload of ['!?', '…。', '  !  ']) {
    const plan = composeGeneralChangePlan('Write «' + payload + '» to punctuation.txt');
    assert.equal(plan?.target, 'punctuation.txt');
    assert.equal(plan?.content, payload);
  }
});
test('seeded explicit content qualifiers license unquoted punctuation', () => {
  const plan = composeGeneralChangePlan('Create a file punctuation.txt containing exactly: !!!');
  assert.equal(plan?.target, 'punctuation.txt');
  assert.equal(plan?.content, '!!!');
});
test('empty or unclosed punctuation operands cannot authorize literal writes', () => {
  for (const prompt of ['Write «» to punctuation.txt', 'Write «!? to punctuation.txt', 'Create a file punctuation.txt containing exactly: "']) {
    assert.notEqual(composeGeneralChangePlan(prompt)?.mode, 'literal_file');
  }
});


test('the unchanged native panic request is not a literal file operand', () => {
  const task = readFileSync(new URL('../fixtures/qwen-memory-original-task.txt', import.meta.url), 'utf8');
  assert.equal(composeGeneralChangePlan(task), null);
});
test('each target owns only its local action, with original Unicode spans', () => {
  const cases = [
  [
    "Choose where to write records.\nCreate file x.txt containing «hello».",
    "hello"
  ],
  [
    "Escribe otra cosa.\nCrea el archivo x.txt con el contenido «hola».",
    "hola"
  ],
  [
    "Напиши что-нибудь.\nСоздай файл x.txt с содержимым «привет».",
    "привет"
  ],
  [
    "Choose where to write records.\n创建 x.txt 内容为 «你好»。",
    "你好"
  ],
  [
    "नोट लिखो।\nबनाओ x.txt ठीक इसी सामग्री के साथ «नमस्ते»।",
    "नमस्ते"
  ],
  [
    "Choose where to write records.\nx.txt में «नमस्ते» लिखो",
    "नमस्ते"
  ],
  [
    "Note İK𐐷 😀 café.\nCreate file x.txt containing «hello».",
    "hello"
  ],
  [
    "Read policy.md.\nNew file: x.txt, contents: «hello»",
    "hello"
  ]
];
  for (const [task, expected] of cases) {
    const plan = composeGeneralChangePlan(task);
    assert.equal(plan?.target, 'x.txt', task);
    assert.equal(plan?.content, expected, task);
  }
});
test('earlier or quoted write cues cannot authorize a local read', () => {
  for (const task of [
  "Choose where to write records.\nRead file x.txt with care.",
  "Write a report elsewhere.\nRead file x.txt containing hello.",
  "Read file x.txt then write it containing hello",
  "The instruction says «write».\nRead file x.txt with care."
]) assert.equal(composeGeneralChangePlan(task), null, task);
});
test('statement punctuation inside a closed payload remains exact data', () => {
  const payload = 'Choose where to write records.\nRead file other.txt with care.';
  const plan = composeGeneralChangePlan('Create file x.txt containing «' + payload + '».');
  assert.equal(plan?.target, 'x.txt');
  assert.equal(plan?.content, payload);
});


test('a target token cannot borrow a file cue from a completed statement', () => {
  for (const task of [
  "Write file.\nx.txt containing «hello».",
  "Write a file.\nfolder/note-α.txt containing «hello».",
  "Создай файл.\nзаметка.txt с содержимым «привет».",
  "Crea el archivo.\nnota.txt con el contenido «hola»."
]) assert.equal(composeGeneralChangePlan(task), null, task);
  const plan = composeGeneralChangePlan('Write file folder/note-α.txt containing «hello».');
  assert.equal(plan?.target, 'folder/note-α.txt');
  assert.equal(plan?.content, 'hello');
});


test('expanding lowercase prefixes preserve unquoted original content', () => {
  for (const prefix of ['İİ', 'İİİ', 'İK𐐷 😀 café', 'KK 😀 中文']) {
    const prompt = 'Note ' + prefix + '.\nCreate file x.txt containing hello.';
    assert.equal(composeGeneralChangePlan(prompt)?.content, 'hello.', prompt);
  }
});
test('raw content spans preserve the original multilingual payload and closer', () => {
  for (const [instruction, expected] of [
    ['Create file x.txt containing «hello».', 'hello'],
    ['Создай файл x.txt с содержимым «привет».', 'привет'],
    ['Crea el archivo x.txt con el contenido «hola».', 'hola'],
    ['创建 x.txt 内容为 «你好»。', '你好'],
    ['बनाओ x.txt ठीक इसी सामग्री के साथ «नमस्ते»।', 'नमस्ते'],
    ['Create file `policy/retention.md` containing Logs are kept for ninety days; backups are kept for a year.', 'Logs are kept for ninety days; backups are kept for a year.'],
    ["Create a file new.txt containing 'hello'.", 'hello'],
  ]) {
    const prompt = 'Note İİK𐐷 😀 café.\n' + instruction;
    assert.equal(composeGeneralChangePlan(prompt)?.content, expected, prompt);
  }
});
test('raw-span mapping rejects interior expansions and split surrogate characters', async () => {
  const { rawLowercaseSpan, rawContentLeadClose } = await import('../../../js/agentic/write_request/lowercase_spans.mjs');
  assert.deepEqual(rawLowercaseSpan('İK😀x', [2, 5]), [1, 4]);
  assert.equal(rawLowercaseSpan('İx', [1, 2]), null);
  assert.equal(rawLowercaseSpan('😀x', [1, 2]), null);
  assert.equal(rawLowercaseSpan('x', [2, 3]), null);
  assert.equal(rawLowercaseSpan('x', [1, 0]), null);
  assert.equal(rawContentLeadClose('😀x', 1), null);
});
test('objective labels map back before quoting and line anchoring', async () => {
  const { objectiveText } = await import('../../../js/agentic/general_planner.mjs');
  assert.equal(objectiveText('Note İİK😀.\nTask: Create file x.txt containing hello'), 'Create file x.txt containing hello');
  const quoted = 'Note İİK😀.\nCreate file x.txt containing «Task: preserve this». ';
  assert.equal(objectiveText(quoted), quoted);
});
test('Unicode prefixes cannot promote cross-statement or quoted owner cues', () => {
  for (const task of [
    'Note İİK😀.\nWrite file.\nx.txt containing «hello».',
    'Note İİK😀.\nThe instruction says «write».\nRead file x.txt containing hello.',
    'Note İİK😀.\nProduce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.',
  ]) assert.notEqual(composeGeneralChangePlan(task)?.mode, 'literal_file', task);
});

test('Unicode offsets preserve real circumfix closers as grammar', () => {
  for (const marker of ['जिसमें Gemfile.lock हो', '把 Gemfile.lock 写入']) {
    const task = 'Note İİK 😀.\nCreate file x.txt ' + marker;
    assert.equal(composeGeneralChangePlan(task)?.content, 'Gemfile.lock', task);
  }
});
test('a pinned line uses original boundaries after expanding lowercase characters', async () => {
  const { pinnedFirstLine } = await import('../../../js/agentic/write_request.mjs');
  assert.equal(pinnedFirstLine('İİ The first line must be exactly `header=ready`'), 'header=ready');
});
