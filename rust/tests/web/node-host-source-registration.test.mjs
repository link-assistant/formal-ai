import test from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host} from '../../../js/agentic/host.mjs';
import {installDefaultNodeSourceHost} from '../../../js/server/default-node-source-bootstrap.mjs';

test('public Node host registration connects the genuine constructor-owned source session', async () => {
  const realm = await installNodeHost(new WorkerHost());
  const installed = host();
  assert.equal(installed.realm, realm);
  assert.equal(installed.sourceOperation, installed.sourceSession.sourceOperation);
  assert.equal(installed.sourceSession.active(), false);
  assert.equal(installed.sourceOperation.hasContext(), false);
  assert.equal(typeof installed.solve, 'function');
  assert.equal(typeof installed.readText, 'function');
  assert.ok(Object.isFrozen(installed.sourceSession));
  assert.ok(Object.isFrozen(installed.sourceOperation));
});

test('receipt JSON and copied public operation methods do not obtain write or import authority', () => {
  const operation = host().sourceOperation;
  for (const receipt of [null, {}, {verified: true}, {status: 0, accepted_operation_sources: []}]) {
    assert.throws(() => operation.acceptedSources(receipt, 'request', 'command', 'workspace'),
      /MissingSourceOperationContext/);
  }
  assert.throws(() => operation.moduleURL('/foreign.mjs', '/parent.mjs'), /MissingSourceOperationContext/);
  assert.throws(() => operation.finalizedCandidateStatus({}, 'request', 'command', '.', 'result.mjs', 'hash'),
    /MissingSourceOperationContext/);
  assert.throws(() => ({...operation}).acceptedSources({}, 'request', 'command', 'workspace'),
    /MissingSourceOperationContext/);
});

test('incomplete requests preserve passthrough without source proof or filesystem preconditions', async () => {
  const session = host().sourceSession;
  let invocations = 0;
  for (const request of ['What is two plus two?', 'Read report.mjs', 'Add exported partial(value).']) {
    const result = await session.run({request, workspace: '/missing-unowned-workspace', tools: []}, () => {
      invocations++;
      assert.equal(host().sourceOperation.hasContext(), false);
      assert.throws(() => host().sourceOperation.acceptedSources({}, request, 'command', 'workspace'),
        /MissingSourceOperationContext/);
      return 'passthrough-result';
    });
    assert.equal(result, 'passthrough-result');
  }
  assert.equal(invocations, 3);
  assert.equal(session.active(), false);
});

test('bootstrap delegates one fresh Node registration and preserves the public realm API', async () => {
  const previous = host();
  const realm = await installDefaultNodeSourceHost(new WorkerHost());
  const installed = host();
  assert.equal(installed.realm, realm);
  assert.equal(installed.sourceOperation, installed.sourceSession.sourceOperation);
  assert.notEqual(installed.sourceSession, previous.sourceSession);
  assert.notEqual(installed.sourceOperation, previous.sourceOperation);
  assert.throws(() => previous.sourceOperation.acceptedSources({}, 'request', 'command', 'workspace'),
    /MissingSourceOperationContext/);
  assert.equal(installed.sourceOperation.hasContext(), false);
});

test('production argument projection accepts only declared canonical records without issuing authority', async () => {
  const {readFileSync} = await import('node:fs');
  const {lex, topLevelItems} = await import('../../../scripts/self-translation/lexer.mjs');
  const {readArguments} = await import('../../../js/agentic/workspace_change.mjs');
  const source = readFileSync(new URL('../../../js/server/node-source-session-host.mjs', import.meta.url), 'utf8');
  const declarations = topLevelItems(lex(source, 'JavaScript')).filter(item =>
    item.tokens[0]?.text === 'function' && item.tokens[1]?.text === 'declaredPhysicalReadOperand');
  assert.equal(declarations.length, 1);
  const declaration = source.slice(declarations[0].start, declarations[0].end);
  assert.equal(declarations[0].tokens[2].text, '(');
  assert.deepEqual(declarations[0].tokens.slice(3, 6).map(token => token.text),
    ['call', ',', 'argumentsValue']);
  // This exact source projection returns only an operand; no host, provider,
  // read receipt, process or write capability is installed or manufactured.
  const project = new Function('readArguments', 'return (' + declaration + ');')(readArguments);
  const invoke = argumentsText => project({arguments: argumentsText}, JSON.parse(argumentsText));
  for (const argumentsText of [readArguments('picker.mjs'), JSON.stringify({file_path: 'picker.mjs'})]) {
    assert.equal(invoke(argumentsText), 'picker.mjs');
  }
  for (const argumentsText of [
    '{}', 'null', '[]', '"picker.mjs"', '17',
    JSON.stringify({file_path: 'picker.mjs', extra: true}),
    JSON.stringify({file_path: 'picker.mjs', filePath: 'caption.mjs', path: 'picker.mjs'}),
    JSON.stringify({path: 'picker.mjs'}),
    '{"file_path":"caption.mjs","file_path":"picker.mjs"}',
    '{"__proto__":{"file_path":"picker.mjs"}}',
  ]) assert.throws(() => invoke(argumentsText), /UnknownPhysicalReadArguments/);
  const canonical = JSON.parse(readArguments('picker.mjs'));
  assert.throws(() => project({arguments: JSON.stringify({file_path: 'foreign.mjs'})}, canonical),
    /UnknownPhysicalReadArguments/);
  assert.equal(host().sourceOperation.hasContext(), false);
});

test('nested test environment remains unknown before creating any physical source operation', async () => {
  const {mkdtempSync, writeFileSync, existsSync, rmSync} = await import('node:fs');
  const {tmpdir} = await import('node:os');
  const {join} = await import('node:path');
  const {deriveCompleteSourceRequest} = await import('../../../js/agentic/module_function/complete-source-preflight.mjs');
  await installNodeHost(new WorkerHost());
  const installed = host();
  const request = 'In this selected-file scratch workspace, add exported renderChoice(selection) to report.mjs. '
    + 'Return the compact canonical summary of the candidate chosen by the selection, or null when no candidate is selected. '
    + 'Discover and compose existing exports from the observed picker.mjs and caption.mjs source modules; reuse their semantics through imports rather than copying their implementations. '
    + 'Read the source modules and destination before authoring. '
    + 'Produce a bounded repair plan identifying actual declaration spans/content identities, arguments/results, import bindings/dependencies, the optional-value guard and effects. '
    + 'Run the supplied report.test.mjs acceptance command, node --test report.test.mjs, and report the real result; do not modify source modules or tests, commit, or push. '
    + 'If safe composition cannot be established, record the exact missing capability before any output write.';
  assert.ok(deriveCompleteSourceRequest(request, installed.readText('data/seed/source-authoring-grammar.lino')));
  const directory = mkdtempSync(join(tmpdir(), 'node-source-nested-refusal-'));
  try {
    for (const file of ['picker.mjs', 'caption.mjs', 'report.test.mjs']) {
      writeFileSync(join(directory, file), 'export const observed = true;\n', {flag: 'wx'});
    }
    let invoked = false;
    await assert.rejects(installed.sourceSession.run({request, workspace: directory,
      tools: ['read', 'write', 'bash']}, () => {invoked = true;}), /nested Node test operation unknown/);
    assert.equal(invoked, false);
    assert.equal(existsSync(join(directory, 'report.mjs')), false);
    assert.equal(installed.sourceOperation.hasContext(), false);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});
