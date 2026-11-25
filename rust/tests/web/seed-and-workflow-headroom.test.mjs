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
  if(kind==='meaning'){
   const records=parseLexiconText(parent+bodies.join(''));
   const bySlug=new Map(records.map(record=>[record.slug,record]));
   assert.equal(bySlug.size,records.length,'partitioned meanings keep unique identities');
   for(const record of parseLexiconText(before))assert.deepEqual(bySlug.get(record.slug),record,
    'every original meaning field survives later additions: '+record.slug);
  }
  else {
   // Main commit 908f48238203a16a403d82508eac3386edef3097 retired
   // precisely these five hard-coded summaries in favor of captured sources.
   // Keep the original partition fixture, and compare every other byte.
   const retiredLanguages=['en','ru','hi','zh','es'];
   const retired=[...before.matchAll(/^  response response_http_fetch_curated_project_(\w+)\n    intent http_fetch_curated_project\n    language (\w+)\n    text "[^\n]*"\n/gmu)];
   assert.deepEqual(retired.map(match=>[match[1],match[2]]),retiredLanguages.map(language=>[language,language]));
   let retained=before;
   for(const [record] of retired)retained=retained.replace(record,'');
   const current=parent+bodies.join('\n');
   // Issue #1632 (R2) dropped the trailing "the unexecuted plan includes …"
   // sentence that named sources whose data was never used; every other byte
   // of those five records must still match the original fixture.
   const unavailable=/^(  response response_web_search_unavailable_(\w+)\n    intent web_search_unavailable\n    language \w+\n    text ")([^\n]*)"\n/gmu;
   const shortened=new Map([...current.matchAll(unavailable)].map(match=>[match[2],match[3]]));
   assert.deepEqual([...shortened.keys()].sort(),[...retiredLanguages].sort());
   retained=retained.replace(unavailable,(record,head,language,text)=>{
    const kept=shortened.get(language);
    assert.ok(kept!==text&&text.startsWith(kept)&&/^ ?\S/u.test(text.slice(kept.length)),'only the trailing unused-source sentence is removed: '+language);
    assert.ok(!/DuckDuckGo/u.test(kept),'no unused source stays named: '+language);
    return head+kept+'"\n';
   });
   assert.ok(!current.includes('response_http_fetch_curated_project_'),'retired source-free summaries stay absent');
   assert.equal(current,retained,'all retained response source bytes are preserved');
  }
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
