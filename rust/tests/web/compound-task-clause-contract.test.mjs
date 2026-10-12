import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { parseLino } from '../../../js/server/lino.mjs';
import { compileCompoundTask } from '../../../js/agentic/compound_task/clause_contract.mjs';

const seed = fs.readFileSync(new URL('../../../data/seed/source-authoring-grammar.lino', import.meta.url), 'utf8');
const document = parseLino(seed);
function request({ source = 'src/config.rs', previous = 'PRIOR', replacement = 'NEXT', proof = '.proof/result.md' } = {}) {
  return [
    `In the file ${source}, rename the constant ${previous} to ${replacement}.`,
    'Change only that file and keep it valid Rust.',
    'This is recursive binary-tree node 1.2 at depth 2.',
    "Solve only this node's task in this fresh temporary repository.",
    'Its harness-evaluated completion criterion is: tracked_source_change.',
    `Apply the change to the tracked file \`${source}\` itself -- the file has to end up modified in the Git worktree, and nothing else may change.`,
    `Then create \`effects/result.lino\` with these exact field lines: \`node_path=1.2\`, \`node_depth=2\`, \`node_kind=leaf\`, and \`result=\` followed by at least four words that state the change you made and that contain the exact text ${replacement}.`,
    `Leave supporting evidence in ${proof}.`,
    'The first line must be exactly node_path=1.2 and the body must state the concrete result.',
    'The harness rejects proof without the separate Git effect.',
    'Use web research when it materially improves factual accuracy.',
    'Do not claim success without evidence.'
  ].join('\n');
}

test('whole source clauses retain immutable independent unresolved Needs', () => {
  const raw = request();
  const result = compileCompoundTask(raw, document);
  assert.ok(result);
  assert.equal(result.clauses.length, 12);
  assert.equal(result.unresolvedNeeds.length, 12);
  assert.equal(result.writeAuthority, false);
  assert.equal(result.compilation, 'pending');
  for (const clause of result.clauses) for (const capture of Object.values(clause.captures)) {
    if (capture.text !== undefined) assert.equal(raw.slice(...capture.span), capture.text);
  }
  assert.throws(() => { result.unresolvedNeeds.length = 0; }, TypeError);
  assert.throws(() => { result.clauses[0].span[0] = 10; }, TypeError);
});
for (const options of [
  { source: 'lib/settings.rs', previous: 'START', replacement: 'FINISH' },
  { source: 'nested/limits.rs', previous: 'threshold', replacement: 'next_threshold' }
]) test('variable technical source bindings ' + options.source, () => {
  const result = compileCompoundTask(request(options), document);
  assert.ok(result);
  assert.equal(result.destinations[0], options.source);
});
for (const prefix of ['Do not read. ', 'Do not write. ', 'Deploy first. ', 'An unknown prerequisite is required. ']) {
  test('independent leading Need is never consumed ' + prefix, () => {
    assert.equal(compileCompoundTask(prefix + request(), document), null);
  });
}
for (const tail of [' Deploy remotely.', ' An unknown prerequisite remains.', ' Read private.txt first.']) {
  test('unconsumed outside tail is never dropped ' + tail, () => {
    assert.equal(compileCompoundTask(request() + tail, document), null);
  });
}
test('destination collision and traversal remain refused', () => {
  assert.equal(compileCompoundTask(request({ proof: 'src/config.rs' }), document), null);
  assert.equal(compileCompoundTask(request({ source: '../escape.rs' }), document), null);
});
test('duplicate grammar ownership remains refused', () => {
  assert.equal(compileCompoundTask(request(), parseLino(seed + '\ncompound-task-grammar\n')), null);
});

test('unknown technical criterion and cardinal remain unowned', () => {
  assert.equal(compileCompoundTask(request().replace('tracked_source_change', 'publish_repository'), document), null);
  assert.equal(compileCompoundTask(request().replace('at least four words', 'at least unknowable words'), document), null);
});
