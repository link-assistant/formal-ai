#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {
  fileURLToPath,
  pathToFileURL}
from 'node:url';
import {
  requirementEvidenceMatrix,
  checkRequirementEvidenceMatrix}
from './lib/requirement-evidence-matrix.mjs';
export function main(args=process.argv.slice(2)) {
  const mode=args[0];
  if(!['--write',
  '--check'].includes(mode))throw new Error('explicit --write or --check required');
  const options=new Map();
  for(let index=1;
  index<args.length;
  index+=2){
    if(!['--root',
    '--output',
    '--declarations',
    '--candidates'].includes(args[index])||!args[index+1]||options.has(args[index]))throw new Error('invalid matrix arguments');
    options.set(args[index],
    args[index+1]);
  }
  const root=path.resolve(options.get('--root')??path.join(path.dirname(fileURLToPath(import.meta.url)),
  '..'));
  const output=options.get('--output');
  if(!output)throw new Error('explicit matrix output required');
  const declaredIds=options.has('--declarations')?JSON.parse(fs.readFileSync(options.get('--declarations'),
  'utf8')):[];
  const candidateManifests=options.has('--candidates')?JSON.parse(fs.readFileSync(options.get('--candidates'),
  'utf8')):[];
  if(!Array.isArray(declaredIds)||declaredIds.some(value=>typeof value!=='string'||!/^R\d[A-Za-z0-9_-]*$/u.test(value))||!Array.isArray(candidateManifests)||candidateManifests.some(value=>typeof value!=='string'))throw new Error('invalid explicit matrix scope');
  const config={
    declaredIds,
    candidateManifests}
  ;
  const matrix=mode==='--check'?checkRequirementEvidenceMatrix(JSON.parse(fs.readFileSync(output,
  'utf8')),
  root,
  config):requirementEvidenceMatrix(root,
  config);
  if(mode==='--write')fs.writeFileSync(output,
  JSON.stringify(matrix,
  null,
  2)+'\n');
  console.log(JSON.stringify({
    records:matrix.records.length,
    counts:matrix.counts,
    freshAcceptance:matrix.freshAcceptance}
  ));
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href)main();
