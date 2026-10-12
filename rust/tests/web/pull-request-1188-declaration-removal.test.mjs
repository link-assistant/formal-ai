// PR #1188 dogfooding (docs/case-studies/pull-request-1188/formal-ai-dogfood.md):
// `Delete the functions a, b and c from code_plans.js.` read the file and
// answered with the read output, because a removal had to quote its text. A
// removal that names functions (the seeded `coding_declaration_noun`) and one
// path now removes each function the file declares under one of the
// request's names (a seeded `function_declaration_keyword` right before the
// name), whole, with the comment and attribute lines directly above it.
// Twin of the Rust cases in rust/tests/unit/pull_request_1188_line_removal.rs.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const TOOLS = ['read', 'edit', 'bash', 'write'];

function execute(files, tool, args) {
  const path = args.filePath ?? args.file_path ?? args.path;
  if (tool === 'read') return files.get(path) ?? `Error: File not found: ${path}`;
  if (tool === 'edit') {
    files.set(path, files.get(path).replace(args.oldString, () => args.newString));
    return '';
  }
  if (tool === 'write') {
    files.set(path, args.content);
    return '';
  }
  const digest = /^sha256sum -- (\S+)$/u.exec(args.command ?? '');
  if (digest) return `${createHash('sha256').update(files.get(digest[1]) ?? '').digest('hex')}  ${digest[1]}\n`;
  return `Error: ${tool} is not simulated`;
}

async function drive(prompt, workspace) {
  const files = new Map(Object.entries(workspace));
  const messages = [{ role: 'user', content: prompt }];
  for (let step = 0; step < 6; step += 1) {
    const plan = await planChatStep(messages, TOOLS);
    if (!plan || plan.kind === 'final') return { files, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const id = `call_${step}`;
    const result = execute(files, call.tool, JSON.parse(call.arguments));
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { files, answer: null };
}

const JS = [
  '// helpers',
  '',
  '/**',
  ' * Double a value.',
  ' */',
  'function double(value) {',
  '  return value * 2;',
  '}',
  '',
  '/** Keep me. */',
  'function keep(value) {',
  '  return double(value);',
  '}',
  '',
  'export function triple(value) {',
  '  if (value) {',
  '    return value * 3;',
  '  }',
  '  return 0;',
  '}',
  '',
].join('\n');

const RUST = [
  'use std::fmt;',
  '',
  '/// Parse a header.',
  '#[must_use]',
  'pub fn parse_header(line: &str) -> Option<&str> {',
  '    line.strip_prefix("# ")',
  '}',
  '',
  'fn keep() {}',
  '',
].join('\n');

test('a named function goes whole, with its doc comment, and the blank line before the next one', async () => {
  const { files, answer } = await drive('Delete the function double from util.js.', { 'util.js': JS });
  assert.equal(files.get('util.js'), [
    '// helpers',
    '',
    '/** Keep me. */',
    'function keep(value) {',
    '  return double(value);',
    '}',
    '',
    'export function triple(value) {',
    '  if (value) {',
    '    return value * 3;',
    '  }',
    '  return 0;',
    '}',
    '',
  ].join('\n'));
  assert.equal(answer, 'Removed `double` from `util.js` and observed the result.');
});

test('several names are removed together and the answer names those the file declared', async () => {
  const { files, answer } = await drive('Delete the functions double, triple and missing from util.js.', { 'util.js': JS });
  assert.equal(files.get('util.js'), '// helpers\n\n/** Keep me. */\nfunction keep(value) {\n  return double(value);\n}\n');
  assert.equal(answer, 'Removed `double`, `triple` from `util.js` and observed the result.');
});

test('a Rust function goes with its doc comment and attributes, in the request language', async () => {
  const { files, answer } = await drive('Удали функцию parse_header из lib.rs.', { 'lib.rs': RUST });
  assert.equal(files.get('lib.rs'), 'use std::fmt;\n\nfn keep() {}\n');
  assert.equal(answer, 'Из `lib.rs` удалено `parse_header`, результат проверен.');
});

test('a name the file does not declare is not a removal anyone can verify', async () => {
  const { files, answer } = await drive('Delete the function absent from util.js.', { 'util.js': JS });
  assert.equal(files.get('util.js'), JS);
  assert.notEqual(answer, null);
  assert.ok(!answer.startsWith('Removed'), answer);
});
