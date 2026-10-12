import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { rustSourceForTask, verifiedSourceDescription } from '../../../js/agentic/code_task.mjs';
import { sourceDescriptionContract } from '../../../js/agentic/code_task/source_contract.mjs';
await installNodeHost(new WorkerHost());
const tools = ['read_file', 'write_file', 'run_command'];
const requests = ['Create a new file rust/src/renamed.rs in this repository containing a single public Rust function calculate that takes an f64 and returns it divided by 3.5.', 'Создай файл rust/src/measure_ru.rs с одной публичной функцией на Rust с именем measure_ru, которая принимает f64 и делит его на 2.5.', '在这个仓库中创建文件 rust/src/measure_zh.rs，其中包含一个名为 measure_zh 的公共 Rust 函数，接受 f64 并返回它除以 2.5。', 'इस रिपॉजिटरी में rust/src/measure_hi.rs फ़ाइल बनाएँ जिसमें measure_hi नाम का एक सार्वजनिक Rust फ़ंक्शन हो जो f64 लेता है और उसे 2.5 से विभाजित करके लौटाता है।', 'Crea un archivo rust/src/measure_es.rs que contiene una función pública Rust llamada measure_es que toma un f64 y lo devuelve dividido por 2.5.'];
for (const request of requests) {
  test('a complete source declaration binds its request: ' + request, async () => {
    const artifact = rustSourceForTask(request);
    assert(artifact);
    const contract = verifiedSourceDescription(request);
    assert(contract);
    assert.equal(contract.wholeRequestConsumed, true);
    assert.equal(contract.compilation, 'pending');
    assert.deepEqual(contract.unknownEffects, ['module_initialization', 'tool_execution']);
    for (const capture of contract.captures) {
      assert.equal(request.slice(...capture.span), capture.text);
    }
    const plan = await planChatStepResolved([{
      role: 'user',
      content: request
    }], tools);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'read_file');
    assert.equal(JSON.parse(plan.calls[0].arguments).path, artifact.path);
  });
}
test('a destination spelling does not provide a function name', () => {
  const artifact = rustSourceForTask(requests[0]);
  assert.equal(artifact.content, 'pub fn calculate(value: f64) -> f64 {\n    value / 3.5\n}\n');
});
test('a source-only contract cannot consume independent required work', async () => {
  for (const suffix of [' Then deploy it.', ' Do not write any files.', ' Read README.md first.', ' Also register this module.']) {
    const request = requests[0] + suffix;
    assert.equal(verifiedSourceDescription(request), null);
    const plan = await planChatStepResolved([{
      role: 'user',
      content: request
    }], tools);
    assert.equal(plan.kind, 'final');
  }
});
test('the expression binds the declared operand, return type, and finite nonzero divisor', () => {
  const artifact = rustSourceForTask(requests[0]);
  for (const content of [artifact.content.replace('value /', 'other /'), artifact.content.replace('3.5', '0.0'), artifact.content.replace('-> f64', '-> i64')]) {
    assert.equal(sourceDescriptionContract(requests[0], {
      ...artifact,
      content
    }), null);
  }
});
test('existing source is read and kept before any replacement', async () => {
  const request = requests[0];
  const first = await planChatStepResolved([{
    role: 'user',
    content: request
  }], tools);
  const call = first.calls[0];
  const id = 'existing-source';
  const messages = [{
    role: 'user',
    content: request
  }, {
    role: 'assistant',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: call.tool,
        arguments: call.arguments
      }
    }]
  }, {
    role: 'tool',
    name: call.tool,
    tool_call_id: id,
    content: 'pub fn existing() {}\n'
  }];
  const plan = await planChatStepResolved(messages, tools);
  assert.equal(plan.kind, 'final');
  assert(plan.answer.toLowerCase().includes('unchanged'));
});

test('explicit technical type captures retain case in every language', async () => {
  for (const request of requests) {
    const artifact = rustSourceForTask(request);
    const changed = request.replace('f64', 'F64');
    assert.equal(sourceDescriptionContract(changed, artifact), null);
    assert.equal(verifiedSourceDescription(changed), null);
    const plan = await planChatStepResolved([{ role: 'user', content: changed }], tools);
    assert.equal(plan.kind, 'final');
  }
});
test('natural language case and an uppercase destination do not change type authority', () => {
  const request = requests[0].replace('Create', 'CREATE').replace('renamed.rs', 'F64_measure.rs');
  const contract = verifiedSourceDescription(request);
  assert(contract);
  assert.equal(contract.output['parameter-type'], 'f64');
  assert.equal(contract.captures.find(capture => capture.role === 'parameter-type').text, 'f64');
  assert.equal(verifiedSourceDescription(request.replace('an f64', 'an F64')), null);
});

test('missing source contracts retain grounded multilingual Gap provenance', async () => {
  const { renderSeededOutcome, planVerifiedGeneratedSourceStep } = await import('../../../js/agentic/code_task.mjs');
  const { canDeliverFinal } = await import('../../../js/agentic/final_result.mjs');
  for (const request of requests) {
    const text = renderSeededOutcome('coding-source-authoring-contract-missing', request, '');
    assert.equal(typeof text, 'string');
    assert(text.length > 0);
    const plan = planVerifiedGeneratedSourceStep(request + ' Then deploy it.', [], tools);
    assert.equal(plan.kind, 'final');
    assert.equal(plan.answer, text);
    assert.equal(plan.result.disposition, 'gap');
    assert.equal(plan.result.origin, 'source-description-goal-coverage-unbound');
    assert.equal(canDeliverFinal(plan), false);
  }
  assert.equal(renderSeededOutcome('unregistered_source_contract', requests[0], ''), null);
});
