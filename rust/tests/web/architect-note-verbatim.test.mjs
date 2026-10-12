// R1188-U14: the 2026-10-08 vision is recorded verbatim as an architect note
// and folded into VISION.md. Every quoted message in the note is the owner's
// message byte for byte, as collect-user-messages.mjs collected it.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const NOTE = 'docs/architect-notes/2026-10-08-architecture-naming-ci-speed-and-text-understanding.md';
const MESSAGES = 'docs/case-studies/pull-request-1188/user-messages.md';

/** The owner messages of the collected file, by number. */
function messages() {
  const byNumber = new Map();
  for (const match of readFileSync(MESSAGES, 'utf8').matchAll(/^## (\d+)\. \S+\n[\s\S]*?```text\n([\s\S]*?)\n```/gmu)) {
    byNumber.set(Number(match[1]), match[2]);
  }
  return byNumber;
}

/** Each quote of the note: the message number it cites and its unquoted text. */
function quotes() {
  const found = [];
  const text = readFileSync(NOTE, 'utf8');
  for (const match of text.matchAll(/message (\d+) of `user-messages\.md`:\n\n((?:>.*\n?)+)/gu)) {
    const body = match[2].trimEnd().split('\n').map((line) => line.replace(/^> ?/u, '')).join('\n');
    found.push({ number: Number(match[1]), body });
  }
  return found;
}

test('every quote of the note is the cited owner message, byte for byte', () => {
  const owner = messages();
  const cited = quotes();
  assert.ok(cited.length >= 10, `the note quotes ${cited.length} messages`);
  for (const { number, body } of cited) assert.equal(body, owner.get(number), `message ${number}`);
});

test('VISION.md and the notes index link the note', () => {
  const name = NOTE.split('/').at(-1);
  assert.ok(readFileSync('VISION.md', 'utf8').includes(`docs/architect-notes/${name}`));
  assert.ok(readFileSync('docs/architect-notes/README.md', 'utf8').includes(`(${name})`));
});
