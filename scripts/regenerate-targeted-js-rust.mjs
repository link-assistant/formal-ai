import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {Worker} from 'node:worker_threads';
assert.equal(process.env.GITHUB_ACTIONS,'true','Translation regeneration is hosted-only');
const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const upstream=path.resolve(process.argv[2]??'.meta-language');
const {execFileSync}=await import('node:child_process');
assert.equal(execFileSync('git',['-C',upstream,'rev-parse','HEAD'],{encoding:'utf8'}).trim(),'eb0574d5528babd3b4aadece81590d8e2230b99b');
assert(fs.existsSync(path.join(upstream,'js/node_modules')),'Install pinned runtime dependencies in the hosted workflow first');
const api=await import(pathToFileURL(path.join(repo,'scripts/translate-js-rust.mjs')));
const {namedImports,resolveImport}=await import(pathToFileURL(path.join(repo,'scripts/lib/translation-workarounds.mjs')));
const {renderBlockers}=await import(pathToFileURL(path.join(repo,'scripts/lib/translation-blockers.mjs')));
const {selfTranslationSignatures}=await import(pathToFileURL(path.join(upstream,'js/src/self-translation.js')));
const read=p=>fs.readFileSync(path.join(repo,p),'utf8');
const targets=['js/agentic/crate/quoted_segments.mjs','js/agentic/crate/task_decomposition_stated_task.mjs'];
const ledgerBefore=read(api.LEDGER_FILE),committed=api.parseLedger(ledgerBefore),scope=api.scopeModules();
assert(targets.every(target=>scope.includes(target)),'Only existing registered modules may be regenerated');
const companion=targets[1],companionProjectionBefore=read(api.projectionPath(companion));
const untouched=new Map(scope.filter(module=>!targets.includes(module)).map(module=>[api.projectionPath(module),read(api.projectionPath(module))]));
assert.equal(committed.commit,'eb0574d5528babd3b4aadece81590d8e2230b99b');
const signatures=new Map();
const inScope=new Set(scope);
const dependencies=new Set();
for(const target of targets) for(const entry of namedImports(read(target))) {const dep=resolveImport(target,entry.specifier,inScope);if(dep&&!targets.includes(dep))dependencies.add(dep);}
assert.equal(dependencies.size,0,'Reviewed quote/stated targets have no registered same-directory dependencies');
assert.deepEqual(namedImports(read(targets[0])),[{specifier:'../ascii_character.mjs',names:[{imported:'isAsciiAlphanumeric',local:'isAsciiAlphanumeric'}]}]);
assert.deepEqual(namedImports(read(companion)),[{specifier:'../write_str.mjs',names:[{imported:'isAlphanumeric',local:'isAlphanumeric'},{imported:'trimEndMatches',local:'trimEndMatches'}]}]);
for(const target of targets) for(const entry of namedImports(read(target))) assert.equal(resolveImport(target,entry.specifier,inScope),null,'Out-of-directory imports retain the normal unbound/carry policy');
for(const dependency of dependencies){
 const projection=read(api.projectionPath(dependency));
 const measured=api.readProjection(projection);
 const source=read(dependency);
 assert.equal(measured.sha256,createHash('sha256').update(source).digest('hex'));
 const exported=api.upstreamBlocks(projection).blocks.filter(block=>block.kind==='translated'&&/\bexport\s/u.test(block.source));
 // The current six dependencies only export translated literal constants; no function/body binding is reconstructed.
 assert(exported.every(block=>/^\s*(?:\/\*\*[\s\S]*?\*\/\s*)?export const [A-Z_]+ = (?:'[^']*'|[0-9]+);\s*$/u.test(block.source)));
 const expectedNames=dependency.endsWith('/language.mjs')?['UNKNOWN','ENGLISH']:dependency.endsWith('/task_decomposition.mjs')?['DEFAULT_SPLIT_DEPTH_BOUND']:[];
 assert.equal(exported.length,expectedNames.length,'Committed exported constant count');
 const recovered=selfTranslationSignatures(exported.map(block=>block.source).join('\n'),'JavaScript');
 assert.deepEqual(recovered.map(item=>item.name),expectedNames,'Committed exported constant identities');
 signatures.set(dependency,recovered);
}
console.log(JSON.stringify({phase:'dependency-signatures',dependencies:[...signatures].map(([module,items])=>({module,names:items.map(item=>item.name)}))}));
const worker=new Worker(path.join(repo,'scripts/translate-js-rust.mjs'),{workerData:{role:'translate',dir:upstream}});
const results=new Map();
try{
 for(const target of targets){
  const text=read(target),imports={},bound={};
  for(const entry of namedImports(text)){const dep=resolveImport(target,entry.specifier,inScope);assert(!dep||signatures.has(dep),`Missing dependency signatures: ${dep}`);imports[entry.specifier]=dep?signatures.get(dep):[];bound[entry.specifier]=dep?imports[entry.specifier].map(item=>item.name):null;}
  const result=await new Promise((resolve,reject)=>{
   const finish=(error,message)=>{
    clearTimeout(responseTimer);
    worker.off('message',onMessage);
    worker.off('error',onError);
    worker.off('exit',onExit);
    if(error)reject(error);else resolve(message);
   };
   const onMessage=message=>finish(message.path===target?null:new Error('TargetedTranslationResponseMismatch'),message);
   const onError=error=>finish(error);
   const onExit=()=>finish(new Error('TargetedTranslationWorkerExited'));
   const responseTimer=setTimeout(()=>finish(new Error('TargetedTranslationWorkerTimeout')),60000);
   worker.once('message',onMessage);
   worker.once('error',onError);
   worker.once('exit',onExit);
   try{worker.postMessage({path:target,text,imports,bound,signatures:true});}catch(error){finish(error);}
  });
  signatures.set(target,result.signatures);
  const measured=api.measure(target,{...result,text});results.set(target,measured);
  console.log(JSON.stringify({phase:'translated',module:target,translated:measured.translated,workaround:measured.workaround,inferred:measured.inferred,carried:measured.carried,ms:result.ms}));
 }
}finally{await worker.terminate();}
assert.equal(results.get(companion).projection,companionProjectionBefore,'Unchanged stated-task companion projection bytes');
const modules=scope.map(module=>results.get(module)??{...api.readProjection(read(api.projectionPath(module))),path:module,projection:read(api.projectionPath(module))});
assert.deepEqual(api.regressions(committed.modules,modules),[]);
for(const module of modules.filter(module=>!targets.includes(module.path))){
 const old=committed.modules.find(row=>row.path===module.path);
 assert(old,'Every untouched module must have a committed row');
 for(const key of ['translated','workaround','inferred','carried'])assert.equal(module[key],old[key],'Untouched ledger measurement');
}
const ledger=api.renderLedger(committed.commit,modules);
const blockers=renderBlockers(modules.flatMap(module=>module.blockers.map((reasons,index)=>({module:module.path,reason:module.refusals[index],blockers:reasons}))));
const outputs=new Map(targets.map(target=>[api.projectionPath(target),results.get(target).projection]));
outputs.set(api.LEDGER_FILE,ledger);outputs.set(api.BLOCKERS_FILE,blockers);
const changed=[];
for(const [file,content]of outputs){if(!fs.existsSync(path.join(repo,file))||read(file)!==content){fs.mkdirSync(path.dirname(path.join(repo,file)),{recursive:true});fs.writeFileSync(path.join(repo,file),content);changed.push(file);}}
const verifyProblems=api.verify(repo);
console.log(JSON.stringify({phase:'written',changed,totals:api.parseLedger(ledger).totals,verify:verifyProblems},null,2));
assert.deepEqual(verifyProblems,[],'Generated source projections and ledger must verify');

for(const [file,content]of untouched)assert.equal(read(file),content,'Untouched projection bytes');
const output=path.join(repo,'translation-regeneration');
assert(!fs.existsSync(output),'Regeneration output must start absent');
const files=[...outputs.keys()];
const identities=files.map(file=>({path:file,bytes:Buffer.byteLength(read(file)),sha256:createHash('sha256').update(read(file)).digest('hex')}));
assert.equal(identities.length,4,'Only the four generated source files may be uploaded');
assert(identities.every(file=>file.bytes<=262144),'Generated source file size bound');
assert(identities.reduce((total,file)=>total+file.bytes,0)<=524288,'Generated source artifact size bound');
const inputs=[...new Set([...targets,...dependencies,'js/agentic/ascii_character.mjs','js/agentic/write_str.mjs','scripts/regenerate-targeted-js-rust.mjs'])].map(file=>({path:file,sha256:createHash('sha256').update(read(file)).digest('hex')}));
const manifest={source:execFileSync('git',['rev-parse','HEAD'],{cwd:repo,encoding:'utf8'}).trim(),upstream:committed.commit,run:process.env.GITHUB_RUN_ID,attempt:process.env.GITHUB_RUN_ATTEMPT,inputs,files:identities};
for(const file of files){const destination=path.join(output,file);fs.mkdirSync(path.dirname(destination),{recursive:true});fs.copyFileSync(path.join(repo,file),destination);}
fs.writeFileSync(path.join(output,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
console.log(JSON.stringify({phase:'artifact-ready',manifest},null,2));
