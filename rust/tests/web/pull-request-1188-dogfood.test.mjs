// PR #1188 dogfooding ledger (docs/case-studies/pull-request-1188/formal-ai-dogfood.md):
// every task the JavaScript Formal AI failed while driving the real
// link-assistant Agent CLI, replayed through the planner the JS server runs
// (`planChatStep`) with the Agent CLI's advertised tools and its tool-result
// shapes (`read` → numbered `<file>` block, `write`/`edit` → empty string,
// `bash` → stdout). The workspace is an in-memory map, so a regression shows
// up as the exact wrong file content the CLI run produced.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const AGENT_CLI_TOOLS = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read', 'task',
  'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];

const pathOf = (args) => args.filePath ?? args.file_path ?? args.path;

function agentRead(text) {
  const body = text.split('\n').map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
  return `<file>\n${body}\n\n(End of file - total ${text.split('\n').length} lines)\n</file>`;
}

/** The Agent CLI's tools over an in-memory workspace; the bash tool knows `cat` and `sha256sum`. */
function execute(files, tool, args) {
  if (tool === 'read') {
    const path = pathOf(args);
    return files.has(path) ? agentRead(files.get(path)) : `Error: File not found: ${path}`;
  }
  if (tool === 'write') {
    files.set(pathOf(args), args.content);
    return '';
  }
  if (tool === 'edit') {
    const path = pathOf(args);
    const text = files.get(path) ?? '';
    if (!text.includes(args.oldString)) return 'Error: oldString not found in content';
    if (text.indexOf(args.oldString) !== text.lastIndexOf(args.oldString)) {
      return 'Error: Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.';
    }
    files.set(path, text.replace(args.oldString, () => args.newString));
    return '';
  }
  if (tool === 'bash') {
    const digest = /^sha256sum -- (\S+)$/.exec(args.command);
    if (digest) return `${createHash('sha256').update(files.get(digest[1]) ?? '').digest('hex')}  ${digest[1]}\n`;
    const cat = /^cat (\S+)$/.exec(args.command);
    if (cat) return files.get(cat[1]) ?? `cat: ${cat[1]}: No such file or directory`;
    const exists = (path) => files.has(path) || files.has(`${path}/`);
    const test = /^test (!? ?)-([ed]) (\S+)$/.exec(args.command);
    if (test) {
      const holds = test[2] === 'd' ? files.has(`${test[3]}/`) : exists(test[3]);
      return holds === (test[1] === '') ? '' : `Error: exit status 1`;
    }
    const mkdir = /^mkdir (\S+)$/.exec(args.command);
    if (mkdir) {
      files.set(`${mkdir[1]}/`, '');
      return '';
    }
    if (args.command.startsWith('git add -A && git commit')) return '5498cccac709f99fc533d81030c088d0b292e594\n';
    if (args.command === 'python3 greet.py') return 'Hello, World\n';
    if (/^node --check \S+$/u.test(args.command)) return '';
    if (/^node --test( \S+)?$/u.test(args.command)) return '# pass 2\n# fail 0\n';
  }
  return `Error: ${tool} is not simulated`;
}

async function drive(prompt, workspace, maxSteps = 8) {
  const files = new Map(Object.entries(workspace));
  const messages = [{ role: 'user', content: prompt }];
  const calls = [];
  for (let step = 0; step < maxSteps; step += 1) {
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    if (!plan || plan.kind === 'final') return { calls, files, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const id = `call_${step}`;
    const result = execute(files, call.tool, JSON.parse(call.arguments));
    calls.push(call.tool);
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, files, answer: null };
}

describe('PR #1188 dogfood: an append keeps the file it appends to', () => {
  // Ladder row T4 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('`Append the line \'third line\' to notes.txt.` adds one line instead of overwriting the file', async () => {
    const { calls, files, answer } = await drive("Append the line 'third line' to notes.txt.", {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first line\nsecond line\nthird line\n');
    assert.equal(answer, 'Appended `third line` to the end of `notes.txt` and observed the result.');
    assert.equal(files.has('.formal-ai/general-change-plan.lino'), false);
  });

  test('a prepend puts the quoted line first', async () => {
    const { files, answer } = await drive('Prepend "# Notes" to notes.txt', { 'notes.txt': 'first line\n' });
    assert.equal(files.get('notes.txt'), '# Notes\nfirst line\n');
    assert.equal(answer, 'Added `# Notes` to the start of `notes.txt` and observed the result.');
  });

  test('a file without a final newline gains the line without one', async () => {
    const { files } = await drive("Append 'second' to n.txt", { 'n.txt': 'no newline' });
    assert.equal(files.get('n.txt'), 'no newline\nsecond');
  });

  test('appending to a missing file creates it with the line', async () => {
    const { calls, files, answer } = await drive("Append 'hello' to new.txt", {});
    assert.deepEqual(calls, ['read', 'write', 'bash']);
    assert.equal(files.get('new.txt'), 'hello\n');
    assert.equal(answer, 'Appended `hello` to the end of `new.txt` and observed the result.');
  });

  test('the position words are seeded per language, not English-only', async () => {
    const { files, answer } = await drive('Добавь строку «третья строка» в конец notes.txt', {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.equal(files.get('notes.txt'), 'first line\nsecond line\nтретья строка\n');
    assert.equal(answer, 'В конец `notes.txt` добавлено `третья строка`, результат проверен.');
  });
});

describe('PR #1188 dogfood: a replacement edits only what it names', () => {
  const README = '# Project\n\nA small tool.\nThis is a smal project.\n';

  // Ladder row T7 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('`smal` -> `small` fixes the word, not the `smal` inside `small`', async () => {
    const { calls, files, answer } = await drive("Replace 'smal' with 'small' in README.md.", { 'README.md': README });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('README.md'), '# Project\n\nA small tool.\nThis is a small project.\n');
    assert.equal(answer, 'Replaced `smal` with `small` in `README.md` and observed the result.');
  });

  // Ladder row T8 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('a single-quoted positional insert places the line after its anchor', async () => {
    const { calls, files, answer } = await drive("Insert 'middle' after the line 'first line' in notes.txt.", {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first line\nmiddle\nsecond line\n');
    assert.equal(answer, 'Inserted `middle` after `first line` in `notes.txt` and observed the result.');
  });

  test('an anchor that occurs twice does not say where the line goes', async () => {
    const { files, answer } = await drive("Insert 'x' before the line 'same' in t.txt.", { 't.txt': 'same\nsame\n' });
    assert.equal(files.get('t.txt'), 'same\nsame\n');
    assert.equal(answer, 'The line `same` occurs 2 times in `t.txt`, so I cannot tell which one places the insert, and nothing was changed. Quote more of the line, or name it by its line number.');
  });
});

describe('PR #1188 dogfood: a removal takes out what it quotes', () => {
  // Ladder row T9 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('`Delete the line \'second line\' from notes.txt.` removes that line', async () => {
    const { calls, files, answer } = await drive("Delete the line 'second line' from notes.txt.", {
      'notes.txt': 'first line\nsecond line\n',
    });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first line\n');
    assert.equal(answer, 'Removed `second line` from `notes.txt` and observed the result.');
  });

  test('quoted text inside a line is removed from that line only', async () => {
    const { files } = await drive("Remove 'smal ' from README.md.", { 'README.md': '# P\n\nThis is a smal project.\n' });
    assert.equal(files.get('README.md'), '# P\n\nThis is a project.\n');
  });

  test('deleting a file is still a file deletion, not a text removal', async () => {
    const { calls } = await drive('Delete the file notes.txt.', { 'notes.txt': 'x\n' });
    assert.deepEqual(calls, ['bash']);
  });

  test('text found nowhere is reported, not written', async () => {
    const { files, answer } = await drive("Delete the line 'absent' from notes.txt.", { 'notes.txt': 'first line\n' });
    assert.equal(files.get('notes.txt'), 'first line\n');
    // Nothing was planned, so nothing failed to verify: the text is not there (G87).
    assert.equal(answer, '`absent` does not occur in `notes.txt`, so nothing was changed.');
  });
});

describe('PR #1188 dogfood: shell requests run what they name', () => {
  // Ladder row T13 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('`Run python3 greet.py.` runs the command without the sentence\'s full stop', async () => {
    const { calls, answer } = await drive('Run python3 greet.py.', { 'greet.py': 'print("Hello, World")\n' });
    assert.deepEqual(calls, ['bash']);
    assert.equal(answer, 'The `python3 greet.py` command completed. Output:\n\n```text\nHello, World\n```');
  });

  test('`Create a directory named src.` makes the directory as a verified recipe, not a listing', async () => {
    const { calls, files, answer } = await drive('Create a directory named src.', {});
    assert.deepEqual(calls, ['bash', 'bash', 'bash']);
    assert.equal(files.has('src/'), true);
    assert.equal(answer, 'Completed the action `mkdir src` and verified it with `test -d src`.');
  });

  // Ladder row T15 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('a commit carries the message the request quotes and pushes only where a remote exists', async () => {
    const files = new Map();
    const messages = [{ role: 'user', content: "Commit all changes with the message 'initial notes'." }];
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(JSON.parse(plan.calls[0].arguments).command,
      "git add -A && git commit -q -m 'initial notes' && if test -n \"$(git remote)\"; then git push -q origin HEAD; fi && git rev-parse HEAD");
    assert.equal(files.size, 0);
  });
});

// Ladder row T16 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
describe('PR #1188 dogfood: a filler word inside a cue phrase keeps the cue', () => {
  for (const [prompt, command] of [
    ['Show me git status.', 'git status'],
    ['Show me the git log.', 'git log'],
    ['Покажи мне статус git', 'git status'],
  ]) {
    test(`${prompt} runs ${command}`, async () => {
      const plan = await planChatStep([{ role: 'user', content: prompt }], AGENT_CLI_TOOLS);
      assert.equal(plan.kind, 'tool_calls');
      assert.equal(plan.calls[0].tool, 'bash');
      assert.equal(JSON.parse(plan.calls[0].arguments).command, command);
    });
  }
});

describe('PR #1188 dogfood: a setting changes on the line that assigns it', () => {
  const CONFIG = '{\n  "debug": false,\n  "name": "demo"\n}\n';

  // Ladder row T17 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('`Change the value of "debug" to true in config.json.` writes a bare boolean', async () => {
    const { calls, files, answer } = await drive('Change the value of "debug" to true in config.json.', { 'config.json': CONFIG });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('config.json'), '{\n  "debug": true,\n  "name": "demo"\n}\n');
    assert.equal(answer, 'Set `debug` to `true` in `config.json` and observed the result.');
  });

  test('a string value keeps the file\'s quoting', async () => {
    const { files } = await drive('Set the value of name to prod in config.json.', { 'config.json': CONFIG });
    assert.equal(files.get('config.json'), '{\n  "debug": false,\n  "name": "prod"\n}\n');
  });

  test('a YAML key is found by its own separator', async () => {
    const { files } = await drive('Change the setting debug to true in app.yml.', { 'app.yml': 'debug: false\nname: demo\n' });
    assert.equal(files.get('app.yml'), 'debug: true\nname: demo\n');
  });
});

describe('PR #1188 dogfood: a named source file says which language to write', () => {
  test('`Create hello.py that prints "Hello, World!" and run it.` writes hello.py and checks its output', async () => {
    const messages = [{ role: 'user', content: 'Create hello.py that prints "Hello, World!" and run it.' }];
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'write');
    const args = JSON.parse(plan.calls[0].arguments);
    assert.equal(args.filePath, 'hello.py');
    assert.equal(args.content,
      '# python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile hello.py\n# python3 hello.py\n# Emit the requested text followed by a newline.\nprint("Hello, World!")\n');
  });

  test('a file name no catalogued language saves as names no language', async () => {
    const { programContractAnswer } = await import('../../../js/agentic/crate/coding_program_contract.mjs');
    assert.equal(programContractAnswer('Create notes.txt that prints "Hello, World!".'), null);
    assert.equal(programContractAnswer('Change greet.py so it prints "Hi".'), null);
  });
});

describe('PR #1188 dogfood: a description of code is never written as a file\'s bytes', () => {
  test('`Add a function multiply(a, b) to math.mjs …` keeps math.mjs and adds only the function', async () => {
    const before = 'export function add(a, b) {\n  return a + b;\n}\n';
    const { files } = await drive(
      'Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, and run node --test to confirm it passes.',
      { 'math.mjs': before, 'math.test.mjs': "import { add } from './math.mjs';\n" },
    );
    assert.equal(files.get('math.mjs'), `${before}\nexport function multiply(a, b) {\n  return a * b;\n}\n`);
    assert.equal(files.has('.formal-ai/general-change-plan.lino'), false);
  });

  test('a request to author a function never writes a clause of itself as the file', async () => {
    const { composeGeneralChangePlan } = await import('../../../js/agentic/general_planner.mjs');
    assert.equal(composeGeneralChangePlan(
      'Write a Python function add(a, b) that returns their sum in add.py and run it with 2 and 3.'), null);
    const lino = 'substitution_rules\n  id "learned_program_plan_rules"';
    assert.equal(composeGeneralChangePlan(`Create file data/seed/learned-program-rules.lino containing\n${lino}`).content, lino);
  });

  test('literal content still writes the file', async () => {
    const { composeGeneralChangePlan } = await import('../../../js/agentic/general_planner.mjs');
    assert.equal(composeGeneralChangePlan('Create a file a.txt containing hello').content, 'hello');
    assert.equal(composeGeneralChangePlan("Write 'function f() {}' to f.js").content, 'function f() {}');
  });
});

describe('PR #1188 dogfood: "their sum" of two parameters reads both of them', () => {
  const answer = (name, operation, symbol) =>
    `Coding task formalized for Python function \`${name}\`.\nDiscovered structural parts: ${operation}.\n` +
    `\`\`\`python\ndef ${name}(a, b):\n    return a ${symbol} b\n\`\`\`\n` +
    'Verification status: unverified in the browser boundary; run the Rust/native solver to execute the derived program and its tests.';

  test('`add(a, b) that returns their sum` is `a + b`, not `sum(a)`', async () => {
    const { solve } = await import('../../../js/agentic/host.mjs');
    const result = await solve('Write a Python function add(a, b) that returns their sum.', []);
    assert.equal(result.intent, 'write_program');
    assert.equal(result.answer, answer('add', 'reduce_sum', '+'));
  });

  test('`multiply(a, b) that returns a times b` is `a * b`: `a` is not read by `math`', async () => {
    const { solve } = await import('../../../js/agentic/host.mjs');
    const result = await solve('Write a Python function multiply(a, b) that returns a times b.', []);
    assert.equal(result.answer, answer('multiply', 'reduce_product', '*'));
  });
});

describe('PR #1188 dogfood: a compacted ladder leaf does not rewrite the list it already changed', () => {
  const LEAF = 'Atomic task L01: Edit the tracked file `rust/src/web_search_core.rs`: add "wikiquote" to the WEB_SEARCH_PROVIDERS list. Change only that file and keep it valid Rust.';
  const UPDATED = 'pub const WEB_SEARCH_PROVIDERS: [&str; 4] = ["duckduckgo", "brave", "startpage", "wikiquote"];\n';

  test('re-read after the summary envelope -> already present at once, never a second write', async () => {
    const messages = [
      { role: 'user', content: 'What did we do so far?' },
      { role: 'assistant', content: `Conversation summary: Atomic task L01 adds wikiquote to the provider list.\n\nTitle: Add wikiquote\n\nUser turns:\n  1. ${LEAF}` },
      { role: 'user', content: 'Continue if you have next steps' },
    ];
    const calls = [];
    let answer = null;
    for (let step = 0; step < 6; step += 1) {
      const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
      if (!plan || plan.kind === 'final') {
        answer = plan ? plan.answer : null;
        break;
      }
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      assert.ok(!(call.tool === 'write' && String(args.filePath).endsWith('rust/src/web_search_core.rs')), JSON.stringify(calls));
      calls.push(call.tool);
      const id = `c${step}`;
      const result = call.tool === 'read' ? agentRead(UPDATED) : call.tool === 'bash' ? UPDATED : '';
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      messages.push({ role: 'tool', tool_call_id: id, content: result });
    }
    assert.deepEqual(calls, ['read']);
    assert.equal(answer, '`rust/src/web_search_core.rs` already lists "wikiquote"; nothing needed to change.');
  });
});

describe('PR #1188 dogfood: a synthesized function is written to the named file and called', () => {
  const PROMPT = 'Write a Python function add(a, b) that returns their sum in add.py and run it with 2 and 3.';
  const CHECK = 'python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile';

  // Ladder row T19 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('write add.py -> check -> call with 2 and 3 -> report', async () => {
    const { solve } = await import('../../../js/agentic/host.mjs');
    const { planSymbolicCommandReroute } = await import('../../../js/agentic/command_reroute.mjs');
    const symbolic = await solve(PROMPT, []);
    const messages = [{ role: 'user', content: PROMPT }];
    const planned = [];
    let plan = null;
    for (let step = 0; step < 6; step += 1) {
      plan = planSymbolicCommandReroute(messages, AGENT_CLI_TOOLS, symbolic);
      if (!plan || plan.kind !== 'tool_calls') break;
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      planned.push([call.tool, args.command ?? args.filePath]);
      const id = `c${step}`;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      messages.push({ role: 'tool', tool_call_id: id, content: String(args.command ?? '').includes('print(add(2, 3))') ? '5\n' : '' });
      if (call.tool === 'write') assert.equal(args.content, 'def add(a, b):\n    return a + b\n');
    }
    assert.deepEqual(planned, [
      ['write', 'add.py'],
      ['bash', `${CHECK} add.py`],
      ['bash', 'python3 -B -c "from add import add; print(add(2, 3))"'],
    ]);
    assert.equal(plan.kind, 'final');
    assert.equal(plan.answer,
      'Created and verified `add.py` through the agentic CLI harness.\n\n```python\ndef add(a, b):\n    return a + b\n\n```\n\n' +
      `Commands executed by the harness:\n- \`${CHECK} add.py\`\n- \`python3 -B -c "from add import add; print(add(2, 3))"\`\n\n` +
      'Actual tool output:\n\n```text\n5\n```');
  });

  test('call arguments are the numbers and quoted literals after the last run verb', async () => {
    const { statedCallArguments } = await import('../../../js/agentic/command_reroute.mjs');
    assert.deepEqual(statedCallArguments('Write f(x, y) in f.py and run it with 2 and 3.'), ['2', '3']);
    assert.deepEqual(statedCallArguments('Write greet(name) in g.py and run it with "Ada".'), ['"Ada"']);
    assert.deepEqual(statedCallArguments('Write f(x) with 2 parameters in f.py.'), []);
  });
});

describe('PR #1188 dogfood: a typo without its correction is corrected by discovery', () => {
  // Ladder row T6 of docs/case-studies/pull-request-1188/formal-ai-dogfood.md.
  test('`Fix the typo \'smal\' in README.md.` -> small, from the seed vocabulary', async () => {
    const { calls, files, answer } = await drive("Fix the typo 'smal' in README.md.", { 'README.md': '# P\n\nA small tool.\nThis is a smal project.\n' });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('README.md'), '# P\n\nA small tool.\nThis is a small project.\n');
    assert.equal(answer, 'Replaced `smal` with `small` in `README.md` and observed the result.');
  });

  test('the correction is the unique most frequent word one edit away, in the word\'s capitalisation', async () => {
    const { correctedSpelling } = await import('../../../js/agentic/crate/spelling.mjs');
    assert.equal(correctedSpelling('smal'), 'small');
    assert.equal(correctedSpelling('Projet'), 'Project');
    assert.equal(correctedSpelling('teh'), 'the');
  });
});

describe('PR #1188 dogfood: an unquoted line is grounded between its lead and the destination cue', () => {
  for (const [prompt, line] of [
    ['Append the line third to notes.txt.', 'third'],
    ['Append the line go to bed to notes.txt.', 'go to bed'],
    ['Добавь строку третья в конец notes.txt', 'третья'],
  ]) {
    test(`${prompt} appends ${line}`, async () => {
      const { files } = await drive(prompt, { 'notes.txt': 'first line\n' });
      assert.equal(files.get('notes.txt'), `first line\n${line}\n`);
    });
  }

  test('without a line lead nothing is guessed and nothing is written', async () => {
    const { files } = await drive('Append third to notes.txt.', { 'notes.txt': 'first line\n' });
    assert.equal(files.get('notes.txt'), 'first line\n');
  });
});

describe('PR #1188 dogfood: code and blank lines are stated in seeded words', () => {
  const LIB = 'fn old() -> u8 {\n    1\n}\nfn keep() {}\n';

  test('a multi-line replacement written with \\n edits the function, never the whole file', async () => {
    const { calls, files, answer } = await drive('In src/lib.rs replace "fn old() -> u8 {\\n    1\\n}" with "fn old() -> u8 {\\n    2\\n}".', { 'src/lib.rs': LIB });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('src/lib.rs'), 'fn old() -> u8 {\n    2\n}\nfn keep() {}\n');
    assert.equal(answer, 'Replaced `fn old() -> u8 {\n    1\n}` with `fn old() -> u8 {\n    2\n}` in `src/lib.rs` and observed the result.');
  });

  test('an appended empty line is stated as an empty line', async () => {
    const { files, answer } = await drive('Append an empty line to src/lib.rs.', { 'src/lib.rs': LIB });
    assert.equal(files.get('src/lib.rs'), `${LIB}\n`);
    assert.equal(answer, 'Added an empty line to `src/lib.rs` and observed the result.');
  });
});

describe('PR #1188 dogfood: cues inside a quoted payload do not steer routing', () => {
  test('a payload that names another file is appended to the file the request names', async () => {
    const { calls, files, answer } = await drive('Append "/// Append an empty line to notes.txt." to src/lib.rs.', { 'src/lib.rs': 'fn keep() {}\n' });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('src/lib.rs'), 'fn keep() {}\n/// Append an empty line to notes.txt.\n');
    assert.equal(files.has('notes.txt'), false);
    assert.equal(answer, 'Appended `/// Append an empty line to notes.txt.` to the end of `src/lib.rs` and observed the result.');
  });

  test('a quoted "Commit all changes" is text to append, not a commit', async () => {
    const { calls, files } = await drive('Append "Commit all changes" to notes.txt.', { 'notes.txt': 'first\n' });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('notes.txt'), 'first\nCommit all changes\n');
  });
});

describe('PR #1188 dogfood: the task survives repeated compactions', () => {
  const LEAF = 'Atomic task L01: Edit the tracked file `rust/src/web_search_core.rs`: add "wikiquote" to the WEB_SEARCH_PROVIDERS list. Change only that file and keep it valid Rust.';

  test('the server answers a compaction request with the native envelope', async () => {
    const { conversationSummaryEnvelope } = await import('../../../js/agentic/crate/conversation_summary.mjs');
    const envelope = conversationSummaryEnvelope('Provide a detailed but concise summary of our conversation above.', [
      { role: 'user', content: LEAF },
      { role: 'assistant', content: 'Let me open rust/src/web_search_core.rs and read what it says.' },
      { role: 'user', content: 'What did we do so far?' },
    ]);
    assert.ok(envelope.startsWith('Conversation summary: '), envelope);
    assert.ok(envelope.endsWith(`\n\nUser turns:\n  1. ${LEAF}\n  2. What did we do so far?`), envelope);
  });

  test('a nested envelope whose first user turn is the recall question still yields the task', async () => {
    const messages = [
      { role: 'user', content: 'What did we do so far?' },
      { role: 'assistant', content: `Conversation summary: What did we do so far? Continue if you have next steps. What did we do so far? Conversation summary: ${LEAF}\n\nTitle: Add wikiquote\n\nUser turns:\n  1. What did we do so far?\n  2. Continue if you have next steps` },
      { role: 'user', content: 'Continue if you have next steps' },
    ];
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'read');
    assert.equal(JSON.parse(plan.calls[0].arguments).filePath, 'rust/src/web_search_core.rs');
  });
});

describe('PR #1188 dogfood: a re-summarized envelope head trails the client residue', () => {
  test('the head sentences that state work are the task', async () => {
    const LEAF = 'Atomic task L01: Edit the tracked file `rust/src/web_search_core.rs`: add "wikiquote" to the WEB_SEARCH_PROVIDERS list. Change only that file and keep it valid Rust.';
    const messages = [
      { role: 'user', content: 'What did we do so far?' },
      { role: 'assistant', content: `Conversation summary: What did we do so far? Continue if you have next steps. What did we do so far? Conversation summary: ${LEAF} What did we do so far? Title: What did we do so. User turns:.\n\nTitle: What did we do so\n\nUser turns:\n  1. What did we do so far?\n  2. Continue if you have next steps\n  3. What did we do so far?` },
      { role: 'user', content: 'Continue if you have next steps' },
    ];
    const plan = await planChatStep(messages, AGENT_CLI_TOOLS);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'read');
    assert.equal(JSON.parse(plan.calls[0].arguments).filePath, 'rust/src/web_search_core.rs');
  });
});

describe('PR #1188 dogfood: a reduction over three parameters reads all three', () => {
  for (const [name, operation, symbol] of [['add3', 'reduce_sum', '+'], ['mul3', 'reduce_product', '*']]) {
    test(`${name}(a, b, c) is a ${symbol} b ${symbol} c`, async () => {
      const { solve } = await import('../../../js/agentic/host.mjs');
      const word = operation === 'reduce_sum' ? 'sum' : 'product';
      const result = await solve(`Write a Python function ${name}(a, b, c) that returns their ${word}.`, []);
      assert.equal(result.answer,
        `Coding task formalized for Python function \`${name}\`.\nDiscovered structural parts: ${operation}.\n` +
        `\`\`\`python\ndef ${name}(a, b, c):\n    return a ${symbol} b ${symbol} c\n\`\`\`\n` +
        'Verification status: unverified in the browser boundary; run the Rust/native solver to execute the derived program and its tests.');
    });
  }
});

describe('PR #1188 dogfood: an unquoted output is bound only when it reads as an utterance', () => {
  const HELLO = '# python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile hello.py\n# python3 hello.py\n'
    + '# Emit the requested text followed by a newline.\nprint("Hello, World!")\n';

  test('`Create hello.py that prints Hello, World! and run it.` writes hello.py and checks its output', async () => {
    const plan = await planChatStep([{ role: 'user', content: 'Create hello.py that prints Hello, World! and run it.' }], AGENT_CLI_TOOLS);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'write');
    const args = JSON.parse(plan.calls[0].arguments);
    assert.equal(args.filePath, 'hello.py');
    assert.equal(args.content, HELLO);
  });

  test('`Write a Python program hello.py that prints …` names its file without a write cue', async () => {
    const plan = await planChatStep([{ role: 'user', content: 'Write a Python program hello.py that prints Hello, World! and run it.' }], AGENT_CLI_TOOLS);
    const args = JSON.parse(plan.calls[0].arguments);
    assert.equal(args.filePath, 'hello.py');
    assert.equal(args.content, HELLO);
  });

  test('the utterance ends at a seeded clause separator or the sentence that carries it', async () => {
    const { boundOutputLiterals } = await import('../../../js/agentic/crate/intent_formalization_obligations.mjs');
    assert.deepEqual(boundOutputLiterals('write a program that prints Hello, World! and run it'), ['Hello, World!']);
    assert.deepEqual(boundOutputLiterals('Напиши программу на Python, которая выводит Привет, мир! и запусти её'), ['Привет, мир!']);
    assert.deepEqual(boundOutputLiterals('Write hello.py that prints Hi. Then print "Bye".'), ['Hi', 'Bye']);
  });

  test('a description of a computation is never bound as text to print', async () => {
    const { boundOutputLiterals } = await import('../../../js/agentic/crate/intent_formalization_obligations.mjs');
    for (const prompt of [
      'Write a Python program that prints the sum of a and b and run it',
      'Write a program that prints Fibonacci numbers up to 100',
      'Write a program that prints FizzBuzz',
      'Write a program that prints FizzBuzz for 1 to 100',
      'Write a program that prints prime numbers below 50',
      'Print the greeting! Then "Hello"',
    ]) assert.deepEqual(boundOutputLiterals(prompt), [], prompt);
  });
});

describe('PR #1188 dogfood: a program the request asks to run is written in the seeded language when it names none', () => {
  const MAIN = (text) => '# python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile main.py\n# python3 main.py\n'
    + `# Emit the requested text followed by a newline.\nprint("${text}")\n`;

  test('the seed names the language and the reason, and the catalog runs it', async () => {
    const { readText } = await import('../../../js/agentic/host.mjs');
    const { findChildValue, parseLinoRoot } = await import('../../../js/agentic/write_lino.mjs');
    const { programLanguageBySlug } = await import('../../../js/agentic/crate/coding_catalog.mjs');
    const root = parseLinoRoot(readText('data/meta/stdout-program-contracts.lino')).children[0];
    const language = findChildValue(root, 'unnamed_language');
    assert.equal(language, 'python');
    assert.match(findChildValue(root, 'unnamed_language_reason'), /issue #906/);
    assert.equal(programLanguageBySlug(language).save_as, 'main.py');
  });

  for (const [prompt, text] of [
    ['write a program that prints Hello, World! and run it', 'Hello, World!'],
    ['Напиши программу, которая выводит Привет, мир! и запусти её', 'Привет, мир!'],
    ['写一个打印 "Ni hao" 的程序并运行', 'Ni hao'],
    ['एक प्रोग्राम लिखो जो "Namaste" प्रिंट करे और उसे चलाओ', 'Namaste'],
  ]) {
    test(`\`${prompt}\` writes main.py and checks its output instead of searching the web`, async () => {
      const plan = await planChatStep([{ role: 'user', content: prompt }], AGENT_CLI_TOOLS);
      assert.equal(plan.kind, 'tool_calls');
      assert.equal(plan.calls[0].tool, 'write');
      const args = JSON.parse(plan.calls[0].arguments);
      assert.equal(args.filePath, 'main.py');
      assert.equal(args.content, MAIN(text));
    });
  }

  test('a verb-final language binds the quoted output its print verb follows (seeded `verb_final`)', async () => {
    const { boundOutputLiterals } = await import('../../../js/agentic/crate/intent_formalization_obligations.mjs');
    assert.deepEqual(boundOutputLiterals('एक प्रोग्राम लिखो जो "Namaste" प्रिंट करे और उसे चलाओ'), ['Namaste']);
    assert.deepEqual(boundOutputLiterals('Python में एक प्रोग्राम लिखें जो "Alpha" प्रिंट करता है'), ['Alpha']);
    // A quoted value no print verb follows, or one a clause separator parts from it, is not output.
    assert.deepEqual(boundOutputLiterals('"notes.txt" में "x" को "y" से बदलो'), []);
    assert.deepEqual(boundOutputLiterals('"a.txt" बनाओ और "b" प्रिंट करो'), ['b']);
  });

  test('a request that does not ask to run the program does not get a language chosen for it (issue #906)', async () => {
    const { programContractAnswer } = await import('../../../js/agentic/crate/coding_program_contract.mjs');
    assert.equal(programContractAnswer('Write a program that prints Hello, World!'), null);
    assert.equal(programContractAnswer('Write a program that prints "Hi"'), null);
  });
});

describe('PR #1188 dogfood: a request that names a line deletes whole lines', () => {
  const TABLE = '| id | value |\n| --- | --- |\n| R56kfQp | drop me |\n| R1 | keep |\n';
  const KEPT = '| id | value |\n| --- | --- |\n| R1 | keep |\n';

  test('`Delete the line containing \'| R56kfQp |\' from t.md.` removes the row, not the quoted cells', async () => {
    const { calls, files, answer } = await drive("Delete the line containing '| R56kfQp |' from t.md.", { 't.md': TABLE });
    assert.deepEqual(calls, ['read', 'edit', 'bash']);
    assert.equal(files.get('t.md'), KEPT);
    assert.equal(answer, 'Deleted 1 line(s) containing `| R56kfQp |` from `t.md` and observed the result.');
  });

  test('`Remove the line …` and `Delete lines containing …` read the same seeded line meaning', async () => {
    const remove = await drive("Remove the line '| R56kfQp |' from t.md.", { 't.md': TABLE });
    assert.equal(remove.files.get('t.md'), KEPT);
    const plural = await drive("Delete lines containing 'R' from t.md.", { 't.md': TABLE });
    assert.equal(plural.files.get('t.md'), '| id | value |\n| --- | --- |\n');
    assert.equal(plural.answer, 'Deleted 2 line(s) containing `R` from `t.md` and observed the result.');
  });

  test('ru, hi and zh name the line in their own words; a sentence-final verb is still the verb', async () => {
    for (const [prompt, expected] of [
      ["Удали из t.md строку, содержащую '| R56kfQp |'.", 'Из `t.md` удалены строки, содержащие `| R56kfQp |` (1), результат проверен.'],
      ["t.md से '| R56kfQp |' वाली पंक्ति हटाओ।", '`t.md` से `| R56kfQp |` वाली 1 पंक्ति(याँ) हटाईं और परिणाम सत्यापित किया।'],
      ["删除 t.md 中包含 '| R56kfQp |' 的行。", '已从 `t.md` 中删除包含 `| R56kfQp |` 的 1 行并验证了结果。'],
    ]) {
      const { files, answer } = await drive(prompt, { 't.md': TABLE });
      assert.equal(files.get('t.md'), KEPT, prompt);
      assert.equal(answer, expected, prompt);
    }
  });

  test('without a line named, quoted text is still removed from inside its line', async () => {
    const { files, answer } = await drive("Remove 'drop me' from t.md.", { 't.md': TABLE });
    assert.equal(files.get('t.md'), '| id | value |\n| --- | --- |\n| R56kfQp |  |\n| R1 | keep |\n');
    assert.equal(answer, 'Removed `drop me` from `t.md` and observed the result.');
  });
});

describe('PR #1188 dogfood: a function and its test are added to existing ES modules (T1)', () => {
  const MATH = 'export function add(a, b) {\n  return a + b;\n}\n';
  const TEST = "import { test } from 'node:test';\nimport assert from 'node:assert/strict';\nimport { add } from './math.mjs';\n\n"
    + "test('add', () => {\n  assert.equal(add(2, 3), 5);\n});\n";
  const PROMPT = 'Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, '
    + 'and run node --test to confirm it passes.';

  test('read both -> write the module and the test -> node --check -> node --test -> report', async () => {
    const { calls, files, answer } = await drive(PROMPT, { 'math.mjs': MATH, 'math.test.mjs': TEST }, 10);
    assert.deepEqual(calls, ['read', 'read', 'write', 'write', 'bash', 'bash']);
    assert.equal(files.get('math.mjs'), `${MATH}\nexport function multiply(a, b) {\n  return a * b;\n}\n`);
    assert.equal(files.get('math.test.mjs'), TEST.replace('{ add }', '{ add, multiply }')
      + "\ntest('multiply', () => {\n  assert.equal(multiply(2, 3), 6);\n});\n");
    assert.equal(answer, 'Created and verified `math.mjs` through the agentic CLI harness.\n\n'
      + `\`\`\`javascript\n${files.get('math.mjs')}\n\`\`\`\n\nCommands executed by the harness:\n\n`
      + `\`math.test.mjs\`\n\n\`\`\`text\n${files.get('math.test.mjs')}\n\`\`\`\n`
      + '- `node --check math.mjs`\n- `node --test`\n\nActual tool output:\n\n```text\n# pass 2\n# fail 0\n```');
  });

  test('the expected value is the specification evaluated, not the synthesized code run', async () => {
    const { moduleFunctionRequest } = await import('../../../js/agentic/module_function.mjs');
    assert.deepEqual(moduleFunctionRequest(PROMPT), {
      name: 'multiply', parameters: ['a', 'b'], at: 15, module: 'math.mjs', test: 'math.test.mjs', language: 'javascript',
      clause: 'Add a function multiply(a, b) to math.mjs that returns a times b', command: 'node --test',
    });
    // A file no seeded extension names, or a test asked for without its file, is not this route's.
    assert.equal(moduleFunctionRequest('Add a function multiply(a, b) to math.txt that returns a times b.'), null);
    assert.equal(moduleFunctionRequest('Add a function multiply(a, b) to math.mjs that returns a times b, and add a test.'), null);
  });

  test('a relation the request names (`returns their sum`, `their difference`) is the expected value, in every seeded language', async () => {
    const both = (operator) => `${MATH}\nexport function both(a, b) {\n  return a ${operator} b;\n}\n`;
    const tested = (expected) => TEST.replace('{ add }', '{ add, both }')
      + `\ntest('both', () => {\n  assert.equal(both(2, 3), ${expected});\n});\n`;
    for (const [prompt, operator, expected] of [
      ['Add a function both(a, b) to math.mjs that returns their sum, add a test for it to math.test.mjs, and run node --test.', '+', 5],
      ['Add a function both(a, b) to math.mjs that returns their product, add a test for it to math.test.mjs, and run node --test.', '*', 6],
      ['Добавь функцию both(a, b) в math.mjs, которая возвращает их сумму, добавь тест для неё в math.test.mjs и запусти node --test.', '+', 5],
      ['math.mjs में एक फ़ंक्शन both(a, b) जोड़ो जो उनका योग लौटाता है, math.test.mjs में उसका टेस्ट जोड़ो और node --test चलाओ।', '+', 5],
      ['在 math.mjs 中添加一个函数 both(a, b)，返回它们的乘积，在 math.test.mjs 中为它添加测试，然后运行 node --test。', '*', 6],
      ['Add a function both(a, b) to math.mjs that returns their difference, add a test for it to math.test.mjs, and run node --test.', '-', -1],
      ['Добавь функцию both(a, b) в math.mjs, которая возвращает их разность, добавь тест для неё в math.test.mjs и запусти node --test.', '-', -1],
      ['math.mjs में एक फ़ंक्शन both(a, b) जोड़ो जो उनका अंतर लौटाता है, math.test.mjs में उसका टेस्ट जोड़ो और node --test चलाओ।', '-', -1],
      ['在 math.mjs 中添加一个函数 both(a, b)，返回它们的差值，在 math.test.mjs 中为它添加测试，然后运行 node --test。', '-', -1],
    ]) {
      const { calls, files } = await drive(prompt, { 'math.mjs': MATH, 'math.test.mjs': TEST }, 10);
      assert.deepEqual(calls, ['read', 'read', 'write', 'write', 'bash', 'bash'], prompt);
      assert.equal(files.get('math.mjs'), both(operator), prompt);
      assert.equal(files.get('math.test.mjs'), tested(expected), prompt);
    }
  });

  test('the stated command is run in every language, after the run verb or before a final one', async () => {
    const { moduleFunctionRequest } = await import('../../../js/agentic/module_function.mjs');
    for (const prompt of [
      'Добавь функцию both(a, b) в math.mjs, которая возвращает их сумму, добавь тест для неё в math.test.mjs и запусти node --test.',
      'math.mjs में एक फ़ंक्शन both(a, b) जोड़ो जो उनका योग लौटाता है, math.test.mjs में उसका टेस्ट जोड़ो और node --test चलाओ।',
      '在 math.mjs 中添加一个函数 both(a, b)，返回它们的乘积，在 math.test.mjs 中为它添加测试，然后运行 node --test。',
    ]) assert.equal(moduleFunctionRequest(prompt).command, 'node --test', prompt);
  });

  test('the browser composer lowers the same IR to JavaScript from the seeded realization', async () => {
    const { solve } = await import('../../../js/agentic/host.mjs');
    const result = await solve('Write a JavaScript function multiply(a, b) that returns a times b.', []);
    assert.equal(result.answer,
      'Coding task formalized for JavaScript function `multiply`.\nDiscovered structural parts: reduce_product.\n'
      + '```javascript\nexport function multiply(a, b) {\n  return a * b;\n}\n```\n'
      + 'Verification status: unverified in the browser boundary; run the Rust/native solver to execute the derived program and its tests.');
  });
});

describe('PR #1188 dogfood: an edit-shaped request never becomes a file deletion', () => {
  const NOTES = 'keep one\ndrop me\nkeep two\n';

  test('held-out en/ru/hi/zh probes are declined honestly and leave the file byte-identical', async () => {
    for (const [prompt, expected] of [
      ['Delete the file t.md word drop.',
        'No edit I can verify reads this request against `t.md`, so nothing was changed. The request names text inside the file, not the file itself, so I did not delete it.'],
      ['Удалить файл t.md слово drop',
        'Ни одна проверяемая правка не прочитала этот запрос для `t.md`, поэтому ничего не изменено. Запрос говорит о тексте внутри файла, а не о самом файле, поэтому файл не удалён.'],
      ['t.md से drop शब्द हटाओ।',
        '`t.md` के लिए इस अनुरोध को कोई सत्यापन-योग्य संपादन नहीं पढ़ सका, इसलिए कुछ नहीं बदला गया। अनुरोध फ़ाइल के भीतर के पाठ के बारे में है, फ़ाइल के बारे में नहीं, इसलिए फ़ाइल नहीं हटाई गई।'],
      ['删除文件 t.md 里的文字 drop',
        '没有可验证的编辑能针对 `t.md` 读懂这个请求，因此未做任何更改。请求说的是文件中的文本，而不是文件本身，所以没有删除该文件。'],
    ]) {
      const { calls, files, answer } = await drive(prompt, { 't.md': NOTES });
      assert.deepEqual(calls, [], prompt);
      assert.equal(files.get('t.md'), NOTES, prompt);
      assert.equal(answer, expected, prompt);
    }
  });

  test('a request that plainly deletes the file is still a deletion, in every language', async () => {
    for (const prompt of ['Delete the file t.md', 'Удали файл t.md', 't.md हटाओ', '删除文件 t.md']) {
      const plan = await planChatStep([{ role: 'user', content: prompt }], AGENT_CLI_TOOLS);
      assert.equal(plan.calls[0].tool, 'bash', prompt);
      assert.equal(JSON.parse(plan.calls[0].arguments).command, 'rm t.md', prompt);
    }
  });

  test('the destructive commands are the seed\'s `destructive true` intents, not a list in code', async () => {
    const { shellIntentVocabulary } = await import('../../../js/agentic/crate/seed_shell_intents.mjs');
    assert.deepEqual(shellIntentVocabulary().intents.filter((intent) => intent.destructive).map((intent) => intent.command), ['rmdir', 'rm']);
  });
});
