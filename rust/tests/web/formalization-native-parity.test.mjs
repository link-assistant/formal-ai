import fs from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import {installTextHost} from '../../../scripts/lib/text-capability-measures.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {grammarRoot,grammarVocabulary} from '../../../js/agentic/crate/formalization_grammar.mjs';
import {countSourceTerms} from '../../../js/agentic/crate/typed_meaning.mjs';
import {formalizeText,statementTerms} from '../../../js/agentic/crate/text_formalization.mjs';
installTextHost();
const vectors=JSON.parse(fs.readFileSync(new URL('../fixtures/formalization-native-parity.json',import.meta.url)));
test('shared held-out grammar vectors preserve raw case local override and unknown-language refusal',()=>{
 const current=host();
 try{
  installHost({...current,readText:file=>file==='data/seed/text-formalization.lino'?vectors.grammar:current.readText(file)});
  assert.equal(grammarRoot().name,'text-formalization');
  for(const vector of vectors.vocabulary)assert.deepEqual(grammarVocabulary(vector.language,vector.kind),vector.values);
 }finally{installHost(current);}
});
test('shared source-custody vectors count grouped tokens once even when logical terms repeat',()=>{
 for(const vector of vectors.counts){
  const [statement]=formalizeText(vector.text,vector.language);
  const terms=Array.from({length:vector.repeat},()=>statementTerms(statement)).flat();
  assert.equal(countSourceTerms(statement,terms),vector.expected,JSON.stringify(vector));
 }
});
