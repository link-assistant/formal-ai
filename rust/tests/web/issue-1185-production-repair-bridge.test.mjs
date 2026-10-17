import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import * as hostApi from '../../../js/agentic/host.mjs';
const { host, installHost } = hostApi;
const validateProgram = (...args) => typeof hostApi.validateProgram === 'function' ? hostApi.validateProgram(...args) : null;
import { validateRepairProgram } from '../../../js/server/repair-program-syntax.mjs';
import { planSymbolicCommandReroute } from '../../../js/agentic/command_reroute.mjs';
import { attemptsFrom, failedStep } from '../../../js/agentic/repair_loop.mjs';

const fixtures = JSON.parse(readFileSync(new URL('../fixtures/discovery-repair/held-out.json', import.meta.url)));
const tools = ['write_file', 'bash', 'web_search', 'web_fetch'];
const call = (id, name, args) => ({ role: 'assistant', content: '',
  tool_calls: [{ id, type: 'function', function: { name, arguments: args } }] });
const result = (id, name, content) => ({ role: 'tool', content, tool_call_id: id, name });
async function cold() {
  await installNodeHost(new WorkerHost({ WebAssembly: {
    instantiate() { throw Error('LocalWasmExecutionForbidden'); },
    compile() { throw Error('LocalWasmExecutionForbidden'); },
  } }));
}
function session(fixture, fix = fixture.fix) {
  const recipe = { language: fixture.language, path: fixture.path, source: fixture.source,
    supporting_files: [], commands: [fixture.command] };
  const answer = { intent: 'write_program', answer: 'Source-backed work item', execution_recipe: recipe };
  const messages = [{ role: 'user', content: 'Implement the supplied work item and verify its generated artifact.' }];
  let sequence = 0;
  const next = () => planSymbolicCommandReroute(messages, tools, answer);
  function observe(plan, output) {
    assert.equal(plan.calls.length, 1);
    const item = plan.calls[0], id = 'observed-' + sequence++;
    messages.push(call(id, item.tool, item.arguments), result(id, item.tool, output));
    return item;
  }
  const source = next();
  assert.equal(source.calls[0].tool, 'write_file');
  observe(source, 'Output: written\nExit Code: 0');
  const run = next();assert.equal(run.calls[0].tool, 'bash');
  observe(run, 'Output:\n' + fixture.diagnostic + '\nExit Code: 1');
  const search = next();assert.equal(search.calls[0].tool, 'web_search');
  observe(search, fixture.url);
  const fetch = next();assert.equal(fetch.calls[0].tool, 'web_fetch');
  observe(fetch, fixture.message + '\n```' + fixture.language + '\n' + fix + '\n```\n');
  const record = next();assert.equal(record.calls[0].tool, 'write_file');
  const document = JSON.parse(record.calls[0].arguments);
  assert.match(document.content, /repair_edit/u);
  assert(document.content.includes(fixture.url));
  observe(record, 'Output: recorded\nExit Code: 0');
  return { next, observe, messages, fixture, record: document };
}
for (const fixture of fixtures) {
  test(fixture.name + ': cold production route renders retrieved code before retry and requires a receipt', async () => {
    await cold();const current = session(fixture);
    const failure = failedStep(fixture.language, fixture.diagnostic,
      { artifact_path: fixture.path, failed_command: fixture.command, validate_program: validateProgram });
    assert.equal(attemptsFrom(current.messages, failure)[0].applied, false, 'recorded is not applied');
    const repair = current.next();assert.equal(repair.calls[0].tool, 'write_file');
    const candidate = JSON.parse(repair.calls[0].arguments);
    assert.equal(candidate.path, fixture.path);assert.equal(candidate.content, fixture.fix + '\n');
    assert.notEqual(candidate.content, fixture.source);
    current.observe(repair, 'Output: written\nExit Code: 0');
    assert.equal(attemptsFrom(current.messages, failure)[0].applied, true);
    assert.equal(attemptsFrom(current.messages, failure)[0].resolved, false);
    const retry = current.next();assert.equal(retry.calls[0].tool, 'bash');
    assert.equal(JSON.parse(retry.calls[0].arguments).command, fixture.command);
    current.observe(retry, 'Output: verified by external harness fixture\nExit Code: 0');
    assert.equal(attemptsFrom(current.messages, failure)[0].resolved, true);
    assert.equal(current.next().kind, 'final');
  });
  test(fixture.name + ': invalid fetched rendering produces no artifact write or unchanged retry', async () => {
    await cold();const current = session(fixture, 'export const invalid = ;');
    const stopped = current.next();assert.equal(stopped.kind, 'final');
    assert.match(stopped.answer, /syntax_invalid/u);
    assert(!stopped.calls?.length);
  });
}
test('unavailable grammar stays a named gap and never retries unchanged source', async () => {
  await cold();const installed = host();installHost({ ...installed, validateProgram: undefined });
  const current = session(fixtures[0]);const stopped = current.next();
  assert.equal(stopped.kind, 'final');assert.match(stopped.answer, /unvalidated/u);
});
test('an unrelated later write cannot be counted as the retained repair', async () => {
  await cold();const current = session(fixtures[0]);
  current.observe({ calls: [{ tool: 'write_file', arguments: JSON.stringify({ path: fixtures[0].path,
    content: 'export const unrelated = 991;\n' }) }] }, 'Output: written\nExit Code: 0');
  const stopped = current.next();assert.equal(stopped.kind, 'final');assert.match(stopped.answer, /repair_write_mismatch/u);
});
test('a failed repair write does not authorize a retry', async () => {
  await cold();const current = session(fixtures[0]);const repair = current.next();
  assert.equal(repair.calls[0].tool, 'write_file');
  current.observe(repair, 'Error: write permission denied');
  const next = current.next();assert(next.kind === 'final' || next.calls?.[0]?.tool === 'write_file');
  assert.notEqual(next.calls?.[0]?.tool, 'bash');
});
test('syntax parsing never evaluates retrieved JavaScript and does not auto-grant missing toolchains', async () => {
  await cold();delete globalThis.__repairFetchedCodeRan;
  assert.equal(validateRepairProgram('javascript', 'globalThis.__repairFetchedCodeRan = true; throw new Error("must not execute");'), true);
  assert.equal(globalThis.__repairFetchedCodeRan, undefined);
  assert.equal(validateRepairProgram('typescript', 'const heldOut: number = 17;'), true);
  assert.equal(validateRepairProgram('javascript', 'const heldOut: number = 17;'), false);
  assert.equal(validateRepairProgram('rust', 'fn main() {}'), null);
  const fixture = { ...fixtures[0], diagnostic: 'node: command not found', source: 'export const harmless = 1;\n' };
  const recipe = { language: fixture.language, source: fixture.source, path: fixture.path,
    supporting_files: [], commands: [fixture.command] };
  const messages = [{ role: 'user', content: 'Implement the supplied work item and verify its generated artifact.' }];
  let plan = planSymbolicCommandReroute(messages, tools, { execution_recipe: recipe });
  messages.push(call('s', 'write_file', plan.calls[0].arguments), result('s', 'write_file', 'written'));
  plan = planSymbolicCommandReroute(messages, tools, { execution_recipe: recipe });
  messages.push(call('r', 'bash', plan.calls[0].arguments), result('r', 'bash', 'Output: node: command not found\nExit Code: 127'));
  const stopped = planSymbolicCommandReroute(messages, tools, { execution_recipe: recipe });
  assert.equal(stopped.kind, 'final');assert.match(stopped.answer, /FORMAL_AI_INSTALL_GRANT=workspace/u);
});

test('a parsed but unchanged repair stops without a retry', async () => {
  await cold();
  const fixture = { ...fixtures[0], source: fixtures[0].fix + '\n' };
  const current = session(fixture);
  const stopped = current.next();
  assert.equal(stopped.kind, 'final');
  assert.match(stopped.answer, /unchanged_fix/u);
  assert(!stopped.calls?.length);
});

test('missing or unsupported parser import leaves Node host loadable with no validation grant', () => {
  const adapter = new URL('../../../js/server/repair-program-syntax.mjs', import.meta.url).href;
  const nodeHost = new URL('../../../js/agentic/node-host.mjs', import.meta.url).href;
  for (const failure of ['ERR_MODULE_NOT_FOUND', 'UnsupportedParserRuntime']) {
    const loader = 'export async function resolve(s,c,n){if(s === "@babel/parser")' +
      '{throw Object.assign(new Error("parser unavailable"),{code:' + JSON.stringify(failure) + '});}return n(s,c,n);}';
    const code = 'const syntax=await import(' + JSON.stringify(adapter) + ');' +
      'await import(' + JSON.stringify(nodeHost) + ');' +
      'console.log(JSON.stringify(syntax.validateRepairProgram("javascript","export const safe = 1;")));';
    const observed = spawnSync(process.execPath, [...process.execArgv,
      '--experimental-loader', 'data:text/javascript,' + encodeURIComponent(loader),
      '--input-type=module', '-e', code], { encoding: 'utf8', timeout: 15000, maxBuffer: 65536 });
    assert.equal(observed.status, 0, observed.stderr);
    assert.equal(observed.stdout.trim(), 'null');
  }
});
