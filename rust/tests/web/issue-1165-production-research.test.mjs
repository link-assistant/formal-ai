import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host, installHost } from '../../../js/agentic/host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { moduleFunctionRequest, signature } from '../../../js/agentic/module_function.mjs';
import { codingResearchQuery, deriveCodingProcedure, independentAcceptance, planCodingResearchStep } from '../../../js/agentic/coding_research.mjs';
import { readFileSync } from 'node:fs';
const fixtures = JSON.parse(readFileSync(new URL('../fixtures/coding-research/held-out.json', import.meta.url)));
const tools = ['read_file', 'write_file', 'bash', 'web_search', 'web_fetch'];
const call = (id, name, args) => ({ role: 'assistant', content: '', tool_calls: [
  { id, type: 'function', function: { name, arguments: args } }] });
const result = (id, name, content) => ({ role: 'tool', content, tool_call_id: id, name });
async function cold(fixture, source = fixture.source) {
  await installNodeHost(new WorkerHost({ WebAssembly: {
    instantiate() { throw Error('LocalWasmExecutionForbidden'); },
    compile() { throw Error('LocalWasmExecutionForbidden'); },
  } }));
  const messages = [{ role: 'user', content: fixture.prompt }];
  let sequence = 0;
  const next = () => planChatStepResolved(messages, tools);
  function observe(plan, output, metadata = null) {
    assert.equal(plan.kind, 'tool_calls', JSON.stringify(plan));
    assert.equal(plan.calls.length, 1);
    const item = plan.calls[0], id = 'observed-' + sequence++;
    const receipt = result(id, item.tool, output);
    if (metadata) receipt.source_read = metadata;
    messages.push(call(id, item.tool, item.arguments), receipt);
    return item;
  }
  async function throughFetch() {
    for (let rung = 0; rung < 6; rung++) {
      const plan = await next();
      const item = plan.calls?.[0];
      assert(item, JSON.stringify(plan));
      const args = JSON.parse(item.arguments);
      if (item.tool === 'read_file') {
        const path = args.path ?? args.filePath ?? args.file_path;
        const absent = path === fixture.module;
        observe(plan, absent ? 'ENOENT: file not found' : fixture.acceptance, absent
          ? {path, success:false, complete:false, format:'raw', error_code:'ENOENT', error:'ENOENT', exit_code:1}
          : {path, success:true, complete:true, format:'raw'});
      } else if (item.tool === 'web_search') {
        assert(!args.query.includes(fixture.prompt));
        assert(!args.query.includes('acceptance'));
        observe(plan, fixture.url);
      } else if (item.tool === 'web_fetch') {
        assert.equal(args.url, fixture.url);
        observe(plan, '<!doctype html><html><body><h1>Reference declarations</h1><pre><code class="language-' + fixture.language + '">' +
          source.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;') + '</code></pre></body></html>');
        return;
      } else assert.fail('unexpected pre-research tool: ' + JSON.stringify(item));
    }
    assert.fail('bounded discovery did not fetch');
  }
  return { messages, next, observe, throughFetch };
}
for (const fixture of fixtures) {
  test(fixture.name + ': cold actual planner searches, formalizes and rebinds held-out source before verification', async () => {
    const current = await cold(fixture);await current.throughFetch();
    const write = await current.next();
    assert.equal(write.calls?.[0].tool, 'write_file', JSON.stringify(write));
    const args = JSON.parse(write.calls[0].arguments);
    assert.equal(args.path, fixture.module);
    assert.equal(args.content, fixture.expected);
    assert.equal(write.research.status, 'proposal');
    assert.equal(write.research.verified, false);
    assert.equal(write.research.url, fixture.url);
    assert.match(write.research.sha256, /^[a-f0-9]{64}$/u);
    assert.equal(write.research.bindings[0].to, fixture.function);
    current.observe(write, 'written');
    const verify = await current.next();
    assert.equal(verify.calls?.[0].tool, 'bash', JSON.stringify(verify));
    assert.equal(JSON.parse(verify.calls[0].arguments).command, 'node --test acceptance.test.mjs');
    // Synthetic externally observed TAP fixture; actual process proof is in the E2E test.
    current.observe(verify, 'Output: TAP version 13\n# tests 1\n# pass 1\n# fail 0\n# skipped 0\n# cancelled 0\nExit Code: 0');
    const final = await current.next();
    assert.equal(final.kind, 'final');
    // A fresh empty request context must rediscover; no program-answer cache exists.
    const again = await cold(fixture);await again.throughFetch();
    assert.equal((await again.next()).calls?.[0].tool, 'write_file');
  });
}
test('side effects and invalid syntax in fetched code never authorize a write', async () => {
  for (const source of ['function unsafe(value) { return process.exit(); }',
    'function incomplete(value) { return +; }', 'globalThis.__fetchedExecuted = true; function good(value) { return value; }']) {
    const current = await cold(fixtures[0], source);await current.throughFetch();
    const stopped = await current.next();
    assert.equal(stopped.kind, 'final');
    assert.equal(stopped.result?.disposition, 'gap');
    assert.match(stopped.answer, /NoValidatedSourceProcedure/u);
    assert.equal(globalThis.__fetchedExecuted, undefined);
  }
});
test('missing parser, no matching code and incomplete acceptance remain honest gaps', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  installHost({ ...host(), validateProgram: undefined });
  assert.match((await current.next()).answer, /GrammarUnavailable/u);
  const request = moduleFunctionRequest(fixtures[0].prompt, { requireTestContract: false });
  assert.equal(deriveCodingProcedure(request, { codeBlocks: [{ language: 'python', text: 'pass' }] }), null);
  assert(!codingResearchQuery(request).includes('node --test'));
  assert(!codingResearchQuery(request).includes(fixtures[0].module));
});
test('a failed independent verification never becomes discovery success', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  current.observe(await current.next(), 'written');
  const verify = await current.next();
  current.observe(verify, 'Output:\nacceptance mismatch\nExit Code: 1');
  const stopped = await current.next();
  assert.equal(stopped.kind, 'final');
  assert.match(stopped.answer, /failed|exit code 1/iu);
});
test('a failed proposed write never authorizes execution', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  current.observe(await current.next(), 'Error: EACCES permission denied');
  const stopped = await current.next();
  assert.equal(stopped.kind, 'final');
  assert(!stopped.calls?.some(item => item.tool === 'bash'));
});

test('research bypasses generated test lookup only for explicit independent acceptance', async () => {
  await cold(fixtures[0]);
  assert.deepEqual(moduleFunctionRequest(fixtures[0].prompt),
    moduleFunctionRequest(fixtures[0].prompt, { requireTestContract: false }));
  assert.equal(moduleFunctionRequest(fixtures[1].prompt), null);
  assert.equal(moduleFunctionRequest(fixtures[1].prompt, { requireTestContract: false }).language, 'typescript');
  const request = moduleFunctionRequest(fixtures[0].prompt);
  assert.equal(independentAcceptance(request, fixtures[0].acceptance), true);
  assert.equal(independentAcceptance(request, "import {readFileSync} from 'node:fs'; console.log(readFileSync('frost.mjs','utf8'));"), false);
  assert.equal(independentAcceptance({ ...request, command: 'cat frost.mjs' }, fixtures[0].acceptance), false);
  assert.equal(independentAcceptance({ ...request, command: 'node --test' }, fixtures[0].acceptance), false);
  const missing = fixtures[0].prompt.replace(', add a test in acceptance.test.mjs, and run node --test acceptance.test.mjs.', '.');
  assert.match(planCodingResearchStep(missing, [{role:'user',content:missing}], tools).answer, /IndependentVerificationRequired/u);
  const other = fixtures[0].prompt.replaceAll('frost.mjs', 'frost.py');
  assert.match(planCodingResearchStep(other, [{role:'user',content:other}], tools).answer, /UnsupportedGrammar/u);
});
test('an ambiguous or interrupted verification receipt cannot claim success', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  current.observe(await current.next(), 'written');
  current.observe(await current.next(), 'Output: still running, no final process status');
  const stopped = await current.next();
  assert.equal(stopped.kind, 'final');
  assert.equal(stopped.result?.disposition, 'gap');
});

test('a fetched page from a different search query cannot authorize source emission', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  const search = current.messages.flatMap(message => message.tool_calls ?? [])
    .find(item => item.function.name === 'web_search');
  search.function.arguments = JSON.stringify({query:'unrelated stale search'});
  const next = await current.next();
  assert.equal(next.calls?.[0].tool, 'web_search');
  assert(!next.calls?.some(item => item.tool === 'write_file'));
});
test('ordinary supported module synthesis keeps its original read and write route', async () => {
  await cold(fixtures[0]);
  const prompt = 'Add a function product(a,b) that returns a*b to p.mjs, add a test for it to p.test.mjs, and run node --test p.test.mjs.';
  const messages = [{role:'user',content:prompt}];
  for (let rung=0;rung<3;rung++) {
    const plan = await planChatStepResolved(messages, tools);
    assert.equal(plan.kind, 'tool_calls', JSON.stringify(plan));
    const item = plan.calls[0];
    assert.equal(plan.research, undefined);
    if (item.tool === 'write_file') {
      assert.equal(JSON.parse(item.arguments).path, 'p.mjs');
      return;
    }
    assert.equal(item.tool, 'read_file');
    const args = JSON.parse(item.arguments), path = args.path ?? args.filePath ?? args.file_path;
    const receipt = result('ordinary-'+rung, item.tool, 'Error: ENOENT: file not found');
    receipt.source_read = {path,success:false,complete:false,format:'raw',error_code:'ENOENT',error:'ENOENT',exit_code:1};
    messages.push(call('ordinary-'+rung,item.tool,item.arguments),receipt);
  }
  assert.fail('ordinary synthesis did not emit its existing recipe');
});

test('unsupported acceptance token representation is a refusal, never an exception', async () => {
  await cold(fixtures[0]);
  const request = moduleFunctionRequest(fixtures[0].prompt);
  const escaped = "import { frost as \\u0066rost } from './frost.mjs'; import assert from 'node:assert/strict'; assert.equal(frost('x'), 'x');";
  assert.doesNotThrow(() => independentAcceptance(request, escaped));
  assert.equal(independentAcceptance(request, escaped), false);
});

test('zero-test or incomplete Node receipts cannot verify either a new or an existing researched procedure', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  const beforeWrite = current.messages.slice();
  current.observe(await current.next(), 'written');
  const run = await current.next(), item = run.calls[0];
  const outputs = [
    ['Output: \nExit Code: 0', 'VerificationReceiptIncomplete'],
    ['Output: TAP version 13\n# tests 0\n# pass 0\n# fail 0\n# skipped 0\nExit Code: 0', 'NoTestsExecuted'],
    ['Output: TAP version 13\n# tests 2\n# pass 0\n# fail 0\n# skipped 2\nExit Code: 0', 'NoTestsExecuted'],
    ['Output: TAP version 13\n# tests 1\n# pass 1\n# fail 1\n# skipped 0\nExit Code: 0', 'VerificationReceiptIncomplete'],
    ["Output: TAP version 13\n# tests 2\n# pass 1\n# fail 0\n# skipped 0\n# cancelled 0\nExit Code: 0", 'VerificationReceiptIncomplete'],
    ["Output: TAP version 13\n# tests 1\n# pass 2\n# fail 0\n# skipped 0\n# cancelled 0\nExit Code: 0", 'VerificationReceiptIncomplete'],
    ["Output: TAP version 13\n# tests 1\n# pass 1\n# pass 1\n# fail 0\n# skipped 0\nExit Code: 0", 'VerificationReceiptIncomplete'],
    ["Output: TAP version 13\n# tests 9007199254740992\n# pass 1\n# fail 0\n# skipped 0\nExit Code: 0", 'VerificationReceiptIncomplete'],
    ["Output: TAP version 13\n# tests 2\n# pass 1\n# fail 0\n# skipped 0\n# cancelled 1\nExit Code: 0", 'VerificationReceiptIncomplete'],
  ];
  const existing = structuredClone(beforeWrite);
  const read = existing.find(message => message.source_read?.path === fixtures[0].module);
  read.content = fixtures[0].expected;
  read.source_read = {path:fixtures[0].module,success:true,complete:true,format:'raw'};
  for (const [output, reason] of outputs) {
    for (const messages of [current.messages, existing]) {
      const observed = [...messages, call('tap-counter', item.tool, item.arguments), result('tap-counter', item.tool, output)];
      const stopped = await planChatStepResolved(observed, tools);
      assert.equal(stopped.kind, 'final', JSON.stringify(stopped));
      assert.equal(stopped.result?.disposition, 'gap', JSON.stringify(stopped));
      assert.match(stopped.answer, new RegExp(reason, 'u'));
      assert.notEqual(stopped.research?.verified, true);
    }
  }
});

test('a later source write outside the bounded pure grammar never authorizes execution', async () => {
  const current = await cold(fixtures[0]);await current.throughFetch();
  current.observe(await current.next(), 'written');
  current.messages.push(call('unrelated-source', 'write_file', JSON.stringify({path:fixtures[0].module,
    content:'globalThis.__unexpected = true; export function frost(value) { return value; }'})),
  result('unrelated-source', 'write_file', 'written'));
  const stopped = await current.next();
  assert.equal(stopped.kind, 'final');
  assert.match(stopped.answer, /UnsupportedRepairSource/u);
  assert(!stopped.calls?.some(item => item.tool === 'bash'));
});

test('exact literal arity includes zero-parameter greetings and refuses missing or extra arguments', async () => {
  await cold(fixtures[0]);
  const zero = fixtures.find(fixture => fixture.name === 'unseen-zero-arg-greeting');
  const zeroRequest = moduleFunctionRequest(zero.prompt);
  assert.deepEqual(signature('cairnSignal( )').parameters, []);
  assert.equal(signature('cairnSignal(,)'), null);
  assert.deepEqual(zeroRequest.parameters, []);
  assert.equal(independentAcceptance(zeroRequest, zero.acceptance), true);
  assert.equal(independentAcceptance(zeroRequest,
    zero.acceptance.replace('cairnSignal()', "cairnSignal('extra')")), false);
  const oneRequest = moduleFunctionRequest(fixtures[0].prompt);
  assert.equal(independentAcceptance(oneRequest,
    fixtures[0].acceptance.replace("frost('unseen-719')", 'frost()')), false);
  assert.equal(independentAcceptance(oneRequest,
    fixtures[0].acceptance.replace("frost('unseen-719')", "frost('unseen-719', 'extra')")), false);
  const two = { ...oneRequest, name:'paired', parameters:['left','right'], module:'paired.mjs' };
  const acceptance = "import {paired} from './paired.mjs'; import assert from 'node:assert/strict'; assert.equal(paired('first', 'second'), 'first');";
  assert.equal(independentAcceptance(two, acceptance), true);
  assert.equal(independentAcceptance(two, acceptance.replace("'first', 'second'", "'first'")), false);
  assert.equal(independentAcceptance(two, acceptance.replace("'first', 'second'", "'first', externalValue")), false);
});


test('actual research route preserves URL-standard eligibility before requesting a fetch', async () => {
  const vectors = JSON.parse(readFileSync(new URL('../fixtures/coding-research/url-standard.json', import.meta.url)));
  for (const vector of vectors) {
    const fixture = { ...fixtures[0], url: vector.url };
    const current = await cold(fixture);
    let observed = null;
    for (let rung = 0; rung < 8; rung++) {
      const plan = await current.next();
      if (plan.kind === 'final') {
        assert.equal(vector.fetch, false, vector.url);
        assert.equal(plan.result?.disposition, 'gap');
        assert.match(plan.answer, /NoValidatedSourceProcedure/u);
        observed = false;
        break;
      }
      assert.equal(plan.calls.length, 1);
      const item = plan.calls[0], args = JSON.parse(item.arguments);
      if (item.tool === 'web_fetch') {
        assert.equal(args.url, vector.url);
        observed = true;
        break;
      }
      if (item.tool === 'web_search') current.observe(plan, fixture.url);
      else {
        assert.equal(item.tool, 'read_file', 'URL classification cannot authorize a write or verification');
        const target = args.path ?? args.filePath ?? args.file_path;
        const absent = target === fixture.module;
        current.observe(plan, absent ? 'ENOENT: file not found' : fixture.acceptance, absent
          ? {path:target, success:false, complete:false, format:'raw', error_code:'ENOENT', error:'ENOENT', exit_code:1}
          : {path:target, success:true, complete:true, format:'raw'});
      }
    }
    assert.equal(observed, vector.fetch, vector.url);
  }
});
