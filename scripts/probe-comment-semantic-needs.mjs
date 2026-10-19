#!/usr/bin/env node
// This read-only capability probe reports existing gaps; it never treats copied prose as semantics.
import {installTextHost} from './lib/text-capability-measures.mjs';
import {formalizeDeeply} from '../js/agentic/crate/formalization_concept_links.mjs';
import {formalizeText,deformalizeStatement} from '../js/agentic/crate/text_formalization.mjs';
installTextHost();
const input='Some handler may throw an error.';
const statements=formalizeText(input,'en');
let deep;
try {
  formalizeDeeply('quorvex','comment-probe',{
    lookup(){return {kind:'found',senses:[{content_id:'declared-probe',lemma:'quorvex',gloss:'a value'}]};}
  },null,1);
  deep={available:true};
} catch(error) {
  deep={available:false,reason:error.message};
}
console.log(JSON.stringify({
  syntheticLookup:true,
  deepFormalization:deep,
  source:input,
  actualTerms:statements.map(statement=>({subject:statement.subject,terms:statement.terms,unknown:statement.unknown})),
  actualEnglish:statements.map(statement=>deformalizeStatement(statement,'en')),
  required:['ordered existential scope','permission modality','abrupt-completion effect'],
  status:deep.available?'inspect-semantics':'capability-gap'
},null,2));
if(!deep.available)process.exitCode=1;
