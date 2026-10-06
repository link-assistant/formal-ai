// Core agentic-planner modules ported to js/agentic/ (shell routing, capability
// routing, web-search intent, mutating recipes, language detection, and the
// small request recognisers), checked against the Rust expectations.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let shell;
let mutating;
let router;
let webSearch;
let language;
let restart;
let envelope;
let positional;
let audit;
let intent;
let rules;
let scope;
let fallback;
let skills;

before(async () => {
  await installNodeHost(new WorkerHost());
  shell = await import('../../../js/agentic/shell_command.mjs');
  mutating = await import('../../../js/agentic/mutating_action.mjs');
  router = await import('../../../js/agentic/capability_router.mjs');
  webSearch = await import('../../../js/agentic/crate/solver_handlers_web_search.mjs');
  language = await import('../../../js/agentic/crate/language.mjs');
  restart = await import('../../../js/agentic/restart_feedback.mjs');
  envelope = await import('../../../js/agentic/harness_envelope.mjs');
  positional = await import('../../../js/agentic/positional_edit.mjs');
  audit = await import('../../../js/agentic/statement_audit.mjs');
  intent = await import('../../../js/agentic/intent_router.mjs');
  rules = await import('../../../js/agentic/crate/rule_interpreter.mjs');
  scope = await import('../../../js/agentic/crate/tool_scope.mjs');
  fallback = await import('../../../js/agentic/shell_file_fallback.mjs');
  skills = await import('../../../js/agentic/crate/skill_compiler.mjs');
});

const user = (content) => ({ role: 'user', content });

/** Drive a mutating recipe the way a client would, every step succeeding. */
function driveRecipe(command, prompt) {
  const messages = [user(prompt)];
  const commands = [];
  for (let step = 0; step <= 12; step += 1) {
    const plan = mutating.planStep(command, messages, ['exec_command'], prompt);
    if (!plan || plan.kind !== 'tool_calls') return { commands, plan };
    const call = plan.calls[0];
    const id = `call_${messages.length}`;
    const ran = JSON.parse(call.arguments).command;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: `Command: ${ran}\nOutput: (empty)\nExit Code: 0` });
    commands.push(ran);
  }
  return { commands, plan: null };
}

describe('issue #749 shell routing', () => {
  test('the chat table resolves to the expected commands', () => {
    for (const [prompt, expected] of [
      ['show current directory', 'pwd'],
      ['show environment variables', 'env'],
      ['copy a.txt to b.txt', 'cp a.txt b.txt'],
      ['what changed in git', 'git diff'],
      ['search for TODO in the code', "rg --fixed-strings -- 'TODO' ."],
      ['delete the file old.txt', 'rm old.txt'],
      ['удали файл old.txt', 'rm old.txt'],
      ['फ़ाइल old.txt हटाओ', 'rm old.txt'],
      ['删除文件 old.txt', 'rm old.txt'],
      ['execute date', 'date'],
      ['run bash: echo hi there', 'echo hi there'],
      ['"execute echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT"', 'echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT'],
      ['execute sort names.txt', 'sort names.txt'],
      ["bash -c 'printf hello'", "bash -c 'printf hello'"],
      ['powershell Get-ChildItem', 'powershell Get-ChildItem'],
    ]) {
      assert.equal(shell.shellCommandForTask(prompt), expected, prompt);
    }
  });

  test('explicit shell forms pass the complete command through', () => {
    for (const [prompt, expected] of [
      ['run bash: echo hi', 'echo hi'],
      ['run the command wc -l foo.txt', 'wc -l foo.txt'],
      ['execute ln -s a b', 'ln -s a b'],
      ['execute arbitrary-tool --alpha beta', 'arbitrary-tool --alpha beta'],
      ["bash -c 'ls -la'", "bash -c 'ls -la'"],
      ['powershell Get-Location', 'powershell Get-Location'],
      ["pwsh -c 'Get-ChildItem'", "pwsh -c 'Get-ChildItem'"],
      ['execute echo show current directory', 'echo show current directory'],
    ]) {
      assert.equal(shell.shellCommandForTask(prompt), expected, prompt);
    }
  });

  test('natural intents cover file, vcs and search tasks', () => {
    for (const [prompt, expected] of [
      ['create an empty file called note.txt', 'touch note.txt'],
      ['rename old.txt to new.txt', 'mv old.txt new.txt'],
      ['remove the directory build', 'rmdir build'],
      ['create a symbolic link from a to b', 'ln -s a b'],
      ['show metadata for Cargo.toml', 'stat Cargo.toml'],
      ['show git log', 'git log'],
      ['commit my changes', 'git commit'],
      ['скопируй a.txt в b.txt', 'cp a.txt b.txt'],
    ]) {
      assert.equal(shell.shellCommandForTask(prompt), expected, prompt);
    }
  });

  test('local search is shell-routed in every language', () => {
    for (const prompt of ['find FIXME in the repository', 'grep for error in local files', 'найди TODO в коде',
      'कोड में TODO खोजें', '在代码中搜索 TODO']) {
      const command = shell.shellCommandForTask(prompt);
      assert.ok(command && command.startsWith("rg --fixed-strings -- '") && command.endsWith("' ."), `${prompt}: ${command}`);
    }
  });

  test('passthrough prefixes have language parity', () => {
    for (const [prompt, expected] of [
      ['run env', 'env'],
      ['run shell: wc -l file.txt', 'wc -l file.txt'],
      ['run powershell: Get-Date', 'Get-Date'],
      ['выполни date', 'date'],
      ['запусти команду wc -l file.txt', 'wc -l file.txt'],
      ['запусти bash: echo привет мир', 'echo привет мир'],
      ['चलाओ date', 'date'],
      ['कमांड चलाओ stat file.txt', 'stat file.txt'],
      ['bash चलाओ: echo नमस्ते', 'echo नमस्ते'],
      ['执行 date', 'date'],
      ['执行命令 stat file.txt', 'stat file.txt'],
      ['运行 bash: echo 你好', 'echo 你好'],
    ]) {
      assert.equal(shell.shellCommandForTask(prompt), expected, prompt);
    }
  });

  test('the full command taxonomy passes through `execute`', () => {
    for (const command of ['pwd', 'cd workspace', 'ls -la', 'uname -a', 'which cargo', 'chmod 644 note.txt',
      'cut -d: -f1 note.txt', "awk '{print $1}' note.txt", "find . -name '*.rs'", 'git commit -m test',
      'go test ./...', 'kill 1234']) {
      assert.equal(shell.shellCommandForTask(`execute ${command}`), command, command);
    }
  });
});

describe('verified mutating recipes (issue #944)', () => {
  test('mv and cp expand into checked recipes; read-only commands do not', () => {
    assert.deepEqual(mutating.verifiedRecipe('mv a.txt b/c.txt'), [
      'test -e a.txt', 'test ! -e b/c.txt', 'mkdir -p -- b', 'mv a.txt b/c.txt', 'test -e b/c.txt', 'test ! -e a.txt',
    ]);
    assert.deepEqual(mutating.verifiedRecipe('cp a.txt b.txt'), [
      'test -e a.txt', 'test ! -e b.txt', 'mkdir -p -- .', 'cp a.txt b.txt', 'test -e b.txt', 'test -e a.txt',
    ]);
    assert.equal(mutating.verifiedRecipe('ls'), null);
    assert.equal(mutating.verifiedRecipe('rm old.txt'), null);
  });

  test('the recipe runs step by step and reports the checks it rests on', () => {
    const { commands, plan } = driveRecipe('cp a.txt b.txt', 'copy a.txt to b.txt');
    assert.equal(commands.length, 6);
    assert.equal(commands[3], 'cp a.txt b.txt');
    assert.equal(plan.kind, 'final');
    assert.ok(plan.answer.includes('`test -e b.txt`'), plan.answer);
  });

  test('a failing precondition blocks the action', () => {
    const prompt = 'copy a.txt to b.txt';
    const call = { id: 'call_1', type: 'function', function: { name: 'exec_command', arguments: '{"command":"test -e a.txt"}' } };
    const messages = [user(prompt), { role: 'assistant', content: '', tool_calls: [call] },
      { role: 'tool', tool_call_id: 'call_1', name: 'exec_command', content: 'Command: test -e a.txt\nOutput: (empty)\nExit Code: 1' }];
    const plan = mutating.planStep('cp a.txt b.txt', messages, ['exec_command'], prompt);
    assert.equal(plan.kind, 'final');
    assert.ok(plan.answer.includes('test -e a.txt'), plan.answer);
  });
});

describe('language detection', () => {
  test('detects the supported languages', () => {
    assert.equal(language.detect('please show the files in this folder'), 'en');
    assert.equal(language.detect('покажи файлы в этой папке'), 'ru');
    assert.equal(language.detect('इस फ़ोल्डर में फ़ाइलें दिखाओ'), 'hi');
    assert.equal(language.detect('显示这个文件夹中的文件'), 'zh');
    assert.equal(language.detect('¿cuáles son los archivos en esta carpeta?'), 'es');
  });
});

describe('capability router', () => {
  test('classifies and selects advertised tools', () => {
    assert.equal(router.classifyTool('exec_command'), 'run');
    assert.equal(router.classifyTool('browser_click'), null);
    assert.equal(router.classifyTool('tool_search'), null);
    assert.equal(router.toolFor(['run_shell_command', 'web_search'], 'search'), 'web_search');
    assert.equal(router.toolFor(['web_search', 'mcp__brave__web_search'], 'search'), 'mcp__brave__web_search');
    assert.ok(router.isHostedResearchTool('Web_Search'));
  });

  test('reported folder variants route to one ls contract (issue #758)', () => {
    for (const prompt of ['list files in this folder', 'покажи файлы в этой папке', 'список файлов', 'какие файлы в этой папке']) {
      const plan = router.planRoutedCapabilityStep(prompt, [user(prompt)], ['exec_command'], router.RoutingStage.NamedOrLocal);
      assert.deepEqual(plan, { kind: 'tool_calls', calls: [{ tool: 'exec_command', arguments: '{"command":"ls"}' }] }, prompt);
    }
  });

  test('local search aliases never become web search calls', () => {
    for (const alias of ['grep', 'grep_search', 'search', 'codesearch', 'Grep']) {
      const prompt = 'search the repository for CAPABILITY_SENTINEL';
      const plan = router.planRoutedCapabilityStep(prompt, [user(prompt)], [alias], router.RoutingStage.NamedOrLocal);
      assert.equal(plan.calls[0].tool, alias);
      assert.equal(JSON.parse(plan.calls[0].arguments).pattern, 'CAPABILITY_SENTINEL');
    }
  });

  test('glob patterns route to the glob tool', () => {
    const prompt = 'find files matching **/*.rs';
    const plan = router.planRoutedCapabilityStep(prompt, [user(prompt)], ['glob'], router.RoutingStage.NamedOrLocal);
    assert.equal(plan.calls[0].arguments, '{"path":".","pattern":"**/*.rs"}');
  });
});

describe('web-search intent', () => {
  test('extracts the query and its kind', () => {
    for (const [prompt, query, kind] of [
      ['search the web for rust async traits', 'rust async traits', 'explicit_prefix'],
      ['google rust borrow checker', 'rust borrow checker', 'semantic_action'],
      ['найди в интернете цены на нефть', 'цены на нефть', 'explicit_prefix'],
      ['financial records for boeing', 'financial records for boeing', 'records_information_request'],
    ]) {
      assert.deepEqual(webSearch.extractWebSearchRequest(prompt, prompt.toLowerCase()), { query, kind }, prompt);
    }
    for (const prompt of ['delete the file old.txt', 'show current directory', 'what changed in git']) {
      assert.equal(webSearch.webSearchQueryFor(prompt), null, prompt);
    }
  });

  test('cleans queries and rejects bare URLs', () => {
    assert.equal(webSearch.cleanSearchQuery(' "(hello  world)"... '), 'hello world)"');
    assert.equal(webSearch.normalizeUrlCandidate('example.com/x'), 'https://example.com/x');
    assert.equal(webSearch.normalizeUrlCandidate('notes.txt'), null);
  });

  test('plans a web search with the advertised tool', () => {
    const prompt = 'search the web for rust async traits';
    const plan = intent.planWebSearchStep(prompt, [user(prompt)], ['run_shell_command', 'web_search']);
    assert.deepEqual(plan, { kind: 'tool_calls', calls: [{ tool: 'web_search', arguments: '{"query":"rust async traits"}' }] });
    const discovery = intent.planWebSearchStep(prompt, [user(prompt)], ['tool_search']);
    assert.equal(discovery.calls[0].arguments, '{"max_results":5,"query":"web search"}');
  });
});

describe('restart feedback', () => {
  test('only the marked fenced listing is consumed', () => {
    assert.deepEqual(restart.changedPaths('⚠️ UNCOMMITTED CHANGES DETECTED\n```text\n?? Main.kt\n M README.md\n```'), ['Main.kt', 'README.md']);
    assert.equal(restart.changedPaths('UNCOMMITTED CHANGES DETECTED\n```\n?? ../outside\n```'), null);
    assert.equal(restart.changedPaths('?? Main.kt'), null);
  });

  test('unreadable or requested feedback blocks readiness', () => {
    assert.equal(restart.feedbackNeedsChanges('{"comments":[],"reviews":[]}'), false);
    assert.equal(restart.feedbackNeedsChanges('{"comments":[{"body":"Fix the output"}],"reviews":[]}'), true);
    assert.equal(restart.feedbackNeedsChanges('{"comments":[],"reviews":[{"state":"CHANGES_REQUESTED"}]}'), true);
    assert.equal(restart.feedbackNeedsChanges('{}'), true);
    assert.equal(restart.feedbackNeedsChanges('authentication failed'), true);
  });
});

describe('small recognisers', () => {
  test('summarize envelopes answer with the leading sentence', () => {
    assert.equal(envelope.summarizeRequest('The following is the text to summarize: <text>\nFix the bug in a.rs. Then test.\n</text>'), 'Fix the bug in a.rs.');
    assert.equal(envelope.summarizeRequest('Fix the bug'), null);
  });

  test('positional inserts compose an anchored edit', () => {
    assert.deepEqual(positional.composePositionalInsert('Add a line "b" directly after the line "a" in notes.txt'), ['notes.txt', 'a', 'a\nb']);
    assert.equal(positional.literalText(': "x\\ny"'), 'x\ny');
  });

  test('statement audits choose the evidence command only when named', () => {
    assert.ok(audit.isStatementAuditTask('Audit the repository requirements with probabilities'));
    assert.ok(!audit.isStatementAuditTask('list files'));
    assert.ok(audit.commandFor('use evidence.json').includes('--evidence evidence.json'));
    assert.ok(!audit.commandFor('audit').includes('--evidence'));
  });

  test('handler rules recognise continuation cues', () => {
    assert.ok(rules.handlerMatches('agentic_continuation', 'continue'));
    assert.ok(rules.handlerMatches('agentic_continuation', 'продолжай'));
    assert.ok(!rules.handlerMatches('agentic_continuation', 'continue the migration in src/queue.rs'));
  });

  test('identity arguments are grounded only by the request', () => {
    assert.ok(scope.isIdentityArgument('owner'));
    assert.ok(!scope.isIdentityArgument('query'));
  });

  test('a shell-less client appends through read and write tools', () => {
    const command = "printf '%s\\n' 'hello' >> notes.txt";
    const prompt = 'append hello to notes.txt';
    const read = fallback.planStep(prompt, [user(prompt)], ['read_file', 'write_file'], command);
    assert.equal(read.calls[0].tool, 'read_file');
    assert.equal(fallback.planStep(prompt, [user(prompt)], ['read_file', 'write_file', 'run_shell_command'], command), null);
    const discover = fallback.planStep(prompt, [user(prompt)], ['tool_search'], command);
    assert.equal(discover.calls[0].arguments, '{"max_results":2,"query":"select:write_file,run_shell_command"}');
  });

  test('skill descriptions need a when/then pair with code spans', () => {
    assert.equal(skills.looksLikeSkillDescription('list files'), false);
  });
});
