// Search/fetch below are synthetic. Files, process output and exit statuses are real.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { executeResult } from '../../../experiments/js_dogfood/drive.mjs';
import { stepOutcome, StepOutcome } from '../../../js/agentic/tool_result.mjs';
const fixtures = JSON.parse(readFileSync(new URL('../fixtures/coding-research/held-out.json', import.meta.url)));
const supported = fixtures.filter(fixture => fixture.language === 'javascript' || Number(process.versions.node.split('.')[0]) >= 22);
const tools = ['read', 'write', 'bash', 'web_search', 'web_fetch'];
async function session(workspace, fixture) {
  await installNodeHost(new WorkerHost({ WebAssembly: {
    instantiate() { throw Error('LocalWasmExecutionForbidden'); },
    compile() { throw Error('LocalWasmExecutionForbidden'); },
  } }));
  const messages = [{ role: 'user', content: fixture.prompt }];
  const observations = [];
  for (let rung = 0; rung < 12; rung++) {
    const plan = await planChatStepResolved(messages, tools);
    assert(plan, 'production planner must return a plan');
    if (plan.kind === 'final') return { plan, observations };
    assert.equal(plan.calls.length, 1);
    const item = plan.calls[0], args = JSON.parse(item.arguments), id = 'real-effect-' + rung;
    let receipt;
    if (item.tool === 'web_search') {
      receipt = { content: fixture.url };
      observations.push({ transport: 'synthetic-search', query: args.query });
    } else if (item.tool === 'web_fetch') {
      assert.equal(args.url, fixture.url);
      receipt = { content: '<!doctype html><html><body><pre><code class="language-' + fixture.language + '">'
        + fixture.source + '</code></pre></body></html>' };
      observations.push({ transport: 'synthetic-fetch', url: args.url });
    } else {
      assert(['read', 'write', 'bash'].includes(item.tool));
      if (item.tool === 'bash') assert.equal(args.command, 'node --test acceptance.test.mjs');
      if (item.tool === 'write') {
        assert.equal(args.path ?? args.filePath ?? args.file_path, fixture.module);
        assert.equal(args.content, fixture.expected);
      }
      // NODE_TEST_CONTEXT belongs to this outer test runner, not the independent child test.
      const outerContext = process.env.NODE_TEST_CONTEXT;
      try {
        if (item.tool === 'bash') delete process.env.NODE_TEST_CONTEXT;
        receipt = executeResult(workspace, item);
      } finally {
        if (outerContext === undefined) delete process.env.NODE_TEST_CONTEXT;
        else process.env.NODE_TEST_CONTEXT = outerContext;
      }
      observations.push({ tool: item.tool, arguments: item.arguments, receipt });
    }
    messages.push({ role: 'assistant', content: '', tool_calls: [
      { id, type: 'function', function: { name: item.tool, arguments: item.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, name: item.tool, ...receipt });
  }
  assert.fail('bounded session exhausted');
}
for (const fixture of supported) {
  test(fixture.name + ': actual files and independent Node process verify a proposal; cold second session performs no source write', async () => {
    const workspace = mkdtempSync(join(tmpdir(), 'formal-cold-source-e2e-'));
    try {
      writeFileSync(join(workspace, 'package.json'), '{"type":"module"}\n');
      writeFileSync(join(workspace, 'acceptance.test.mjs'), fixture.acceptance);
      const first = await session(workspace, fixture);
      assert.equal(first.observations.filter(item => item.tool === 'write').length, 1);
      const verification = first.observations.filter(item => item.tool === 'bash');
      assert.equal(verification.length, 1);
      assert.equal(stepOutcome(verification[0].receipt.content), StepOutcome.Succeeded,
        verification[0].receipt.content);
      assert.match(verification[0].receipt.content, /^(?:#|ℹ) tests 1$/mu);
      assert.match(verification[0].receipt.content, /^(?:#|ℹ) pass 1$/mu);
      assert.equal(readFileSync(join(workspace, fixture.module), 'utf8'), fixture.expected);
      assert.equal(readFileSync(join(workspace, 'acceptance.test.mjs'), 'utf8'), fixture.acceptance);
      const before = statSync(join(workspace, fixture.module)).mtimeMs;
      const second = await session(workspace, fixture);
      assert.equal(second.observations.filter(item => item.transport === 'synthetic-search').length, 1);
      assert.equal(second.observations.filter(item => item.transport === 'synthetic-fetch').length, 1);
      assert.equal(second.observations.filter(item => item.tool === 'write').length, 0);
      const again = second.observations.filter(item => item.tool === 'bash');
      assert.equal(again.length, 1);
      assert.equal(stepOutcome(again[0].receipt.content), StepOutcome.Succeeded);
      assert.equal(second.plan.result?.origin, 'coding-research-verified-existing');
      assert.equal(second.plan.research?.verified, true);
      assert.equal(statSync(join(workspace, fixture.module)).mtimeMs, before);
      assert.equal(readFileSync(join(workspace, fixture.module), 'utf8'), fixture.expected);
      assert.equal(readFileSync(join(workspace, 'acceptance.test.mjs'), 'utf8'), fixture.acceptance);
      const broken = fixture.acceptance.replace(/, ('[^']*')\);/u, ", 'deliberate-independent-mismatch');");
      assert.notEqual(broken, fixture.acceptance);
      writeFileSync(join(workspace, 'acceptance.test.mjs'), broken);
      const negative = await session(workspace, fixture);
      const rejected = negative.observations.filter(item => item.tool === 'bash');
      assert.equal(rejected.length, 1);
      assert.equal(stepOutcome(rejected[0].receipt.content), StepOutcome.Failed);
      assert.match(rejected[0].receipt.content, /^(?:#|ℹ) fail 1$/mu);
      assert.equal(negative.plan.result?.disposition, 'failure', JSON.stringify(negative.plan));
      assert.equal(negative.plan.research?.verified, false);
      assert.equal(negative.observations.filter(item => item.tool === 'write').length, 0);
      assert.equal(readFileSync(join(workspace, fixture.module), 'utf8'), fixture.expected);
      assert.equal(statSync(join(workspace, fixture.module)).mtimeMs, before);
      const actualRuns = [...verification, ...again, ...rejected];
      console.log('research_real_effects ' + JSON.stringify({fixture:fixture.name,
        httpTransport:'synthetic', processAdapter:'maintained executeResult/shellCapture',
        outcomes:actualRuns.map(item => stepOutcome(item.receipt.content)),
        writes:[first,second,negative].map(run => run.observations.filter(item => item.tool === 'write').length),
        sourceMtimeUnchanged:statSync(join(workspace,fixture.module)).mtimeMs === before}));
    } finally { rmSync(workspace, { recursive: true, force: true }); }
  });
}
