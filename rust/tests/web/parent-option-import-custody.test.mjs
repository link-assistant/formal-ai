import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {generateOptions,optionTestImportContract} from '../../../scripts/lib/native-option-registry.mjs';
const producer="#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)] pub enum Shade{#[default] Dark,Light} impl Shade{#[must_use] pub const fn label(self)->&'static str{match self{Self::Dark=>\"dark\",Self::Light=>\"light\"}}#[must_use] pub fn parse(input:&str)->Option<Self>{match input{\"dark\"=>Some(Self::Dark),\"light\"=>Some(Self::Light),_=>None}}}";
function qualify(parent){
  const root=fs.mkdtempSync(path.join(os.tmpdir(),'parent-option-import-'));
  try{
    for(const [relative,source]of Object.entries({'rust/src/lib.rs':'pub mod fixture;','rust/src/fixture.rs':producer,'rust/Cargo.toml':'[package]\nname="fixture"\nversion="0.0.0"\n','rust/tests/unit/mod.rs':parent,'rust/tests/unit/specification/mod.rs':'mod fixture;'})){
      const target=path.join(root,relative);fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,source);
    }
    const binding=generateOptions(producer,'rust/src/fixture.rs','fixture','pub mod fixture;').programs.find(item=>item.kind==='nativeOptionStringParse');
    assert.ok(binding);
    return optionTestImportContract('use formal_ai::fixture::Shade;',root,binding);
  }finally{fs.rmSync(root,{recursive:true,force:true});}
}
for(const imported of ['use ::formal_ai::source_fetch;','use alternate::nested::observer;','use crate::observations as retained_observations;','use super::observations;','use alternate::工具;']){
  test('named non-Option parent import retains source custody: '+imported,()=>{
    const witnesses=qualify(imported);assert.equal(witnesses.filter(item=>item.path==='rust/tests/unit/mod.rs').length,1);
    assert.match(witnesses.find(item=>item.path==='rust/tests/unit/mod.rs').sha256,/^[0-9a-f]{64}$/u);
  });
}
for(const imported of ['use custom::*;','use custom::{Thing};','use custom::Option;','use custom::Choice as Some;','use custom::Choice as None;','use custom::Option as Harmless;','use custom::Thing as std;','use custom:::Thing;','use custom:Thing;','use ::crate::Thing;','use self;','use custom::Thing as _;','use custom::Thing as Alias as Other;','use custom::Thing; use other::Thing;','#[cfg(feature="unknown")] use custom::Thing;','extern crate custom;']){
  test('unproven or shadowing parent import refuses: '+imported,()=>assert.throws(()=>qualify(imported)));
}
