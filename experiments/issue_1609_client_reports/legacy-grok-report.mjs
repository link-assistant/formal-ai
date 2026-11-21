import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

// The historical community client keeps its history in this process. Exercise
// its stock streaming/tool loop with the runner's recording GitHub command.
const [packageDirectory, endpoint, dialog] = process.argv.slice(2);
assert.ok(packageDirectory && endpoint && dialog);
const base = new URL(endpoint);
assert.equal(base.hostname, '127.0.0.1');
const fromPackage = relative => import(pathToFileURL(path.resolve(packageDirectory, relative)));
const { GrokAgent } = await fromPackage('dist/agent/grok-agent.js');
const { ConfirmationService } = await fromPackage('dist/utils/confirmation-service.js');
const client = new GrokAgent('formal-ai-local', `${base.href.replace(/\/$/u, '')}/dialog/${dialog}`, 'formal-ai', 3);
const confirmation = ConfirmationService.getInstance();
confirmation.on('confirmation-requested', request => {
  const command = request.filename;
  confirmation.confirmOperation(typeof command === 'string' && command.startsWith('formal-ai report create-issue '), false);
});

for (const [prompt, expected] of [
  ['Hello', /Hi[!,]/u],
  ['Report', /GitHub issue/u],
  ['GitHub issue', /Server log/u],
  ['Server log', /Filed the issue on GitHub: https:\/\/github\.com\/link-assistant\/formal-ai\/issues\/9999/u],
]) {
  let answer = '';
  for await (const chunk of client.processUserMessageStream(prompt)) {
    if (chunk.type === 'content') answer += chunk.content;
  }
  assert.match(answer, expected);
  process.stdout.write(`> ${prompt}\n${answer}\n`);
}
assert.ok(client.getChatHistory().some(turn => turn.type === 'tool_result'));
