import test from 'node:test';
import assert from 'node:assert/strict';
import {eliminateLastOwnedClones,lowerGeneratedRust} from '../../../scripts/lib/research-rust-lowering.mjs';
const options={ownedValueModel:'pinned-meta-language-owned-values/v1',pure:true};
const clones=source=>(source.match(/\.clone\(\)/gu)??[]).length;
test('owned SSA final use moves while earlier reuse stays cloned',()=>{
 const source='pub fn sample(x: String) -> String { let a = x.clone(); crate::join(a.clone(), x.clone()) }';
 const result=eliminateLastOwnedClones(source);
 assert.equal(clones(result.source),1);
 assert(result.source.includes('let a = x.clone()'));
 assert(result.source.includes('crate::join(a, x)'));
});
test('exclusive branches independently transfer their final owned value',()=>{
 const result=eliminateLastOwnedClones('pub fn choose(x: String, flag: bool) -> String { if flag { x.clone() } else { x.clone() } }');
 assert.equal(clones(result.source),0);
 assert.equal(result.observations.length,2);
});
test('loop fixed point retains values across iterations and permits assignment kills',()=>{
 const retained='pub fn held(mut x: String, quit: bool) -> String { loop { crate::observe(x.clone()); if quit { return x.clone(); }; continue; } }';
 const result=eliminateLastOwnedClones(retained);
 assert.equal(clones(result.source),1);
 assert(result.source.includes('observe(x.clone())'));
 const updated='pub fn update(mut x: String, keep: bool) -> String { loop { return if keep { x = crate::advance(x.clone()); continue; } else { x.clone() }; } }';
 assert.equal(clones(eliminateLastOwnedClones(updated).source),0);
});
test('borrowed aliases owned captures custom drop and unsupported loops never get guessed moves',()=>{
 for(const source of [
  'pub fn borrowed(x: String) -> String { let r = &x; crate::observe(r); x.clone() }',
  'pub fn captured(x: String) -> String { let f = || x.clone(); crate::observe(f); x.clone() }',
  'impl Drop for ResearchValue { fn drop(&mut self) {} } pub fn value(x: ResearchValue) -> ResearchValue { x.clone() }',
  'pub fn unsupported(x: String) -> String { while crate::again() { crate::observe(x.clone()); } x.clone() }',
  'pub fn borrowed_match(x: ResearchValue) -> String { match &x { ResearchValue::Value(value) => value.clone() } }',
 ])assert.equal(eliminateLastOwnedClones(source).source,source);
});
test('pure generated attributes are item-specific ABI and Number contracts only',()=>{
 const source='pub fn value(text: String, count: f64) -> bool { count == 1f64 && text == "x" }';
 const result=lowerGeneratedRust(source,options);
 assert(result.source.includes('#[must_use]'));
 assert.deepEqual(result.allowances.map(row=>row.lint).sort(),['float_cmp','needless_pass_by_value']);
 assert(!result.source.includes('clippy::all'));
 assert.throws(()=>lowerGeneratedRust(source),/pinned-meta/u);
 assert.throws(()=>lowerGeneratedRust(source,{...options,pure:false}));
 assert.throws(()=>lowerGeneratedRust('pub fn value() { unknown!(); }',options),/OpaqueGeneratedRustMacro/);
});
test('positive branch rewrite complements whole conditions and never a partial not',()=>{
 const partial='pub fn value(a: bool, b: bool) -> f64 { if !a && b { 1f64 } else { 2f64 } }';
 assert(!lowerGeneratedRust(partial,options).operations.includes('positive-condition'));
 const whole='pub fn value(a: bool, b: bool) -> f64 { if !(a && b) { 1f64 } else { 2f64 } }';
 assert(lowerGeneratedRust(whole,options).operations.includes('positive-condition'));
 for(const a of [false,true])for(const b of [false,true])assert.equal(!(a&&b)?1:2,(a&&b)?2:1);
 for(const a of [NaN,-0,0,1,Infinity])for(const b of [NaN,-0,0,1,Infinity])assert.equal(a!==b?1:2,a===b?2:1);
});
test('literal spelling preserves non-normalized Unicode and tuple/unit argument boundaries',()=>{
 const source='pub fn value() -> bool { crate::accept(()) && crate::accept((1f64, 2f64)) && " " == " " }';
 const result=lowerGeneratedRust(source,options);
 assert(result.source.includes('accept(())'));
 assert(result.source.includes('accept((1f64, 2f64))'));
 assert.equal((result.source.match(/\\u\{2000\}/gu)??[]).length,2);
 assert(!result.source.includes('" "'));
});
test('only a whole immutable string receiver is lowered for empty comparisons',()=>{
 const source='pub fn value(text: String) -> bool { text.clone() != "" }';
 const result=lowerGeneratedRust(source,options);
 assert(result.source.includes('(text).is_empty()'));
 assert(!result.source.includes('().is_empty()'));
});
test('guarded index casts keep exact guards and reject unexplained narrowing',()=>{
 const source='pub fn index(index: f64) -> Option<usize> { if index >= 0.0 && index.fract() == 0.0 && index < 9007199254740992.0 { Some(index as usize) } else { None } }';
 const result=lowerGeneratedRust(source,options);
 assert(result.source.includes('9_007_199_254_740_992.0'));
 assert(result.allowances.some(row=>row.lint==='cast_possible_truncation'));
 assert.throws(()=>lowerGeneratedRust('pub fn index(index: f64) -> usize { index as usize }',options),/UnprovenGeneratedIndexCast/);
});

test('parenthesized condition receivers never absorb the if keyword',()=>{
 const source='pub fn value(text: String) -> bool { if (text) == "" { true } else { false } }';
 const result=lowerGeneratedRust(source,options);
 assert(!result.source.includes('(if '));
 assert(result.source.includes('if ((text)).is_empty()'));
});

test('generic clone implementations field writes and moved captures remain outside ownership lowering',()=>{
 for(const source of [
 'pub fn generic<T: Clone>(value: T) -> T { let x = value.clone(); x.clone() }',
 'pub fn field(mut value: ResearchRecord) -> ResearchRecord { value.name = crate::consume(value.clone()); value.clone() }',
 'pub fn closure(value: String) -> String { let work = move || value.clone(); crate::use_work(work); value.clone() }',
 ])assert.equal(eliminateLastOwnedClones(source).source,source);
});


test('single-character pattern lowering decodes Rust escapes without changing their content', () => {
  for (const literal of ["\"\\u{0}\"","\"\\x41\"","\"\\u{1f680}\""]) {
    const source = 'pub fn observed(value: String) -> bool { value.contains(' + literal + ') }';
    const result = lowerGeneratedRust(source, options);
    assert(result.operations.includes('single-character-pattern'), literal);
    assert(result.source.includes("contains('" + literal.slice(1, -1) + "')"), literal);
  }
  for (const literal of ['""', '"ab"']) {
    const result = lowerGeneratedRust('pub fn observed(value: String) -> bool { value.contains(' + literal + ') }', options);
    assert(!result.operations.includes('single-character-pattern'), literal);
  }
});
