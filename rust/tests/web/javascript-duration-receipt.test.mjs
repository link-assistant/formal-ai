import test from 'node:test';
import assert from 'node:assert/strict';
import { importDuration, byteSha } from '../../../scripts/import-javascript-test-duration.mjs';
const root = '/source';
const command = 'node --test /source/rust/tests/web/renamed-observation.test.mjs';
const request = { prompt: `Run ${command}`, allowedCommands: [command] };
const captured = text => ({ prompt: request.prompt, boundary: { allowedCommands: [command], denied: [] },
  ok: false, result: { transcript: [{ tool: 'bash', arguments: JSON.stringify({ command }), result: text }] } });
const tap = 'Output: TAP version 13\nok 1 - retained observable\n1..1\n# tests 1\n# pass 1\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n# duration_ms 1234.567\n\nExit Code: 0';
const table = 'javascript-test-durations\n  test "rust/tests/web/original.test.mjs"\n    seconds 7.1\n';
const bytes = value => Buffer.from(JSON.stringify(value));
const definition = Buffer.from('source-owned complete original test definition');
const apply = (text = tap, base = table) => importDuration(base, bytes(request), bytes(captured(text)), definition, root);
test('actual inner exit and complete TAP supply a local weight despite markerless outer false', () => {
  const result = apply();
  assert.equal(result.seconds, 1.234567);
  assert.ok(result.text.startsWith(table));
  assert.ok(result.text.includes(`source-sha256 ${byteSha(definition)}`));
  assert.ok(result.text.includes('runtime-source-closure Unknown'));
  assert.equal(apply(tap, result.text).text, result.text);
});
test('all failure, cancellation, skipped, todo and empty summaries refuse', () => {
  for (const key of ['fail', 'cancelled', 'skipped', 'todo']) assert.throws(() => apply(tap.replace(`# ${key} 0`, `# ${key} 1`)));
  assert.throws(() => apply(tap.replace('# tests 1', '# tests 0')));
  assert.throws(() => apply(tap.replace('Exit Code: 0', 'Exit Code: 1')));
  assert.throws(() => apply(tap.replace('ok 1', 'not ok 1')));
});
test('missing and duplicate measurements cannot be inferred', () => {
  assert.throws(() => apply(tap.replace('# duration_ms 1234.567\n', '')));
  assert.throws(() => apply(tap.replace('# duration_ms 1234.567', '# duration_ms 1234.567\n# duration_ms 9')));
  assert.throws(() => apply(tap, table + table.slice(table.indexOf('  test'))));
  assert.throws(() => apply(tap, apply().text.replace('seconds 1.234567', 'seconds 8')));
});
test('wrong request, multiple executions and outside-source commands refuse', () => {
  const receipt = captured(tap);
  receipt.result.transcript.push(receipt.result.transcript[0]);
  assert.throws(() => importDuration(table, bytes(request), bytes(receipt), definition, root));
  assert.throws(() => importDuration(table, bytes({ ...request, prompt: 'different' }), bytes(captured(tap)), definition, root));
  assert.throws(() => importDuration(table, bytes(request), bytes(captured(tap)), definition, '/different'));
});

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { runImport } from '../../../scripts/import-javascript-test-duration.mjs';
test('source/request byte guards and physical definition identity are mandatory on check and import', () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'duration-source-'));
  try {
    const source = path.join(workspace, 'rust/tests/web/renamed-observation.test.mjs');
    fs.mkdirSync(path.dirname(source), { recursive: true });
    fs.writeFileSync(source, definition);
    const closedCommand = `node --test ${source}`;
    const closedRequest = { prompt: `Run ${closedCommand}`, allowedCommands: [closedCommand] };
    const closedReceipt = { ...captured(tap), prompt: closedRequest.prompt,
      boundary: { allowedCommands: [closedCommand], denied: [] },
      result: { transcript: [{ tool: 'bash', arguments: JSON.stringify({ command: closedCommand }), result: tap }] } };
    const input = path.join(workspace, 'request.json'), output = path.join(workspace, 'receipt.json');
    fs.writeFileSync(input, bytes(closedRequest));
    fs.writeFileSync(output, bytes(closedReceipt));
    const tablePath = path.join(workspace, 'table.lino'), generated = path.join(workspace, 'generated.lino');
    fs.writeFileSync(tablePath, table);
    const pin = filename => ({ path: filename, sha256: byteSha(fs.readFileSync(filename)) });
    const config = { root: workspace, table: tablePath, output: generated,
      request: pin(input), receipt: pin(output), definition: pin(source) };
    assert.equal(runImport(config).seconds, 1.234567);
    const check = { ...config, table: generated };
    assert.equal(runImport(check, true).seconds, 1.234567);
    fs.appendFileSync(source, '\nchanged');
    assert.throws(() => runImport(check, true), /stale or unbound input/u);
    fs.writeFileSync(source, definition);
    const unrelated = path.join(workspace, 'unrelated.test.mjs');
    fs.writeFileSync(unrelated, definition);
    assert.throws(() => runImport({ ...check, definition: pin(unrelated) }, true), /definition path identity/u);
    fs.appendFileSync(input, ' ');
    assert.throws(() => runImport(check, true), /stale or unbound input/u);
  } finally { fs.rmSync(workspace, { recursive: true, force: true }); }
});

test('positive complete top-level TAP plan and test points are required', () => {
  assert.throws(() => apply(tap.replace('1..1\n', '')));
  assert.throws(() => apply(tap.replace('ok 1 - retained observable\n', '')));
  assert.throws(() => apply(tap.replace('1..1', '1..2')));
  assert.throws(() => apply(tap.replace('ok 1 -', 'ok 2 -')));
  assert.throws(() => apply(tap.replace('ok 1 - retained observable', 'ok 1 - retained observable # SKIP absent')));
});
test('unsafe integer counts cannot become equal by Number rounding', () => {
  assert.throws(() => apply(tap.replace('# tests 1', '# tests 9007199254740993')
    .replace('# pass 1', '# pass 9007199254740992')));
  assert.throws(() => apply(tap.replace('1..1', '1..9007199254740993')));
});
test('nested test totals are not confused with top-level completed plans', () => {
  const nested = tap.replace('ok 1 - retained observable',
    '    ok 1 - first nested\n    ok 2 - second nested\n    1..2\nok 1 - retained observable')
    .replace('# tests 1', '# tests 2').replace('# pass 1', '# pass 2');
  assert.equal(apply(nested).seconds, 1.234567);
});
