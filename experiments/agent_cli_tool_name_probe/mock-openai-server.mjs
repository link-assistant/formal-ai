#!/usr/bin/env node
// Minimal OpenAI-compatible server that streams exactly one tool call, to isolate
// whether `agent --output-format stream-json --compact-json` carries the tool's
// name through to its own stream (issue #715 case study, formal-ai).
//
// It depends on nothing: no formal-ai, no model, no network. Run it, point the
// agent CLI at it, and read the `tool_use` event the CLI prints.
//
//   node experiments/agent_cli_tool_name_probe/mock-openai-server.mjs 8931 &
//   agent --base-url http://127.0.0.1:8931/api/openai/v1 --model mock/mock \
//         --permission-mode auto --output-format stream-json --compact-json \
//         --disable-stdin --prompt 'Write hi to a file called hi.txt'
//
// Expected: the printed tool_use event names `write`.
// Observed (agent 0.25.0): "name": "unknown", "input": {}.

import { createServer } from 'node:http';

const port = Number(process.argv[2] || 8931);
const MODEL = 'formal-ai';

const sse = (payload) => `data: ${JSON.stringify(payload)}\n\n`;

// Omitting usage makes the CLI retry the round as a provider API error
// (link-assistant/agent#249), which turns this probe into a backoff loop.
const USAGE = { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 };

// One assistant turn: a single tool call to `write`, streamed as OpenAI streams
// it — the name and arguments ride in `delta.tool_calls[].function`.
function toolCallStream() {
  const base = { id: 'chatcmpl-probe', object: 'chat.completion.chunk', created: 0, model: MODEL };
  let body = '';
  body += sse({
    ...base,
    choices: [{
      index: 0,
      delta: {
        role: 'assistant',
        tool_calls: [{
          index: 0,
          id: 'call_probe_0',
          type: 'function',
          function: { name: 'write', arguments: JSON.stringify({ filePath: 'hi.txt', content: 'hi\n' }) },
        }],
      },
      finish_reason: null,
    }],
  });
  body += sse({ ...base, choices: [{ index: 0, delta: {}, finish_reason: 'tool_calls' }], usage: USAGE });
  body += 'data: [DONE]\n\n';
  return body;
}

let completions = 0;

createServer((req, res) => {
  let raw = '';
  req.on('data', (c) => { raw += c; });
  req.on('end', () => {
    const url = req.url || '';
    console.error(`[mock] ${req.method} ${url}`);

    if (url.endsWith('/models')) {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({ object: 'list', data: [{ id: MODEL, object: 'model', owned_by: 'mock' }] }));
      return;
    }

    if (url.endsWith('/chat/completions')) {
      // Stream the tool call once, then answer every later round with plain text
      // so the CLI terminates instead of calling `write` forever. Counting the
      // rounds keeps that deterministic; sniffing the request body for a tool
      // result does not, which is why the first draft of this probe looped.
      completions += 1;
      res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
      if (completions > 1) {
        const base = { id: 'chatcmpl-done', object: 'chat.completion.chunk', created: 0, model: MODEL };
        res.end(
          sse({ ...base, choices: [{ index: 0, delta: { role: 'assistant', content: 'done' }, finish_reason: null }] }) +
          sse({ ...base, choices: [{ index: 0, delta: {}, finish_reason: 'stop' }], usage: USAGE }) +
          'data: [DONE]\n\n',
        );
      } else {
        res.end(toolCallStream());
      }
      return;
    }

    res.writeHead(404, { 'content-type': 'application/json' });
    res.end('{}');
  });
}).listen(port, '127.0.0.1', () => console.error(`[mock] listening on 127.0.0.1:${port}`));
