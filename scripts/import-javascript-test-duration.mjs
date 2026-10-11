// Import captured single-file Node TAP observations; local weights grant no source authority.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
export const byteSha = bytes => createHash('sha256').update(bytes).digest('hex');

export function importDuration(table, requestBytes, receiptBytes, definitionBytes, root) {
  const request = JSON.parse(requestBytes), receipt = JSON.parse(receiptBytes);
  assert.equal(receipt.prompt, request.prompt, 'request identity');
  const transcript = receipt.result?.transcript;
  assert.equal(transcript?.length, 1, 'one captured execution required');
  const tool = transcript[0];
  assert.equal(tool.tool, 'bash');
  const command = JSON.parse(tool.arguments).command;
  const match = /^node --test (\/[^\s]+\.test\.mjs)$/u.exec(command);
  assert.ok(match, 'closed single-file Node test command required');
  assert.deepEqual(request.allowedCommands, [command]);
  assert.deepEqual(receipt.boundary?.allowedCommands, [command]);
  assert.deepEqual(receipt.boundary?.denied, []);
  assert.equal(request.prompt, `Run ${command}`);
  const relative = path.relative(root, match[1]).split(path.sep).join('/');
  assert.ok(/^rust\/tests\/web\/[^/]+\.test\.mjs$/u.test(relative), 'owned top-level test definition');
  const output = tool.result;
  assert.equal(typeof output, 'string');
  assert.match(output, /^Output: TAP version 13\n/u);
  assert.match(output, /\nExit Code: 0\s*$/u);
  assert.doesNotMatch(output, /^not ok /mu);
  const single = (pattern, integer = false) => {
    const matches = [...output.matchAll(pattern)];
    assert.equal(matches.length, 1, 'missing or duplicate TAP summary');
    const value = Number(matches[0][1]);
    assert.ok(Number.isFinite(value));
    if (integer) assert.ok(Number.isSafeInteger(value), "unsafe integer count");
    return value;
  };
  const total = single(/^# tests (\d+)$/gmu, true);
  assert.ok(total > 0, 'empty execution is not measurement');
  assert.equal(single(/^# pass (\d+)$/gmu, true), total);
  for (const name of ['fail', 'cancelled', 'skipped', 'todo']) {
    assert.equal(single(new RegExp(`^# ${name} (\\d+)$`, 'gmu'), true), 0);
  }
  const milliseconds = single(/^# duration_ms (\d+(?:\.\d+)?)$/gmu);
  assert.ok(Number.isFinite(milliseconds) && milliseconds > 0);
  const plan = single(/^1\.\.(\d+)$/gmu, true);
  assert.ok(plan > 0 && total >= plan, "positive complete top-level plan");
  const points = [...output.matchAll(/^ok (\d+)(?: - .*)?$/gmu)];
  assert.equal(points.length, plan, "missing or duplicate top-level test point");
  for (const [index, point] of points.entries()) {
    const id = Number(point[1]);
    assert.ok(Number.isSafeInteger(id));
    assert.equal(id, index + 1, "contiguous completed test points");
    assert.doesNotMatch(point[0], /\s+#\s*(?:skip|todo)\b/iu);
  }
  const seconds = milliseconds / 1000;
  const rows = [...table.matchAll(/^  test "([^"]+)"\n([\s\S]*?)(?=^  test |$(?![\s\S]))/gmu)];
  assert.equal(new Set(rows.map(row => row[1])).size, rows.length, 'duplicate existing duration');
  const block = `  test "${relative}"\n    seconds ${seconds}\n    receipt-sha256 ${byteSha(receiptBytes)}\n    request-sha256 ${byteSha(requestBytes)}\n    source-sha256 ${byteSha(definitionBytes)}\n    measurement-kind local-single-file\n    runtime-source-closure Unknown\n    execution-source-authentication Unknown\n    definition-binding import-time-only\n`;
  const existing = rows.find(row => row[1] === relative);
  if (existing) {
    assert.equal(existing[0], block, 'existing measurement is not this exact receipt and definition');
    return { text: table, relative, seconds, runtimeSourceClosure: 'Unknown' };
  }
  assert.ok(table.endsWith('\n'), 'preserve original final newline');
  return { text: table + block, relative, seconds, runtimeSourceClosure: 'Unknown' };
}

export function runImport(config, check = false) {
  const pinned = entry => {
    const bytes = fs.readFileSync(entry.path);
    assert.equal(byteSha(bytes), entry.sha256, 'stale or unbound input');
    return bytes;
  };
  const request = pinned(config.request), receipt = pinned(config.receipt);
  const definition = pinned(config.definition);
  const command = JSON.parse(JSON.parse(receipt).result.transcript[0].arguments).command;
  assert.equal(config.definition.path, command.slice("node --test ".length), "definition path identity");
  const table = fs.readFileSync(config.table, 'utf8');
  const result = importDuration(table, request, receipt, definition, config.root);
  if (check) assert.equal(table, result.text, 'measured duration is absent or stale');
  else fs.writeFileSync(config.output, result.text);
  return { file: result.relative, seconds: result.seconds, local: true,
    runtimeSourceClosure: result.runtimeSourceClosure, hostedDeadlineAcceptance: 'Pending' };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  assert.ok(process.argv.length === 3 || (process.argv.length === 4 && process.argv[3] === '--check'));
  console.log(JSON.stringify(runImport(JSON.parse(fs.readFileSync(process.argv[2])), process.argv[3] === '--check')));
}
