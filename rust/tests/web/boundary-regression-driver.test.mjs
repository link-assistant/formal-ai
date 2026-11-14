// Intended owner: rust/tests/web/boundary-regression-driver.test.mjs.
// No regression body is supplied to the solver; assertions remain outside workspace.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
const blocked = () => {
  throw Error('LocalWasmExecutionForbidden');
};
globalThis.WebAssembly = {
  ...WebAssembly,
  instantiate: blocked,
  compile: blocked,
  Module: blocked,
  Instance: blocked
};
await installNodeHost(new WorkerHost({
  WebAssembly: globalThis.WebAssembly
}));
const request = 'Create a regression test in tests/range.test.mjs for function contains in src/range.mjs with an inclusive upper bound. Preserve the source, comments and other functions.';
const source = `// Preserve the actual source. 雪
export function contains(value,minimum,maximum){
  return value >= minimum && value <= maximum;
}
`;
const tools = ['read', 'write', 'edit'];
function fixture(context) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'boundary-driver-'));
  context.after(() => fs.rmSync(directory, {
    recursive: true,
    force: true
  }));
  fs.mkdirSync(path.join(directory, 'src'));
  fs.writeFileSync(path.join(directory, 'src/range.mjs'), source);
  return directory;
}
test('actual maintained planner owns typed Create regression with exclusive source-bound bytes', async context => {
  const directory = fixture(context);
  assert.equal(fs.existsSync(path.join(directory, 'tests/range.test.mjs')), false);
  const result = await drive(planChatStep, directory, request, {
    tools,
    steps: 8,
    allowedCommands: []
  });
  const writes = result.transcript.filter(entry => entry.tool === 'write');
  assert.equal(writes.length, 1);
  assert.equal(JSON.parse(writes[0].arguments).path, 'tests/range.test.mjs');
  assert.equal(writes[0].source_creation.success, true);
  assert.equal(writes[0].source_creation.exclusive, true);
  assert.equal(writes[0].semantic_creation.maintainedPlanMatch, true);
  assert.equal(fs.readFileSync(path.join(directory, 'src/range.mjs'), 'utf8'), source);
  assert.equal(fs.readFileSync(path.join(directory, 'tests/range.test.mjs'), 'utf8'), JSON.parse(writes[0].arguments).content);
  const report = JSON.parse(result.answer);
  assert.equal(report.physicalReadbackVerified, true);
  assert.equal(report.independentlyAccepted, false);
});
test('arbitrary injected Write cannot borrow the typed semantic creation authority', async context => {
  const directory = fixture(context);
  const injected = async () => ({
    kind: 'tool_calls',
    calls: [{
      tool: 'write',
      arguments: JSON.stringify({
        filePath: 'tests/range.test.mjs',
        content: 'injected unrelated bytes'
      })
    }]
  });
  const result = await drive(injected, directory, request, {
    tools,
    steps: 8,
    allowedCommands: []
  });
  assert.equal(result.stop, 'UnboundSemanticRegressionPlan');
  assert.equal(fs.existsSync(path.join(directory, 'tests/range.test.mjs')), false);
});
test('forwarded reads do not authorize an injected candidate body or another target', async context => {
  for (const variant of ['body', 'target']) {
    const directory = fixture(context);
    const injected = async (messages, offered) => {
      const plan = await planChatStep(messages, offered);
      if (plan?.kind !== 'tool_calls' || plan.calls[0].tool !== 'write') return plan;
      const argumentsValue = JSON.parse(plan.calls[0].arguments);
      if (variant === 'body') argumentsValue.content += '\n// injected bytes';else for (const alias of ['path', 'filePath', 'file_path']) if (Object.hasOwn(argumentsValue, alias)) argumentsValue[alias] = 'tests/unrelated.test.mjs';
      return {
        kind: 'tool_calls',
        calls: [{
          tool: 'write',
          arguments: JSON.stringify(argumentsValue)
        }]
      };
    };
    const result = await drive(injected, directory, request, {
      tools,
      steps: 8,
      allowedCommands: []
    });
    assert.equal(result.stop, 'UnboundSemanticRegressionPlan');
    assert.equal(fs.existsSync(path.join(directory, 'tests/range.test.mjs')), false);
    assert.equal(fs.existsSync(path.join(directory, 'tests/unrelated.test.mjs')), false);
  }
});
test('physical source drift after observation prevents any semantic creation', async context => {
  const directory = fixture(context);
  let changed = false;
  const injected = async (messages, offered) => {
    const plan = await planChatStep(messages, offered);
    if (!changed && plan?.kind === 'tool_calls' && plan.calls[0].tool === 'write') {
      changed = true;
      fs.appendFileSync(path.join(directory, 'src/range.mjs'), '// external source changed\n');
    }
    return plan;
  };
  const result = await drive(injected, directory, request, {
    tools,
    steps: 8,
    allowedCommands: []
  });
  const writes = result.transcript.filter(entry => entry.tool === 'write');
  assert.equal(writes.length, 1);
  assert.equal(writes[0].is_error, true);
  assert.equal(fs.existsSync(path.join(directory, 'tests/range.test.mjs')), false);
});
test('symbolic target parent never redirects creation or a target read outside workspace', async context => {
  const directory = fixture(context),
    outside = fs.mkdtempSync(path.join(os.tmpdir(), 'boundary-outside-'));
  context.after(() => fs.rmSync(outside, {
    recursive: true,
    force: true
  }));
  fs.symlinkSync(outside, path.join(directory, 'tests'), 'dir');
  const result = await drive(planChatStep, directory, request, {
    tools,
    steps: 8,
    allowedCommands: []
  });
  assert.equal(result.transcript.some(entry => entry.tool === 'write'), false);
  assert.equal(fs.existsSync(path.join(outside, 'range.test.mjs')), false);
});
