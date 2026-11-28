import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../../scripts/check-file-size.mjs';
import {parseRegistry} from '../../../scripts/generate-seed-registry.mjs';

const root=fileURLToPath(new URL('../../../',import.meta.url)).replace(/\/$/u,'');
const pairs=[
  ['multilingual-responses','multilingual-responses-administration','response'],
  ['meanings-response-intents','meanings-response-intents-additional','meaning'],
  ['meanings-response-intents-handlers','meanings-response-intents-handlers-additional','meaning'],
  ['meanings-routing-vocabulary','meanings-routing-vocabulary-additional','meaning'],
  ['multilingual-responses-synthesis','multilingual-responses-synthesis-runtime','response'],
];
test('canonical seed records and package mirrors retain headroom without losing consumers',()=>{
  const paths=pairs.flatMap(([first,second])=>[first,second].flatMap(name=>['data/seed/'+name+'.lino','rust/embedded/data/seed/'+name+'.lino']));
  assert.deepEqual(checkListedFiles(root,new Set(paths)),{warnings:[],violations:[],embeddedDataViolations:[],errors:[]});
  const registry=parseRegistry(readFileSync(root+'/data/meta/seed-registry.lino','utf8'));
  const web=readFileSync(root+'/js/seed-files.js','utf8');
  for(const [first,second,kind] of pairs)for(const name of [first,second]){
    const row=registry.seeds.find(seed=>seed.name===name);assert(row,name+' remains registered');
    assert(row.bundle&&row.lexicon.has(kind),name+' retains its bundled lexicon consumer');
    assert.equal(row.web,kind==='response',name+' retains original browser eligibility');
    assert.equal(web.includes('"seed/'+name+'.lino"'),row.web,name+' browser registration follows original eligibility');
    assert.equal(readFileSync(root+'/data/seed/'+name+'.lino','utf8'),readFileSync(root+'/rust/embedded/data/seed/'+name+'.lino','utf8'),name+' package bytes');
  }
});


test('moved meanings retain priority and moved multilingual templates remain reachable',async()=>{
  const {installHost}=await import('../../../js/agentic/host.mjs');
  const {parseLino}=await import('../../../js/server/lino.mjs');
  const {lexicon,parseLexiconText,meaningFiles}=await import('../../../js/agentic/crate/seed_meanings.mjs');
  const {responseFor}=await import('../../../js/agentic/crate/seed.mjs');
  const read=path=>readFileSync(root+'/'+path,'utf8');
  installHost({parseLino,readText:read});
  const actual=lexicon(),files=meaningFiles();
  const boundaries=[
    ['meanings-response-intents-additional','web_open_in_new_tab','calendar-duration-label'],
    ['meanings-response-intents-handlers-additional','installation_conversion_heading','opaque-label'],
    ['meanings-routing-vocabulary-additional','install_steps','operand'],
  ];
  for(const [name,first,last] of boundaries){
    const path='data/seed/'+name+'.lino',index=files.indexOf(path);assert(index>=0);
    const entries=parseLexiconText(read(path));assert.equal(entries[0].slug,first);assert.equal(entries.at(-1).slug,last);
    const start=files.slice(0,index).reduce((total,file)=>total+parseLexiconText(read(file)).length,0);
    assert.deepEqual(actual.slice(start,start+entries.length),entries,'complete moved records retain declaration priority');
  }
  const responses=parseLino(read('data/seed/multilingual-responses-synthesis-runtime.lino')).children.filter(child=>child.name==='response');
  assert.equal(responses[0].id,'response_coding_rosetta_passed_en');
  assert.equal(responses.at(-1).id,'response_nl_tool_browser_unavailable_es');
  for(const entry of responses){
    const value=name=>entry.children.find(child=>child.name===name)?.value;
    assert.equal(responseFor(value('intent'),value('language')),value('text'),entry.id+' remains an actual localized lookup');
  }
});
