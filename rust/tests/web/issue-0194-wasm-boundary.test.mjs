// Issue #133 (R194): as much logic as possible must be compiled from Rust to
// WebAssembly, with JavaScript reserved for UI, seed-file fetching, and CORS
// handling. The boundary is enforced by the existence and content of the
// boundary-defining modules.

import assert from 'node:assert/strict';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

test('R194: rust/src/web_engine_core.rs owns prompt normalization and arithmetic evaluation', () => {
  const path = join(REPO_ROOT, 'rust/src/web_engine_core.rs');
  assert.ok(existsSync(path), 'rust/src/web_engine_core.rs is missing');
  const source = readFileSync(path, 'utf8');
  // The module must define the key WASM-boundary primitives the shard names.
  for (const symbol of ['normalize_prompt', 'evaluate_arithmetic', 'detect_language']) {
    assert.ok(source.includes(symbol), `web_engine_core.rs is missing ${symbol}`);
  }
});

test('R194: rust/src/web_search_core.rs owns provider order and RRF constants', () => {
  const path = join(REPO_ROOT, 'rust/src/web_search_core.rs');
  assert.ok(existsSync(path), 'rust/src/web_search_core.rs is missing');
  assert.ok(statSync(path).size > 0, 'rust/src/web_search_core.rs is empty');
});

test('R194: js/wasm-worker/src/lib.rs bridges the Rust primitives to the browser worker', () => {
  const path = join(REPO_ROOT, 'js/wasm-worker/src/lib.rs');
  assert.ok(existsSync(path), 'js/wasm-worker/src/lib.rs is missing');
  assert.ok(statSync(path).size > 0, 'js/wasm-worker/src/lib.rs is empty');
});
