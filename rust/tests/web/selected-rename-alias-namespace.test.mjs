import assert from 'node:assert/strict';
import {test} from 'node:test';
import {mkdtempSync,mkdirSync,readFileSync,writeFileSync,rmSync,realpathSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {selectedRenamePlan} from '../../../experiments/formal_ai_subagent/selected-rename-by-rule.mjs';
const origin=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const hash=value=>createHash('sha256').update(value).digest('hex');
function fixture(run){
 const root=realpathSync(mkdtempSync(join(tmpdir(),'guarded-alias-')));
 const values=new Map();
 function add(path){
  if(values.has(path))return;
  const bytes=readFileSync(join(origin,path));values.set(path,bytes);
  for(const match of bytes.toString().matchAll(/^\s*include "([^"]+)"\s*$/gmu))add(match[1]);
 }
 for(const path of ['experiments/formal_ai_subagent/rename-by-rule.mjs',
  'experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs',
  'scripts/self-translation/lexer.mjs','scripts/measure-abbreviations.mjs',
  'scripts/lib/links-notation-names.mjs','data/meta/notation-rules.lino',
  'data/meta/abbreviation-rules.lino','data/meta/rename-map.lino'])add(path);
 values.set('rust/Cargo.toml',Buffer.from('[lib]\nname = "fixture_library"\n'));
 values.set('rust/src/lib.rs',Buffer.from('pub mod documentation;\npub use documentation as text_documents;\n'));
 values.set('rust/src/documentation/mod.rs',Buffer.from('pub mod doc;\n'));
 values.set('rust/src/documentation/doc.rs',Buffer.from('// α😀\npub fn value()->u8{7}\n'));
 const consumer='rust/tests/unit/another/doc.rs';
 values.set(consumer,Buffer.from('use fixture_library::text_documents::doc::value;\n'));
 for(const [path,bytes]of values){mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);}
 const inventory=[...values.keys()].sort();
 const manifest={inventory,inventorySha256:hash(JSON.stringify(inventory)),
  prefix:'rust/src/documentation',sources:[...values].map(([path,bytes])=>({path,sha256:hash(bytes)}))};
 try{run(root,manifest,consumer);}finally{rmSync(root,{recursive:true,force:true});}
}
test('physical selected namespace closes a real namesake import without changing source bytes',()=>fixture((root,manifest,consumer)=>{
 const before=readFileSync(join(root,consumer),'utf8');
 const plan=selectedRenamePlan(root,manifest);
 assert.equal(plan.rows.find(row=>row.beforePath===consumer).after,
  'use fixture_library::text_documents::document::value;\n');
 for(const row of plan.rows)assert.equal(row.inverse,row.before);
 assert.equal(readFileSync(join(root,consumer),'utf8'),before);
 assert.equal(plan.namespaceRaceProof,false);
 assert.equal(plan.completeness,'SelectedReferencesOnly');
}));
for(const path of ['experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs',
 'scripts/self-translation/lexer.mjs'])test('unbound '+path+' refuses',()=>fixture((root,manifest)=>{
 manifest.sources=manifest.sources.filter(row=>row.path!==path);
 assert.throws(()=>selectedRenamePlan(root,manifest),/Unbound alias transformation/u);
}));
test('rebound fabricated transformer bytes cannot replace imported physical origin',()=>fixture((root,manifest)=>{
 const path='experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs';
 const bytes=Buffer.concat([readFileSync(join(root,path)),Buffer.from('\n// forged origin\n')]);
 writeFileSync(join(root,path),bytes);manifest.sources.find(row=>row.path===path).sha256=hash(bytes);
 assert.throws(()=>selectedRenamePlan(root,manifest),/Unbound alias transformation/u);
}));
test('source hash drift refuses before any rename effects',()=>fixture((root,manifest)=>{
 writeFileSync(join(root,'rust/src/lib.rs'),'pub mod unknown;');
 assert.throws(()=>selectedRenamePlan(root,manifest),/Source drift/u);
}));
test('conditional physical alias refuses complete plan',()=>fixture((root,manifest)=>{
 const path='rust/src/lib.rs';const bytes=Buffer.from('pub mod documentation;\n#[cfg(feature="unknown")] pub use documentation as text_documents;\n');
 writeFileSync(join(root,path),bytes);manifest.sources.find(row=>row.path===path).sha256=hash(bytes);
 assert.throws(()=>selectedRenamePlan(root,manifest),/Unsupported exported alias/u);
}));
