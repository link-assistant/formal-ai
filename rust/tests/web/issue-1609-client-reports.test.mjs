import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { startCli, temporaryHome } from './support/structured-log-fixture.mjs';
import { startServer } from '../../../js/server/main.mjs';

const CLAUDE_SYSTEM = [
  'Do not open downloaded files or write in a different directory before confirming the user trusts them.',
  'You can read files, and then run commands using the Bash tool.',
];
const OPENCODE_SYSTEM = [
  'Output text to communicate with the user.',
  'You MUST answer concisely with fewer than 4 lines of text (not including tool use or code generation), unless user asks for detail.',
  'Answer the user\'s question directly, without elaboration, explanation, or details.',
  'If there is a simple answer, answer it directly instead.',
];
const TITLE_SYSTEM = [
  'You are a title generator. You output ONLY a thread title. Nothing else.',
  '<task>',
  'Generate a brief title that would help the user find this conversation later.',
  'Your output must be:',
  '- A single line',
  '- ≤50 characters',
  '- No explanations',
  '</task>',
].join('\n');
const ISSUE_URL = 'https://github.com/link-assistant/formal-ai/issues/9999';
const NATIVE = process.env.FORMAL_AI_TEST_NATIVE_CLI;
const user = content => ({ role: 'user', content });
const assistant = content => ({ role: 'assistant', content });
const system = content => ({ role: 'system', content });
const bashSchema = { type: 'object', properties: { command: { type: 'string' } }, required: ['command'] };
const commandSchema = { type: 'object', properties: { cmd: { type: 'string' } }, required: ['cmd'] };

function chat(id, headers, guidance, name = 'bash', schema = bashSchema) {
  return {
    id, headers, guidance, endpoint: '/api/openai/v1/chat/completions',
    request: (messages, tools) => ({
      messages, tools: tools ? [{ type: 'function', function: { name, parameters: schema } }] : [],
    }),
    unpack(body) {
      const { content, tool_calls: calls } = body.choices[0].message;
      const call = calls?.[0];
      return {
        text: content,
        command: call && JSON.parse(call.function.arguments).command,
        description: call && JSON.parse(call.function.arguments).description,
        reply: output => [{ role: 'assistant', content, tool_calls: calls }, {
          role: 'tool', name, tool_call_id: call.id, content: output,
        }],
      };
    },
  };
}

function gemini(name, endpoint, id) {
  const turn = (role, parts) => ({ role, parts });
  return {
    name, endpoint, id, nativeHarness: true, headers: { 'x-formal-ai-dialog-id': id },
    request: (messages, tools) => ({
      contents: messages.map(message => 'parts' in message ? message
        : turn(message.role === 'assistant' ? 'model' : 'user', [{ text: message.content }])),
      tools: tools ? [{ functionDeclarations: [{ name: 'run_shell_command', parameters: bashSchema }] }] : [],
    }),
    unpack(body) {
      const { parts } = body.candidates[0].content;
      const call = parts.find(part => part.functionCall)?.functionCall;
      return {
        text: parts.flatMap(part => part.text ?? []).join('\n'),
        command: call?.args.command,
        reply: output => [turn('model', parts), turn('user', [{
          functionResponse: { id: call.id, name: call.name, response: { output } },
        }])],
      };
    },
  };
}

const QWEN_ID = '100a1cc7-2483-46c2-803c-2ee707ab6c17';
const GROK_ID = '100a1cc7-2483-46c2-803c-2ee707ab6c49';
const AIDER_ID = 'aider_issue1609';
const qwenChat = chat(QWEN_ID, { 'x-formal-ai-dialog-id': QWEN_ID },
  ["Respond in the same language as the user's input."], 'run_shell_command');
const grokChat = chat(GROK_ID, { 'x-grok-session-id': GROK_ID }, [], 'run_terminal_command', {
  ...bashSchema,
  properties: { ...bashSchema.properties, description: { type: 'string' } },
  required: ['command', 'description'],
});

const clients = [
  {
    name: 'Claude Code', nativeHarness: true, id: 'f19dae24-d36f-48c9-b17b-da2db2ccd240', endpoint: '/api/anthropic/v1/messages',
    request(messages, tools) {
      return {
        system: CLAUDE_SYSTEM.map(text => ({ type: 'text', text })), messages, max_tokens: 1024,
        metadata: { user_id: JSON.stringify({ session_id: this.id }) },
        tools: tools ? [{ name: 'Bash', input_schema: bashSchema }] : [],
      };
    },
    unpack(body) {
      const call = body.content.find(part => part.type === 'tool_use');
      return {
        text: body.content.filter(part => part.type === 'text').map(part => part.text).join('\n'),
        command: call?.input.command,
        reply: output => [assistant(body.content), user([{ type: 'tool_result', tool_use_id: call.id, content: output }])],
      };
    },
  },
  {
    name: 'OpenCode',
    ...chat('ses_issue1609', { 'x-opencode-session-id': 'ses_issue1609' }, OPENCODE_SYSTEM),
  },
  {
    name: 'Link Assistant Agent', nativeHarness: true,
    ...chat('agent_issue1609', { 'x-formal-ai-dialog-id': 'agent_issue1609' }, []),
  },
  {
    name: 'Codex', nativeHarness: true, id: '34c14b94-e8cc-4c32-a792-12a162fcd9fe', endpoint: '/api/openai/v1/responses',
    headers: { 'x-codex-turn-metadata': JSON.stringify({ thread_id: '34c14b94-e8cc-4c32-a792-12a162fcd9fe' }) },
    request: (input, tools) => ({
      input, tools: tools ? [{ type: 'function', name: 'exec_command', parameters: commandSchema }] : [],
    }),
    unpack(body) {
      const call = body.output.find(item => item.type === 'function_call');
      return {
        text: body.output.flatMap(item => item.content ?? []).filter(part => part.type === 'output_text').map(part => part.text).join('\n'),
        command: call && JSON.parse(call.arguments).cmd,
        reply: output => [assistant(body.output[0].content.map(part => part.text).join('\n')),
          call, { type: 'function_call_output', call_id: call.call_id, output }],
      };
    },
  },
  gemini('Gemini', '/api/gemini/v1beta/models/formal-ai:generateContent', 'gemini_issue1609'),
  gemini('Vertex', '/api/vertex/v1/projects/formal-ai/locations/global/publishers/google/models/formal-ai:generateContent', 'vertex_issue1609'),
  {
    ...qwenChat, name: 'Qwen Code', nativeHarness: true,
    request(messages, tools) {
      return qwenChat.request(messages.map(message => {
        if (message.role !== 'user' || typeof message.content !== 'string') return message;
        return user([
          { type: 'text', text: '<system-reminder>Local environment</system-reminder>' },
          { type: 'text', text: message.content },
          { type: 'text', text: 'Current memory catalog (full snapshot; supersedes all earlier memory catalogs):\n<system-reminder>Private profile</system-reminder>' },
        ]);
      }), tools);
    },
  },
  {
    ...grokChat, name: 'Grok Build', nativeHarness: true,
    endpoint: '/api/openai/v1/dialog/grok_factory_scope/chat/completions',
    request(messages, tools) {
      const wireTurns = messages.map(message => message.role === 'user'
        ? user(`<user_query>\n${message.content}\n</user_query>`) : message);
      return grokChat.request([
        user('<user_info>Local profile</user_info>'), user('<rules>Local policy</rules>'),
        ...wireTurns, user('<system-reminder>MCP server connected</system-reminder>'),
      ], tools);
    },
  },
];

async function ask(run, client, messages, tools = false) {
  const response = await fetch(run.url + client.endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...client.headers },
    body: JSON.stringify({ model: 'formal-ai', stream: false, ...client.request(messages, tools) }),
    signal: AbortSignal.timeout(18_000),
  });
  const body = await response.text();
  assert.ok(response.ok, body);
  return client.unpack(JSON.parse(body));
}

function writeFixture(file, ...records) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, records.map(record => JSON.stringify(record)).join('\n') + '\n');
}

function checkNativeExport(run, client, logs, command) {
  const history = ['Hello', 'Report', 'GitHub issue', 'Both logs'];
  const session = client.id;
  const config = path.join(run.home, 'client-transcript');
  const env = { ...run.env };
  let tool;
  let nativeId = session;
  if (client.name === 'Claude Code') {
    tool = 'claude';
    env.CLAUDE_CONFIG_DIR = config;
    writeFixture(path.join(config, 'projects', 'test-project', `${session}.jsonl`),
      ...history.map(text => ({ sessionId: session, type: 'user', message: user(text) })));
  } else if (client.name === 'Codex') {
    tool = 'codex';
    env.CODEX_HOME = config;
    writeFixture(path.join(config, 'sessions', '2026', '11', '21', `rollout-2026-11-21T14-00-00-${session}.jsonl`),
      ...history.map(text => ({ type: 'response_item', payload: { type: 'message', ...user(text) } })));
  } else if (client.name === 'Qwen Code') {
    tool = 'qwen';
    env.HOME = config;
    env.QWEN_CODE_SESSION_ID = session;
    const store = path.join(config, '.qwen', 'projects', 'work', 'chats');
    writeFixture(path.join(store, `${session}.jsonl`), ...history.map(text => ({
      sessionId: session, type: 'user', message: { role: 'user', parts: [{ text }] },
    })));
    writeFixture(path.join(store, 'unrelated.jsonl'), {
      sessionId: 'unrelated', type: 'user', message: { role: 'user', parts: [{ text: 'A different conversation' }] },
    });
  } else if (client.name === 'Grok Build') {
    tool = 'grok';
    env.GROK_HOME = config;
    const store = path.join(config, 'sessions', '%2Fwork');
    writeFixture(path.join(store, session, 'chat_history.jsonl'),
      { type: 'user', content: [{ type: 'text', text: '<user_info>Private profile</user_info>' }] },
      ...history.map((text, prompt_index) => ({
        type: 'user', prompt_index, content: [{ type: 'text', text: `<user_query>\n${text}\n</user_query>` }],
      })));
    writeFixture(path.join(store, 'unrelated', 'chat_history.jsonl'), {
      type: 'user', prompt_index: 0, content: 'A different conversation',
    });
  } else if (client.name === 'Aider') {
    tool = 'aider';
    const old = '# aider chat started at 2026-11-20 10:00:00\n\n#### A different conversation\n\n';
    const current = '# aider chat started at 2026-11-21 10:00:00\n\n'
      + history.map(text => `#### ${text}  \n\n> Tokens: 2 sent, 1 received.  \n\n`).join('');
    env.FORMAL_AI_REPORT_DIALOG_ID = session;
    env.FORMAL_AI_REPORT_AIDER_HISTORY = config;
    env.FORMAL_AI_REPORT_AIDER_HISTORY_START = String(Buffer.byteLength(old));
    writeFileSync(config, old + current);
  } else if (client.name === 'Gemini' || client.name === 'Vertex') {
    tool = 'gemini';
    nativeId = '79dd7290-ac1c-4c3f-a490-cd47e612eb82';
    env.GEMINI_CLI_HOME = path.join(config, session);
    env.FORMAL_AI_REPORT_DIALOG_ID = session;
    const store = path.join(env.GEMINI_CLI_HOME, '.gemini', 'tmp', 'work', 'chats');
    writeFixture(path.join(store, 'session-matching.jsonl'), { sessionId: nativeId },
      { type: 'user', content: [{ text: '<session_context>Gemini environment</session_context>' }] },
      ...history.map(text => ({ type: 'user', content: [{ text }] })));
    writeFixture(path.join(store, 'session-other.jsonl'), { sessionId: '00000000-0000-4000-a000-000000000000' },
      { type: 'user', content: [{ text: 'A different conversation' }] });
  } else {
    tool = 'agent';
    nativeId = 'ses_matching';
    env.XDG_DATA_HOME = config;
    env.FORMAL_AI_REPORT_DIALOG_ID = session;
    const store = path.join(config, 'link-assistant-agent', 'storage');
    for (const [id, turns] of [[nativeId, history], ['ses_other', ['A different conversation']]]) {
      turns.forEach((text, index) => {
        const message = `msg_${id}_${index}`;
        writeFixture(path.join(store, 'message', id, `${message}.json`), { id: message, sessionID: id, role: 'user' });
        writeFixture(path.join(store, 'part', message, 'text.json'), { type: 'text', text });
      });
    }
  }
  const encoded = execFileSync(command, ['context', 'export', '--session', session, '--source', 'both',
    '--log-dir', logs, '--format', 'json'], { encoding: 'utf8', env: { ...env, FORMAL_AI_REPORT_CLIENT: tool } });
  const exported = JSON.parse(encoded);
  assert.equal(exported.metadata.dialog_id, session);
  const environmentMessages = tool === 'gemini' ? 1 : tool === 'aider' ? history.length : 0;
  assert.equal(exported.metadata.harness_message_count, history.length + environmentMessages);
  assert.ok(exported.metadata.server_message_count > 0);
  assert.equal(exported.metadata.harness_error, null);
  assert.equal(exported.metadata.server_error, null);
  assert.equal(exported.harness.source.tool, tool);
  assert.equal(exported.harness.session.id, nativeId);
  assert.doesNotMatch(encoded, /A different conversation/u);
}

for (const [server, binary] of [['JavaScript', undefined], ['native', NATIVE]]) {
  test(`${server} keeps released-client greetings, titles, and issue receipts usable`, {
    skip: server === 'native' && !binary, timeout: binary ? 300_000 : 60_000,
  }, async t => {
    const logs = temporaryHome(t);
    const env = {
      FORMAL_AI_DIALOG_LOG_DIR: logs,
      FORMAL_AI_MEMORY_PATH: path.join(logs, 'memory.lino'),
      FORMAL_AI_LOG_DIR: path.join(logs, 'diagnostics'),
      FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false',
      FORMAL_AI_ALLOW_NETWORK: 'false',
      FORMAL_AI_DREAMING: 'false',
      OPENAI_API_KEY: 'formal-ai',
    };
    const run = binary
      ? await startCli(t, { binary, silent: false, args: ['--agent-mode'], lifetime: 315_000, env })
      : await startServer({ agentMode: true, env });
    if (binary) await run.ready();
    else t.after(() => new Promise(resolve => run.server.close(resolve)));
    await t.test('MCP and Telegram greet and open the Formal AI issue form', async () => {
      const request = async (endpoint, payload) => {
        const response = await fetch(run.url + endpoint, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(payload),
          signal: AbortSignal.timeout(18_000),
        });
        const body = await response.text();
        assert.ok(response.ok, body);
        return JSON.parse(body);
      };
      for (const [prompt, expected] of [
        ['Hello', /Hi[!,]/u],
        ['Привет', /Здравствуйте/u],
        ['Report issue', /https:\/\/github\.com\/link-assistant\/formal-ai\/issues\/new$/u],
      ]) {
        const mcp = await request('/mcp', {
          jsonrpc: '2.0', method: 'tools/call', id: 1,
          params: { name: 'formal_ai_chat', arguments: { prompt } },
        });
        const telegram = await request('/telegram/webhook', {
          message: { message_id: 1, chat: { id: 7, type: 'private' }, text: prompt },
        });
        assert.match(mcp.result.content[0].text, expected);
        assert.match(telegram.text, expected);
      }
    });
    for (const client of clients) {
      await t.test(client.name, async () => {
        const messages = client.guidance?.map(system) ?? [];
        const greetings = [
          { language: 'en', greeting: 'Hello', expected: /^Hi[!,]/u },
          { language: 'ru', greeting: 'Привет', expected: /^Здравствуйте/u },
          { language: 'hi', greeting: 'नमस्ते', expected: /^नमस्ते/u },
          { language: 'zh', greeting: '你好', expected: /^你好/u },
          { language: 'es', greeting: 'Hola', expected: /^¡Hola!/u },
        ];
        for (const { language, greeting, expected } of
          binary && client.name !== 'OpenCode' ? greetings.slice(0, 2) : greetings) {
          const response = await ask(run, client, [...messages, user(greeting)]);
          assert.match(response.text, expected, language);
        }
        const history = [...messages, user('Hello'), assistant('Hi!'), user('Report')];
        const destination = await ask(run, client, history, true);
        assert.match(destination.text, /GitHub issue/u);
        history.push(assistant(destination.text), user('GitHub issue'));
        const scope = await ask(run, client, history, true);
        assert.match(scope.text, /Both logs/u);
        history.push(assistant(scope.text), user('Both logs'));
        const submission = await ask(run, client, history, true);
        assert.match(submission.command, /^formal-ai report create-issue /u);
        assert.ok(submission.command.includes(`--session '${client.id}' --source both `), submission.command);
        assert.match(submission.text, /file the report in the Formal AI GitHub repository/u);
        if (client.name === 'Grok Build') assert.match(submission.description, /file the report/u);
        for (const [receipt, expected] of [
          [ISSUE_URL, /^Filed the issue on GitHub: https:\/\/github\.com\/link-assistant\/formal-ai\/issues\/9999/u],
          [`Exit Code: 1\n${ISSUE_URL}`, /couldn't file the issue/u],
        ]) {
          const final = await ask(run, client, [...history, ...submission.reply(receipt)], true);
          assert.match(final.text, expected);
          assert.equal(final.command, undefined);
        }
        const log = path.join(logs, `${client.id}.conversation.lino`);
        assert.ok(existsSync(log), log);
        assert.match(readFileSync(log, 'utf8'), /Report/u);
        if (binary && client.nativeHarness) checkNativeExport(run, client, logs, binary);
      });
    }
    await t.test('ordinary shell explanations describe the selected subcommand', async () => {
      for (const action of ['git status', 'git log']) {
        const plan = await ask(run, clients[1], [user(`Run ${action}`)], true);
        assert.equal(plan.command, action);
        assert.equal(plan.text, `I’ll run \`${action}\`.`);
      }
    });
    await t.test('Aider can confirm and run its own report command without function tools', async () => {
      const id = AIDER_ID;
      const aiderChat = chat(id, {}, []);
      const format = 'Show any edits using the *file listing* format.';
      const notice = user('I am not sharing any files that you can edit yet.');
      const example = user('Change the greeting to be more casual');
      const aider = {
        ...aiderChat, name: 'Aider',
        endpoint: `/api/openai/v1/dialog/${id}/chat/completions`,
        request(messages, tools) {
          const latest = messages.at(-1);
          return aiderChat.request([
            system(format), example, assistant('Ok.'), ...messages.slice(0, -1),
            notice, assistant('Ok.'), user(`${latest.content}\n\n${format}`),
          ], tools);
        },
      };
      const history = [user('Hello'), assistant('Hi!'), user('Report issue')];
      const destination = await ask(run, aider, history);
      assert.match(destination.text, /GitHub issue/u);
      history.push(assistant(destination.text), user('3'));
      const scope = await ask(run, aider, history);
      assert.match(scope.text, /Both logs/u);
      history.push(assistant(scope.text), user('1'));
      const submission = await ask(run, aider, history);
      assert.equal(submission.command, undefined);
      assert.match(submission.text, /`\/run`/u);
      assert.match(submission.text, /formal-ai report create-issue --session 'aider_issue1609' --source both /u);
      assert.match(submission.text, /--title 'Formal AI: `Hello`'/u);
      assert.doesNotMatch(submission.text, /Filed the issue|```/u);
      assert.ok(existsSync(path.join(logs, `${id}.conversation.lino`)));
      const uncorrelated = await ask(run, chat(null, {}, []), history);
      assert.doesNotMatch(uncorrelated.text, /report create-issue/u);
      const malformed = await fetch(run.url + aider.endpoint.replace(id, '%2F-invalid'), { method: 'POST' });
      assert.equal(malformed.status, 404);
      const models = await fetch(run.url + aider.endpoint.replace('chat/completions', 'models'));
      assert.ok(models.ok);
      if (binary) checkNativeExport(run, aider, logs, binary);
    });
    await t.test('OpenCode title requests stay plain and bounded', async () => {
      for (const greeting of ['Hello', 'Привет']) {
        const { text } = await ask(run, clients[1], [
          system(TITLE_SYSTEM), user('Generate a title for this conversation:\n'), user(greeting),
        ]);
        assert.match(text, /^\S[^\r\n]*$/u);
        assert.ok(Array.from(text).length <= 50, text);
        assert.doesNotMatch(text, /Cannot satisfy|можно помочь|may I help/u);
        if (greeting === 'Привет') assert.match(text, /\p{Script=Cyrillic}/u);
      }
    });
  });
}
