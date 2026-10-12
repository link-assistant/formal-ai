import assert from 'node:assert/strict';
import test from 'node:test';
import {installTextHost} from '../../../scripts/lib/text-capability-measures.mjs';
import {formalizeText} from '../../../js/agentic/crate/text_formalization.mjs';
import {summarizeByDependency} from '../../../js/agentic/crate/dependency_summarization.mjs';
import {restatementFacts,verifiedRestatement,summarizeWithRestatement} from '../../../js/agentic/crate/concise_dependency_restatement.mjs';
installTextHost();
const original = 'The Moon orbits Earth. Earth is orbited by the Moon. The Moon has craters. Craters cover the Moon. Paris is a city.';
function first(text,language='en') { return formalizeText(text,language)[0]; }
function check(text,language='en') {
  const source = first(text,language);
  const candidate = verifiedRestatement(source);
  assert.equal(candidate.originalText,source.text);
  assert.deepEqual(candidate.originalFacts,restatementFacts(source));
  if (candidate.rewritten) {
    assert.ok(Array.from(candidate.text).length < Array.from(source.text).length);
    assert.deepEqual(formalizeText(candidate.text,language).map(restatementFacts),[candidate.originalFacts]);
  } else assert.equal(candidate.text,source.text);
  return candidate;
}
test('generated shorter wording preserves negation and exact source facts',()=>{
  const candidate=check('The Moon is not a satellite.');
  assert.equal(candidate.rewritten,true);
  assert.equal(candidate.originalFacts.polarity,'denied');
});
test('legacy graph, duplicate decisions and exact output remain unchanged',()=>{
  const legacy=summarizeByDependency(original,'en'),candidate=summarizeWithRestatement(original,'en');
  assert.deepEqual(candidate.statements,legacy.statements);
  assert.deepEqual(candidate.removed,legacy.removed);
  assert.deepEqual(candidate.kept,legacy.kept);
  assert.equal(candidate.duplicates,legacy.duplicates);
  assert.equal(candidate.text,'The Moon orbits Earth. The Moon has craters.');
});
test('complete core admission reparses the ordered kept facts',()=>{
  const candidate=summarizeWithRestatement(original,'en');
  if(candidate.coreRestatementVerified) assert.deepEqual(
    formalizeText(candidate.restatedText,'en').map(restatementFacts),
    candidate.kept.map(index=>restatementFacts(candidate.statements[index].statement)));
  else assert.equal(candidate.restatedText,candidate.text);
});
test('quoted authored facts retain original wording',()=>{
  for(const text of ['The Moon is "a satellite".','The Moon is “not a satellite”.']) {
    const candidate=check(text);assert.equal(candidate.rewritten,false);
    assert.equal(candidate.reason,'quoted-or-malformed-source');
  }
});
test('malformed quotation cannot become a shortening',()=>{
  const candidate=check('The Moon is "a satellite.');
  assert.equal(candidate.rewritten,false);
});
test('multiple negation scopes are retained instead of flattened',()=>{
  const candidate=check('The Moon is not never ready.');
  assert.equal(candidate.rewritten,false);assert.equal(candidate.reason,'multiple-negation-scopes');
});
test('number surfaces and opaque unknown surfaces stay exact',()=>{
  for(const text of ['The Moon has 2027 satellites.','The Moon has 2 satellites.','The Moon is not quuxp7.']) check(text);
});
test('source facts preserve term order and multiplicity',()=>{
  const source=first('The Moon is not quuxp7.');
  const facts=restatementFacts(source);
  const extra={...source,terms:[...source.terms,...source.terms]};
  assert.notDeepEqual(restatementFacts(extra),facts);
});
test('anaphor dependencies retain the complete original graph',()=>{
  const text='The Moon is not a satellite. It has 2027 craters. The Moon is bright.';
  const legacy=summarizeByDependency(text,'en'),candidate=summarizeWithRestatement(text,'en');
  assert.deepEqual(candidate.statements,legacy.statements);
  assert.deepEqual(candidate.kept,legacy.kept);
});
test('missing and empty cores do not invent any statement',()=>{
  const candidate=summarizeWithRestatement('','en');
  assert.equal(candidate.restatedText,'');assert.deepEqual(candidate.restatements,[]);
});
test('every declared language applies the same verified fallback rule',()=>{
  for(const [language,text] of [['en','The Moon is not a satellite.'],['ru','Луна не спутник.'],
    ['hi','चंद्रमा उपग्रह नहीं है।'],['zh','月球不是行星。'],['es','La Luna no es un satélite.']]) check(text,language);
});
test('formal preservation scope does not claim unmodeled semantic equivalence',()=>{
  assert.match(summarizeWithRestatement(original,'en').preservationScope,/unmodeled semantics unknown/u);
});
