import { recipeSourceResult, recipeSourceUpdateCommand } from '../../../js/agentic/recipe_resume.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { chatCompletionFromPlan, responseFromPlan } from '../../../js/server/agentic.mjs';
import test from 'node:test';
import crypto from 'node:crypto';
import { Progress, qualifiedCommandAttempt } from '../../../js/agentic/progress.mjs';
import { recipeProgressAfterLatestUser, planSymbolicCommandReroute } from '../../../js/agentic/command_reroute.mjs';
import { recipeSourceCommand } from '../../../js/agentic/recipe_resume.mjs';
import { handleAnthropicMessages } from '../../../js/server/anthropic.mjs';
import { createResponse } from '../../../js/server/responses.mjs';
import { parseResponsesRequest } from '../../../js/server/responses-input.mjs';
import { createMemory } from '../../../js/server/memory.mjs';
import { feedbackNeedsChanges } from '../../../js/agentic/restart_feedback.mjs';
import { recipeDeliveryReference } from '../../../js/agentic/recipe_resume.mjs';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { createChatCompletion } from '../../../js/server/openai.mjs';
import { parseChatRequest } from '../../../js/server/chat-request.mjs';
import { ensureNodeHost } from '../../../js/server/node-host-install.mjs';
import { boundedCommand } from '../../../js/server/bounded-command.mjs';
import { invalidArguments } from '../../../js/agentic/crate/tool_call_validation.mjs';
const originalTask = fs.readFileSync(new URL('../fixtures/issue-1364-claude-restart.txt', import.meta.url), 'utf8');
const issues = JSON.parse(fs.readFileSync(new URL('../fixtures/issue-1364-hello-world-issues.json', import.meta.url), 'utf8'));
const historicalSources = JSON.parse(fs.readFileSync(new URL('../fixtures/issue-1364-original-sources.json', import.meta.url), 'utf8'));
const logicalRoot = '/tmp/gh-issue-solver-1792505721477';
function relativeClientPath(value) {
  if (path.isAbsolute(value)) {
    assert(value.startsWith(logicalRoot + '/'));
    value = value.slice(logicalRoot.length + 1);
  }
  assert(value !== '' && !path.isAbsolute(value) && !value.split('/').some(part => part === '..' || part === ''));
  return value;
}
function decodeAddPatch(input) {
  const lines = input.trimEnd().split('\n');
  assert.equal(lines.shift(), '*** Begin Patch');
  assert.equal(lines.pop(), '*** End Patch');
  const header = lines.shift();
  assert(header.startsWith('*** Add File: '));
  assert(lines.every(line => line.startsWith('+')));
  return {
    path: header.slice('*** Add File: '.length),
    content: lines.map(line => line.slice(1)).join('\n') + '\n'
  };
}
function clientTools(protocol) {
  const commandKey = protocol === 'codex' ? 'cmd' : 'command';
  const run = {
    type: 'object',
    properties: {
      [commandKey]: {
        type: 'string'
      }
    },
    required: [commandKey],
    additionalProperties: false
  };
  const source = {
    type: 'object',
    properties: {
      file_path: {
        type: 'string',
        description: 'Absolute client workspace path'
      },
      content: {
        type: 'string'
      }
    },
    required: ['file_path', 'content'],
    additionalProperties: false
  };
  const read = {
    type: 'object',
    properties: {
      file_path: {
        type: 'string',
        description: 'Absolute client workspace path'
      }
    },
    required: ['file_path'],
    additionalProperties: false
  };
  if (protocol === 'claude') return [{
    name: 'Bash',
    input_schema: run
  }, {
    name: 'Write',
    input_schema: source
  }, {
    name: 'Read',
    input_schema: read
  }];
  if (protocol === 'codex') return [{
    type: 'function',
    name: 'exec_command',
    parameters: run
  }, {
    type: 'custom',
    name: 'apply_patch',
    description: 'Apply a source patch in the client workspace'
  }];
  return [{
    type: 'function',
    function: {
      name: 'bash',
      parameters: run
    }
  }, {
    type: 'function',
    function: {
      name: 'write',
      parameters: source
    }
  }, {
    type: 'function',
    function: {
      name: 'read',
      parameters: read
    }
  }];
}
async function nextClient(context, history, tools, protocol) {
  if (protocol === 'claude') {
    const response = await handleAnthropicMessages(context, {
      body: JSON.stringify({
        model: 'formal-ai',
        messages: history,
        tools,
        max_tokens: 4096
      })
    });
    assert.equal(response.statusCode, 200, response.body);
    const message = JSON.parse(response.body);
    return {
      final: message.content.filter(block => block.type === 'text').map(block => block.text).join('\n'),
      calls: message.content.filter(block => block.type === 'tool_use').map(block => ({
        id: block.id,
        name: block.name,
        args: block.input
      })),
      wire: message
    };
  }
  if (protocol === 'codex') {
    const response = await createResponse(context, parseResponsesRequest(JSON.stringify({
      model: 'formal-ai',
      input: history,
      tools
    })));
    return {
      final: response.output.filter(item => item.type === 'message').flatMap(item => item.content).map(item => item.text ?? '').join('\n'),
      calls: response.output.filter(item => ['function_call', 'custom_tool_call'].includes(item.type)).map(item => ({
        id: item.call_id,
        name: item.name,
        args: item.type === 'custom_tool_call' ? decodeAddPatch(item.input) : JSON.parse(item.arguments),
        custom: item.type === 'custom_tool_call'
      })),
      wire: response.output
    };
  }
  const response = await createChatCompletion(context, parseChatRequest(JSON.stringify({
    model: 'formal-ai',
    messages: history,
    tools
  })));
  const message = response.choices[0].message;
  return {
    final: message.content,
    calls: (message.tool_calls ?? []).map(call => ({
      id: call.id,
      name: call.function.name,
      args: JSON.parse(call.function.arguments)
    })),
    wire: message
  };
}
function appendClient(history, next, call, result, protocol) {
  const output = JSON.stringify({
    exit_code: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
    complete: result.cleanup_complete
  });
  if (protocol === 'claude') history.push({
    role: 'assistant',
    content: next.wire.content
  }, {
    role: 'user',
    content: [{
      type: 'tool_result',
      tool_use_id: call.id,
      content: output,
      is_error: result.status !== 0
    }]
  });else if (protocol === 'codex') history.push(...next.wire, {
    type: call.custom ? 'custom_tool_call_output' : 'function_call_output',
    call_id: call.id,
    output
  });else history.push(next.wire, {
    role: 'tool',
    name: call.name,
    tool_call_id: call.id,
    content: output
  });
}
async function runFixture(configuration) {
  const issue = issues[configuration.index];
  const receipt = [];
  const reference = issue.html_url.replace('/issues/1', '/pull/2');
  const task = configuration.protocol === 'claude' ? originalTask : 'Issue to solve: ' + issue.html_url + '\nYour prepared Pull Request: ' + reference + '\nYour prepared working directory: ' + logicalRoot + '\nImplement every numbered requirement, preserve the existing branch, and continue until verified. Complete blocked local verification through the requested workflow honestly.';
  async function execute(cwd, file, argumentsValue, environment) {
    const result = await boundedCommand(file, argumentsValue, {
      cwd,
      env: environment,
      timeout: 15000,
      stdoutLimitBytes: 2097152,
      stderrLimitBytes: 1048576,
      encoding: 'utf8'
    });
    assert.equal(result.error, undefined, result.stderr);
    assert.equal(result.cleanup_complete, true);
    return result;
  }
  function definitions(run, write) {
    return [{
      type: 'function',
      function: {
        name: run,
        parameters: {
          type: 'object',
          properties: {
            command: {
              type: 'string'
            }
          },
          required: ['command'],
          additionalProperties: false
        }
      }
    }, {
      type: 'function',
      function: {
        name: write,
        parameters: {
          type: 'object',
          properties: {
            path: {
              type: 'string'
            },
            content: {
              type: 'string'
            }
          },
          required: ['path', 'content'],
          additionalProperties: false
        }
      }
    }];
  }
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-recipe-full-api-'));
  const base = fixture;
  try {
    const cwd = fixture + '/work';
    const origin = fixture + '/origin.git';
    const bin = fixture + '/bin';
    fs.mkdirSync(cwd);
    fs.mkdirSync(bin);
    const git = async (...args) => {
      const result = await execute(cwd, 'git', args);
      assert.equal(result.status, 0, result.stderr);
      return result.stdout.trim();
    };
    await git('init', '--bare', origin);
    await git('init', '-b', 'retained-work');
    await git('config', 'user.name', 'Hermetic fixture');
    await git('config', 'user.email', 'fixture@example.invalid');
    fs.writeFileSync(cwd + '/README.md', 'Initial fixture source.\n');
    const historical = historicalSources.sources.find(entry => entry.path === configuration.source);
    assert(historical);
    const sourceContext = {
      worker: new WorkerHost()
    };
    await ensureNodeHost(sourceContext);
    const generated = programContractAnswer(issue.body)?.execution_recipe?.source;
    assert.equal(typeof generated, 'string');
    const generatedLines = generated.split('\n');
    assert.equal(historical.instructionLines.length, historical.instructionLineCount);
    const beforeSource = [...historical.instructionLines, ...generatedLines.slice(historical.instructionLineCount)].join('\n');
    const sourceBytes = Buffer.from(beforeSource, 'utf8');
    const blob = crypto.createHash('sha1').update(Buffer.concat([Buffer.from('blob ' + sourceBytes.length + '\0'), sourceBytes])).digest('hex');
    assert.equal(blob, historical.gitBlob);
    assert.equal(sourceBytes.length, historical.bytes);
    assert.equal(crypto.createHash('sha256').update(sourceBytes).digest('hex'), historical.sha256);
    fs.writeFileSync(cwd + '/' + configuration.source, sourceBytes);
    const retainedArtifacts = [];
    const remember = (relative, data) => {
      const target = path.join(cwd, relative);
      fs.mkdirSync(path.dirname(target), {
        recursive: true
      });
      fs.writeFileSync(target, data);
      retainedArtifacts.push({
        path: relative,
        sha256: crypto.createHash('sha256').update(data).digest('hex')
      });
    };
    if (configuration.protocol !== 'agent') {
      const number = new URL(issue.html_url).pathname.split('/').at(-1);
      const evidence = 'dev/log/self-authored/issue-' + number + '/evidence/';
      remember(evidence + configuration.protocol + '-stream.jsonl', Buffer.alloc(0));
      // Synthetic bytes use the pinned producer grammar; original private session bytes are absent.
      remember(evidence + 'session-id.txt', Buffer.from('formal-ai session synthetic-historical-session\nformal-ai model formal-ai/0.3.55\n'));
    }
    if (configuration.protocol === 'codex') remember('main', Buffer.from([0x7f, 0x45, 0x4c, 0x46, 0x00, 0x53, 0x59, 0x4e, 0x54, 0x48, 0x45, 0x54, 0x49
    , 0x43]));
    await git('add', 'README.md', configuration.source, ...retainedArtifacts.map(entry => entry.path));
    await git('commit', '-qm', 'initial');
    await git('remote', 'add', 'origin', origin);
    await git('push', '-qu', 'origin', 'retained-work');
    const statePath = fixture + '/state.json';
    const state = {
      head: await git('rev-parse', 'HEAD'),
      branch: 'retained-work',
      isDraft: true,
      reference: issue.html_url.replace('/issues/1', '/pull/2'),
      issue,
      workflow: 'Run ' + configuration.source,
      calls: []
    };
    fs.writeFileSync(statePath, JSON.stringify(state));
    const fakeGh = `#!/usr/bin/env python3
import json, os, sys
p=os.environ['FA_RECIPE_FIXTURE_STATE']
s=json.load(open(p))
a=sys.argv[1:]
s['calls'].append(a)
if a[:2]==['issue','view']:
    assert a[2]==s['issue']['html_url']
    key=a[a.index('--json')+1]
    print(s['issue'][key])
elif a[:2]==['pr','view']:
    assert a[2]==s['reference']
    if '--json' in a:
        fields=a[a.index('--json')+1]
        if fields=='headRefOid,headRefName,isDraft,state,url':
            print(json.dumps(dict(headRefOid=s['head'],headRefName=s['branch'],isDraft=s['isDraft'],state='OPEN',url=s['reference'])))
        elif fields=='comments,reviews':
            print(json.dumps(dict(comments=[],reviews=[])))
        else:
            raise RuntimeError('unexpected fixture PR fields '+fields)
    else:
        print('No comments.')
elif a[:2]==['pr','checks']:
    assert a[2]==s['reference']
    assert not ('--watch' in a and '--json' in a)
    if '--json' in a:
        print(json.dumps([dict(bucket='pass',name='run',workflow=s['workflow'])]))
    else:
        assert '--watch' in a and '--fail-fast' in a
        print('All checks passed.')
elif a[:2]==['pr','edit']:
    assert a[2]==s['reference']
elif a[:2]==['pr','ready']:
    assert a[2]==s['reference'] and s['isDraft']
    s['isDraft']=False
else:
    raise RuntimeError('unexpected fixture gh arguments '+repr(a))
json.dump(s,open(p,'w'))
`;
    fs.writeFileSync(bin + '/gh', fakeGh, {
      mode: 0o700
    });
    for (const compiler of ['kotlinc', 'java', 'rustc', 'scalac', 'scala']) fs.writeFileSync(bin + '/' + compiler, '#!/usr/bin/env python3\nimport sys\nsys.stderr.write("fixture toolchain unavailable\\n")\nsys.exit(127)\n', {
      mode: 0o700
    });
    for (const networkTool of ['curl', 'wget']) fs.writeFileSync(bin + '/' + networkTool, '#!/usr/bin/env python3\nimport sys\nsys.stderr.write("unexpected external network operation in hermetic replay\\n")\nsys.exit(97)\n', {
      mode: 0o700
    });
    const environment = {
      ...process.env,
      PATH: bin + path.delimiter + process.env.PATH,
      FA_RECIPE_FIXTURE_STATE: statePath
    };
    const tools = clientTools(configuration.protocol);
    for (let run = 0; run < 2; run++) {
      const messages = [{
        role: 'user',
        content: task
      }];
      const context = {
        worker: new WorkerHost(),
        agentMode: true,
        memory: createMemory({
          ...process.env,
          FORMAL_AI_MEMORY_PATH: fixture + '/memory-' + run + '.lino'
        })
      };
      await ensureNodeHost(context);
      if (run === 0) {
        const own = issue.html_url.replace('/issues/1', '/pull/2');
        const handoff = reference => 'Your prepared Pull Request: ' + reference;
        assert.deepEqual(recipeDeliveryReference(task, issue.html_url), {
          kind: 'bound',
          reference: own
        });
        assert.equal(recipeDeliveryReference(handoff('https://github.com/foreign/repository/pull/2'), issue.html_url).kind, 'refused');
        assert.equal(recipeDeliveryReference(handoff(own) + '\n' + handoff(own.replace('/pull/2', '/pull/3')), issue.html_url).kind, 'refused');
        assert.equal(recipeDeliveryReference(handoff(own.replace('/pull/2', '/pull/0')), issue.html_url).kind, 'refused');
        assert.equal(recipeDeliveryReference('```text\n' + handoff(own) + '\n```', issue.html_url).kind, 'absent');
        assert.equal(recipeDeliveryReference('"' + handoff(own) + '"', issue.html_url).kind, 'absent');
        assert.equal(recipeDeliveryReference('Solve ' + issue.html_url, issue.html_url).kind, 'absent');
      }
      if (run === 0) {
        const empty = JSON.stringify({
          comments: [],
          reviews: []
        });
        assert.equal(feedbackNeedsChanges(empty), false);
        assert.equal(feedbackNeedsChanges(JSON.stringify({
          exit_code: 0,
          stdout: empty,
          stderr: '',
          complete: true
        })), false);
        assert.equal(feedbackNeedsChanges('Output: ' + empty + '\nExit Code: 0'), false);
        for (const extra of [{
          exit_code: 127
        }, {
          signal: 'SIGTERM'
        }, {
          timed_out: true
        }, {
          complete: false
        }]) assert.equal(feedbackNeedsChanges(JSON.stringify({
          exit_code: 0,
          stdout: empty,
          stderr: '',
          complete: true,
          ...extra
        })), true);
        assert.equal(feedbackNeedsChanges(JSON.stringify({
          stdout: empty
        })), true);
        assert.equal(feedbackNeedsChanges(JSON.stringify({
          exit_code: 0,
          stdout: '{}',
          complete: true
        })), true);
        assert.equal(feedbackNeedsChanges(JSON.stringify({
          exit_code: 0,
          stdout: JSON.stringify({
            comments: [{
              body: 'Fix output'
            }],
            reviews: []
          }),
          complete: true
        })), true);
      }
      if (run === 0) {
        const command = 'owned exact command';
        const transcript = (argumentsValue, result = {
          exit_code: 0,
          stdout: '',
          complete: true
        }) => [{
          role: 'user',
          content: 'Run the owned command.'
        }, {
          role: 'assistant',
          content: '',
          tool_calls: [{
            id: 'alias-frame',
            type: 'function',
            function: {
              name: 'exec_command',
              arguments: JSON.stringify(argumentsValue)
            }
          }]
        }, {
          role: 'tool',
          name: 'exec_command',
          tool_call_id: 'alias-frame',
          content: JSON.stringify(result)
        }];
        for (const key of ['command', 'cmd', 'script']) assert.equal(qualifiedCommandAttempt(Progress.scan(transcript({
          [key]: command
        })), command)?.binding, 'exact');
        const previousDescriptor = Object.getOwnPropertyDescriptor(Object.prototype, 'cmd');
        try {
          Object.defineProperty(Object.prototype, 'cmd', {
            value: command,
            configurable: true
          });
          assert.equal(qualifiedCommandAttempt(Progress.scan(transcript({
            unrelated: true
          })), command), null);
        } finally {
          if (previousDescriptor) Object.defineProperty(Object.prototype, 'cmd', previousDescriptor);else delete Object.prototype.cmd;
        }
        for (const args of [{
          cmd: command,
          command: 'foreign command'
        }, {
          cmd: command,
          workdir: '/foreign/workspace'
        }]) assert.equal(qualifiedCommandAttempt(Progress.scan(transcript(args)), command), null);
        const source = 'expected source\n';
        const recipe = {
          language: 'javascript',
          path: 'owned.js',
          source,
          supporting_files: [],
          commands: ['node owned.js']
        };
        const task = 'Issue to solve: https://github.com/fixture/project/issues/1\nYour prepared Pull Request: https://github.com/fixture/project/pull/2';
        const probe = recipeSourceCommand(recipe.path);
        const initial = [{
          role: 'user',
          content: task
        }, {
          role: 'assistant',
          content: '',
          tool_calls: [{
            id: 'probe-frame',
            type: 'function',
            function: {
              name: 'exec_command',
              arguments: JSON.stringify({
                cmd: probe
              })
            }
          }]
        }, {
          role: 'tool',
          name: 'exec_command',
          tool_call_id: 'probe-frame',
          content: JSON.stringify({
            exit_code: 0,
            stdout: JSON.stringify({
              schema: 'workspace-source-v1',
              path: recipe.path,
              success: true,
              complete: true,
              absent: false,
              source
            }),
            complete: true
          })
        }];
        for (const [tool, args] of [['write', {
          path: recipe.path,
          content: 'other bytes'
        }], ['Edit', {
          path: recipe.path,
          old_string: source,
          new_string: 'other bytes'
        }], ['MultiEdit', {
          path: recipe.path,
          edits: [{
            old_string: source,
            new_string: 'other bytes'
          }]
        }]]) {
          const changed = [...initial, {
            role: 'assistant',
            content: '',
            tool_calls: [{
              id: 'mutation-frame',
              type: 'function',
              function: {
                name: tool,
                arguments: JSON.stringify(args)
              }
            }]
          }, {
            role: 'tool',
            name: tool,
            tool_call_id: 'mutation-frame',
            content: JSON.stringify({
              exit_code: 0,
              stdout: '',
              complete: true
            })
          }];
          for (const staleRead of [false, true]) {
            const history = staleRead ? [...changed, {
              role: 'assistant',
              content: '',
              tool_calls: [{
                id: 'stale-read',
                type: 'function',
                function: {
                  name: 'Read',
                  arguments: JSON.stringify({
                    path: recipe.path
                  })
                }
              }]
            }, {
              role: 'tool',
              name: 'Read',
              tool_call_id: 'stale-read',
              content: source,
              source_read: {
                path: recipe.path,
                success: true,
                complete: true,
                format: 'raw'
              }
            }] : changed;
            const progress = recipeProgressAfterLatestUser(history, 'write', recipe, recipe.commands, true);
            assert.equal(progress.source_invalidated, true, tool);
            assert.equal(progress.files_written, 0, tool);
            assert.equal(planSymbolicCommandReroute(history, ['exec_command', 'write', 'Read'], {
              execution_recipe: recipe
            }, '').kind, 'final');
          }
        }
        const plain = planSymbolicCommandReroute([{
          role: 'user',
          content: 'Issue to solve: https://github.com/fixture/project/issues/1'
        }], ['bash', 'write'], {
          execution_recipe: recipe
        }, '');
        assert.equal(plain.kind, 'tool_calls');
        assert.equal(plain.calls[0].tool, 'write');
      }
      const calls = [];
      const initialHead = await git('rev-parse', 'HEAD');
      let final = null;
      for (let step = 0; step < 28; step++) {
        fs.writeFileSync(base + '/client-matrix-v3-codex-current-attempt.json', JSON.stringify({
          run,
          calls,
          messages,
          tools
        }, null, 2));
        const next = await nextClient(context, messages, tools, configuration.protocol);
        if (!next.calls.length) {
          final = next.final;
          break;
        }
        assert.equal(next.calls.length, 1);
        const call = next.calls[0];
        const args = call.args;
        const kind = ['write', 'apply_patch'].includes(call.name.toLowerCase()) ? 'write' : call.name.toLowerCase() === 'read' ? 'read' : 'bash';
        const definition = tools.find(tool => (tool.function?.name ?? tool.name) === call.name);
        const schema = definition.function?.parameters ?? definition.parameters ?? definition.input_schema;
        if (!call.custom) assert.equal(invalidArguments(args, schema), null);
        calls.push({
          tool: kind,
          declaredTool: call.name,
          arguments: args,
          protocol: configuration.protocol
        });
        let result;
        if (kind === 'write') {
          const target = path.join(cwd, relativeClientPath(args.path ?? args.file_path ?? args.filePath));
          fs.mkdirSync(path.dirname(target), {
            recursive: true
          });
          fs.writeFileSync(target, args.content);
          result = {
            status: 0,
            stdout: '',
            stderr: '',
            cleanup_complete: true
          };
        } else {
          assert.equal(kind, 'bash', 'unexpected Read route must be explicitly admitted');
          result = await execute(cwd, '/bin/sh', ['-c', args.command ?? args.cmd], environment);
          const state = JSON.parse(fs.readFileSync(statePath));
          state.head = await git('rev-parse', 'HEAD');
          fs.writeFileSync(statePath, JSON.stringify(state));
        }
        calls[calls.length - 1].observation = {
          status: result.status,
          stdout: result.stdout,
          stderr: result.stderr,
          cleanup_complete: result.cleanup_complete
        };
        appendClient(messages, next, call, result, configuration.protocol);
      }
      const tracked = (await git('ls-files')).split('\n');
      for (const artifact of retainedArtifacts) {
        assert(!tracked.includes(artifact.path), artifact.path);
        assert.equal(crypto.createHash('sha256').update(fs.readFileSync(cwd + '/' + artifact.path)).digest('hex'), artifact.sha256);
      }
      const head = await git('rev-parse', 'HEAD');
      receipt.push({
        run,
        calls,
        final,
        initialHead,
        head,
        status: await git('status', '--porcelain', '--untracked-files=all'),
        state: JSON.parse(fs.readFileSync(statePath))
      });
      fs.writeFileSync(base + '/client-matrix-v3-' + configuration.protocol + '-run-' + run + '.json', JSON.stringify(receipt.at(-1), null, 2), {
        flag: 'wx'
      });
      assert.equal(typeof final, 'string');
      assert.match(final, /verified by CI/u);
      assert.doesNotMatch(final, /Created and verified/u);
      assert.equal(receipt.at(-1).status, '');
      assert.equal(receipt.at(-1).state.isDraft, false);
      if (run === 0) {
        assert.notEqual(head, initialHead);
        assert(calls.some(call => call.tool === 'write'));
        assert.match(fs.readFileSync(cwd + '/' + configuration.source, 'utf8'), /Hello, World!/u);
        assert(fs.readdirSync(cwd + '/.github/workflows').some(file => file.endsWith('.yml')));
      } else {
        assert.equal(head, initialHead);
        assert.equal(calls.filter(call => call.tool === 'write').length, 0);
        assert(!calls.some(call => (call.arguments.command ?? call.arguments.cmd)?.includes('git commit')));
        const operations = receipt.at(-1).state.calls.slice(receipt[0].state.calls.length);
        assert(!operations.some(args => args[0] === 'pr' && ['edit', 'ready'].includes(args[1])));
      }
    }
    fs.writeFileSync(base + '/client-matrix-v3-' + configuration.protocol + '-proof.json', JSON.stringify({
      runs: receipt.map(({
        calls,
        state,
        ...rest
      }) => ({
        ...rest,
        toolCalls: calls.length,
        sourceWrites: calls.filter(call => call.tool === 'write').length
      })),
      scope: 'Hermetic full API replay; physical Git/source; fake unavailable compiler and GitHub checks, not real GitHub or native acceptance',
      authorshipCredit: 0
    }, null, 2), {
      flag: 'wx'
    });
    console.log('CLIENT_TWO_PASS_' + configuration.protocol.toUpperCase());
  } finally {
    fs.rmSync(fixture, {
      recursive: true,
      force: true
    });
  }
}
for (const configuration of [{
  index: 0,
  protocol: 'claude',
  source: 'Main.kt'
}, {
  index: 1,
  protocol: 'codex',
  source: 'main.rs'
}, {
  index: 2,
  protocol: 'agent',
  source: 'Main.scala'
}]) test('historical ' + configuration.protocol + ' full API preserves owned source and resumes without writes or commits', {
  timeout: 120000
}, () => runFixture(configuration));
test('tool identities are stable for one normalized request and distinct after history progress', () => {
  const tools = [{
    type: 'function',
    function: {
      name: 'bash',
      parameters: {
        type: 'object',
        properties: {
          command: {
            type: 'string'
          }
        },
        required: ['command'],
        additionalProperties: false
      }
    }
  }];
  const request = {
    model: 'formal-ai',
    messages: [{
      role: 'user',
      content: 'Run the owned command.'
    }],
    tools
  };
  const plan = {
    kind: 'tool_calls',
    calls: [{
      tool: 'bash',
      arguments: JSON.stringify({
        command: 'printf owned'
      })
    }]
  };
  const identify = request => {
    const chat = chatCompletionFromPlan({}, request, 'fixed owned prompt', plan, 1);
    const response = responseFromPlan({}, request, request, 'fixed owned prompt', plan, 1);
    return {
      chat: chat.choices[0].message.tool_calls[0].id,
      responses: response.output.find(item => item.type === 'function_call').call_id
    };
  };
  const first = identify(request);
  assert.deepEqual(identify(request), first);
  assert.equal(first.chat, first.responses);
  const progressed = identify({
    ...request,
    messages: [...request.messages, {
      role: 'assistant',
      content: 'The prior operation completed.'
    }]
  });
  assert.notEqual(progressed.chat, first.chat);
  assert.notEqual(progressed.responses, first.responses);
  assert.equal(progressed.chat, progressed.responses);
});
test('guarded full-source updates reject stale identity and digest and preserve atomic output ownership', async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-source-preimage-'));
  const target = path.join(directory, 'owned.js');
  const original = 'old source\n',
    tampered = 'bad source\n',
    replacement = 'new source\n';
  try {
    fs.writeFileSync(target, original, {
      mode: 0o640
    });
    const run = command => boundedCommand('/bin/sh', ['-c', command], {
      cwd: directory,
      timeout: 15000,
      stdoutLimitBytes: 1048576,
      stderrLimitBytes: 1048576,
      encoding: 'utf8'
    });
    const probe = async () => {
      const result = await run(recipeSourceCommand('owned.js'));
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.error, undefined);
      assert.equal(result.cleanup_complete, true);
      return recipeSourceResult(JSON.stringify({
        exit_code: 0,
        stdout: result.stdout,
        complete: true
      }), 'owned.js');
    };
    const before = await probe();
    assert.equal(before.source, original);
    assert.equal(before.sha256, crypto.createHash('sha256').update(original).digest('hex'));
    fs.writeFileSync(target, tampered);
    const stale = await run(recipeSourceUpdateCommand('owned.js', replacement, before));
    assert.notEqual(stale.status, 0);
    assert.equal(stale.cleanup_complete, true);
    assert.equal(fs.readFileSync(target, 'utf8'), tampered);
    const current = await probe();
    const badDigest = await run(recipeSourceUpdateCommand('owned.js', replacement, {
      ...current,
      sha256: before.sha256
    }));
    assert.notEqual(badDigest.status, 0);
    assert.equal(badDigest.cleanup_complete, true);
    assert.equal(fs.readFileSync(target, 'utf8'), tampered);
    const updated = await run(recipeSourceUpdateCommand('owned.js', replacement, current));
    assert.equal(updated.status, 0, updated.stderr);
    assert.equal(updated.error, undefined);
    assert.equal(updated.cleanup_complete, true);
    assert.equal(fs.readFileSync(target, 'utf8'), replacement);
    assert.equal(fs.statSync(target).mode & 0o777, 0o640);
    assert.deepEqual(fs.readdirSync(directory), ['owned.js']);
    assert.equal(recipeSourceUpdateCommand('owned.js', replacement, {
      absent: false,
      source: original
    }), null);
    assert.equal(recipeSourceUpdateCommand('owned.js', replacement, {
      ...current,
      absent: true
    }), null);
  } finally {
    fs.rmSync(directory, {
      recursive: true,
      force: true
    });
  }
});
