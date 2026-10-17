import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {parse} from '@babel/parser';
import {installTextHost} from '../../../scripts/lib/text-capability-measures.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {structuralRuleValues,numericIdentityPrefix,mergeNumericWords,answerCompatibility} from '../../../js/agentic/crate/dialogue_formalization.mjs';
import {renderTerm} from '../../../js/agentic/crate/round_trip_translation.mjs';
import {WordKind} from '../../../js/agentic/crate/text_formalization.mjs';
installTextHost();
const seedPath='data/seed/text-formalization.lino';
const source=fs.readFileSync(new URL('../../../js/worker/formal_ai_worker_translation_composition.js',import.meta.url),'utf8');
const ast=parse(source,{sourceType:'script'});
const node=ast.program.body.find(node=>node.type==='FunctionDeclaration'&&node.id.name==='targetTranslationSurfaceComplete');
const complete=vm.runInNewContext('('+source.slice(node.start,node.end)+')',{crateModule:()=>({structuralRuleValues})});
test('all four structural artifacts are rejected while complete multilingual surfaces survive',()=>{
 assert.deepEqual(structuralRuleValues('target-surface','reject-fragment'),['response:','-->','{{','[[']);
 for(const marker of ['response:','-->','{{','[['])assert.equal(complete('before '+marker+' after','en'),false);
 for(const [text,language]of [['lexical surface','en'],['полная фраза','ru'],['पूर्ण पाठ','hi'],['完整文字','zh']])assert.equal(complete(text,language),true);
});
test('numeric writers and roundtrip readers retain exact resolved/unresolved schemas',()=>{
 assert.equal(answerCompatibility(null,null).reason,'missing typed evidence');
 assert.equal(numericIdentityPrefix(false),'number:');assert.equal(numericIdentityPrefix(true),'number-unresolved:');
 for(const [surface,language,expected]of [['987,654','en','number:987654'],['987,65','en','number-unresolved:987,65'],['१२,३४५','hi','number-unresolved:१२,३४५']]){
  const [word]=mergeNumericWords([{surface,kind:WordKind.Number,id:'old'}],[],language);assert.equal(word.id,expected);
  assert.equal(renderTerm(word,language,'en').translated,!expected.startsWith('number-unresolved:'));
 }
});
test('grammar facts drive both marker and numeric behavior rather than hidden source literals',()=>{
 const current=host(),seed=current.readText(seedPath);
 try{
  installHost({...current,readText:file=>file===seedPath?seed.replace('reject-fragment "[["','reject-fragment "[["\n    reject-fragment "heldout-marker"').replace('unresolved-prefix "number-unresolved:"','unresolved-prefix "heldout-number:"'):current.readText(file)});
  assert.equal(complete('a heldout-marker surface','en'),false);
  const [word]=mergeNumericWords([{surface:'987,65',kind:WordKind.Number,id:'old'}],[],'en');
  assert.equal(word.id,'heldout-number:987,65');assert.equal(renderTerm(word,'en','en').translated,false);
 }finally{installHost(current);}
});
test('missing or ambiguous structural grammar refuses instead of certifying a surface',()=>{
 const current=host(),seed=current.readText(seedPath);
 try{
  for(const changed of [seed.replace('structural-rule target-surface','structural-rule absent-surface'),seed+'\n  structural-rule target-surface\n    reject-fragment bad\n']){
   installHost({...current,readText:file=>file===seedPath?changed:current.readText(file)});assert.throws(()=>complete('normal surface','en'));
  }
 }finally{installHost(current);}
});
