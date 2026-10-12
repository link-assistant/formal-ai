import assert from 'node:assert/strict';

import {
test
} from 'node:test';

import {
sourceQualifiedAliasReferences as refs
} from '../../../experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs';

const move={
from:'pkg/src/domain/catalog/brief.rs',to:'pkg/src/domain/catalog/description.rs'
};

function sources(body='pub struct First {} pub enum Second { Value }') {
return new Map([['pkg/Cargo.toml','[lib]\nname="library"\n'],['pkg/src/lib.rs','pub(crate) mod domain; pub use domain::catalog as catalog_alias;'],['pkg/src/domain/mod.rs','pub mod catalog;'],[move.from,body]]);

}
const original='// 😀 preserved\nuse library::catalog_alias::brief::{First, Second};\nfn inspect() { let value = First {}; }';

const run=(text=original,context=sources())=>refs(text,'pkg/tests/check.rs',[move],context);

test('owned plain group preserves full item bindings and UTF16 offsets',()=>{
const r=run();
assert.equal(r.length,1);
assert.equal(original.slice(r[0].start,r[0].end),'brief');
const after=original.slice(0,r[0].start)+r[0].to+original.slice(r[0].end);
assert.equal(after,original.replace('::brief::','::description::'));
const context=sources();
context.set(move.to,context.get(move.from));
context.delete(move.from);
const reverse=refs(after,'pkg/tests/check.rs',[{
from:move.to,to:move.from
}],context);
assert.equal(reverse.length,1);
assert.equal(after.slice(0,reverse[0].start)+reverse[0].to+after.slice(reverse[0].end),original);

});

test('trailing comma preserves both names',()=>assert.equal(run(original.replace('Second}','Second,}')).length,1));

test('already renamed selected span retains physical original item ownership',()=>assert.equal(run(original.replace('::brief::','::description::')).length,0));

test('unselected sibling import remains unchanged and earns no references',()=>assert.equal(run('use library::catalog_alias::other::{First, Second};').length,0));

for(const [name,text]of[
 ['nested group','use library::catalog_alias::brief::{First, nested::{Second}};'],
 ['wildcard','use library::catalog_alias::brief::*;'],
 ['self','use library::catalog_alias::brief::{self, First};'],
 ['item alias','use library::catalog_alias::brief::{First as Other, Second};'],
 ['duplicate','use library::catalog_alias::brief::{First, First};'],
 ['unknown item','use library::catalog_alias::brief::{First, Missing};'],
 ['reexport','pub use library::catalog_alias::brief::{First, Second};'],
 ['nested import','fn action(){ use library::catalog_alias::brief::{First, Second}; }'],
 ['foreign namespace binding','fn library() {} '+original],
 ['competing import','use foreign::First; '+original],
 ['shadow variable',original+' let First = 1;'],
 ['shadow parameter',original+' fn another(First: u8) {}'],
 ['shadow declaration',original+' struct First {}'],
 ['attribute import','#[cfg(feature="x")] '+original],
])test(name+' refuses',()=>assert.throws(()=>run(text)));

test('missing owned module refuses',()=>{
const c=sources();
c.delete(move.from);
assert.throws(()=>run(original,c));

});

test('private item refuses',()=>assert.throws(()=>run(original,sources('struct First {} pub enum Second { Value }'))));

test('conditional public item refuses',()=>assert.throws(()=>run(original,sources('#[cfg(feature="x")] pub struct First {} pub enum Second { Value }'))));


test('raw nonselected fixture is unchanged only by exact moved identifier absence',()=>assert.equal(run('const DATA = r#"brief_v1 fixture"#;').length,0));

test('raw selected identifier retains unsupported refusal',()=>assert.throws(()=>run('const DATA = r#"brief fixture"#;')));

test('nonRust companion does not mask exact Rust identifier absence',()=>assert.equal(refs('const DATA = r#"brief_v1 fixture"#;','pkg/tests/check.rs',[move,{
from:'meta/brief.lino',to:'meta/description.lino'
}],sources()).length,0));


for (const [name,suffix] of [
 ['tuple local',' fn action(){let (First, another) = values;}'],
 ['tuple parameter',' fn action((First, another): Pair) {}'],
 ['closure parameter',' fn action(){let transform = |First| First;}'],
]) test(name+' refuses after immutable genuine acceptance',()=>{
 assert.throws(()=>run(original+suffix));
});

for (const [name,body] of [
 ['for binding','fn action(){for First in values { use_value(First); }}'],
 ['match binding','fn action(){match value { Some(First) => use_value(First), None => {} }}'],
 ['struct binding','fn action(){let Pair { x: other, y: First } = value; use_value(First);}'],
 ['if-let binding','fn action(){if let Pair { x: other, y: First } = value { use_value(First); }}'],
]) test(name+' refuses selected group item shadow',()=>assert.throws(()=>run('use library::catalog_alias::brief::{First, Second};\n'+body)));
test('trait impl for clause is a type position rather than loop binding',()=>{
 const text='use library::catalog_alias::brief::{First, Second};\nimpl SomeTrait for Other { fn make()->First { First {} } }';
 const result=run(text);
 assert.equal(result.length,1);
 assert.equal(text.slice(result[0].start,result[0].end),'brief');
});
