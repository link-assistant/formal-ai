#!/usr/bin/env node
// Shared canonical snapshot policy compiled through the pinned checked meta intermediate.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {workflowPin} from './translate-js-rust.mjs';
import {loadResearchCompiler, generateCodingResearchProgram} from './generate-coding-research-program.mjs';
export const SNAPSHOT_SOURCE = 'js/provenance/portable/snapshot.mjs';
export async function generateSnapshotProgram(root, compilerDirectory){
 const pin=workflowPin(fs.readFileSync(path.join(root,'.github/workflows/layered-ci.yml'),'utf8'));
 const compiler=await loadResearchCompiler(compilerDirectory,pin);
 const sources=[{path:SNAPSHOT_SOURCE,source:fs.readFileSync(path.join(root,SNAPSHOT_SOURCE),'utf8')}];
 return generateCodingResearchProgram(sources,compiler,{
  sources:[SNAPSHOT_SOURCE],nativeDirectory:'rust/src/provenance_snapshot_generated',
  metaPath:'data/meta/provenance-snapshot-program.lino',namespace:'provenance_snapshot_generated',
  generatorPath:'scripts/generate-provenance-snapshot.mjs',metaRecord:'provenance-snapshot-program',
 });
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
 const mode=process.argv[2]??'--check';
 if(mode!=='--check'&&mode!=='--write')throw Error('snapshot_generation_mode_required');
 const at=process.argv.indexOf('--meta-language');
 const compiler=at<0?process.env.FORMAL_AI_META_LANGUAGE:process.argv[at+1];
 const result=await generateSnapshotProgram(root,compiler);
 const drift=[];
 for(const [relative,text] of Object.entries(result.outputs)){
  const target=path.join(root,relative);
  if(!fs.existsSync(target)||fs.readFileSync(target,'utf8')!==text){
   drift.push(relative);
   if(mode==='--write'){fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,text);}
  }
 }
 if(mode==='--check'&&drift.length)throw Error('snapshot_program_drift:'+drift.join(','));
 console.log('Canonical snapshot program matches pinned checked IR');
}
