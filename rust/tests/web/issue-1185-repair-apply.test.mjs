// Issue #1185 R3 apply half, JavaScript root: the `repair_edit` record is read
// back and rendered into the artifact's source, mirroring
// `repair_edit_record_renders_back_into_the_source` and
// `repair_loop_applies_the_recorded_fix_before_the_retry` in
// rust/tests/unit/agentic-coding/issue_1185_repair_apply.rs. This root has no CST engine, so
// the loop applies only with a validator supplied, and without one it goes on
// to the retry exactly as the Rust build does with `meta-language` off.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import {
  MAX_REPAIR_RUNGS, failedStep, formalizeDiagnostic, repairEditDocument, repairStep,
} from '../../../js/agentic/repair_loop.mjs';
import { parseRepairEdit, renderRepairEdit, spliceRepairEdit } from '../../../js/agentic/repair_apply.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const SOURCE = 'fn main() {\n    let left: f64 = 1.5;\n    let right: i32 = 2;\n    println!("{left}");\n    println!("{right}");\n    let total = left + right;\n    println!("{total}");\n}\n';
const REPORTED = 'error[E0308]: mismatched types\n --> src/main.rs:6:33\n  |\n6 |     let total = left + right;\n  |                          ^ expected `f64`, found `i32`\n';
const FIX = 'let total = left + f64::from(right);';
const RENDERED = 'fn main() {\n    let left: f64 = 1.5;\n    let right: i32 = 2;\n    println!("{left}");\n    println!("{right}");\n    let total = left + f64::from(right);\n    println!("{total}");\n}\n';

const record = (fix) => repairEditDocument('rust', formalizeDiagnostic('rust', REPORTED)[0], fix, 'https://example.org/e0308');

test('the repair_edit record reads back into its fields', () => {
  assert.deepEqual(parseRepairEdit(record(FIX)), {
    language: 'rust', file: 'src/main.rs', line: 6, error_code: 'E0308', retained: FIX,
  });
  assert.equal(parseRepairEdit(record(null)).retained, null);
  assert.equal(parseRepairEdit('formal_clause\n  quantifier forall\n'), null);
});

test('the record renders back into the source at the located line', () => {
  assert.deepEqual(spliceRepairEdit(parseRepairEdit(record(FIX)), SOURCE), { source: RENDERED });
  assert.deepEqual(renderRepairEdit(record(FIX), SOURCE, () => true), { source: RENDERED });
  const multiLine = 'let right = f64::from(right);\nlet total = left + right;';
  assert.equal(
    spliceRepairEdit(parseRepairEdit(record(multiLine)), SOURCE).source.split('\n').slice(5, 7).join('\n'),
    '    let right = f64::from(right);\n    let total = left + right;',
  );
});

test('every refusal is a named gap', () => {
  assert.deepEqual(renderRepairEdit('not a record', SOURCE, () => true), { gap: 'not_a_record' });
  assert.deepEqual(renderRepairEdit(record(null), SOURCE, () => true), { gap: 'no_retained_fix' });
  assert.deepEqual(renderRepairEdit(record(FIX), 'fn main() {}\n', () => true), { gap: 'line_out_of_range', line: 6, lines: 1 });
  assert.deepEqual(renderRepairEdit(record(FIX), SOURCE), { gap: 'unvalidated', language: 'rust' });
  assert.deepEqual(renderRepairEdit(record(FIX), SOURCE, () => false), { gap: 'syntax_invalid', language: 'rust' });
});

const call = (id, name, args) => ({
  role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name, arguments: args } }],
});
const result = (id, name, content) => ({ role: 'tool', content, tool_call_id: id, name });
const TOOLS = ['web_search', 'web_fetch', 'write_file', 'bash'];
const page = `The error E0308 mismatched types: convert the integer before adding it.\n\`\`\`rust\n${FIX}\n\`\`\`\n`;

function transcriptThroughTheRecord(failure) {
  const base = [
    { role: 'user', content: 'run the generated program and verify it' },
    call('c0', 'write_file', JSON.stringify({ path: 'src/main.rs', content: SOURCE })),
    result('c0', 'write_file', 'written'),
    call('c1', 'web_search', '{ "query": "rust E0308 mismatched types" }'),
    result('c1', 'web_search', 'https://example.org/e0308'),
    call('c2', 'web_fetch', '{ "url": "https://example.org/e0308" }'),
    result('c2', 'web_fetch', JSON.stringify({ content: page, exit_code: 0 })),
  ];
  const recorded = repairStep(base, TOOLS, failure, 0, MAX_REPAIR_RUNGS);
  assert.equal(recorded.kind, 'record_fix');
  assert.equal(recorded.plan.calls[0].tool, 'write_file');
  const args = recorded.plan.calls[0].arguments;
  return [...base, call('c3', 'write_file', args), result('c3', 'write_file', 'recorded')];
}

function failure(validate) {
  return failedStep('rust', REPORTED, {
    exit_code: 1, failed_command: 'rustc --edition 2021 src/main.rs', artifact_path: 'src/main.rs', validate_program: validate,
  });
}

test('with a validator the recorded fix is rendered into the artifact before the retry', () => {
  const validated = failure(() => true);
  const messages = transcriptThroughTheRecord(validated);
  const applied = repairStep(messages, TOOLS, validated, 0, MAX_REPAIR_RUNGS);
  assert.equal(applied.kind, 'apply_fix');
  assert.equal(applied.plan.calls[0].tool, 'write_file');
  const written = JSON.parse(applied.plan.calls[0].arguments);
  assert.equal(written.path, 'src/main.rs');
  assert.equal(written.content, RENDERED);
  const after = [...messages, call('c4', 'write_file', JSON.stringify({ path: 'src/main.rs', content: RENDERED })), result('c4', 'write_file', 'written')];
  assert.equal(repairStep(after, TOOLS, validated, 0, MAX_REPAIR_RUNGS).kind, 'retry');
});

test('without a CST engine the loop goes on to the retry and leaves the artifact untouched', () => {
  const unvalidated = failure(null);
  const messages = transcriptThroughTheRecord(unvalidated);
  assert.equal(repairStep(messages, TOOLS, unvalidated, 0, MAX_REPAIR_RUNGS).kind, 'retry');
});
