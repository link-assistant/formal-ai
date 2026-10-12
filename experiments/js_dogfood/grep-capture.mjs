// Capture complete ripgrep output before rendering bounded previews.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { closeSync, mkdtempSync, openSync, readSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { StringDecoder } from 'node:string_decoder';

const MAXIMUM_RECORD_CHARACTERS = 256 * 1024;
const MAXIMUM_RENDERED_MATCHES = 80;
const MAXIMUM_LINE_CHARACTERS = 400;

/** Oversized JSON records remain in the raw capture; memory stays bounded. */
function renderCapture(file) {
  const descriptor = openSync(file, 'r');
  const buffer = Buffer.alloc(32 * 1024);
  const decoder = new StringDecoder('utf8');
  const digest = createHash('sha256');
  const groups = new Map();
  let pending = '';
  let oversized = false;
  let omittedRecords = 0;
  let previewClipped = false;
  let rendered = 0;
  let total = null;
  function accept(line) {
    if (!line) return;
    if (line.length > MAXIMUM_RECORD_CHARACTERS) { omittedRecords += 1; return; }
    const record = JSON.parse(line);
    if (record.type === 'summary') { total = record.data.stats.matched_lines; return; }
    if (record.type !== 'match') return;
    if (rendered >= MAXIMUM_RENDERED_MATCHES) { previewClipped = true; return; }
    const path = record.data.path.text ?? Buffer.from(record.data.path.bytes, 'base64').toString('utf8');
    const original = record.data.lines.text ?? Buffer.from(record.data.lines.bytes, 'base64').toString('utf8');
    const text = original.replace(/\r?\n$/u, '');
    const clipped = text.length > MAXIMUM_LINE_CHARACTERS;
    previewClipped ||= clipped;
    if (!groups.has(path)) groups.set(path, []);
    groups.get(path).push('  Line ' + record.data.line_number + ': ' + text.slice(0, MAXIMUM_LINE_CHARACTERS) + (clipped ? ' [line preview truncated]' : ''));
    rendered += 1;
  }
  function consume(text) {
    let rest = text;
    for (;;) {
      const end = rest.indexOf('\n');
      if (end < 0) break;
      if (oversized) { omittedRecords += 1; oversized = false; }
      else accept(pending + rest.slice(0, end));
      pending = '';
      rest = rest.slice(end + 1);
    }
    if (oversized) return;
    pending += rest;
    if (pending.length > MAXIMUM_RECORD_CHARACTERS) { pending = ''; oversized = true; }
  }
  try {
    for (;;) {
      const length = readSync(descriptor, buffer, 0, buffer.length, null);
      if (!length) break;
      digest.update(buffer.subarray(0, length));
      consume(decoder.write(buffer.subarray(0, length)));
    }
    consume(decoder.end());
    if (oversized) omittedRecords += 1;
    else accept(pending);
  } finally { closeSync(descriptor); }
  if (!Number.isSafeInteger(total)) throw new Error('ripgrep capture has no complete match-count summary');
  const truncated = previewClipped || omittedRecords > 0 || total !== rendered;
  const body = [...groups].map(([path, hits]) => [path + ':', ...hits].join('\n')).join('\n\n');
  return { total, rendered, body, truncated, omittedRecords, sha256: digest.digest('hex'), bytes: statSync(file).size };
}

/** Regular-file stdout avoids pipe capacity limits and preserves original bytes. */
export function grepCapture(argv) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-grep-capture-'));
  const file = join(directory, 'ripgrep.jsonl');
  const descriptor = openSync(file, 'wx', 0o600);
  let result;
  try { result = spawnSync('rg', argv, { stdio: ['ignore', descriptor, 'pipe'], encoding: 'utf8', maxBuffer: 1024 * 1024 }); }
  finally { closeSync(descriptor); }
  if (result.error || result.status !== 0) {
    rmSync(directory, { recursive: true, force: true });
    if (!result.error && result.status === 1) return 'No files found';
    throw result.error ?? new Error('ripgrep exited ' + result.status + ': ' + result.stderr);
  }
  try {
    const captured = renderCapture(file);
    if (!captured.total) { rmSync(directory, { recursive: true, force: true }); return 'No files found'; }
    const output = 'Found ' + captured.total + ' matches\n' + captured.body;
    if (!captured.truncated) { rmSync(directory, { recursive: true, force: true }); return output; }
    return output + '\n\nResults truncated: ' + captured.rendered + ' of ' + captured.total
      + ' matching lines rendered; ' + captured.omittedRecords + ' oversized JSON records omitted.'
      + '\nComplete ripgrep JSON: ' + file + '\nSHA256: ' + captured.sha256 + '\nBytes: ' + captured.bytes;
  } catch (error) { rmSync(directory, { recursive: true, force: true }); throw error; }
}
