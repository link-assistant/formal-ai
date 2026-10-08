// PR #1188 dogfooding (T90, gap G12): "Find all usages of add." searched the
// web for the sentence, "Where is add used in this project?" grepped the
// whole question, "Search for add in the files of this directory." listed the
// directory and "Find all usages of add in this directory." looked for a file
// named all-usages-of-add. Workspace content search now has its own seeded
// vocabulary (`workspace_content_search_form`, every registered language):
// the arm greps the named identifier or quoted literal -- the client's grep
// tool when advertised, else `grep -rn` through the shell -- and answers with
// the file:line hits, or a seeded not-found naming the pattern and the scope.
// The Rust twin is rust/tests/unit/pull_request_1188_workspace_search.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const AGENT_CLI_TOOLS = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read', 'task',
  'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];
const REDUCED_TOOLS = ['read', 'write', 'edit', 'bash'];
const ROOT = '/work/demo';
const GREP_HITS = `Found 2 matches\n${ROOT}/m.mjs:\n  Line 1: export function add(a, b) {\n\n`
  + `${ROOT}/m.test.mjs:\n  Line 6:   assert.strictEqual(add(1, 2), 3);`;
const SHELL_HITS = './m.mjs:1:export function add(a, b) {\n./m.test.mjs:6:  assert.strictEqual(add(1, 2), 3);\n';
const LISTED = '- `m.mjs:1`: export function add(a, b) {\n- `m.test.mjs:6`: assert.strictEqual(add(1, 2), 3);';

/** The conversation after the user's request and, when given, one tool call with its result. */
function conversation(prompt, call = null, result = null) {
  const messages = [
    { role: 'system', content: `<env>\n  Working directory: ${ROOT}\n</env>` },
    { role: 'user', content: prompt },
  ];
  if (call !== null) {
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id: 'c0', type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: 'c0', name: call.tool, content: result });
  }
  return messages;
}

/** The first planned call and the answer given after `result` comes back. */
async function searchOnce(prompt, tools, result) {
  const plan = await planChatStep(conversation(prompt), tools);
  assert.equal(plan?.kind, 'tool_calls', `${prompt} planned ${JSON.stringify(plan)}`);
  const [call] = plan.calls;
  const answer = await planChatStep(conversation(prompt, call, result), tools);
  assert.equal(answer?.kind, 'final', `${prompt} then planned ${JSON.stringify(answer)}`);
  return { tool: call.tool, args: JSON.parse(call.arguments), answer: answer.answer };
}

describe('workspace content search greps the named identifier', () => {
  for (const prompt of [
    'Find all usages of add.',
    'Find all usages of add in this directory.',
    'Where is add used in this project?',
    'Search for add in the files of this directory.',
    'Grep for add.',
  ]) {
    test(`${prompt} lists the file:line hits`, async () => {
      const { tool, args, answer } = await searchOnce(prompt, AGENT_CLI_TOOLS, GREP_HITS);
      assert.equal(tool, 'grep');
      assert.deepEqual(args, { path: '.', pattern: '\\badd\\b' });
      assert.equal(answer, `Found 2 line(s) mentioning \`add\` in \`.\`:\n${LISTED}`);
    });
  }

  test('every registered language asks in its own words', async () => {
    const cases = [
      ['Найди все использования add.', `Найдено строк с \`add\` в \`.\`: 2.\n${LISTED}`],
      ['इस निर्देशिका में add के सभी उपयोग खोजें।', `\`.\` में \`add\` वाली 2 पंक्ति(याँ) मिलीं:\n${LISTED}`],
      ['查找 add 的所有用法。', `在 \`.\` 中找到 2 行提到 \`add\`：\n${LISTED}`],
      ['¿Dónde se usa add?', `Se encontraron 2 línea(s) que mencionan \`add\` en \`.\`:\n${LISTED}`],
    ];
    for (const [prompt, expected] of cases) {
      const { tool, args, answer } = await searchOnce(prompt, AGENT_CLI_TOOLS, GREP_HITS);
      assert.equal(tool, 'grep', prompt);
      assert.equal(args.pattern, '\\badd\\b', prompt);
      assert.equal(answer, expected, prompt);
    }
  });

  test('a pattern found nowhere is a seeded not-found naming the pattern and the scope', async () => {
    const { answer } = await searchOnce('Find all usages of sub.', AGENT_CLI_TOOLS, 'No files found');
    assert.equal(answer, 'No line in `.` mentions `sub`.');
  });

  test('without a grep tool the shell runs grep -rn', async () => {
    const { tool, args, answer } = await searchOnce('Find all usages of add.', REDUCED_TOOLS, SHELL_HITS);
    assert.equal(tool, 'bash');
    assert.equal(args.command, "grep -rnHw --exclude-dir=.git -- 'add' '.'");
    assert.equal(answer, `Found 2 line(s) mentioning \`add\` in \`.\`:\n${LISTED}`);
    const none = await searchOnce('Find all usages of sub.', REDUCED_TOOLS, '');
    assert.equal(none.answer, 'No line in `.` mentions `sub`.');
  });

  test('a scope phrase after the searched word is the scope, not part of the query (G36)', async () => {
    for (const prompt of ["Search for 'foo' in this project.", 'Search for foo in the workspace.']) {
      const plan = await planChatStep(conversation(prompt), REDUCED_TOOLS);
      assert.equal(JSON.parse(plan.calls[0].arguments).command, "grep -rnHw --exclude-dir=.git -- 'foo' '.'", prompt);
    }
  });

  test('a quoted literal is searched as fixed text within the named folder', async () => {
    const plan = await planChatStep(conversation("Grep for 'return a' in lib/"), REDUCED_TOOLS);
    assert.equal(JSON.parse(plan.calls[0].arguments).command, "grep -rnHF --exclude-dir=.git -- 'return a' 'lib'");
  });

  test('a named file is the scope, and a request to change the uses is not a search', async () => {
    const scoped = await planChatStep(conversation('Count the lines containing foo in f.txt.'), REDUCED_TOOLS);
    assert.equal(JSON.parse(scoped.calls[0].arguments).command, "grep -rnHw --exclude-dir=.git -- 'foo' 'f.txt'");
    for (const prompt of ['Rename all usages of add to plus.', 'Remove all uses of add.']) {
      const plan = await planChatStep(conversation(prompt), REDUCED_TOOLS);
      assert.ok(!(plan?.calls?.[0]?.arguments ?? '').includes('grep -rn'), `${prompt}: ${JSON.stringify(plan)}`);
    }
  });

  test('a file-name request still locates the file', async () => {
    const plan = await planChatStep(conversation('Find the file m.mjs in this directory.'), AGENT_CLI_TOOLS);
    assert.equal(plan.calls[0].tool, 'bash');
    assert.match(JSON.parse(plan.calls[0].arguments).command, /^find /u);
  });

  test('a search the request sends to the web is not grepped', async () => {
    const plan = await planChatStep(conversation('Find usages of add online.'), AGENT_CLI_TOOLS);
    assert.notEqual(plan?.calls?.[0]?.tool, 'grep');
  });
});

// T91 (gap G11): "Run the tests in m.test.mjs." ran `bun test`, the
// whole-suite command of whichever marker file the server's own directory
// held. A named test file runs with the runtime its extension needs, from the
// seeded `test_file_runners` group of data/seed/shell-intents.lino.
describe('a named test file runs with its own runtime', () => {
  for (const [prompt, command] of [
    ['Run the tests in m.test.mjs.', 'node --test m.test.mjs'],
    ['Запусти тесты в m.test.mjs.', 'node --test m.test.mjs'],
    ['Run the tests in test_m.py.', 'python3 -m pytest test_m.py'],
  ]) {
    test(prompt, async () => {
      const plan = await planChatStep(conversation(prompt), REDUCED_TOOLS);
      assert.equal(plan.calls[0].tool, 'bash');
      assert.equal(JSON.parse(plan.calls[0].arguments).command, command);
    });
  }
});

// T92 (gap G10): "Add a function sub(a, b) that returns a - b to m.mjs." only
// read the module: the browser composer discovers no structure in a body
// stated as an expression, and the returned value was read up to the end of
// the clause ("a - b to m.mjs"). The seeded integer operation that meets the
// specification at every sample pair is the function, as natively, and the
// returned value is the longest expression right after the verb.
describe('a function whose body is stated as an expression is added', () => {
  test('sub(a, b) returning a - b joins the module and is checked', async () => {
    const MODULE = 'export function add(a, b) {\n  return a + b;\n}\n';
    let file = MODULE;
    const messages = conversation('Add a function sub(a, b) that returns a - b to m.mjs.');
    const tools = [];
    let answer = null;
    for (let step = 0; step < 6 && answer === null; step += 1) {
      const plan = await planChatStep(messages, REDUCED_TOOLS);
      if (!plan || plan.kind === 'final') {
        answer = plan?.answer ?? '';
        break;
      }
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      let result = '';
      if (call.tool === 'read') result = file;
      if (call.tool === 'write') file = args.content;
      tools.push(call.tool);
      const id = `s${step}`;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: result });
    }
    assert.deepEqual(tools, ['read', 'write', 'bash']);
    assert.equal(file, `${MODULE}\nexport function sub(a, b) {\n  return a - b;\n}\n`);
    assert.match(answer, /^Created and verified `m\.mjs`/u);
  });
});

// T93 (gap G14): "Fix the bug in m.mjs: add should return the sum." on a
// correct module fell to the unknown answer. The stated expectation is checked
// first -- the module read, the function run at the contract's samples
// through its seeded probe -- and a function that meets it is no defect found.
describe('a bug report with a stated expectation is checked first', () => {
  const MODULE = 'export function add(a, b) {\n  return a + b;\n}\n\nexport function mul(a, b) {\n  return a * b;\n}\n';

  /** Read returns the module, the shell returns `printed`; the calls and the answer. */
  async function check(prompt, printed) {
    const messages = conversation(prompt);
    const calls = [];
    for (let step = 0; step < 4; step += 1) {
      const plan = await planChatStep(messages, REDUCED_TOOLS);
      if (!plan || plan.kind === 'final') return { calls, answer: plan?.answer ?? null };
      const [call] = plan.calls;
      calls.push(call.tool === 'bash' ? JSON.parse(call.arguments).command : call.tool);
      const id = `e${step}`;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: call.tool === 'read' ? MODULE : printed });
    }
    return { calls, answer: null };
  }

  const PROBE = 'node --input-type=module -e "import { add } from \'./m.mjs\'; console.log(add(2, 3));"';

  test('a function that meets the expectation is no defect found', async () => {
    const { calls, answer } = await check('Fix the bug in m.mjs: add should return the sum.', '5\n');
    assert.deepEqual(calls, ['read', PROBE]);
    assert.equal(answer, 'No defect found: `add(2, 3)` in `m.mjs` returned `5`, the expected `5`, so `m.mjs` is unchanged.');
  });

  test('a function that misses it is a confirmed defect, the file unchanged', async () => {
    const { answer } = await check('Fix the bug in m.mjs: add should return a * b.', '5\n');
    assert.equal(answer, 'The defect is confirmed: `add(2, 3)` in `m.mjs` returned `5`, but the request expects `6`. `m.mjs` is unchanged.');
  });

  test('every registered language states the expectation in its own words', async () => {
    const { calls, answer } = await check('Исправь ошибку в m.mjs: add должна возвращать сумму.', '5\n');
    assert.deepEqual(calls, ['read', PROBE]);
    assert.equal(answer, 'Ошибка не найдена: `add(2, 3)` в `m.mjs` вернула `5` — ожидаемое `5`, поэтому `m.mjs` не изменён.');
    const hindi = await check('m.mjs में बग ठीक करें: add को योग लौटाना चाहिए।', '5\n');
    assert.deepEqual(hindi.calls, ['read', PROBE]);
  });
});

// T96 (the backstop behind G13, G17 and T29): the literal-file general change
// plan wrote its target without reading it, so each new misreading of an edit
// request destroyed a file ("Replace all uses of add with plus in m.mjs."
// wrote the words as m.mjs). Unless the first write verb is a seeded
// whole-file write, the target is read first, and an existing non-empty file
// is replaced only when the request says so.
describe('a literal-file write never replaces an existing file unasked', () => {
  /** Run `prompt` over the in-memory `files`; the tools act as the Agent CLI's do. */
  async function run(prompt, files) {
    const messages = conversation(prompt);
    const calls = [];
    for (let step = 0; step < 6; step += 1) {
      const plan = await planChatStep(messages, REDUCED_TOOLS);
      if (!plan || plan.kind === 'final') return { calls, answer: plan?.answer ?? null };
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      const path = args.filePath ?? args.path;
      let result = '';
      if (call.tool === 'read') result = files.has(path) ? files.get(path) : `Error: File not found: ${path}`;
      if (call.tool === 'write') files.set(path, args.content);
      if (call.tool === 'bash') result = files.get(args.command.replace(/^cat /u, '')) ?? '';
      calls.push(call.tool === 'bash' ? args.command : `${call.tool} ${path}`);
      const id = `w${step}`;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: result });
    }
    return { calls, answer: null };
  }

  test('an edit misread as a whole-file write reads the file and keeps it', async () => {
    const files = new Map([['m.mjs', 'export function add(a, b) {\n  return a + b;\n}\n']]);
    const { calls, answer } = await run('Replace all uses of add with plus in m.mjs.', files);
    assert.deepEqual(calls, ['read m.mjs']);
    assert.equal(answer, 'Left `m.mjs` unchanged: it already has content, and the request does not say to replace the whole file.');
    assert.equal(files.get('m.mjs'), 'export function add(a, b) {\n  return a + b;\n}\n');
  });

  test('a missing target is created after the read finds nothing', async () => {
    const files = new Map();
    const { calls } = await run("Put 'hello' into notes.txt.", files);
    assert.deepEqual(calls, ['read notes.txt', 'write .formal-ai/general-change-plan.lino', 'write notes.txt', 'cat notes.txt']);
    assert.equal(files.get('notes.txt'), 'hello');
  });

  test('an existing target is kept for that verb, and replaced for a whole-file write', async () => {
    const kept = new Map([['notes.txt', 'old']]);
    const { answer } = await run("Put 'hello' into notes.txt.", kept);
    assert.equal(kept.get('notes.txt'), 'old');
    assert.match(answer, /^Left `notes\.txt` unchanged/u);
    const replaced = new Map([['a.txt', 'old']]);
    const { calls } = await run('Create a file a.txt containing hello', replaced);
    assert.equal(calls[0], 'write .formal-ai/general-change-plan.lino');
    assert.equal(replaced.get('a.txt'), 'hello');
  });
});

// T98 (gap G29): "List the files in src." listed the workspace root. The
// listed directory is the path-shaped word after a seeded place preposition,
// unless it belongs to a seeded scope phrase ("in this directory").
describe('a listing request lists the directory it names', () => {
  for (const [prompt, tools, expected] of [
    ['List the files in src.', AGENT_CLI_TOOLS, ['list', { path: 'src' }]],
    ['Покажи файлы в src.', AGENT_CLI_TOOLS, ['list', { path: 'src' }]],
    ['List the files in this directory.', AGENT_CLI_TOOLS, ['list', { path: '.' }]],
    ['List the files in src.', REDUCED_TOOLS, ['bash', { command: "ls 'src'" }]],
    ['List the files in the current folder.', REDUCED_TOOLS, ['bash', { command: 'ls' }]],
  ]) {
    test(`${prompt} (${tools.length} tools)`, async () => {
      const plan = await planChatStep(conversation(prompt), tools);
      assert.deepEqual([plan.calls[0].tool, JSON.parse(plan.calls[0].arguments)], expected);
    });
  }
});

// T99 (gap G26): "Which functions does src/m.mjs export?" dumped the module.
// The answer names the functions declared behind a seeded export marker.
describe('a question about what a module exports is answered from its declarations', () => {
  const MODULE = 'export function add(a, b) {\n  return a + b;\n}\n\nfunction helper() {}\n\nexport async function mul(a, b) {\n  return a * b;\n}\n';
  for (const [prompt, expected] of [
    ['Which functions does src/m.mjs export?', '`src/m.mjs` exports 2 function(s): `add`, `mul`.'],
    ['Какие функции экспортирует src/m.mjs?', '`src/m.mjs` экспортирует функции (2): `add`, `mul`.'],
  ]) {
    test(prompt, async () => {
      const first = await planChatStep(conversation(prompt), REDUCED_TOOLS);
      assert.equal(first.calls[0].tool, 'read');
      const answer = await planChatStep(conversation(prompt, first.calls[0], MODULE), REDUCED_TOOLS);
      assert.equal(answer.answer, expected);
    });
  }
});

// T100 (gap G27): "Summarize README.md in one sentence." answered with the
// file's first line. The named file's text goes to the summarization handlers.
describe('a summary of a named file summarizes its text', () => {
  test('Summarize README.md in one sentence.', async () => {
    const TEXT = 'The river flooded the town after three days of rain. Residents moved to the school on the hill. Volunteers brought food and blankets.\n';
    const prompt = 'Summarize README.md in one sentence.';
    const first = await planChatStep(conversation(prompt), REDUCED_TOOLS);
    assert.equal(first.calls[0].tool, 'read');
    const answer = await planChatStep(conversation(prompt, first.calls[0], TEXT), REDUCED_TOOLS);
    assert.equal(answer.kind, 'final');
    assert.ok(answer.answer.startsWith('The river flooded the town after three days of rain.'), answer.answer);
    assert.ok(!answer.answer.includes('Volunteers'), answer.answer);
  });
});
