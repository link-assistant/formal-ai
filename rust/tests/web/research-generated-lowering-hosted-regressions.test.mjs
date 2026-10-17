import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { eliminateLastOwnedClones, lowerGeneratedRust } from '../../../scripts/lib/research-rust-lowering.mjs';
const options = {
  ownedValueModel: 'pinned-meta-language-owned-values/v1',
  pure: true
};
const clones = source => (source.match(/\.clone\(\)/gu) ?? []).length;
function formatted(source) {
  const result = spawnSync('rustfmt', ['--edition', '2024', '--emit', 'stdout', '--config', 'skip_children=true'], {
    input: source,
    encoding: 'utf8',
    timeout: 15000
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
test('nested owning match observes each physical pattern binding once', () => {
  const source = 'pub fn nested(value: ResearchValue, flag: bool) -> String { match value { ResearchValue::Outer(inner) => { match inner { ResearchInner::Text(text) => { if flag { text.clone() } else { crate::finish(text.clone()) } } } } } }';
  const result = eliminateLastOwnedClones(source);
  assert.equal(clones(result.source), 0);
  assert.equal(result.observations.length, 2);
  const borrowed = source.replace('match value', 'match &value');
  assert.equal(eliminateLastOwnedClones(borrowed).source, borrowed);
});
test('terminal boxed rebindings transfer their local final value only', () => {
  const source = 'pub fn unpack(value: Box<ResearchValue>) -> ResearchValue { let value = *value; value.clone() }';
  const result = lowerGeneratedRust(source, options);
  assert.equal(clones(result.source), 0);
  assert(result.operations.includes('terminal-owned-rebinding'));
  const retained = 'pub fn unpack(value: Box<ResearchValue>) -> ResearchValue { let value = *value; crate::observe(value.clone()); value.clone() }';
  assert.equal(clones(lowerGeneratedRust(retained, options).source), 2, 'unproven additional local operations stay untouched');
});
test('terminality follows every nested if and match arm; fallthrough keeps else', () => {
  const cases = ['pub fn value(flag: bool, other: bool) { loop { if flag { if other { continue; } else { continue; } } else { crate::observe(); continue; } } }', 'pub fn value(flag: bool, value: ResearchValue) { loop { if flag { match value { ResearchValue::A => { continue; } ResearchValue::B => { continue; } } } else { crate::observe(); continue; } } }'];
  for (const source of cases) {
    const result = lowerGeneratedRust(source, options);
    assert(result.operations.includes('terminal-branch'));
    assert(!formatted(result.source).split('\n').some(line => /\}\s+\{/u.test(line)), 'separate adjacent statement blocks physically');
  }
  const fallthrough = 'pub fn value(flag: bool, other: bool) { if flag { if other { return; } else { crate::observe(); } } else { crate::other(); } }';
  const result = lowerGeneratedRust(fallthrough, options);
  assert(formatted(result.source).includes('else'), 'nonterminal path cannot lose the alternative');
});
test('array lookup retains lazy panic and calls identical Clone trait method directly', () => {
  const source = 'pub fn at<T: Clone>(values: &[T], index: Option<usize>) -> T { match index.and_then(|index| values.get(index)) {\n            Some(value) => value.clone(),\n            None => panic!("array index out of range"),\n        } }';
  const result = lowerGeneratedRust(source, options);
  assert(result.source.includes('Clone::clone'));
  assert(result.source.includes('|| panic!("array index out of range")'));
  assert(!result.source.includes('|value| value.clone()'));
});
