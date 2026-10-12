// Issue #1021: the behaviour range, pinned on the JavaScript roots.
//
// rust/tests/unit/issue_1021_behaviour_range.rs pins every reported prompt of
// issue #1021 with held-out paraphrases on the Rust solver. The same prompts are
// driven here through the JavaScript twins: the agentic planner
// (js/agentic/planner.mjs, the Agent CLI path) for the shell rows, and the
// browser worker (js/worker/) for the coding rows. A green run means the
// generalization holds on the first root too, not only on the Rust one.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { createWorkerContext, evaluate } from './support/browser-runtime.mjs';

let planner;
let mutating;
let shell;
let worker;

before(async () => {
  await installNodeHost(new WorkerHost());
  planner = await import('../../../js/agentic/planner.mjs');
  mutating = await import('../../../js/agentic/mutating_action.mjs');
  shell = await import('../../../js/agentic/shell_command.mjs');
  worker = createWorkerContext();
  await evaluate(worker, 'loadSeed()');
});

/** The longest recipe the seed declares is six steps; twelve is a runaway. */
const MAX_PLAN_STEPS = 12;

/**
 * The shell command the agentic planner resolves for `prompt`, or null. A
 * verified recipe (issue #944) is driven to its end with every step succeeding
 * and its mutating action picked out, as the Rust `shell_command` helper does;
 * a single read-only command is returned as planned.
 */
async function shellCommand(prompt) {
  const messages = [{ role: 'user', content: prompt }];
  const commands = [];
  while (commands.length <= MAX_PLAN_STEPS) {
    const plan = await planner.planChatStep(messages, ['exec_command']);
    if (!plan || plan.kind !== 'tool_calls') break;
    const call = plan.calls[0];
    const command = JSON.parse(call.arguments).command;
    if (!command) break;
    const id = `call_${messages.length}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: `Command: ${command}\nOutput: (empty)\nExit Code: 0` });
    commands.push(command);
    // A read-only command is the whole plan; only a recipe opens with a check.
    if (commands.length === 1 && !command.startsWith('test ')) break;
  }
  const resolved = shell.shellCommandForTask(prompt);
  const recipe = resolved === null ? null : mutating.expand(resolved);
  if (recipe === null) return commands[0] ?? null;
  assert.deepEqual(commands, recipe.steps, 'every declared precondition, preparation, action and postcondition is driven');
  return commands[recipe.action] ?? null;
}

const solve = (prompt) => worker.solve(prompt, [], {}, {}, [], {});

describe('R1021-3/4/5/9: shell requests on the agentic planner', () => {
  test('a command-naming noun is not an argument (#866, #867)', async () => {
    for (const [prompt, expected] of [
      ['Execute ls command', 'ls'],
      ['Execute the ls command', 'ls'],
      ['run ls command', 'ls'],
      ['Run the ls command please', 'ls'],
      ['выполни команду ls', 'ls'],
      ['запусти ls команду', 'ls'],
      ['execute pwd command', 'pwd'],
      ['run whoami command', 'whoami'],
      ['execute git status command', 'git status'],
      ['run the df -h command', 'df -h'],
    ]) {
      assert.equal(await shellCommand(prompt), expected, prompt);
    }
  });

  test('a prose listing request routes to ls in any word order, english, russian, hindi and spanish (#865)', async () => {
    for (const prompt of [
      'List me files here',
      'list me the files here',
      'here, show me files',
      'could you enumerate the files in this folder',
      'which files do we have here?',
      'give me the contents of the current directory',
      'перечисли мне файлы здесь',
      'покажи мне файлы в этой папке',
      'यहाँ फ़ाइलें दिखाओ',
      'lista los archivos aquí',
      'muéstrame los archivos de la carpeta actual',
      'aquí, enseña los ficheros',
      '¿cuáles archivos hay en este directorio?',
    ]) {
      assert.equal(await shellCommand(prompt), 'ls', prompt);
    }
    assert.notEqual(await shellCommand('show me the current directory'), 'ls');
    assert.notEqual(await shellCommand('list the running processes'), 'ls');
  });

  test('a bare command is the request (#868)', async () => {
    for (const command of ['ls', 'pwd', 'whoami', 'ls -la', 'df -h']) {
      assert.equal(await shellCommand(command), command, command);
    }
  });

  test('a move between absolute or home-relative paths is performed; a traversal is not (#824)', async () => {
    for (const [prompt, expected] of [
      ['Move /Users/konard/Desktop/Archive/hive-control-center to ~/Code/Archive/link-assistant',
        'mv /Users/konard/Desktop/Archive/hive-control-center ~/Code/Archive/link-assistant'],
      ['move /tmp/report.txt to ~/Documents/report.txt', 'mv /tmp/report.txt ~/Documents/report.txt'],
      ['please move the file /var/log/build.log to ~/archive/build.log', 'mv /var/log/build.log ~/archive/build.log'],
      ['перемести файл /tmp/report.txt в ~/Documents/report.txt', 'mv /tmp/report.txt ~/Documents/report.txt'],
    ]) {
      assert.equal(await shellCommand(prompt), expected, prompt);
    }
    for (const prompt of ['move ../secrets/key.pem to ~/key.pem', 'move /tmp/report.txt to ../../elsewhere/report.txt']) {
      assert.equal(await shellCommand(prompt), null, prompt);
    }
  });

  test('a cue that names its object still takes a plain name', async () => {
    for (const [prompt, expected] of [
      ['remove the directory build', 'rmdir build'],
      ['delete the file old.txt', 'rm old.txt'],
      ['copy a.txt to b.txt', 'cp a.txt b.txt'],
      ['rename old.txt to new.txt', 'mv old.txt new.txt'],
      ['create a symbolic link from a to b', 'ln -s a b'],
    ]) {
      assert.equal(await shellCommand(prompt), expected, prompt);
    }
  });
});

describe('R1021-6/7: a named exercise is a program, not a file operation', () => {
  test('neither the exercise nor its Rosetta Code URL reaches the shell (#862, #863)', async () => {
    for (const prompt of [
      'Give me example of how to do copy stdin to stdout in Rust',
      'Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust',
      'show me how to copy stdin to stdout',
      'how do I copy standard input to standard output in Rust?',
      'write a program that copies stdin to stdout',
      'rename the variable counter to total in this function',
    ]) {
      assert.equal(await shellCommand(prompt), null, prompt);
    }
  });

  test('the exercise is answered as a catalogued program in english, russian, hindi, chinese and spanish', async () => {
    for (const [prompt, language] of [
      ['Give me example of how to do copy stdin to stdout in Rust', 'rust'],
      ['Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust', 'rust'],
      ['write a program that copies stdin to stdout in Python', 'python'],
      ['Write a Go program that copies standard input to standard output', 'go'],
      ['напиши программу на C, которая копирует стандартный ввод в стандартный вывод', 'c'],
      ['скопировать stdin в stdout на php', 'php'],
      ['मानक इनपुट को मानक आउटपुट में कॉपी करें python', 'python'],
      ['用 Java 将标准输入复制到标准输出', 'java'],
      ['copiar stdin a stdout en JavaScript', 'javascript'],
    ]) {
      const answer = await solve(prompt);
      assert.equal(answer.intent, 'write_program', prompt);
      assert.ok(answer.evidence.includes(`response:write_program:copy_stdin_to_stdout:${language}`), `${prompt}: ${answer.evidence}`);
    }
  });

  test('the stdin answer pipes the input it was verified against into the run command', async () => {
    const answer = await solve('copy stdin to stdout in Rust');
    assert.equal(answer.content, [
      'Here is a minimal Rust copy standard input to standard output program:',
      '',
      '```rust',
      'use std::io::{self, Read, Write};',
      '',
      'fn main() -> io::Result<()> {',
      '    let mut input = Vec::new();',
      '    io::stdin().read_to_end(&mut input)?;',
      '    io::stdout().write_all(&input)',
      '}',
      '```',
      '',
      'Execution status: not run - the browser sandbox cannot invoke a rust toolchain.',
      '',
      'Copy the snippet into a rust environment to verify.',
      '',
      'Expected output after verification:',
      '```text',
      'hello',
      'world',
      '```',
      '',
      'How it works:',
      'The program performs the requested task and prints its result to standard output.',
      '',
      'How to test it yourself:',
      '1. Install the Rust toolchain from https://rustup.rs.',
      '2. Save the code above to a file named `main.rs`.',
      '3. Check that it compiles: `rustc main.rs -o main`.',
      "4. Run it: `printf 'hello\\nworld\\n' | ./main`.",
      '5. Compare the output with the expected output shown above.',
    ].join('\n'));
    assert.equal(answer.programExecution.runCommand, "printf 'hello\\nworld\\n' | ./main");
  });

  test('a task that reads no input keeps its plain run command', async () => {
    const answer = await solve('write me hello world program in Rust');
    assert.equal(answer.programExecution.runCommand, './main');
    assert.ok(!answer.content.includes('printf'), answer.content);
  });
});

describe('R1021-8: a framework-named request is answered in that framework (#723)', () => {
  test('Laravel is its own target; an uncatalogued framework falls back to its language', async () => {
    for (const [prompt, intent] of [
      ['напиши мне код на PHP Laravel', 'write_script_laravel'],
      ['write me PHP Laravel code', 'write_script_laravel'],
      ['PHP Laravel में कोड लिखें', 'write_script_laravel'],
      ['用 PHP Laravel 写代码', 'write_script_laravel'],
      ['write me PHP Symfony code', 'write_script_php'],
      ['write me some PHP code', 'write_script_php'],
      ['PHP में कोड लिखें', 'write_script_php'],
      ['用 PHP 写代码', 'write_script_php'],
      ['write me Ruby on Rails code', 'write_script_ruby'],
      ['напиши мне код на Python Django', 'write_script_python'],
    ]) {
      assert.equal((await solve(prompt)).intent, intent, prompt);
    }
  });
});

describe('R1021-31: asking for code is a coding request whatever the asking verb', () => {
  test('a named language is answered with its minimal script in every asking verb', async () => {
    for (const [prompt, intent] of [
      ['мне нужен код на пхп', 'write_script_php'],
      ['дай мне код на php', 'write_script_php'],
      ['мне нужен код на python', 'write_script_python'],
      ['I need PHP code', 'write_script_php'],
      ['I want PHP code', 'write_script_php'],
      ['give me python code', 'write_script_python'],
      ['मुझे php कोड चाहिए', 'write_script_php'],
      ['我需要 php 代码', 'write_script_php'],
      ['escribe código en Python', 'write_script_python'],
    ]) {
      assert.equal((await solve(prompt)).intent, intent, prompt);
    }
  });

  test('code naming no language reaches the honest dead end in english, russian, hindi, chinese and spanish', async () => {
    for (const prompt of [
      'мне нужен код', 'дай мне код', 'напиши мне код', 'мне нужна программа',
      'I need code', 'give me code', 'I want code', 'write me a program',
      'मुझे कोड चाहिए', '我需要代码', '给我代码', 'necesito código', 'escribe código', 'dame código',
    ]) {
      assert.equal((await solve(prompt)).intent, 'write_program_request_unspecified', prompt);
    }
  });

  test('the dead end is worded in the language it was asked in', async () => {
    assert.equal((await solve('I need code')).content,
      'I will not guess what to write. This reads as a program request, but it names neither what the program must do nor which programming language to write it in.\n\nSay what the program should do and which language it should be written in, and I will derive it.');
    assert.equal((await solve('мне нужен код')).content,
      'Я не угадываю, что писать. Это похоже на запрос программы, но в нём не названо ни что программа должна делать, ни на каком языке программирования её написать.\n\nСкажите, что должна делать программа и на каком языке, и я её выведу.');
  });

  test('an asking verb that names another subject is not a coding request', async () => {
    for (const prompt of [
      'I need information about Rust',
      'I need to find a python tutorial',
      'give me the code of this repository',
      'I need a code review',
      'дай мне код этого репозитория',
      'I need the code of this file',
      'give me a code example for sorting',
    ]) {
      const { intent } = await solve(prompt);
      assert.ok(!intent.startsWith('write_'), `${prompt} routed to ${intent}`);
    }
  });
});
