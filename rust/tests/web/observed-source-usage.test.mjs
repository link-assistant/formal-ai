import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parse } from '@babel/parser';
import { createHash } from 'node:crypto';
import { babelSourceFrontend } from '../../../scripts/lib/source-cst-babel.mjs';
import { readSyntaxRules } from '../../../js/meta/readable-source.js';
import { observedSourceUsage } from '../../../js/agentic/observed_source_usage.mjs';
const forbiddenWasm = () => {
  throw Error('LocalProductWasmForbidden');
};
const originalWasm = globalThis.WebAssembly;
if (process.env.CI !== 'true') globalThis.WebAssembly = {
  ...originalWasm,
  compile: forbiddenWasm,
  instantiate: forbiddenWasm,
  Module: forbiddenWasm,
  Instance: forbiddenWasm
};
test.after(() => {
  globalThis.WebAssembly = originalWasm;
});
const rules = readSyntaxRules(readFileSync('data/meta/source-model/syntax-rules.lino', 'utf8'));
const vectors = JSON.parse(readFileSync('data/benchmarks/fixtures/source-usage/vectors.json', 'utf8'));
const extract = row => observedSourceUsage({
  ...row,
  frontend: {
    parse: babelSourceFrontend({
      parse
    }, row.owner),
    rules
  }
});
function witnesses(result, source) {
  const bytes = Buffer.from(source);
  for (const item of [...result.imports, ...result.calls, ...result.references, ...result.gaps]) {
    if (!item.span) continue;
    const full = bytes.subarray(...item.span);
    assert.ok(full.toString().startsWith(item.source));
    assert.ok(Buffer.byteLength(item.source) <= 512);
  }
}
test('pure Babel actual complete CST produces import-linked syntax facts across independent variants', () => {
  for (const row of vectors.cases.filter(row => row.language === 'JavaScript')) {
    const result = extract(row);
    assert.equal(result.sourceIdentity, createHash('sha256').update(row.source).digest('hex'));
    assert.deepEqual(result.imports.map(item => item.module), row.modules, row.name);
    assert.deepEqual(result.imports.flatMap(item => item.bindings.map(binding => binding.local)), row.bindings, row.name);
    assert.deepEqual(result.calls.map(item => item.binding), row.calls, row.name);
    assert.deepEqual(result.calls.flatMap(item => item.options.map(option => option.name)), row.options, row.name);
    for (const kind of row.gapKinds) assert.ok(result.gaps.some(gap => gap.kind === kind), row.name + ': ' + kind);
    witnesses(result, row.source);
  }
});
test('original JS adapter facts name actual imports, calls and option fields without API semantics', () => {
  const owner = 'js/web-search-component-entry.js',
    source = readFileSync(owner, 'utf8');
  const result = extract({
    owner,
    source,
    language: 'JavaScript'
  });
  assert.ok(result.imports.some(item => item.module === '@link-assistant/web-search'));
  assert.ok(result.calls.some(item => item.binding === 'getDefaultProviderIds'));
  const merge = result.calls.find(item => item.binding === 'mergeResults');
  assert.ok(merge);
  assert.deepEqual(merge.options.map(item => item.name), ['strategy', 'rrfK', 'removeDuplicates']);
  assert.equal(result.scope, 'static-source-observations-only; no execution or external API semantics');
  witnesses(result, source);
});
test('escaped module literals and dynamic imports stay explicit unresolved evidence', () => {
  for (const source of ['import {invoke} from "mod\\u0075le"; invoke();', 'const module = import("dynamic-module");']) {
    const result = extract({
      owner: 'src/unseen.mjs',
      source,
      language: 'JavaScript'
    });
    assert.equal(result.calls.length, 0);
    assert.ok(result.gaps.length > 0);
  }
});
test('CI installed vendored Rust frontend matches shared syntax witnesses', {
  skip: process.env.CI !== 'true',
  timeout: 20000
}, async () => {
  const {
    installSourceUsageRustFrontend
  } = await import('../../../js/server/source-usage-rust-frontend.mjs');
  const {
    makeObservedSourceUsageOperation
  } = await import('../../../js/server/observed-source-usage.mjs');
  const frontend = await installSourceUsageRustFrontend();
  assert.equal(await installSourceUsageRustFrontend(), frontend);
  const operation = makeObservedSourceUsageOperation(frontend);
  for (const row of vectors.cases.filter(row => row.language === 'Rust')) {
    const result = operation({
      owner: row.owner,
      source: row.source,
      language: row.language
    });
    assert.equal(result.sourceIdentity, createHash('sha256').update(row.source).digest('hex'));
    assert.deepEqual(result.imports.map(item => item.module), row.modules, row.name);
    assert.deepEqual(result.imports.flatMap(item => item.bindings.map(binding => binding.local)), row.bindings, row.name);
    assert.deepEqual(result.calls.map(item => item.binding), row.calls, row.name);
    assert.deepEqual(result.calls.flatMap(item => item.options.map(option => option.name)), row.options, row.name);
    if (row.referenceRoots) assert.deepEqual(result.references.map(item => item.root), row.referenceRoots, row.name);
    for (const kind of row.gapKinds) assert.ok(result.gaps.some(gap => gap.kind === kind), row.name + ': ' + kind);
    assert.equal(result.scope, 'static-source-observations-only; no execution or external API semantics');
    witnesses(result, row.source);
  }
});
