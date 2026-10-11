// PR #1188 G100: content that is one quoted literal is the literal, whatever
// the pair of quotes. A file created "with exactly this content: «…»" kept the
// guillemets as its first and last characters, on one line or several.
// PR #1188 G101: a fenced block's info string (```js) is not content, and the
// line break before the closing fence ends the last line. Twins:
// `clean_content` in rust/src/agentic_coding/write_request.rs and
// `fenced_body` in rust/src/agentic_coding/markdown_section.rs.
import { before, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let cleanContent;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ cleanContent } = await import('../../../js/agentic/write_request.mjs'));
});

test('one quoted literal in any pair of quotes is the content', () => {
  assert.equal(cleanContent('«first line\nsecond line»'), 'first line\nsecond line');
  assert.equal(cleanContent('«one line»'), 'one line');
  assert.equal(cleanContent('“quoted”.'), 'quoted');
  assert.equal(cleanContent("'hello'."), 'hello');
});

test('content that is more than one literal keeps its quotes', () => {
  assert.equal(cleanContent('«a» and «b»'), '«a» and «b»');
  assert.equal(cleanContent('say «hi» twice'), 'say «hi» twice');
});

test('a fenced block is its lines, without the info string', () => {
  assert.equal(cleanContent('```js\nexport const x = 1;\n```'), 'export const x = 1;\n');
  assert.equal(cleanContent('```\n  indented\n```'), '  indented\n');
  assert.equal(cleanContent('```one line```'), 'one line');
});
