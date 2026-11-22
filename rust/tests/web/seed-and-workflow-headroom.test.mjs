import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../../scripts/check-file-size.mjs';
import {parseRegistry} from '../../../scripts/generate-seed-registry.mjs';

const root=fileURLToPath(new URL('../../../',import.meta.url)).replace(/\/$/u,'');
const read=path=>readFileSync(root+'/'+path,'utf8');
const pairs=[
 ['meanings-translation',['meanings-occupations'],'meaning'],
 ['multilingual-responses-agentic',['multilingual-responses-composition','multilingual-responses-quantity-constraint'],'response'],
];
test('grown seed owners, their package copies and the staged workflow stay below warning limits',()=>{
 const paths=['.github/workflows/release-staged.yml',...pairs.flatMap(([first,children])=>[first,...children].flatMap(name=>['data/seed/'+name+'.lino','rust/embedded/data/seed/'+name+'.lino']))];
 assert.deepEqual(checkListedFiles(root,new Set(paths)),{warnings:[],violations:[],embeddedDataViolations:[],errors:[]});
});
test('partitioning preserves every original seed record and every bundle, lexicon and web consumer',async()=>{
 const {installHost}=await import('../../../js/agentic/host.mjs');
 const {parseLino}=await import('../../../js/server/lino.mjs');
 const {parseLexiconText}=await import('../../../js/agentic/crate/seed_meanings.mjs');
 installHost({parseLino,readText:read});
 const registry=parseRegistry(read('data/meta/seed-registry.lino'));
 const web=read('js/seed-files.js');
 for(const [first,children,kind] of pairs){
  const before=gunzipSync(readFileSync(new URL('../../../experiments/issue-1580/fixtures/seed-and-workflow-headroom/'+first+'.lino.before.gz',import.meta.url))).toString();
  const parent=read('data/seed/'+first+'.lino');
  const bodies=children.map(name=>read('data/seed/'+name+'.lino')).map(child=>child.slice(child.indexOf('\n')+1));
  if(kind==='meaning')assert.deepEqual(parseLexiconText(parent+bodies.join('')),parseLexiconText(before),'concise lexemes preserve every original meaning field');
  // Restore the original separator between files; record bytes stay exact.
  else assert.equal(parent+bodies.join('\n'),before,'response source bytes are preserved');
  for(const name of [first,...children]){
   const row=registry.seeds.find(seed=>seed.name===name);assert(row,name+' is registered');
   assert(row.bundle&&row.web&&row.lexicon.has(kind),name+' retains all three original consumers');
   assert(web.includes('"seed/'+name+'.lino"'),name+' is fetched by the browser');
   assert.equal(read('rust/embedded/data/seed/'+name+'.lino'),read('data/seed/'+name+'.lino'));
  }
 }
});
test('actual lexicon and localized response lookups consume every moved record',async()=>{
 const {installHost}=await import('../../../js/agentic/host.mjs');
 const {parseLino}=await import('../../../js/server/lino.mjs');
 const {lexicon,parseLexiconText}=await import('../../../js/agentic/crate/seed_meanings.mjs');
 const {responseFor}=await import('../../../js/agentic/crate/seed.mjs');
 installHost({parseLino,readText:read});
 const all=lexicon();
 for(const entry of parseLexiconText(read('data/seed/meanings-occupations.lino'))){
  assert.deepEqual(all.filter(candidate=>candidate.slug===entry.slug),[entry],entry.slug+' is loaded exactly once');
 }
 for(const file of pairs[1][1])for(const entry of parseLino(read('data/seed/'+file+'.lino')).children.filter(child=>child.name==='response')){
  const value=name=>entry.children.find(child=>child.name===name)?.value;
  assert.equal(responseFor(value('intent'),value('language')),value('text'),entry.id+' remains reachable');
 }
});
