import assert from 'node:assert/strict';
import {readFileSync,existsSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import vm from 'node:vm';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url)).replace(/\/$/u,'');
const read=name=>readFileSync(root+'/'+name,'utf8');
for(const owner of ['js/seed_loader.js','ts/seed_loader.ts'])test(owner+' retains source headroom',()=>{
 assert.deepEqual(checkListedFiles(root,new Set([owner])),{warnings:[],violations:[],embeddedDataViolations:[]});
});
function api(loader,records=true){
 const context=vm.createContext({console,TextEncoder});context.self=context;
 vm.runInContext(read('js/parser-cache-primordials.js'),context);
 if(records&&existsSync(root+'/js/seed-record-readers.js'))vm.runInContext(read('js/seed-record-readers.js'),context);
 vm.runInContext(loader,context);return context.FormalAiSeed;
}
const plain=value=>JSON.parse(JSON.stringify(value));
test('all concept, context and fact fields equal the original reader on real seed documents',()=>{
 const previous=api(gunzipSync(readFileSync(new URL('./fixtures/seed-record-readers/previous-loader.js.gz',import.meta.url))).toString());
 const current=api(read('js/seed_loader.js'));
 for(const file of ['data/seed/concepts.lino','data/seed/concept-contexts.lino','data/seed/facts.lino','data/seed/fact-captures.lino']){
  const text=read(file),oldRoot=previous.parse(text),newRoot=current.parse(text);
  for(const name of ['extractConcepts','extractConceptContexts','extractFacts'])assert.deepEqual(plain(current[name](newRoot)),plain(previous[name](oldRoot)),name+' '+file);
 }
 for(const value of [null,undefined,{}, {name:'other',children:[]}])for(const name of ['extractConcepts','extractConceptContexts','extractFacts'])assert.deepEqual(plain(current[name](value)),plain(previous[name](value)));
});
test('seed evaluation rejects a missing record reader prerequisite',()=>{
 assert.throws(()=>api(read('js/seed_loader.js'),false),/formalAiSeedRecordReaders/u);
});
