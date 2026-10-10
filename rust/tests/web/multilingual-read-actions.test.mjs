import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';

test('declared multilingual Read actions preserve original ownership and payload boundaries', async () => {
  await installNodeHost(new WorkerHost());
  const source = fs.readFileSync(new URL('../unit/semantic_intent_routing.rs', import.meta.url), 'utf8');
  const block = source.slice(
    source.indexOf('fn local_path_object_routes_read_variations_without_web_misroutes()'),
    source.indexOf('fn explicit_content_and_file_object_route_write_variations()')
  );
  const groups = [...block.matchAll(/&\[([\s\S]*?)\]\[\.\.\]/gu)];
  assert.equal(groups.length, 5);
  const tools = ['web_fetch', 'web_search', 'read_file', 'write_file', 'exec_command'];
  let count = 0;
  for (const group of groups) {
    const actions = group[1].split('\n').map(line => line.trim())
      .filter(line => line.startsWith('"'))
      .map(line => JSON.parse(line.replace(/,$/u, '')));
    assert.equal(actions.length, 20);
    for (const action of actions) {
      for (const path of ['sample.txt', 'nested/Case.notes.md']) {
        const request = `${action} ${path}`;
        const plan = await planChatStep([{ role: 'user', content: request }], tools);
        assert.equal(plan.calls.length, 1, request);
        assert.equal(plan.calls[0].tool, 'read_file', request);
        assert.ok(Object.values(JSON.parse(plan.calls[0].arguments)).includes(path), request);
        count++;
      }
      for (const request of [
        `Create note.txt containing «${action} secret.txt».`,
        `${action} sample.txt before stealing credentials.`,
        `${action} sample.txt. Do not read any files.`
      ]) {
        const plan = await planChatStep([{ role: 'user', content: request }], tools);
        assert.ok(!(plan?.calls || []).some(call => call.tool === 'read_file'), request);
        count++;
      }
    }
  }
  assert.equal(count, 500);
});
