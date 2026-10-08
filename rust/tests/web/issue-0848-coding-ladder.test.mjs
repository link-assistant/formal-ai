// Issue #848: the coding-task ladder, pinned on the JavaScript root.
//
// rust/tests/unit/agentic-coding/issue_848_coding_ladder.rs inspects the bytes the offline
// Agent CLI loop writes and reads back. The same dialogues are driven here
// through the agentic planner twin (js/agentic/planner.mjs and its code_task,
// structured_edit, workspace_change and shell_command modules): a passing
// answer is not evidence of a change, so every test asserts the exact bytes
// written, the exact command that observes them, and that the loop finishes.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planner;

before(async () => {
  await installNodeHost(new WorkerHost());
  planner = await import('../../../js/agentic/planner.mjs');
});

const user = (content) => ({ role: 'user', content });
const sha256 = (text) => createHash('sha256').update(text).digest('hex');

/** The single call a planned step makes. */
function onlyCall(plan) {
  assert.equal(plan && plan.kind, 'tool_calls', JSON.stringify(plan));
  assert.equal(plan.calls.length, 1);
  return plan.calls[0];
}

/** Record `call` as made and answered with `result`; `argumentsOverride` is what the client really sent. */
function pushResult(messages, id, call, result, argumentsOverride) {
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: argumentsOverride ?? call.arguments } }] });
  messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: result });
}

async function finalAnswer(messages, tools) {
  const plan = await planner.planChatStep(messages, tools);
  assert.equal(plan.kind, 'final', JSON.stringify(plan));
  return plan.answer;
}

/**
 * Drive a source-creation request until it writes `path`, reading nothing
 * along the way, and return the written bytes and the observing command.
 */
async function generatedSource(task, path) {
  const tools = ['read_file', 'write_file', 'run_command'];
  const messages = [user(task)];
  const write = onlyCall(await planner.planChatStep(messages, tools));
  assert.equal(write.tool, 'write_file', task);
  const written = JSON.parse(write.arguments);
  assert.equal(written.path, path, task);
  pushResult(messages, 'write', write, 'created');
  const verify = onlyCall(await planner.planChatStep(messages, tools));
  pushResult(messages, 'verify', verify, written.content);
  return { content: written.content, verify: JSON.parse(verify.arguments).command, answer: await finalAnswer(messages, tools) };
}

describe('R848-2/R848-7: source creation renders executable bytes and observes them', () => {
  test('function, constant and test requests write exact source', async () => {
    for (const [task, path, expected] of [
      ['Create a new file rust/src/si_units.rs in this repository containing a single public Rust function millimetres_to_metres that takes an f64 and returns it divided by 1000.0.',
        'rust/src/si_units.rs', 'pub fn millimetres_to_metres(value: f64) -> f64 {\n    value / 1000.0\n}\n'],
      ['Create a new file rust/src/ladder_probe.rs in this repository containing a single public Rust constant LADDER_PROBE of type &str with the value probe.',
        'rust/src/ladder_probe.rs', 'pub const LADDER_PROBE: &str = "probe";\n'],
      ['Create a new file rust/tests/unit/ladder_probe.rs in this repository containing a single Rust test named ladder_probe_runs that asserts 1 equals 1.',
        'rust/tests/unit/ladder_probe.rs', '#[test]\nfn ladder_probe_runs() {\n    assert_eq!(1, 1);\n}\n'],
      ['Create a Rust test file rust/tests/integration/ladder_probe.rs in this repository containing one Rust test named ladder_integration_probe asserting 2 plus 2 equals 4.',
        'rust/tests/integration/ladder_probe.rs', '#[test]\nfn ladder_integration_probe() {\n    assert_eq!(2 + 2, 4);\n}\n'],
    ]) {
      const outcome = await generatedSource(task, path);
      assert.equal(outcome.content, expected, task);
      assert.equal(outcome.verify, `cat ${path}`, task);
    }
  });

  test('the same request in russian, chinese and hindi generates the same shape of source', async () => {
    for (const [task, path, name] of [
      ['Создай файл rust/src/ladder_ru.rs с одной публичной функцией на Rust с именем ladder_ru, которая возвращает число 1.', 'rust/src/ladder_ru.rs', 'ladder_ru'],
      ['在这个仓库中创建文件 rust/src/ladder_zh.rs，其中包含一个名为 ladder_zh 的公共 Rust 函数，返回数字 1。', 'rust/src/ladder_zh.rs', 'ladder_zh'],
      ['इस रिपॉजिटरी में rust/src/ladder_hi.rs फ़ाइल बनाएँ जिसमें ladder_hi नाम का एक सार्वजनिक Rust फ़ंक्शन हो जो 1 लौटाता है।', 'rust/src/ladder_hi.rs', 'ladder_hi'],
    ]) {
      assert.equal((await generatedSource(task, path)).content, `pub fn ${name}() -> i64 {\n    1\n}\n`, task);
    }
  });
});

describe('R848-5: repository search uses the named code subject', () => {
  test('seven subject shapes each become one focused grep', async () => {
    for (const [task, expected] of [
      ['In this repository, find where the string example.org is emitted as evidence and tell me the file name.', 'example.org'],
      ['In this repository, find where the function stable_id is defined and tell me the file name.', 'stable_id'],
      ['In this repository, find where the function dialog_id is defined and tell me the file name.', 'dialog_id'],
      ['In this repository, find where the excluded_folders array is defined and tell me the file name.', 'excluded_folders'],
      ['In this repository, find where reciprocalRankFusion is implemented in JavaScript and tell me the file name.', 'reciprocalRankFusion'],
      ['In this repository, find where the Context struct for world models is defined and tell me the file name.', 'Context'],
      ['In this repository, find where the summarize function is defined and tell me the file name.', 'summarize'],
    ]) {
      const call = onlyCall(await planner.planChatStep([user(task)], ['grep']));
      assert.deepEqual([JSON.parse(call.arguments).query, JSON.parse(call.arguments).pattern], [expected, expected], task);
    }
  });
});

describe('R848-6: a structured collection edit transforms the workspace bytes', () => {
  test('read, transform, write and observe the excluded_folders array', async () => {
    const before = 'fn is_excluded(file_path: &str) -> bool {\n    let excluded_folders = ["changelog.d/", "docs/", "examples/"];\n    excluded_folders.iter().any(|folder| file_path.starts_with(folder))\n}\n';
    const after = 'fn is_excluded(file_path: &str) -> bool {\n    let excluded_folders = ["changelog.d/", "docs/", "examples/", "dev/log/"];\n    excluded_folders.iter().any(|folder| file_path.starts_with(folder))\n}\n';
    const tools = ['read_file', 'write_file', 'run_command'];
    const messages = [user('In this repository, in scripts/detect-code-changes.rs, add "dev/log/" to the excluded_folders array.')];
    const read = onlyCall(await planner.planChatStep(messages, tools));
    assert.deepEqual([read.tool, JSON.parse(read.arguments).path], ['read_file', 'scripts/detect-code-changes.rs']);
    pushResult(messages, 'read_1', read, before);
    const write = onlyCall(await planner.planChatStep(messages, tools));
    assert.deepEqual([write.tool, JSON.parse(write.arguments).path, JSON.parse(write.arguments).content],
      ['write_file', 'scripts/detect-code-changes.rs', after]);
    pushResult(messages, 'write_1', write, 'updated');
    const verify = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(verify.arguments).command, 'cat scripts/detect-code-changes.rs');
    pushResult(messages, 'run_1', verify, after);
    assert.ok((await finalAnswer(messages, tools)).includes('observed'));
  });
});

describe('R848-10: symbol refactors and composite module requests terminate verified', () => {
  const TASK = 'In this repository, in rust/src/web_search_core.rs, rename the constant WEB_SEARCH_RRF_K to WEB_SEARCH_FUSION_K everywhere it appears in this file.';
  const BEFORE = 'pub const WEB_SEARCH_RRF_K: f64 = 60.0;\npub fn score(rank: f64) -> f64 { WEB_SEARCH_RRF_K + rank }\n';
  const AFTER = 'pub const WEB_SEARCH_FUSION_K: f64 = 60.0;\npub fn score(rank: f64) -> f64 { WEB_SEARCH_FUSION_K + rank }\n';

  test('an identifier rename is a grounded rewrite, not a file move', async () => {
    const tools = ['read_file', 'write_file', 'run_command'];
    const messages = [user(TASK)];
    const read = onlyCall(await planner.planChatStep(messages, tools));
    assert.deepEqual([read.tool, JSON.parse(read.arguments).path], ['read_file', 'rust/src/web_search_core.rs']);
    pushResult(messages, 'read_rename', read, BEFORE);
    const write = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(write.arguments).content, AFTER);
    pushResult(messages, 'write_rename', write, 'updated');
    const verify = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(verify.arguments).command, 'cat rust/src/web_search_core.rs');
    pushResult(messages, 'verify_rename', verify, AFTER);
    assert.ok((await finalAnswer(messages, tools)).includes('observed'));
  });

  test('an Agent CLI absolute read path gets one bounded whole-word rewrite and a digest check', async () => {
    const tools = ['read', 'edit', 'write', 'bash'];
    const messages = [user(TASK)];
    const read = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(read.tool, 'read');
    pushResult(messages, 'agent_read', read,
      '<file>\n00001| pub const WEB_SEARCH_RRF_K: f64 = 60.0;\n00002| pub fn score(rank: f64) -> f64 { WEB_SEARCH_RRF_K + rank }\n00003| \n\n(End of file - total 3 lines)\n</file>',
      JSON.stringify({ filePath: '/tmp/agent-workspace/rust/src/web_search_core.rs' }));
    const rewrite = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(rewrite.tool, 'bash');
    assert.equal(JSON.parse(rewrite.arguments).command,
      "perl -pi -e 's/\\bWEB_SEARCH_RRF_K\\b/WEB_SEARCH_FUSION_K/g' -- rust/src/web_search_core.rs");
    pushResult(messages, 'agent_rewrite', rewrite, '');
    const verify = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(verify.arguments).command, 'sha256sum -- rust/src/web_search_core.rs');
    pushResult(messages, 'agent_verify', verify, `${sha256(AFTER)}  rust/src/web_search_core.rs\n`);
    assert.ok((await finalAnswer(messages, tools)).includes('observed'));
  });

  const COMPOSITE = 'Create rust/src/ladder_units.rs with a public function metres_to_kilometres dividing an f64 by 1000.0, and register the module in rust/src/lib.rs so it is part of the crate.';
  const MODULE = 'pub fn metres_to_kilometres(value: f64) -> f64 {\n    value / 1000.0\n}\n';
  const LIB_AFTER = 'pub mod language;\npub mod seed;\npub mod ladder_units;\n';

  test('a composite request writes the module, then registers it, observing both', async () => {
    const tools = ['read_file', 'write_file', 'run_command'];
    const messages = [user(COMPOSITE)];
    const create = onlyCall(await planner.planChatStep(messages, tools));
    assert.deepEqual([create.tool, JSON.parse(create.arguments).path, JSON.parse(create.arguments).content],
      ['write_file', 'rust/src/ladder_units.rs', MODULE]);
    pushResult(messages, 'write_module', create, 'created');
    const verifyModule = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(verifyModule.arguments).command, 'cat rust/src/ladder_units.rs');
    pushResult(messages, 'verify_module', verifyModule, MODULE);
    const readLib = onlyCall(await planner.planChatStep(messages, tools));
    assert.deepEqual([readLib.tool, JSON.parse(readLib.arguments).path], ['read_file', 'rust/src/lib.rs']);
    pushResult(messages, 'read_lib', readLib, 'pub mod language;\npub mod seed;\n');
    const writeLib = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(writeLib.arguments).content, LIB_AFTER);
    pushResult(messages, 'write_lib', writeLib, 'updated');
    const verifyLib = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(verifyLib.arguments).command, 'cat rust/src/lib.rs');
    pushResult(messages, 'verify_lib', verifyLib, LIB_AFTER);
    assert.ok((await finalAnswer(messages, tools)).includes('observed'));
  });

  test('an Agent CLI composite registers the module with a compact edit and a digest check', async () => {
    const tools = ['read', 'edit', 'write', 'bash'];
    const messages = [user(COMPOSITE)];
    const create = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(create.tool, 'write');
    pushResult(messages, 'agent_create_module', create, '');
    const verifyModule = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(verifyModule.tool, 'bash');
    pushResult(messages, 'agent_verify_module', verifyModule, MODULE);
    const readLib = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(readLib.tool, 'read');
    pushResult(messages, 'agent_read_lib', readLib,
      '<file>\n00001| pub mod language;\n00002| pub mod seed;\n00003| \n\n(End of file - total 3 lines)\n</file>',
      JSON.stringify({ filePath: '/tmp/agent-workspace/rust/src/lib.rs' }));
    const edit = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(edit.tool, 'edit', 'registration does not echo the whole lib.rs through write');
    assert.deepEqual([JSON.parse(edit.arguments).oldString, JSON.parse(edit.arguments).newString],
      ['pub mod seed;\n', 'pub mod seed;\npub mod ladder_units;\n']);
    pushResult(messages, 'agent_edit_lib', edit, '');
    const verifyLib = onlyCall(await planner.planChatStep(messages, tools));
    assert.equal(JSON.parse(verifyLib.arguments).command, 'sha256sum -- rust/src/lib.rs');
    pushResult(messages, 'agent_verify_lib', verifyLib, `${sha256(LIB_AFTER)}  rust/src/lib.rs\n`);
    assert.ok((await finalAnswer(messages, tools)).includes('observed'));
  });
});
