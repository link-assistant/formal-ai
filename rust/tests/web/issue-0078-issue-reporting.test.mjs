// Issue #78 (R115, R117, R118): the prefilled report keeps its memory note to
// one short paragraph with a link, the long upload walkthrough lives in
// docs/upload-memory.md, and that guide documents both ways to attach the
// export. Issue #187 (R219): the multilingual intent-coverage checker is a
// registered CI gate.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const read = (relative) => readFileSync(join(REPO_ROOT, relative), 'utf8');
const GUIDE = 'https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md';

describe('R115: the prefilled report body keeps a one-paragraph memory note', () => {
  const seed = read('data/seed/agent-info.lino');
  const note = /field issue_report_memory_note\n\s+value "(.*)"\n/.exec(seed)?.[1];

  test('the seeded note is one line that links the upload guide', () => {
    assert.ok(note, 'issue_report_memory_note is not seeded');
    assert.ok(!note.includes('\n'), 'the note spans several lines');
    assert.ok(note.length < 400, `the note is ${note.length} characters long`);
    assert.ok(note.includes(GUIDE), note);
  });

  test('the browser fallback carries the same note as the seed', () => {
    assert.ok(read('js/app/issue-reporting.jsx').includes(`issue_report_memory_note:\n    ${JSON.stringify(note)}`));
  });
});

describe('R117/R118: the upload guide holds the walkthrough and both options', () => {
  const guide = read('docs/upload-memory.md');

  test('the guide walks through each operating system', () => {
    for (const system of ['**macOS**', '**Windows**', '**Linux / WSL / macOS terminal**']) assert.ok(guide.includes(system), system);
  });

  test('step 3 offers a Gist or a .zip attachment', () => {
    const step = guide.slice(guide.indexOf('## Step 3'));
    assert.match(step, /### Option A — Upload as a GitHub Gist/);
    assert.match(step, /### Option B — Wrap the file in a `\.zip` and attach it/);
  });
});

describe('R219: CI fails when a multilingual feature matrix omits a language', () => {
  test('the intent-coverage checker is a registered web-stage gate', () => {
    const gate = read('data/meta/ci-gates/check-multilingual-intent-coverage.lino');
    assert.match(gate, /stage web/);
    assert.match(gate, /check:intent-coverage/);
    assert.match(read('rust/tests/e2e/package.json'), /"check:intent-coverage": "node scripts\/check-multilingual-intent-coverage\.mjs"/);
  });
});
