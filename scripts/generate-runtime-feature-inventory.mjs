#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {buildRuntimeFeatureInventory} from './lib/runtime-feature-inventory.mjs';
export const INVENTORY_FILE='data/meta/runtime-feature-coverage.json';
export function serializeRuntimeFeatureInventory(inventory){
 const fields=Object.entries(inventory).map(([key,value])=>{
  const rendered=Array.isArray(value)
   ? '[\n'+value.map(row=>'    '+JSON.stringify(row)).join(',\n')+'\n  ]'
   : JSON.stringify(value);
  return '  '+JSON.stringify(key)+': '+rendered;
 });
 return '{\n'+fields.join(',\n')+'\n}\n';
}
export function renderRuntimeFeatureInventory(read){
 return serializeRuntimeFeatureInventory(buildRuntimeFeatureInventory(read));
}
export function checkRuntimeFeatureInventory(read,actual){return renderRuntimeFeatureInventory(read)===actual;}
if(process.argv[1]===fileURLToPath(import.meta.url)){
 const args=process.argv.slice(2);if(args.length!==1||!['--check','--write'].includes(args[0]))throw Error('use --check or --write');
 const root=path.resolve(fileURLToPath(new URL('..',import.meta.url)));
 const expected=renderRuntimeFeatureInventory(file=>fs.readFileSync(path.join(root,file),'utf8'));
 const target=path.join(root,INVENTORY_FILE);
 if(args[0]==='--write')fs.writeFileSync(target,expected);
 else if(fs.readFileSync(target,'utf8')!==expected){console.error('runtime feature coverage inventory drift');process.exitCode=1;}
}
