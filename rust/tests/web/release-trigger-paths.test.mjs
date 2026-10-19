import test from 'node:test';
import assert from 'node:assert/strict';
import {projectReleaseTriggerPaths} from '../../../scripts/generate-release-trigger-paths.mjs';
const recipe = "export const RELEASE_PROTOCOL_FILES=Object.freeze({role:Object.freeze(['first.mjs','second.mjs'])});";
const workflow = '# original header\non:\n  pull_request:\n    paths:\n      - first.mjs # original operand\n  workflow_dispatch: {}\npermissions:\n  contents: read\n';
test('declared source projection preserves every original byte and unrelated field', () => {
  const result = projectReleaseTriggerPaths(recipe,workflow,'role');
  assert.deepEqual(result.added,['second.mjs']);
  assert.equal(result.text.slice(0,result.offset)+result.text.slice(result.offset+result.insertion.length),workflow);
  assert.ok(result.text.includes('first.mjs # original operand'));
});
test('check is deterministic and idempotent without rewriting short source', () => {
  const first=projectReleaseTriggerPaths(recipe,workflow,'role');
  const second=projectReleaseTriggerPaths(recipe,first.text,'role');
  assert.equal(second.text,first.text);assert.deepEqual(second.added,[]);
});
test('CRLF source framing is preserved by the exact inverse', () => {
  const source=workflow.replaceAll('\n','\r\n');
  const result=projectReleaseTriggerPaths(recipe,source,'role');
  assert.equal(result.text.slice(0,result.offset)+result.text.slice(result.offset+result.insertion.length),source);
  assert.ok(result.insertion.endsWith('\r\n'));
});
test('computed, dynamic, duplicate and escaping declarations refuse', () => {
  for(const source of [recipe.replace('role:','[role]:'),recipe.replace("'second.mjs'",'dynamicSource'),
    recipe.replace('second.mjs','first.mjs'),recipe.replace('second.mjs','../outside.mjs')]) {
    assert.throws(()=>projectReleaseTriggerPaths(source,workflow,'role'));
  }
});
test('unknown patterns and alias path sequences refuse instead of approximating', () => {
  for(const source of [workflow.replace('first.mjs','**/*.mjs'),workflow.replace('paths:\n      - first.mjs # original operand','paths: [first.mjs]'),
    'paths: &original [first.mjs]\non:\n  pull_request:\n    paths: *original\n']) {
    assert.throws(()=>projectReleaseTriggerPaths(recipe,source,'role'));
  }
});
test('foreign role and duplicate workflow keys refuse', () => {
  assert.throws(()=>projectReleaseTriggerPaths(recipe,workflow,'foreign'));
  assert.throws(()=>projectReleaseTriggerPaths(recipe,workflow+'permissions: {}\n','role'));
});

test('canonical registry const and global intrinsic identity are required', () => {
  for (const prefix of [
    'const Object={freeze:value=>value};',
    'import Object from "other";',
    'function probe(Object) {}',
    'try {} catch (Object) {}',
    'const {Object}=other;'
  ]) assert.throws(()=>projectReleaseTriggerPaths(prefix+recipe,workflow,'role'));
  for (const prefix of [
    'const alias=Object;',
    'const alias=Object.freeze;',
    'Object.freeze=value=>value;',
    'Object["freeze"]=value=>value;',
    'globalThis.Object.freeze=value=>value;'
  ]) assert.throws(()=>projectReleaseTriggerPaths(prefix+recipe,workflow,'role'));
  assert.throws(()=>projectReleaseTriggerPaths(recipe.replace('export const','export let'),workflow,'role'));
  assert.throws(()=>projectReleaseTriggerPaths(recipe+'RELEASE_PROTOCOL_FILES={};',workflow,'role'));
  assert.throws(()=>projectReleaseTriggerPaths(recipe+'RELEASE_PROTOCOL_FILES.role=[];',workflow,'role'));
});
test('every declared role is immutable and closed before projection', () => {
  const shared=recipe.replace('role:Object.freeze(', 'other:Object.freeze(["third.mjs"]),role:Object.freeze(');
  assert.deepEqual(projectReleaseTriggerPaths(shared,workflow,'role').added,['second.mjs']);
  assert.throws(()=>projectReleaseTriggerPaths(shared.replace('other:Object.freeze(["third.mjs"])',
    'other:["third.mjs"]'),workflow,'role'));
  assert.throws(()=>projectReleaseTriggerPaths(shared.replace('"third.mjs"','unknown'),workflow,'role'));
  assert.throws(()=>projectReleaseTriggerPaths(shared.replace('other:', 'role:'),workflow,'role'));
});

 test('maintained publication triggers match every current declared source', async () => {
   const {readFileSync} = await import('node:fs');
   const currentRecipe = readFileSync(new URL('../../../scripts/release-protocol.mjs',import.meta.url),'utf8');
   const currentWorkflow = readFileSync(new URL('../../../.github/workflows/publish-source-networks.yml',import.meta.url),'utf8');
   const projection = projectReleaseTriggerPaths(currentRecipe,currentWorkflow,'source-networks');
   assert.deepEqual(projection.added,[]);
   assert.equal(projection.text,currentWorkflow);
 });

test('anchored existing filters preserve their exact text without replacing explicit declarations', () => {
  for (const filter of ['scripts/**','packages/source-cst-runtime/**','scripts/lib/source-cst-*.mjs']) {
    const original=workflow.replace('first.mjs # original operand',filter+' # original operand');
    const result=projectReleaseTriggerPaths(recipe,original,'role');
    assert.deepEqual(result.added,['first.mjs','second.mjs']);
    assert.equal(result.text.slice(0,result.offset)+result.text.slice(result.offset+result.insertion.length),original);
    assert(result.text.includes(filter+' # original operand'));
    assert.deepEqual(projectReleaseTriggerPaths(recipe,result.text,'role').added,[]);
  }
});
test('retained filters reject traversal, metadata, negation, broad and dynamic patterns', () => {
  for (const filter of ['../outside/**','.git/**','scripts/../outside/**','scripts/**/nested.mjs',
    'scripts/**/**','**/*.mjs','*.mjs','scripts/*','scripts/?name.mjs','scripts/[ab].mjs',
    '!scripts/first.mjs','scripts/${dynamic}/**','/outside/**','scripts/*/outside.mjs']) {
    assert.throws(()=>projectReleaseTriggerPaths(recipe,
      workflow.replace('first.mjs # original operand',filter+' # original operand'),'role'),filter);
  }
  for (const filter of ['scripts/**','scripts/lib/source-cst-*.mjs']) {
    assert.throws(()=>projectReleaseTriggerPaths(recipe.replace('second.mjs',filter),workflow,'role'),
      'filters never become frozen registry source authority');
  }
});
