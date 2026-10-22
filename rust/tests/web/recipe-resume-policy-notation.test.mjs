import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {parseLino,childValue} from '../../../js/server/lino.mjs';
import {installHost} from '../../../js/agentic/host.mjs';
import {recipeDeliveryReference} from '../../../js/agentic/recipe_resume.mjs';
const root=new URL('../../../',import.meta.url);
const readText=relative=>readFileSync(new URL(relative,root),'utf8');
function install(){installHost({readText,parseLino,realm:{}});}
const issue='https://github.com/o/r/issues/1',pull='https://github.com/o/r/pull/2';
test('prepared pull request metadata decodes one working pattern across scalar and JSON quoting layers',()=>{
 const policy=parseLino(readText('data/meta/recipe-resume.lino'));
 const patterns=JSON.parse(childValue(policy,'prepared-pull-request-patterns'));assert.equal(patterns.length,1);
 for(const pattern of patterns)for(const prefix of ['', 'Your ']){
  const match=new RegExp(pattern,'iu').exec(prefix+'prepared pull request: '+pull);
  assert(match);assert.equal(match[1],pull);
 }
 install();
 for(const request of ['prepared pull request: '+pull,'Your prepared pull request: '+pull,
  '\n\tYour\tprepared   pull\trequest:\t'+pull+'  \r\n']){
  assert.deepEqual(recipeDeliveryReference(request,issue),{kind:'bound',reference:pull});
 }
});
test('properly quoted policy retains source-owned cross-repository issue quoted and ambiguous refusals',()=>{
 install();
 for(const reference of ['https://github.com/foreign/r/pull/2',issue,'https://github.com/o/r/pull/0']){
  assert.equal(recipeDeliveryReference('Your prepared pull request: '+reference,issue).kind,'refused');
 }
 assert.equal(recipeDeliveryReference('Your prepared pull request: '+pull+'\nYour prepared pull request: https://github.com/o/r/pull/3',issue).kind,'refused');
 for(const request of ['"Your prepared pull request: '+pull+'"','\u0060\u0060\u0060text\nYour prepared pull request: '+pull+'\n\u0060\u0060\u0060','Solve '+issue]){
  assert.equal(recipeDeliveryReference(request,issue).kind,'absent');
 }
});
