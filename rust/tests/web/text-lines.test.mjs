import test from 'node:test';
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {rustLines} from '../../../js/logging/text-lines.mjs';
import {rustLines as publicRustLines, parseLinksNotation} from '../../../js/server/memory-store.mjs';
import {parseLinksQueryBody} from '../../../js/server/network.mjs';
import {summarizeProxyExchange} from '../../../js/server/dialog-log-summary.mjs';

test('line splitting retains empty, terminal LF, CRLF, lone CR and Unicode behavior', () => {
  const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/physical-lines/vectors.json', import.meta.url)));
  for (const row of fixture.cases) assert.deepEqual(rustLines(row.source), row.expected, row.id);
});
test('public memory export retains the canonical function name, arity and fresh allocations', () => {
  assert.strictEqual(publicRustLines, rustLines);
  assert.equal(publicRustLines.name, 'rustLines');
  assert.equal(publicRustLines.length, 1);
  const first = publicRustLines('a\n'), second = publicRustLines('a\n');
  assert.notStrictEqual(first, second);
  first.push('mutation');
  assert.deepEqual(second, ['a']);
  assert.deepEqual(new publicRustLines('b\n'), ['b']);
});
test('non-string errors and split getter evaluation preserve their original behavior', () => {
  for (const value of [null, undefined, false, 0, [], {}]) assert.throws(() => rustLines(value), TypeError);
  const failure = new Error('original accessor');
  assert.throws(() => rustLines({get split() { throw failure; }}), error => error === failure);
  const effects = [];
  const value = {get split() {
    effects.push('get split');
    return function(separator) {
      effects.push([separator, this === value]);
      return ['one\r', ''];
    };
  }};
  assert.deepEqual(rustLines(value), ['one']);
  assert.deepEqual(effects, ['get split', ['\n', true]]);
});
test('links query consumer retains CRLF envelope and final-line parsing', () => {
  assert.equal(parseLinksQueryBody('request\r\n  query "SELECT links"\r\n'), 'SELECT links');
  assert.equal(parseLinksQueryBody('query "SELECT last"'), 'SELECT last');
  assert.equal(parseLinksQueryBody('request\r\n  ignored "none"\r\n'), null);
});
test('memory parser and proxy SSE consumer retain physical CRLF records', () => {
  const parsed = parseLinksNotation('demo_memory\r\n  event "one"\r\n    role "user"\r\n    content "oak"\r\n');
  assert.equal(parsed.length, 1);
  assert.equal(parsed[0].id, 'one');
  assert.equal(parsed[0].content, 'oak');
  const body = 'data: {"choices":[{"delta":{"content":"oak"}}]}\r\n\r\ndata: [DONE]\r\n\r\n';
  const summary = summarizeProxyExchange('POST', '/v1/chat/completions', '{}', 200, 'text/event-stream', body);
  assert.equal(summary.response_content_preview, 'oak');
});
