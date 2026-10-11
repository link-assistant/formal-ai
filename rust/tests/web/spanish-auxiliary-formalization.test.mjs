import assert from 'node:assert/strict';
import {before,test} from 'node:test';
import {installTextHost} from '../../../scripts/lib/text-capability-measures.mjs';
let text;
before(async()=>{installTextHost();text=await import('../../../js/agentic/crate/text_formalization.mjs');});
test('Spanish present-perfect auxiliaries use the shared grammatical role',()=>{
  for(const auxiliary of ['he','has','ha','hemos','habéis','han']){
    const words=text.readWords(auxiliary+' hablado','es',false);
    assert.equal(words[0].kind,text.WordKind.Function,auxiliary);
    assert.equal(words[1].kind,text.WordKind.Unknown,auxiliary);
    assert.equal(words[1].surface,'hablado');
  }
});
test('auxiliaries keep negation, unknown content and numeric facts',()=>{
  for(const auxiliary of ['he','has','ha','hemos','habéis','han']){
    const [statement]=text.formalizeText('La Luna no '+auxiliary+' observado 42 cráteres.','es');
    assert.equal(statement.polarity,text.Polarity.Denied);
    const terms=text.statementTerms(statement);
    assert(terms.some(term=>term.kind===text.WordKind.Number&&term.surface==='42'));
    assert(terms.some(term=>term.kind===text.WordKind.Unknown&&term.surface==='observado'));
    assert(terms.some(term=>term.kind===text.WordKind.Unknown&&term.surface==='cráteres'));
    assert(!terms.some(term=>term.surface===auxiliary));
  }
});
test('the lexical role does not erase accented or longer content words',()=>{
  for(const word of ['há','hán','hemosfera','habéisimo']){
    assert(!text.functionWords('es').has(word));
    assert.equal(text.readWords(word,'es',false)[0].kind,text.WordKind.Unknown,word);
  }
});
