import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {installTextHost} from '../../../scripts/lib/text-capability-measures.mjs';
import {parseLino} from '../../../js/server/lino.mjs';
import {formalizeText,statementIdentity,deformalizeStatement,clauseStatement} from '../../../js/agentic/crate/text_formalization.mjs';
import {typedMeaning,typedMeaningEntails} from '../../../js/agentic/crate/typed_meaning.mjs';
import {factSurvives,statementNotation} from '../../../js/agentic/crate/page_formalization.mjs';
import {restates,splitDuplicates} from '../../../js/agentic/crate/dependency_summarization.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {factoryOf} from '../../../scripts/generate-worker-crate-modules.mjs';
installTextHost();
const statement=(text,language)=>formalizeText(text,language)[0];
test('questions assertions currencies and units remain distinct through both duplicate paths',()=>{
 for(const [one,two,language]of [['You are located on Earth.','You are located on Earth?','en'],['450000 ₽','450000 $','ru'],['3 years','3 months','en']]){
  const a=statement(one,language),b=statement(two,language);
  assert.notEqual(typedMeaning(a.dialogue),typedMeaning(b.dialogue));assert.notEqual(statementIdentity(a),statementIdentity(b));
  assert.notEqual(statementNotation(a),statementNotation(b));assert.equal(restates(a,b),false);assert.equal(restates(b,a),false);
  assert.equal(splitDuplicates([{statement:a},{statement:b}]).removed.length,0);
  assert.doesNotThrow(()=>parseLino(statementNotation(a)));
 }
});
test('semantic identity ignores numeric spelling custody but preserves every typed obligation',()=>{
 const a=statement('987 654 ₽','ru'),b=statement('987654 ₽','ru');
 assert.equal(typedMeaning(a.dialogue),typedMeaning(b.dialogue));assert.equal(statementIdentity(a),statementIdentity(b));assert(restates(a,b));
 for(const [field,value]of [['currency','USD'],['unit','year'],['period','month'],['comparison','at-most'],['net',true],['uncertainty','unsupported']]){
  const altered=structuredClone(a);altered.dialogue.quantities[0][field]=value;
  assert.notEqual(typedMeaning(a.dialogue),typedMeaning(altered.dialogue));assert(!restates(a,altered));
 }
 const question=statement('Where are you located?','en');const other=structuredClone(question);other.dialogue.question.subject='unknown';assert.notEqual(statementIdentity(question),statementIdentity(other));
});
test('deformalization preserves typed facts and forged metadata never gets a survival pass',()=>{
 for(const [source,language]of [['You are located on Earth?','en'],['450000 ₽','ru'],['450000 $','ru'],['3 years','en']]){
  const actual=statement(source,language),rendered=deformalizeStatement(actual,language),back=clauseStatement(rendered,language,null,false);
  assert.equal(typedMeaning(actual.dialogue),typedMeaning(back.dialogue),source);assert(factSurvives(actual),source);
 }
 const wrong=statement('450000 ₽','ru');wrong.dialogue.quantities[0].currency='USD';assert.equal(factSurvives(wrong),false);
 const q=statement('You are located on Earth?','en');q.dialogue.question.kind='where';assert.equal(factSurvives(q),false);
});
test('actual browser factories preserve the same typed identity without import cycles or raised budgets',async()=>{
 const worker=new WorkerHost();const names=['formalization_grammar','dialogue_formalization','typed_meaning','text_formalization','page_formalization','dependency_summarization'];
 const factories=names.map(name=>factoryOf('crate/'+name+'.mjs',fs.readFileSync(new URL('../../../js/agentic/crate/'+name+'.mjs',import.meta.url),'utf8'),name==='text_formalization'?666:name==='dialogue_formalization'?198:null)).join('\n');
 const result=await worker.run(factories+`;crateInstances.clear();crateHostSeed=null;(()=>{const text=crateModule('crate/text_formalization.mjs'),dedup=crateModule('crate/dependency_summarization.mjs');const a=text.formalizeText('450000 ₽','ru')[0],b=text.formalizeText('450000 $','ru')[0];return [text.statementIdentity(a),text.statementIdentity(b),dedup.restates(a,b)];})()`);
 assert.notEqual(result[0],result[1]);assert.equal(result[2],false);
});

test('typed subset entailment retains earlier obligations and rejects incompatible evidence',()=>{
 const earlier=statement('1969 and 1972','en'),later=statement('1972','en');
 assert(typedMeaningEntails(earlier.dialogue,later.dialogue));assert(!typedMeaningEntails(later.dialogue,earlier.dialogue));
 assert(restates(later,earlier));
 const neutral=statement('web interface','en'),quantified=statement('10 web interface examples','en');
 assert(restates(neutral,quantified));assert(!restates(statement('450000 USD','en'),statement('450000 RUB','en')));
 const relative=formalizeText('The Moon is a body, which reflects light.','en');
 assert(relative.every(value=>!value.dialogue.question));assert(statement('Which body reflects light?','en').dialogue.question);
});

test('embedded declarative fragments and location prepositions do not invent obligations',()=>{
 assert(formalizeText('The cat is an animal, is a small domesticated mammal.','en').every(value=>!value.dialogue.question));
 assert.equal(statement('Из звёзд','ru').dialogue.answer,null);
 assert.equal(statement('In botany','en').dialogue.answer,null);
 assert.deepEqual(statement('In Paris','en').dialogue.answer.locations,['Paris']);
 const inherited=clauseStatement('It supplies energy.','en',{kind:'number',id:'number:17',surface:'17'},true);
 assert.equal(inherited.dialogue.quantities[0].value,'17');assert.deepEqual(inherited.dialogue.quantities[0].sourceComponents,[]);assert(factSurvives(inherited));
});

test('held-out multilingual budget statements retain exact typed evidence and distinguish currencies',()=>{
 const native=fs.readFileSync(new URL('../unit/issue_1188_web_formalization.rs',import.meta.url),'utf8');
 const block=native.slice(native.indexOf('fn held_out_multilingual_budgets_preserve_exact_typed_currency_meaning()'));
 const cases=[...block.matchAll(/language:\s*"(en|ru|hi|zh|es)",\s*text:\s*("(?:[^"\\]|\\.)*")/gu)]
  .map(match=>({language:match[1],text:JSON.parse(match[2])}));
 assert.deepEqual(cases.map(row=>row.language),['en','ru','hi','zh','es']);
 const expected=currency=>JSON.stringify([
  ['schema','dialogue-meaning/v1'],['question',null],
  ['answer',[['kind','quantity'],['polarity',null],['locations',[]]]],
  ['quantities',[[['value','73921'],['comparison','equal'],['currency',currency],
   ['unit',null],['period',null],['net',false],['uncertainty',null]]]],
 ]);
 for(const {text,language} of cases){
  const currency=['en','ru'].includes(language)?'USD':null;
  const statements=formalizeText(text,language);
  assert.equal(statements.length,1,text);
  const actual=statements[0];
  assert.equal(actual.dialogue.quantities.length,1,text);
  assert.equal(actual.dialogue.quantities[0].value,'73921',text);
  assert.equal(actual.dialogue.quantities[0].comparison,'equal',text);
  assert.equal(actual.dialogue.quantities[0].currency,currency,text);
  assert.equal(typedMeaning(actual.dialogue),expected(currency),text);
  const changed=formalizeText(text.replace('USD','RUB'),language);
  assert.equal(changed.length,1,text);
  assert.equal(changed[0].dialogue.quantities[0].currency,currency?'RUB':null,text);
  if(currency!==null)assert.notEqual(typedMeaning(actual.dialogue),typedMeaning(changed[0].dialogue),text);
  const changedAmount=formalizeText(text.replace('73921','73922'),language);
  assert.equal(changedAmount.length,1,text);
  assert.equal(changedAmount[0].dialogue.quantities[0].value,'73922',text);
  assert.notEqual(statementIdentity(actual),statementIdentity(changedAmount[0]),text);
 }
});
