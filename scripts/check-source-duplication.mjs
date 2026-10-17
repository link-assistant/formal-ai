#!/usr/bin/env node
// Run together with hosted Rust AST gate. Only independently rederived callable
// identities can share one definition; raw clone evidence always remains visible.
import {readFileSync,lstatSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {parse} from '@babel/parser';
import traversal from '@babel/traverse';
import {translateJsToTs,tokenize,tsTwinPath} from './translate-es.mjs';
import {ENTRY_MODULES,importClosure,workerFileOf,generatedFiles} from './generate-worker-crate-modules.mjs';
import {structuralCallables,groupClones,assessDuplication,proveDerivedCallables,canonicalCloneGroups,proveFactoryCallables} from './lib/source-duplication.mjs';
const traverse=traversal.default??traversal;
const digest=value=>createHash('sha256').update(value).digest('hex');
export function trackedSourceFiles(root,roots=['js','ts'],run=execFileSync) {
 const listed=run('git',['ls-files','--cached','--others','--exclude-standard','-z','--',...roots],{cwd:root,encoding:'utf8'});
 return [...new Set(listed.split('\0').filter(Boolean))].filter(file=>roots.some(folder=>file.startsWith(folder+'/'))&&/\.(?:js|mjs|cjs|jsx|ts|mts|cts|tsx)$/u.test(file)).sort();
}
function factoryMappings(root) {
 const read=name=>readFileSync(join(root,'js/agentic',name),'utf8');
 const expected=generatedFiles();
 const modules=importClosure(ENTRY_MODULES,read);
 const generator=[['scripts/generate-worker-crate-modules.mjs',new URL('./generate-worker-crate-modules.mjs',import.meta.url)],['scripts/translate-es.mjs',new URL('./translate-es.mjs',import.meta.url)],['js/agentic/crate/es_tokenizer.mjs',new URL('../js/agentic/crate/es_tokenizer.mjs',import.meta.url)]].map(([file,url])=>({file,sha256:digest(readFileSync(url))}));
 return modules.map(name=>{
  const source='js/agentic/'+name,target='js/worker/'+workerFileOf(name),sourceText=read(name),output=expected.get(target);
  if(typeof output!=='string')throw Error('canonical factory list lacks independently generated output');
  const budget='data/meta/worker-line-budget/'+workerFileOf(name).replace(/\.js$/u,'.lino');
  return {source,target,generator:[...generator,{file:budget,sha256:digest(readFileSync(join(root,budget)))}],regenerate:text=>{if(text!==sourceText)throw Error('factory source changed');return output;}};
 });
}
export function scanJavaScriptImplementations(root,roots=['js','ts'],{listSourceFiles=trackedSourceFiles,listFactoryMappings=factoryMappings}={}) {
 const files=[],records=[],gaps=[],parsedFiles=[],sources=[],texts=new Map(),callables=new Map();
 const listed=listSourceFiles(root,roots);
 for(const file of listed){
  let metadata;try{metadata=lstatSync(join(root,file));}catch(error){if(error.code==='ENOENT')continue;throw error;}
  if(metadata.isSymbolicLink()||!metadata.isFile()){gaps.push({file,reason:'implementation source requires regular-file custody'});continue;}
  files.push(file);const source=readFileSync(join(root,file),'utf8'),sourceSha256=digest(source);sources.push({file,sourceSha256});texts.set(file,source);
  try{const rows=structuralCallables(source,file,{parse,traverse});callables.set(file,rows);for(const row of rows)records.push({...row,file,sourceSha256});parsedFiles.push(file);}
  catch(error){gaps.push({file,reason:String(error.message)});}
 }
 if(!files.length)gaps.push({reason:'no implementation sources discovered'});
 const generatorFiles=[['scripts/translate-es.mjs',new URL('./translate-es.mjs',import.meta.url)],['js/agentic/crate/es_tokenizer.mjs',new URL('../js/agentic/crate/es_tokenizer.mjs',import.meta.url)]];
 const generator=generatorFiles.map(([file,url])=>({file,sha256:digest(readFileSync(url))}));
 const proofs=[],expectedTargets=new Set();
 for(const file of files){
  if(!file.startsWith('js/')||!(file.endsWith('.js')||(file.startsWith('js/agentic/')&&file.endsWith('.mjs'))))continue;
  const target='ts/'+tsTwinPath(file.slice(3));expectedTargets.add(target);
  if(!texts.has(target)||!callables.has(file)||!callables.has(target)){const reason='missing or unparsed independently regenerated target';proofs.push({accepted:false,source:file,target,reason});gaps.push({file:target,reason});continue;}
  let proof;try{proof=proveDerivedCallables({sourcePath:file,targetPath:target,source:texts.get(file),target:texts.get(target),sourceCallables:callables.get(file),targetCallables:callables.get(target),translate:translateJsToTs,tokenize,generator});}
  catch(error){proof={accepted:false,source:file,target,reason:'maintained generator refused: '+error.message};}
  proofs.push(proof);if(!proof.accepted)gaps.push({file:target,reason:proof.reason});
 }
 for(const file of files)if(file.startsWith('ts/')&&!expectedTargets.has(file))gaps.push({file,reason:'orphan target has no canonical translator owner'});
 const beforeFactoryIndependentGroups=canonicalCloneGroups(groupClones(records),proofs).length;
 let mappings=[];try{mappings=listFactoryMappings(root);}catch(error){gaps.push({reason:'canonical factory generation refused: '+error.message});}
 for(const mapping of mappings){
  const {source,target,generator,regenerate}=mapping;let proof;
  if(!texts.has(source)||!texts.has(target)||!callables.has(source)||!callables.has(target))proof={accepted:false,kind:'worker_factory',source,target,reason:'canonical factory source or target missing/unparsed'};
  else try{
   proof=proveFactoryCallables({
    sourcePath:source,targetPath:target,source:texts.get(source),target:texts.get(target),
    sourceCallables:callables.get(source),targetCallables:callables.get(target),
    regenerate,tokenize,generator
   });
  }catch(error){
   proof={accepted:false,kind:'worker_factory',source,target,reason:error.message};
  }
  proofs.push(proof);if(!proof.accepted)gaps.push({file:target,reason:proof.reason});
 }
 const clones=groupClones(records),independentClones=canonicalCloneGroups(clones,proofs);
 return {schema:'formal-ai-source-duplication-v2',language:'JavaScript/TypeScript',files:files.length,callables:records.length,sources,
  beforeFactoryIndependentGroups,workerFactoryProofs:proofs.filter(proof=>proof.kind==='worker_factory'&&proof.accepted).length,rawCloneGroups:clones.length,rawCloneSites:clones.reduce((n,g)=>n+g.sites.length,0),clones,
  independentCloneGroups:independentClones.length,independentClones,proofs,
  ...assessDuplication({files,parsedFiles,clones:independentClones,gaps})};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const args=process.argv.slice(2);if(args.some(arg=>arg!=='--inventory'))throw Error('only --inventory is accepted; baselines and exemptions are forbidden');
 const report=scanJavaScriptImplementations(resolve(fileURLToPath(new URL('..',import.meta.url))));console.log(JSON.stringify(report));
 if(!args.includes('--inventory')&&!report.accepted)process.exitCode=1;
}
