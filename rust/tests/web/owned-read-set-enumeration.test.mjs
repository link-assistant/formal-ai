import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { responseArgumentsForTool } from '../../../js/agentic/crate/protocol_responses.mjs';
import { ownedReadPaths } from '../../../js/agentic/file_read/ownership.mjs';
await installNodeHost(new WorkerHost());
test('original strict advertised shared schemas preserve complete arguments', async () => {
  for (const [prompt, name, properties] of [['find files matching **/*.rs', 'glob', {
    pattern: {
      type: 'string'
    },
    path: {
      type: 'string'
    }
  }], ['list files in this folder', 'list_directory', {
    path: {
      type: 'string'
    }
  }], ['read all of these files: Cargo.toml and README.md', 'read_many_files', {
    file_paths: {
      type: 'array',
      items: {
        type: 'string'
      },
      minItems: 1
    }
  }]]) {
    const schema = {
      type: 'object',
      properties,
      required: Object.keys(properties),
      additionalProperties: false
    };
    const tools = [{
      type: 'function',
      function: {
        name,
        parameters: schema
      }
    }, {
      type: 'function',
      function: {
        name: 'web_search',
        parameters: {
          type: 'object',
          properties: {
            query: {
              type: 'string'
            }
          },
          required: ['query']
        }
      }
    }];
    const plan = await planChatStep([{
      role: 'user',
      content: prompt
    }], tools.map(t => t.function.name));
    assert.equal(plan?.kind, 'tool_calls', prompt);
    assert.equal(plan.calls[0].tool, name);
    const args = JSON.parse(responseArgumentsForTool(tools, name, plan.calls[0].arguments, prompt, null));
    assert.deepEqual(Object.keys(args).sort(), Object.keys(properties).sort());
    for (const [key, decl] of Object.entries(properties)) assert.ok(decl.type === 'array' ? Array.isArray(args[key]) && args[key].length >= decl.minItems : typeof args[key] === 'string');
    if (name === 'read_many_files') assert.deepEqual(args.file_paths, ['Cargo.toml', 'README.md']);
  }
});
test('original navigation planning delegation and edit aliases route by intent', async () => {
  for (const [prompt, aliases] of [['find all Rust files matching **/*.rs', ['glob', 'Glob']], ['show the contents of the src directory', ['list_directory', 'list', 'ls', 'LS']], ['create a todo list for this change', ['todo_write', 'todowrite', 'update_plan', 'create_todo_list', 'update_todo_list', 'TodoWrite']], ['delegate this investigation to a subagent', ['task', 'multi_agent_v1', 'invoke_agent', 'Task']], ['read all of these files: Cargo.toml and README.md', ['read_many_files']], ['replace alpha with beta in a.txt and b.txt', ['multi_edit', 'MultiEdit']]]) for (const alias of aliases) {
    const plan = await planChatStep([{
      role: 'user',
      content: prompt
    }], [alias]);
    assert.equal(plan?.kind, 'tool_calls', prompt + alias);
    assert.equal(plan.calls.length, 1);
    assert.equal(plan.calls[0].tool, alias);
  }
});
test('set enumeration modifiers retain complete operands and refuse unknown residual obligations', () => {
  for (const prompt of ['Read all of these files: a.txt and b.txt', 'Read every file: a.txt and b.txt', 'Read each file: a.txt and b.txt']) assert.deepEqual(ownedReadPaths(prompt, 'file_read_action_cue'), ['a.txt', 'b.txt']);
  for (const prompt of ['Read all of these files: a.txt and run malicious', 'Read all of these files: a.txt and unknown words b.txt', 'Read all of these files: a.txt then deploy', 'Do not read a.txt. Read all of these files: a.txt and b.txt']) assert.deepEqual(ownedReadPaths(prompt, 'file_read_action_cue'), [], prompt);
});

test('set modifiers with unknown operand gaps refuse without throwing', () => {
  for (const prompt of [
    'read all of these unowned words files: a.txt',
    'read all unknown files: a.txt',
    'read every invented file: a.txt'
  ]) assert.deepEqual(ownedReadPaths(prompt, 'file_read_action_cue'), []);
});
