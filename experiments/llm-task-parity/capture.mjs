// Capture ungraded OpenAI-compatible responses from an already-running release.
// Usage: node experiments/llm-task-parity/capture.mjs BASE_URL RELEASE > responses.jsonl
import { readFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';

const [baseUrl, version] = process.argv.slice(2);
if (!baseUrl || !version) throw new Error('Provide the server base URL and its verified release version.');

const data = new URL('../../data/benchmarks/llm-task-parity/', import.meta.url);
const prompts = readFileSync(new URL('prompts.jsonl', data), 'utf8').trim().split('\n').map(JSON.parse);
const image = readFileSync(new URL('red-square.png', data)).toString('base64');
const tools = [{
  type: 'function',
  function: {
    name: 'read',
    description: 'Read a local UTF-8 file by path.',
    parameters: { type: 'object', properties: { path: { type: 'string' } }, required: ['path'] },
  },
}];

for (const probe of prompts) {
  for (const language of ['en', 'ru']) {
    const messages = [];
    if (probe.history) {
      const [user, assistant] = probe.history[language];
      messages.push({ role: 'user', content: user }, { role: 'assistant', content: assistant });
    }
    let content = probe[language];
    if (probe.document) {
      const [context, answer] = probe.document[language];
      const paragraphs = Array.from({ length: 80 }, (_, index) => {
        const sentence = index === 53 ? answer : context;
        return `${index + 1}. ${sentence}`;
      });
      content = `${paragraphs.join('\n')}\n\n${content}`;
    }
    if (probe.image) {
      content = [
        { type: 'text', text: content },
        { type: 'image_url', image_url: { url: `data:image/png;base64,${image}` } },
      ];
    }
    messages.push({ role: 'user', content });
    const start = performance.now();
    const result = { version, class: probe.class, id: probe.id, language };
    try {
      const response = await fetch(new URL('/v1/chat/completions', baseUrl), {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ model: 'formal-ai', messages, ...(probe.tools ? { tools } : {}) }),
        signal: AbortSignal.timeout(30000),
      });
      const payload = await response.json();
      const choice = payload.choices?.[0];
      result.status = response.status;
      if (choice) {
        result.content = choice.message?.content ?? null;
        if (choice.message?.tool_calls) result.toolCalls = choice.message.tool_calls;
        result.finishReason = choice.finish_reason;
      } else {
        result.error = payload.error ?? payload;
      }
    } catch (error) {
      result.error = error.message;
    }
    result.elapsedMs = Math.round(performance.now() - start);
    console.log(JSON.stringify(result));
  }
}
