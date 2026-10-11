import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import test from 'node:test';
import { execute } from '../../../experiments/js_dogfood/drive.mjs';
import { harnessReportedFailure, reportedExitCode, observedPayload } from '../../../js/agentic/tool_result.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

function fixture(context) {
  const parent = mkdtempSync(join(tmpdir(), 'formal-ai-driver-grep-'));
  const root = join(parent, 'workspace');
  mkdirSync(root);
  context.after(() => rmSync(parent, { recursive: true, force: true }));
  return { parent, root };
}

await installNodeHost(new WorkerHost());

const search = (root, args) => execute(root, { tool: 'grep', arguments: JSON.stringify(args) });

test('actual ripgrep accepts inline flags and preserves numbered matching evidence', (context) => {
  const { root } = fixture(context);
  writeFileSync(join(root, 'finding.txt'), 'clear\nTODO repair\npending review\n');
  assert.equal(search(root, { path: 'finding.txt', pattern: '(?i)todo|PENDING' }),
    'Found 2 matches\n' + join(root, 'finding.txt') + ':\n  Line 2: TODO repair\n  Line 3: pending review');
});

test('actual zero-match result is distinct from unavailable source and invalid search', (context) => {
  const { root } = fixture(context);
  writeFileSync(join(root, 'finding.txt'), 'clear\n');
  assert.equal(search(root, { path: 'finding.txt', pattern: 'TODO' }), 'No files found');
  assert.equal(JSON.parse(search(root, { path: 'absent.txt', pattern: 'TODO' })).is_error, true);
  assert.equal(JSON.parse(search(root, { path: 'finding.txt', pattern: '(' })).is_error, true);
});

test('search respects file inclusion and keeps colon-bearing path and text', (context) => {
  const { root } = fixture(context);
  writeFileSync(join(root, 'finding:part.mjs'), 'TODO: repair\n');
  writeFileSync(join(root, 'other.txt'), 'TODO: excluded\n');
  const result = search(root, { path: '.', include: '*.mjs', pattern: 'TODO:' });
  assert.equal(result, 'Found 1 matches\n' + join(root, 'finding:part.mjs') + ':\n  Line 1: TODO: repair');
});

test('sandbox sibling whose name shares the root prefix is still outside', (context) => {
  const { parent, root } = fixture(context);
  const sibling = join(parent, 'workspace-other');
  mkdirSync(sibling);
  writeFileSync(join(sibling, 'secret.txt'), 'TODO outside\n');
  const failure = JSON.parse(search(root, { path: join(sibling, 'secret.txt'), pattern: 'TODO' }));
  assert.equal(failure.is_error, true);
  assert.match(failure.error, /^path escapes the sandbox:/u);
});

test('failed adapter results are typed while literal error text stays successful file content', (context) => {
  const { root } = fixture(context);
  writeFileSync(join(root, 'diagnostic.txt'), 'Error: authored example\n');
  const unavailable = JSON.parse(execute(root, { tool: 'codesearch', arguments: '{}' }));
  assert.equal(unavailable.is_error, true);
  const missing = JSON.parse(execute(root, { tool: 'read', arguments: JSON.stringify({ path: 'missing.txt' }) }));
  assert.equal(missing.is_error, true);
  const authored = execute(root, { tool: 'read', arguments: JSON.stringify({ path: 'diagnostic.txt' }) });
  assert.match(authored, /00001\| Error: authored example/u);
  assert.equal(harnessReportedFailure(authored), false);
});

test('actual successful Bash keeps JSON error fields as output with explicit zero status', async (context) => {
  const { root } = fixture(context);
  await installNodeHost(new WorkerHost());
  const command = "node -e 'process.stdout.write(JSON.stringify({error:\"authored value\"}))'";
  const result = execute(root, { tool: 'bash', arguments: JSON.stringify({ command }) });
  assert.equal(reportedExitCode(result), 0);
  assert.equal(harnessReportedFailure(result), false);
  assert.ok(result.includes('Output: {"error":"authored value"}\nExit Code: 0'));
  assert.equal(observedPayload(result), JSON.stringify({ error: "authored value" }));
});

test('large matching line retains complete raw bytes and count without a pipe overflow', (context) => {
  const { root } = fixture(context);
  const text = 'TODO ' + 'λ'.repeat(9 * 1024 * 1024) + '\nTODO short\n';
  writeFileSync(join(root, 'large.txt'), text);
  const result = search(root, { path: 'large.txt', pattern: 'TODO' });
  assert.match(result, /^Found 2 matches\n/u);
  assert.match(result, /Line 2: TODO short/u);
  assert.match(result, /Results truncated: 1 of 2 matching lines rendered; 1 oversized JSON records omitted/u);
  const file = result.match(/^Complete ripgrep JSON: (.+)$/mu)?.[1];
  assert.ok(file);
  context.after(() => rmSync(dirname(file), { recursive: true, force: true }));
  const raw = readFileSync(file);
  assert.equal(result.match(/^SHA256: ([a-f0-9]{64})$/mu)?.[1], createHash('sha256').update(raw).digest('hex'));
  const records = raw.toString('utf8').trimEnd().split('\n').map(JSON.parse);
  assert.equal(records.find((record) => record.type === 'match').data.lines.text, text.split('\n')[0] + '\n');
});

test('many matching Unicode lines keep their real count in a bounded preview', (context) => {
  const { root } = fixture(context);
  writeFileSync(join(root, 'many.txt'), Array.from({ length: 200 }, (_, index) => 'TODO λ🙂 ' + index).join('\n'));
  const result = search(root, { path: 'many.txt', pattern: 'TODO' });
  assert.match(result, /^Found 200 matches\n/u);
  assert.match(result, /Line 1: TODO λ🙂 0/u);
  assert.match(result, /Results truncated: 80 of 200 matching lines rendered/u);
  const file = result.match(/^Complete ripgrep JSON: (.+)$/mu)?.[1];
  context.after(() => rmSync(dirname(file), { recursive: true, force: true }));
  assert.ok(result.length < 10000);
});
