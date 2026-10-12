// Identity/source-selection controls only; this does not prove runtime/profile parity.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { deserialize, serialize } from 'node:v8';
import { createHash } from 'node:crypto';
import { resolve, relative, isAbsolute } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const root = fileURLToPath(new URL('../../../', import.meta.url));
import { tokenize, testFunctions, specificationCases, caseOf } from '../../../scripts/lib/rust-specification-cases.mjs';
import { expression } from '../../../scripts/lib/rust-specification-values.mjs';
const baselineFile = resolve(root, 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8');
const hash = value => createHash('sha256').update(value).digest('hex');
const canonicalPrograms = cases => {
  const copy = structuredClone(cases);
  for (const item of copy) for (const fixture of item.program?.fixtures ?? []) {
    if (isAbsolute(fixture.file)) {
      const path = relative(root, fixture.file);
      assert.ok(!path.startsWith('..'), 'fixture escaped exact source root');
      fixture.file = 'repo:' + path;
    }
  }
  return copy;
};
for (const name of ['renamed_Москва_value', '東京_case', 'हिन्दी_case', 'αβ_case', '𐐀_name', 'e\u0301_case', '_case', 'ascii123', 'r#match', 'r#Москва', 'r#é_case']) {
  test('preserve complete renamed native identifier ' + name, () => {
    const records = testFunctions(tokenize(`#[test] fn ${name}(){ assert_eq!(1,1); }`));
    assert.equal(records.length, 1);
    assert.equal(records[0].name, name.replace(/^r#/u, '').normalize('NFC'));
    assert.deepEqual(records[0].body, tokenize('assert_eq!(1,1);'));
  });
}
for (const name of ['3name', 'emoji_🦄', 'join_\u200d_name', 'join_\u200c_name', 'mark_©', '\u0301bad', 'name-dash', 'name space', 'r#_', 'r#self', 'r#Self', 'r#super', 'r#crate']) {
  test('refuse unsupported header without silently losing declaration ' + name, () => {
    assert.throws(() => testFunctions(tokenize(`#[test] fn ${name}(){ assert!(true); }`)), /unsupported native test identifier\/header/u);
  });
}
test('NFC semantic names retain distinct source hashes and exact string spelling', () => {
  const decomposed = '#[test] fn cafe\u0301_case(){ assert_eq!("e\u0301", "é"); }';
  const composed = decomposed.replace('cafe\u0301_case', 'café_case');
  const left = testFunctions(tokenize(decomposed)),
    right = testFunctions(tokenize(composed));
  assert.equal(left[0].name, right[0].name);
  assert.notEqual(hash(decomposed), hash(composed));
  assert.deepEqual(left[0].body.filter(token => token.kind === 'string').map(token => token.text), ['e\u0301', 'é']);
});
const nativeSources = () => readdirSync(resolve(root, 'rust/tests/unit/specification'), {
  recursive: true
}).map(String).filter(name => name.endsWith('.rs')).sort().map(name => {
  const path = 'rust/tests/unit/specification/' + name;
  return {
    path,
    sha256: hash(readFileSync(resolve(root, path)))
  };
});
// Explicit review command refreshes a versioned witness; normal tests never rewrite it.
if (process.argv.includes('--refresh-reviewed-baseline')) {
  const sourceSha = hash(readFileSync(resolve(root, 'scripts/lib/rust-specification-cases.mjs')));
  assert.ok(process.argv.includes('--reviewed-reader-sha=' + sourceSha), 'explicit current reader source review required');
  const previous = readFileSync(baselineFile);
  assert.ok(process.argv.includes('--previous-baseline-sha=' + hash(previous)), 'exact previous governed witness required');
  const inventory = specificationCases(root);
  writeFileSync(baselineFile, serialize({
    inventory: {
      ...inventory,
      cases: canonicalPrograms(inventory.cases)
    },
    sourceManifest: nativeSources()
  }));
  process.exit(0);
}
const baseline = deserialize(readFileSync(baselineFile));
test('governed inventory preserves complete objects and authoritative native source', () => {
  const inventory = specificationCases(root);
  assert.deepEqual({
    inventory: {
      ...inventory,
      cases: canonicalPrograms(inventory.cases)
    },
    sourceManifest: nativeSources()
  }, baseline, 'native source or reader changed: review complete case delta and explicitly refresh governed witness');
  assert.equal(new Set(inventory.unsupported.map(item => item.id)).size, inventory.unsupported.length);
});

// Raw identifiers retain lexical spelling in expression tokens; identity-only fn names normalize separately.

for (const name of ['true', 'false', 'match', 'if', 'as']) test('unbound raw keyword identifier ' + name + ' cannot become a keyword/literal', () => {
  const tokens = tokenize('r#' + name);
  assert.equal(tokens[0].text, 'r#' + name);
  assert.throws(() => expression(tokens, {
    environment: new Map(),
    calls: new Map()
  }), /unbound native value r#/u);
});
for (const name of ['true', 'false']) test('declared raw ' + name + ' resolves its variable instead of boolean literal', () => {
  const value = expression(tokenize('r#' + name), {
    environment: new Map([['r#' + name, {
      type: 'boolean',
      origin: 'declared-binding'
    }]]),
    calls: new Map()
  });
  assert.equal(value.kind, 'reference');
  assert.equal(value.name, 'r#' + name);
  assert.equal(value.origin, 'declared-binding');
});
test('legacy offline config accepts only actual literal and refuses raw variable spelling', () => {
  const source = 'let solver = UniversalSolver::new(SolverConfig { offline: true, ..SolverConfig::default() }); let response = solver.solve(\"hello\"); assert_eq!(response.intent, \"greeting\");';
  assert.ok(caseOf(tokenize(source)).case);
  const refused = caseOf(tokenize(source.replace('offline: true', 'offline: r#true')));
  assert.equal(refused.case, undefined);
  assert.match(refused.reason, /let solver/);
});
