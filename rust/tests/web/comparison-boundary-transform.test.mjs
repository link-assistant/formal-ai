// Intended owner: rust/tests/web/comparison-boundary-transform.test.mjs.
// Grader vectors do not enter production source, seed data or solver messages.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {parse} from '@babel/parser';
import {babelSourceFrontend} from '../../../scripts/lib/source-cst-babel.mjs';
import {readSyntaxRules,sourceToLinks,sourceFromLinks,ownedSourceLinks} from '../../../js/meta/readable-source.js';
import {comparisonBoundaryRoleKernel,comparisonBoundaryWitnessKernel} from '../../../js/agentic/portable/comparison_boundary_kernel.mjs';
import {selectComparisonBoundaryDefinition,deriveComparisonBoundaryCandidate,deriveComparisonBoundaryRegression} from '../../../scripts/lib/source-cst-boundary-transform.mjs';
const rules=readSyntaxRules(readFileSync('data/meta/source-model/syntax-rules.lino','utf8'));
const frontend={parse:babelSourceFrontend({parse},'unseen.mjs'),rules};
const owned=source=>ownedSourceLinks('src/unseen.mjs',sourceToLinks(source,'JavaScript',frontend));
const forms=JSON.parse(readFileSync('data/meta/comparison-boundary-regression.json','utf8'));
const definition={declaration:'inside',bound:'upper',inclusive:true};
const vectors=JSON.parse(readFileSync('rust/tests/fixtures/comparison-boundary/vectors.json','utf8'));

test('JavaScript semantic kernel obeys shared native qualification role/witness vectors',()=>{
  for(const row of vectors.roles) assert.deepEqual(comparisonBoundaryRoleKernel(...row.operands,...row.operators,row.bound,row.inclusive),row.expected);
  for(const row of vectors.witnesses) assert.equal(comparisonBoundaryWitnessKernel(...row.args),row.expected);
});

test('absent function name binds exactly one structurally suitable interval declaration',()=>{
  const source='export function inside(reading,floor,ceiling) { return reading >= floor && reading < ceiling; }\nexport function unrelated(value) { return value + 1; }\n';
  assert.deepEqual(selectComparisonBoundaryDefinition(owned(source),{bound:'upper',inclusive:true}),definition);
  const another='export function other(x,lo,hi) { return lo <= x && hi > x; }\n';
  assert.throws(()=>selectComparisonBoundaryDefinition(owned(source+another),{bound:'upper',inclusive:true}),/BoundaryUniqueSuitableDeclarationRequired/u);
});

test('recursive CST transform changes only the structurally selected upper relation',()=>{
  for(const [expression,expected] of [
    ['reading >= floor && reading < ceiling','reading >= floor && reading <= ceiling'],
    ['floor <= reading && ceiling > reading','floor <= reading && ceiling >= reading'],
    ['ceiling > reading && reading >= floor','ceiling >= reading && reading >= floor'],
  ]){
    const source='// 雪: preserve this comment and CRLF.\r\nexport function inside(reading, floor, ceiling) {\r\n  return '+expression+';\r\n}\r\nexport function untouched(input) { return input + 8; }';
    const result=deriveComparisonBoundaryCandidate(owned(source),definition,frontend);
    assert.equal(sourceFromLinks(result.document),source.replace(expression,expected));
    assert.equal(result.verification.exactInverse,true);
    assert.equal(result.verification.independentGrammarReparse,true);
    assert.equal(result.verification.arbitrarySemanticEquivalence,false);
    const again=deriveComparisonBoundaryCandidate(result.document,definition,frontend);
    assert.equal(again.changed,false);
    assert.deepEqual(again.document,result.document);
  }
});

test('ambiguous ownership, impure shapes, bound captures and unsupported operands refuse',()=>{
  for(const source of [
    'export function inside(x,lo,hi) { sideEffect(); return x >= lo && x < hi; }',
    'export function inside(x,lo,hi) { return x >= lo || x < hi; }',
    'export function inside(x,lo,hi) { return x >= lo && x < hi + 1; }',
    'export function inside(x,lo,hi) { return x >= lo && x < hi && x !== 3; }',
    'export function inside(x,lo,hi) { return x >= lo && lo < hi; }',
    'export function inside(x,lo,hi) { return x >= lo && x < lo; }',
    'export async function inside(x,lo,hi) { return x >= lo && x < hi; }',
    'export function inside(x=1,lo,hi) { return x >= lo && x < hi; }',
  ]) assert.throws(()=>deriveComparisonBoundaryCandidate(owned(source),definition,frontend),undefined,source);
});

test('regression derives contract witnesses before any candidate source is selected',()=>{
  const before='export function inside(reading,floor,ceiling) { return reading >= floor && reading < ceiling; }';
  const result=deriveComparisonBoundaryRegression(owned(before),definition,{moduleSpecifier:'../src/unseen.mjs',frontend,forms});
  assert.deepEqual(result.samples,[
    {args:[-2,0,2],expected:false},{args:[0,0,2],expected:true},
    {args:[1,0,2],expected:true},{args:[2,0,2],expected:true},{args:[4,0,2],expected:false},
  ]);
  assert.ok(result.source.includes('assert.equal(subject(2, 0, 2), true);'));
  assert.equal(result.contract.independentAcceptance,false);
  assert.equal(sourceFromLinks(owned(before)),before);
});


test('shared regression witnesses bind every legal parameter name literally',()=>{
  for(const row of vectors.regressions){
    const model=ownedSourceLinks(row.owner,sourceToLinks(row.source,'JavaScript',frontend));
    const result=deriveComparisonBoundaryRegression(model,row.definition,{moduleSpecifier:row.moduleSpecifier,frontend,forms});
    assert.deepEqual(result.samples,row.expectedSamples);
    assert.equal(sourceFromLinks(model),row.source);
  }
});
