import test from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {createChatCompletion} from '../../../js/server/openai.mjs';
import {parseChatRequest} from '../../../js/server/chat-request.mjs';
import {agenticOutcome} from '../../../js/server/agentic.mjs';
import {finalResult} from '../../../js/agentic/plan.mjs';
import {planEditStep} from '../../../js/agentic/intent_router.mjs';
import {ensureNodeHost} from '../../../js/server/node-host-install.mjs';
process.env.FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR = 'false';
const blocked = () => { throw Error('LocalWasmExecutionForbidden'); };
const worker = new WorkerHost({WebAssembly: {...WebAssembly, instantiate: blocked, compile: blocked, Module: blocked, Instance: blocked}});
const ctx = {worker, agentMode: true, memory: {events: () => []}};
await ensureNodeHost(ctx, {sourceUsageRustGrammar: false});
const tool = name => ({type: 'function', function: {name, description: name, parameters: {type: 'object', properties: name === 'bash' ? {command: {type: 'string'}} : {path: {type: 'string'}}, required: name === 'bash' ? ['command'] : []}}});
const request = (prompt, names) => parseChatRequest(JSON.stringify({model: 'formal-ai', messages: [{role: 'system', content: '<env>\nWorking directory: /isolated-fixture\n</env>'}, {role: 'user', content: prompt}], tools: names.map(tool)}));
test('grounded edit with Start bash-only tools reports missing capabilities rather than memory completion', async () => {
  const prompts = [
    ['snippet.mjs', 'In snippet.mjs, replace «const label = "before";» with «const label = "after";». Preserve all other content.'],
    ['nested/held-out.mjs', 'In nested/held-out.mjs replace «const memory = "old";» with «const memory = "new";». Preserve all other content.']
  ];
  for (const [target, prompt] of prompts) {
    const missingEdit = planEditStep(prompt, [], ['read']);
    assert.deepEqual(JSON.parse(missingEdit.answer).missingCapabilities, ['edit']);
    assert.equal(JSON.parse(missingEdit.answer).reason, 'EditToolUnavailable');
    assert.equal(finalResult(missingEdit).disposition, 'gap');
    for (const names of [['bash'], ['edit'], ['bash', 'edit']]) {
      const input = request(prompt, names);
      for (let pass = 0; pass < 2; pass++) {
        const response = await createChatCompletion(ctx, input);
        const message = response.choices[0].message;
        assert.deepEqual(message.tool_calls, []);
        const gap = JSON.parse(message.content);
        assert.equal(gap.schema, 'workspace-edit-capability-gap/v1');
        assert.equal(gap.target, target);
        assert.equal(gap.reason, names.includes('edit') ? 'ReadToolUnavailable' : 'EditToolUnavailable');
        assert.deepEqual(gap.missingCapabilities, ['read', 'edit'].filter(name => !names.includes(name)));
        assert.equal(gap.complete, false);
        assert.equal(gap.verified, false);
      }
      const outcome = await agenticOutcome(ctx, input, () => { throw Error('UnexpectedPermissionRefusal'); });
      assert.equal(outcome.kind, 'planned');
      assert.equal(finalResult(outcome.plan).disposition, 'gap');
    }
    const supported = await createChatCompletion(ctx, request(prompt, ['read', 'edit']));
    assert.equal(supported.choices[0].message.tool_calls[0].function.name, 'read');
  }
  const memory = await createChatCompletion(ctx, request('For every memory event perform an unknown operation.', ['bash']));
  assert.match(memory.choices[0].message.content, /program_gap:no_complete_seeded_family/);
});
