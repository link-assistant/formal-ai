import {renderTerm} from '../../../js/agentic/crate/round_trip_translation.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {installTextHost} from '../../../scripts/lib/text-capability-measures.mjs';
import {clauseStatement,sourceTermCount,statementTerms} from '../../../js/agentic/crate/text_formalization.mjs';
import {formalizeDialogue} from '../../../js/agentic/crate/dialogue_formalization.mjs';
import {factSurvives} from '../../../js/agentic/crate/page_formalization.mjs';
installTextHost();
test('canonical numeric identity and independent boundaries survive rendering',()=>{
 for(const [value,language]of [['695,000千米','zh'],['3,827⋅1026','ru'],['225—250','ru'],['1：397','zh'],['१४,९६,००,००० किलोमीटर','hi']])assert(factSurvives(clauseStatement(value,language,null,false)),value);
});
test('logical numeric groups retain exact source components without changing unknowns',()=>{
 for(const [value,expected,count]of [['987 654 blarg','987654',2],['987654 blarg','987654',1],['9,007,199,254,740,993 blarg','9007199254740993',1]]){
  const statement=clauseStatement(value,'en',null,false),quantity=statement.dialogue.quantities[0];
  assert.equal(statementTerms(statement).length,2);assert.equal(sourceTermCount(statement),count+1);
  assert.deepEqual(statement.unknown,['blarg']);assert.equal(quantity.value,expected);
  assert.equal(quantity.sourceComponents.length,count);
  for(const component of quantity.sourceComponents)assert.equal(new TextDecoder().decode(new TextEncoder().encode(value).slice(component.start,component.end)),component.surface);
 }
 const prefixed=clauseStatement('€987 654 blarg','en',null,false).dialogue.quantities[0];
 for(const component of prefixed.sourceComponents)assert.equal(new TextDecoder().decode(new TextEncoder().encode('€987 654 blarg').slice(component.start,component.end)),component.surface);
});

test('declared locale grammar preserves decimal meaning and refuses unsupported grouping',()=>{
 for(const [surface,language,expected]of [['3,827','ru','3.827'],['450 000','ru','450000'],['9,007,199,254,740,993','en','9007199254740993'],['12,34,567.89','hi','1234567.89'],['12.345,67','es','12345.67'],['12,345.67','zh','12345.67']]){
  const quantities=formalizeDialogue(surface,language).quantities;assert.equal(quantities.length,1);assert.equal(quantities[0].value,expected,surface);assert.equal(quantities[0].uncertainty,null);
  assert(factSurvives(clauseStatement(surface,language,null,false)),surface);
 }
 for(const [surface,language]of [['1,23','en'],['1,234,567','hi'],['3.827','ru']]){
  const [quantity]=formalizeDialogue(surface,language).quantities;assert.equal(quantity.value,surface);assert.equal(quantity.uncertainty,'unsupported-locale-number-format');
 }
});

test('translation renders canonical numeric meaning in the target locale',()=>{
 for(const [surface,source,target,expected]of [['3,827','ru','en','3.827'],['1.25','en','ru','1,25'],['12.345,67','es','en','12345.67'],['9,007,199,254,740,993','en','ru','9007199254740993']]){
  const term=statementTerms(clauseStatement(surface,source,null,false))[0];
  assert.equal(renderTerm(term,source,target).target,expected);
  assert.equal(statementTerms(clauseStatement(expected,target,null,false))[0].id,term.id);
 }
});
