import test from 'node:test';
import assert from 'node:assert/strict';
import { prepareRustConstantRename } from '../../../js/server/rust-source-transform.mjs';

for (const [previous, replacement] of [['ALPHA','BETA'], ['threshold','next_threshold'], ['old_value','NEW_VALUE']]) {
  test('source-bound private rename ' + previous, async () => {
    const source = '// α😀 constant note\nconst ' + previous + ': i32 = 7;\n'
      + 'fn probe(value: i32) -> bool { value == ' + previous + ' }\n';
    const result = await prepareRustConstantRename(source, previous, replacement);
    assert.ok(result);
    assert.equal(result.content, source.replaceAll(previous, replacement));
    assert.equal(result.spans.length, 2);
    assert.equal(result.writeAuthority, false);
    assert.equal(result.compilerValidity, 'unknown');
    assert.deepEqual(result.unresolvedNeeds, ['trusted-source-read', 'owned-write', 'native-verification']);
  });
}

test('quoted and comment data are conserved', async () => {
  const source = '// OLD must stay in prose\nconst OLD: &str = "OLD";\nfn probe(value: &str) -> bool { value == OLD }\n';
  const result = await prepareRustConstantRename(source, 'OLD', 'RENAMED');
  assert.ok(result);
  assert.ok(result.content.includes('// OLD must stay in prose'));
  assert.ok(result.content.includes('= "OLD";'));
  assert.equal(result.spans.length, 2);
});

const negatives = [
  ['public item', 'pub const OLD: i32 = 7; fn probe(v: i32) -> bool { v == OLD }'],
  ['parameter shadow', 'const OLD: i32 = 7; fn probe(OLD: i32) -> bool { OLD == 7 }'],
  ['local shadow', 'const OLD: i32 = 7; fn probe() { let OLD = 9; let v = OLD == 7; }'],
  ['collision', 'const OLD: i32 = 7; const NEW: i32 = 8; fn probe(v: i32) -> bool { v == OLD }'],
  ['macro use', 'const OLD: i32 = 7; fn probe() { consume!(OLD); }'],
  ['descendant module', 'const OLD: i32 = 7; mod child;'],
  ['unknown scoped reference', 'const OLD: i32 = 7; fn probe() { let v = crate::OLD; }'],
  ['invalid source', 'const OLD: i32 = ;'],
  ['duplicate definition', 'const OLD: i32 = 7; const OLD: i32 = 8;']
];
for (const [label, source] of negatives) test('refuse ' + label, async () => {
  assert.equal(await prepareRustConstantRename(source, 'OLD', 'NEW'), null);
});
for (const identifier of ['_', 'gen', 'Self', 'super', 'two names']) test('refuse invalid replacement ' + identifier, async () => {
  assert.equal(await prepareRustConstantRename('const OLD: i32 = 7;', 'OLD', identifier), null);
});

test('candidate spans and Needs cannot be dropped after return', async () => {
  const result = await prepareRustConstantRename('const OLD: i32 = 7;', 'OLD', 'NEW');
  assert.ok(result);
  assert.ok(Object.isFrozen(result.spans));
  assert.ok(Object.isFrozen(result.unresolvedNeeds));
  assert.throws(() => { result.spans.length = 0; }, TypeError);
  assert.throws(() => { result.unresolvedNeeds.length = 0; }, TypeError);
  assert.equal(result.spans.length, 1);
  assert.equal(result.unresolvedNeeds.length, 3);
});

for (const [previous, replacement] of [['FIRST', 'SECOND'], ['threshold', 'new_threshold']]) {
  test('unshadowed final block expression ' + previous, async () => {
    const source = 'const ' + previous + ': usize = 7;\n'
      + 'fn observed() -> usize { ' + previous + ' }\n';
    const result = await prepareRustConstantRename(source, previous, replacement);
    assert.ok(result);
    assert.equal(result.content, source.replaceAll(previous, replacement));
    assert.equal(result.spans.length, 2);
    assert.equal(result.writeAuthority, false);
  });
}
for (const [label, source] of [
  ['tail parameter shadow', 'const OLD: usize = 7; fn observed(OLD: usize) -> usize { OLD }'],
  ['tail local shadow', 'const OLD: usize = 7; fn observed() -> usize { let OLD = 9; OLD }'],
  ['tail macro reference', 'const OLD: usize = 7; fn observed() -> usize { consume!(OLD) }'],
  ['tail scoped reference', 'const OLD: usize = 7; fn observed() -> usize { crate::OLD }']
]) test('refuse ' + label, async () => {
  assert.equal(await prepareRustConstantRename(source, 'OLD', 'NEW'), null);
});
