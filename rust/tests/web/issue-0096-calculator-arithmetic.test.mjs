// Issue #96 (R125): non-NSFW calculator prompts are added to the Rust example
// and to the demo-dialog simulator seed so the chat demo can exercise the
// arithmetic path in all four supported languages (en, ru, zh, hi).

import assert from 'node:assert/strict';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

test('R125: rust/examples/try_arithmetic.rs exists and is non-empty', () => {
  const path = join(REPO_ROOT, 'rust/examples/try_arithmetic.rs');
  assert.ok(existsSync(path), 'rust/examples/try_arithmetic.rs is missing');
  assert.ok(statSync(path).size > 0, 'rust/examples/try_arithmetic.rs is empty');
});

test('R125: data/seed/demo-dialogs.lino has calculator dialog entries for all four languages', () => {
  const dialogs = readFileSync(join(REPO_ROOT, 'data/seed/demo-dialogs.lino'), 'utf8');
  // The seed must carry at least four calculator dialogs, one per supported
  // language, all routed to the calculation intent.
  const calculationIntentMatches = [...dialogs.matchAll(/expected_intent\s+calculation/g)];
  assert.ok(
    calculationIntentMatches.length >= 4,
    `expected at least 4 calculation dialogs, found ${calculationIntentMatches.length}`,
  );
  // The four concrete entry names introduced by issue #96 must be present.
  for (const name of [
    'dialog_calculator_english',
    'dialog_calculator_russian',
    'dialog_calculator_chinese',
    'dialog_calculator_hindi',
  ]) {
    assert.ok(dialogs.includes(name), `demo-dialogs.lino is missing ${name}`);
  }
});
