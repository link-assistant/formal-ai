// Issue #1165 R1165-1: data/seed/coding-documentation-captures.lino is
// pre-cached source data, not a store of programs. Every capture's header
// names a byte-for-byte fixture and its SHA-256; every block row is what the
// page formalizer reads from those bytes, so the seed is re-derived here from
// the fixtures and must match byte for byte (with its rust/embedded mirror).
// The native twin is `documentation_captures_are_the_formalized_fixtures` in
// rust/tests/unit/issue_1165_discovery_production.rs.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';

import {
  EMBEDDED_PATH,
  SEED_PATH,
  captureHeaders,
  renderSeed,
} from '../../../scripts/generate-coding-documentation-captures.mjs';
import { REPO_ROOT, createWorkerContext, evaluate } from '../../../js/server/worker-host.mjs';

const context = createWorkerContext();
const seeded = evaluate(context, 'loadSeed()');
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), 'utf8');

test('the captures seed is the formalized fixtures, byte for byte', async () => {
  await seeded;
  const seed = read(SEED_PATH);
  const headers = captureHeaders(context, seed);
  assert.deepEqual(
    headers.map((header) => [header.language, header.url]),
    [
      ['rust', 'https://doc.rust-lang.org/book/ch01-02-hello-world.html'],
      ['go', 'https://go.dev/doc/tutorial/getting-started'],
      ['kotlin', 'https://kotlinlang.org/docs/command-line.html'],
      ['kotlin', 'https://kotlinlang.org/docs/kotlin-tour-hello-world.html'],
      ['scala', 'https://docs.scala-lang.org/scala3/book/taste-hello-world.html'],
    ],
  );
  assert.equal(renderSeed(context, headers), seed, 'regenerate with scripts/generate-coding-documentation-captures.mjs --write');
  assert.equal(read(EMBEDDED_PATH), seed, 'the embedded mirror is byte-identical');
});

test('the worker reads the captures the seed records', async () => {
  await seeded;
  const kotlin = evaluate(context, 'documentationCaptures("kotlin", "hello_world").map((capture) => capture.url)');
  assert.deepEqual([...kotlin], ['https://kotlinlang.org/docs/command-line.html', 'https://kotlinlang.org/docs/kotlin-tour-hello-world.html']);
  assert.equal(evaluate(context, 'documentationCaptures("python", "hello_world").length'), 0);
  assert.equal(evaluate(context, 'documentationRouteActive()'), true);
});
