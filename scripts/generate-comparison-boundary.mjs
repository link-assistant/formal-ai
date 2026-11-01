#!/usr/bin/env node
// Intended owner: scripts/generate-comparison-boundary.mjs.
// Only one canonical portable source enters this profile. It cannot regenerate
// unrelated modules or turn the deferred full translator work into a gate.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {loadResearchCompiler,generateCodingResearchProgram} from './generate-coding-research-program.mjs';
import {workflowPin} from './translate-js-rust.mjs';
export const SOURCE='js/agentic/portable/comparison_boundary_kernel.mjs';
export const PROFILE={sources:[SOURCE],nativeDirectory:'rust/src/comparison_boundary_generated',
  metaPath:'data/meta/comparison-boundary-program.lino',namespace:'comparison_boundary_generated',
  generatorPath:'scripts/generate-comparison-boundary.mjs',metaRecord:'comparison-boundary-program',
  description:'Finite interval role binding and contract witnesses from canonical portable JavaScript.'};
export async function generateComparisonBoundary(directory,compilerDirectory,write=false) {
  const compiler=await loadResearchCompiler(compilerDirectory,workflowPin(fs.readFileSync(path.join(directory,'.github/workflows/layered-ci.yml'),'utf8')));
  const generated=generateCodingResearchProgram([{path:SOURCE,source:fs.readFileSync(path.join(directory,SOURCE),'utf8')}],compiler,PROFILE);
  const drift=[];
  for (const [relative,expected] of Object.entries(generated.outputs)) {
    if (!(relative===PROFILE.metaPath||relative.startsWith(PROFILE.nativeDirectory+'/')))throw Error('BoundaryGenerationScopeViolation');
    const target=path.join(directory,relative);
    if (!fs.existsSync(target)||fs.readFileSync(target,'utf8')!==expected) {
      drift.push(relative);
      if (write) {fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,expected);}
    }
  }
  return {drift,sourceHash:generated.sourceHash,irHash:generated.irHash,compilerPin:generated.compilerPin};
}
if (process.argv[1]&&import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href) {
  const args=process.argv.slice(2);
  if (args.length!==3||!['--check','--write'].includes(args[0])||args[1]!=='--meta-language')throw Error('ExpectedBoundaryModeAndPinnedCompiler');
  const directory=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
  const result=await generateComparisonBoundary(directory,path.resolve(args[2]),args[0]==='--write');
  console.log(JSON.stringify(result));
  if (args[0]==='--check'&&result.drift.length)process.exitCode=1;
}
