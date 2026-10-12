// Literal payloads must remain data while Formal AI plans their writes.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planGeneralChangeStep } from '../../../js/agentic/general_execution.mjs';
import { composeGeneralChangePlan, hasAuthoritativeLiteralWrite } from '../../../js/agentic/general_planner.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
const fence = String.fromCharCode(96).repeat(3);
const openQuote = String.fromCharCode(171);
const closeQuote = String.fromCharCode(187);

async function observe(prompt, expected, initial = null, mutation = 'write') {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-literal-'));
  try {
    if (initial !== null) writeFileSync(join(directory, 'output.mjs'), initial);
    const result = await drive(planChatStep, directory, prompt, { steps: 12 });
    assert.equal(result.stop, 'final', JSON.stringify(result));
    assert.equal(readFileSync(join(directory, 'output.mjs'), 'utf8'), expected);
    assert.ok(result.transcript.some((entry) => entry.tool === mutation));
    assert.ok(!result.transcript.some((entry) => entry.tool === 'glob'));
    return result;
  } finally { rmSync(directory, { recursive: true, force: true }); }
}

const source = [
  "import { test } from 'node:test';",
  'const inventory = [];',
  "const actions = ['Copy source.mjs to target.mjs', 'Delete notes.tmp', 'Move a.mjs to b.mjs'];",
  '// When `input` arrives then `output` follows; this is source documentation.',
  '// task: words here belong to the payload.',
  '// Append this instruction to the end of src/rules.mjs.',
  '// retain these literal words, in JavaScript.',
  '',
].join(String.fromCharCode(10));

test('fenced creation preserves source with task labels, command cues and paths', async () => {
  await observe('Create output.mjs containing\n' + fence + 'javascript\n' + source + fence, source);
});

test('closed quoted whole-content consent replaces an existing source file', async () => {
  const prompt = 'Set the contents of output.mjs to ' + openQuote + source + closeQuote;
  await observe(prompt, source, 'export const previous = true;\n');
});

test('a plain replacement stays an edit even when its new literal names a file write', async () => {
  const replacement = 'Create another.mjs with inventory []';
  const prompt = 'In output.mjs replace ' + openQuote + 'old' + closeQuote + ' with ' + openQuote + replacement + closeQuote;
  assert.equal(hasAuthoritativeLiteralWrite(prompt), false);
  await observe(prompt, 'before ' + replacement + ' after\n', 'before old after\n', 'edit');
});

test('longer fences preserve payloads containing their own Markdown examples', async () => {
  const body = 'const example = ' + JSON.stringify(fence + 'text\ninside\n' + fence) + ';\n';
  for (const length of [4, 7]) {
    const outer = String.fromCharCode(96).repeat(length);
    await observe('Create output.mjs containing\n' + outer + 'javascript\n' + body + outer, body);
  }
});

test('literal verification reads failure narratives and compact JSON as authored data', async () => {
  for (const body of ['Error: this is the diagnostic example.\n', '{"z":"failed: recorded attempt","a":"fixture"}\n']) {
    const result = await observe('Set the contents of output.mjs to ' + openQuote + body + closeQuote, body, 'previous\n');
    assert.match(result.answer, /Completed the general change request/);
  }
});

test('explicit verification failure still vetoes exact literal output', () => {
  const body = 'Error: this is the diagnostic example.\n';
  const prompt = 'Set the contents of output.mjs to ' + openQuote + body + closeQuote;
  const plan = composeGeneralChangePlan(prompt);
  const call = { id: 'verify', type: 'function', function: { name: 'bash', arguments: JSON.stringify({ command: plan.verification_command }) } };
  const messages = [
    { role: 'user', content: prompt },
    { role: 'assistant', content: '', tool_calls: [call] },
    { role: 'tool', tool_call_id: 'verify', name: 'bash', content: 'Output: ' + body + 'Error: \nExit Code: 1' },
  ];
  const result = planGeneralChangeStep(messages, ['bash', 'write', 'read'], plan);
  assert.equal(result.kind, 'final');
  assert.match(result.answer, /exited with code 1/i);
  assert.doesNotMatch(result.answer, /Completed the general change request/);
});

test('exact-content Set forms consume seed leads while preserving the declared literal bytes', async () => {
  const body = 'First paragraph.\n\nWhen tasks arrive, preserve Copy a.mjs to b.mjs as quoted data.\n';
  for (const lead of ['exactly this content', 'with exactly this content', 'with content', 'contents', 'с точно таким содержанием', 'ठीक इसी सामग्री के साथ', '内容与以下完全相同', 'con exactamente este contenido']) {
    const result = await observe('Set the contents of output.mjs to ' + lead + ': ' + openQuote + body + closeQuote, body, 'previous\n');
    assert.match(result.answer, /terminal_state "executed"/);
  }
});

test('seed-like words inside quoted payloads and ordinary leading prose remain content', async () => {
  for (const body of ['exactly what I asked for', 'with content: literal words']) {
    await observe('Set the contents of output.mjs to ' + openQuote + body + closeQuote, body, 'previous\n');
  }
  await observe('Set the contents of output.mjs to exactly what I asked for', 'exactly what I asked for', 'previous\n');
});

test('G125: quoted edit payloads with skill cues stay data', async () => {
  const old = '// When `input` then `output`' + String.fromCharCode(10);
  const next = '// When `input` then `result`' + String.fromCharCode(10);
  await observe('In output.mjs replace ' + openQuote + old + closeQuote
    + ' with ' + openQuote + next + closeQuote, next, old, 'edit');
});

test('G125: genuine unquoted skill teaching keeps its guard', async () => {
  const result = await planChatStep([{ role: 'user', content: 'When `input` then `output`' }], ['read', 'write', 'edit', 'bash']);
  assert.equal(result, null);
});

test('G125: a malformed quoted replacement changes nothing', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-skill-quote-'));
  const initial = '// When `input` then `output`' + String.fromCharCode(10);
  try {
    writeFileSync(join(directory, 'output.mjs'), initial);
    const prompt = 'In output.mjs replace ' + openQuote + initial + closeQuote + ' with ' + openQuote + 'unterminated';
    const result = await drive(planChatStep, directory, prompt, { steps: 12 });
    assert.equal(readFileSync(join(directory, 'output.mjs'), 'utf8'), initial);
    assert.ok(!result.transcript.some((entry) => entry.tool === 'write' || entry.tool === 'edit'));
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
