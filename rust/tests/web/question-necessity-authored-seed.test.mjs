import assert from 'node:assert/strict';
import fs from 'node:fs';import path from 'node:path';import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {parseLino} from '../../../js/server/lino.mjs';
import {bundleFromFiles} from '../../../js/server/seed.mjs';
import {parseRegistry} from '../../../scripts/generate-seed-registry.mjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const basePath='data/seed/question-necessity.lino',extensionPath='data/seed/question-necessity-text-predicates.lino';
const read=relative=>fs.readFileSync(path.join(root,relative),'utf8');
const predicateRows=text=>parseLino(text).children.filter(row=>row.children.some(field=>field.name==='record_type'&&field.value==='question_necessity_text_predicate')).map(row=>Object.fromEntries(row.children.map(field=>[field.name,field.value])));
const expected=[
 ['memory_event','prefix','memory'],['self_trace_event','prefix','question_necessity:'],
 ['workspace_evidence','contains','workspace'],['workspace_evidence','contains','derivation'],
 ['url','prefix','http://'],['url','prefix','https://'],['replayed_question','prefix','your previous question was:'],
].map(([name,operation,value])=>({record_type:'question_necessity_text_predicate',name,operation,value}));
test('the canonical question seed preserves every original authored byte',()=>{
 assert.deepEqual(fs.readFileSync(path.join(root,basePath)),fs.readFileSync(path.join(root,'docs/case-studies/issue-920/self-hosting-authorship/question-necessity.lino')));
 assert.deepEqual(predicateRows(read(basePath)),[]);
});
test('registered extension retains all seven predicate objects in the shared composed policy',()=>{
 const registry=parseRegistry(read('data/meta/seed-registry.lino'));
 for(const name of ['question-necessity','question-necessity-text-predicates'])assert.equal(registry.seeds.find(seed=>seed.name===name)?.bundle,true);
 assert.deepEqual(predicateRows(read(extensionPath)),expected);
 const composed=read(basePath)+'\n'+read(extensionPath);assert.deepEqual(predicateRows(composed),expected);
 const tree=parseLino(composed);assert.equal(tree.children.filter(row=>row.name==='question_necessity_protocol').length,1);assert.equal(tree.children.filter(row=>row.name==='question_necessity_ratchet').length,1);
 const bundle=bundleFromFiles([[basePath,read(basePath)],[extensionPath,read(extensionPath)]]);
 const bundled=parseLino(bundle);const extensionFile=bundled.children.find(node=>node.name==='file'&&node.value===extensionPath);assert(extensionFile);
 assert.deepEqual(predicateRows(extensionFile.children.map(row=>row.name+'\n'+row.children.map(field=>'  '+field.name+' '+JSON.stringify(field.value)).join('\n')).join('\n')),expected);
 const native=read('rust/src/question_necessity.rs');assert(native.includes('const POLICY_SEED: &str = concat!('));assert(native.includes('include_str!("../embedded/'+basePath+'")'));assert(native.includes('include_str!("../embedded/'+extensionPath+'")'));
});
