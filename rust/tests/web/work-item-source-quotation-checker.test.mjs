// Externally specified independent-grader regressions, not solver-authored code.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { checkGeneratedDocumentation } from '../../../scripts/work-item-qualification/source-quotation-claims.mjs';
import { observedManifestEvidence } from '../../../scripts/work-item-qualification/manifest-claims.mjs';
const original = '# Existing guide\n\nRetain these local notes.\n';
const owner = 'web/client.mjs',
  module = 'ribbon-core';
const quotation = 'readPacket(input,{mode:"strict",enabled:true})';
const source = 'import {decode as readPacket} from "ribbon-core";\nexport function load(input){return ' + quotation + ';}\n';
const digest = value => createHash('sha256').update(value).digest('hex');
function inputs(code = source, path = owner) {
  return [{
    path: 'package.json',
    content: '{"dependencies":{"ribbon-core":"2.7"}}'
  }, {
    path,
    content: code,
    sourceIdentity: digest(code)
  }];
}
function section({
  path = owner,
  imported = 'decode',
  from = module,
  quote = quotation,
  language = 'javascript'
} = {}) {
  return 'Source syntax observed in `' + path + '`:\n\nImported `' + imported + '` from `' + from + '` is called in `' + path + '`; the following excerpt records its syntax.\n\n```' + language + '\n' + quote + '\n```\n';
}
const table = ['<!-- formal-ai-manifest-start: package.json -->', 'Dependency declarations from `package.json`:', 'These are literal manifest declarations; they do not establish executable runtime use.', '| Dependency | Declared version | Dependency section | Target qualifier | Literal options |', '| --- | --- | --- | --- | --- |', '| `ribbon-core` | `2.7` | `dependencies` | `unqualified` | `{}` |', '<!-- formal-ai-manifest-end: package.json -->'].join('\n');
function grade(tail, files = inputs(), evidence = observedManifestEvidence(files)) {
  return checkGeneratedDocumentation(original + '\n' + tail, original, files, evidence);
}
const calls = result => result.claims.filter(row => row.kind === 'source-quotation');
test('independent table and aliased-call witnesses account for the generated tail without semantic credit', () => {
  const result = grade(table + '\n\n' + section());
  assert.equal(result.prefixPreserved, true);
  assert.equal(result.complete, false);
  assert.equal(result.semanticAcceptance, 'pending');
  assert.ok(result.claims.some(row => row.status === 'verified-literal-declaration'));
  const call = calls(result)[0];
  assert.equal(call.status, 'verified-static-binding');
  assert.equal(call.sourceIdentity, digest(source));
  assert.equal(call.witnesses[0].local, 'readPacket');
  assert.deepEqual(call.witnesses[0].options.map(row => [row.key, row.valueSource]), [['mode', '"strict"'], ['enabled', 'true']]);
  assert.equal(call.dependency.status, 'verified-literal-declaration');
  assert.ok(result.obligations.some(row => row.id === 'external-api-roles' && row.status === 'pending'));
  assert.ok(result.independence.includes('shared Babel grammar'));
});
test('wrong modules imported names and changed call options contradict frozen syntax', () => {
  for (const tail of [section({
    from: 'different-core'
  }), section({
    imported: 'invented'
  }), section({
    quote: quotation.replace('strict', 'unsafe')
  })]) {
    assert.ok(calls(grade(tail)).some(row => row.status === 'contradicted'));
  }
});
test('shadowed imports comment-only snippets and reassigned bindings never gain static binding credit', () => {
  const codes = ['import {decode as readPacket} from "ribbon-core";\nexport function load(readPacket,input){return ' + quotation + ';}\n', 'import {decode as readPacket} from "ribbon-core";\n// ' + quotation + '\nexport const untouched=1;\n', 'import {decode as readPacket} from "ribbon-core";\nreadPacket=other;\n' + quotation + ';\n'];
  for (const code of codes) assert.ok(calls(grade(section(), inputs(code))).every(row => row.status !== 'verified-static-binding'));
});
test('stale source identities and changed owners stay unknown', () => {
  const files = inputs();
  files[1].sourceIdentity = '0'.repeat(64);
  assert.equal(calls(grade(section(), files))[0].reason, 'frozen-source-identity-mismatch');
  assert.equal(calls(grade(section({
    path: 'web/other.mjs'
  })))[0].reason, 'frozen-source-unavailable');
});
test('nearest npm declaration wins and a Cargo name cannot establish a JavaScript package dependency', () => {
  const nested = [...inputs(), {
    path: 'web/package.json',
    content: '{"dependencies":{}}'
  }];
  const near = calls(grade(section(), nested))[0];
  assert.equal(near.status, 'verified-static-binding');
  assert.equal(near.dependency.status, 'unknown');
  assert.equal(near.dependency.manifest, 'web/package.json');
  const cargo = [{
    path: 'native/Project.toml',
    content: '[dependencies]\nribbon-core="2.7"\n'
  }, inputs()[1]];
  assert.equal(calls(grade(section(), cargo))[0].dependency.reason, 'nearest-npm-manifest-unavailable');
});
test('every unsupported added prose segment remains unknown and changing original prose is contradicted', () => {
  const result = grade(section() + '\nThis library is the persistent knowledge-store backend.\n');
  assert.ok(result.claims.some(row => row.reason === 'unsupported-added-segment' && row.status === 'unknown'));
  assert.equal(result.obligations.find(row => row.id === 'all-added-segments-accounted-for').status, 'unresolved');
  const changed = checkGeneratedDocumentation(original.replace('Retain', 'Delete') + section(), original, inputs(), observedManifestEvidence(inputs()));
  assert.equal(changed.prefixPreserved, false);
  assert.equal(changed.claims[0].status, 'contradicted');
});
test('Rust quotations preserve literal bytes while independent root resolution stays unknown', () => {
  const path = 'native/search.rs',
    quote = 'search::merge(query)',
    files = [{
      path,
      content: 'fn run(){' + quote + ';}\n'
    }];
  const tail = 'Source syntax observed in `' + path + '`:\n\nQualified source reference in `' + path + '`; module-root resolution is unverified.\n\n```rust\n' + quote + '\n```\n';
  const result = grade(tail, files);
  assert.equal(calls(result)[0].status, 'unknown');
  assert.equal(calls(result)[0].quotationVerified, true);
  assert.equal(result.complete, false);
});
test('namespace calls verify static member bindings but dynamic dispatch stays unsupported', () => {
  const code = 'import * as library from "ribbon-core";\nlibrary.decode(input);\nlibrary[method](input);\n';
  const good = grade(section({
    imported: '*',
    quote: 'library.decode(input)'
  }), inputs(code));
  assert.equal(calls(good)[0].status, 'verified-static-binding');
  assert.equal(calls(good)[0].witnesses[0].member, 'decode');
  assert.equal(calls(grade(section({
    imported: '*',
    quote: 'library[method](input)'
  }), inputs(code)))[0].status, 'unknown');
});
test('type-only imports and nonmatching fence languages never certify runtime binding', () => {
  const path = 'web/client.ts',
    code = 'import type {decode as readPacket} from "ribbon-core";\n' + quotation + ';\n';
  assert.ok(calls(grade(section({
    path,
    language: 'typescript'
  }), inputs(code, path))).every(row => row.status !== 'verified-static-binding'));
  assert.ok(grade(section({
    language: 'rust'
  })).claims.some(row => row.reason === 'caption-owner-or-fence-mismatch'));
});
test('manifest facts must bind to the actual frozen manifest bytes', () => {
  const files = inputs(),
    evidence = observedManifestEvidence(files);
  const stale = {
    observations: evidence.observations.map(row => ({
      ...row,
      sourceIdentity: '0'.repeat(64)
    })),
    facts: evidence.facts.map(row => ({
      ...row,
      sourceIdentity: '0'.repeat(64)
    }))
  };
  const result = grade(table, files, stale);
  assert.ok(result.claims.every(row => row.status !== 'verified-literal-declaration'));
});
