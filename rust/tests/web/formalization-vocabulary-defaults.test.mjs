import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {installHost} from '../../../js/agentic/host.mjs';
import {parseLino} from '../../../js/server/lino.mjs';
import {grammarVocabulary} from '../../../js/agentic/crate/formalization_grammar.mjs';
import {seedWords} from '../../../js/agentic/crate/text_formalization.mjs';
const seed=fs.readFileSync(new URL('../../../data/seed/text-formalization.lino',import.meta.url),'utf8');
const install=value=>installHost({readText:()=>value,parseLino});
install(seed);
test('all original locale numeric profiles survive default extraction',()=>{
 const profiles={en:['.',',','3','3'],ru:[',','\u00a0','3','3'],hi:['.',',','3','2'],zh:['.',',','3','3'],es:[',','.','3','3']};
 for(const [language,expected]of Object.entries(profiles)){
  const kinds=['numeric-decimal','numeric-group','numeric-primary-width','numeric-secondary-width'];
  assert.deepEqual(kinds.map(kind=>grammarVocabulary(language,kind)[0]),expected);
  assert.deepEqual(kinds.map(kind=>seedWords(language,kind)[0]),expected);
 }
});
test('unknown locales receive no numeric or generic default support',()=>{
 for(const language of ['xx','','EN'])for(const kind of ['numeric-decimal','numeric-group','numeric-primary-width','numeric-secondary-width'])assert.deepEqual(grammarVocabulary(language,kind),[]);
});
test('local overrides win and absent local kinds inherit generic defaults',()=>{
 install('text-formalization\n  defaults\n    sample inherited\n    override inherited\n  language en\n    override local\n    override second\n  language ru\n    other supported\n');
 assert.deepEqual(grammarVocabulary('en','sample'),['inherited']);
 assert.deepEqual(grammarVocabulary('en','override'),['local','second']);
 assert.deepEqual(grammarVocabulary('ru','sample'),['inherited']);
 assert.deepEqual(grammarVocabulary('xx','sample'),[]);
 assert.deepEqual(grammarVocabulary('en','absent'),[]);
 install(seed);
});
