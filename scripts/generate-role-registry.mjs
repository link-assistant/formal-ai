#!/usr/bin/env node
// JavaScript twin of generate-role-registry.py. The Python writer remains the
// canonical independent parity oracle; local generation does not need Python.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const defaultRoot=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const slug=/^[a-z][a-z0-9_-]*$/u;
export function stripComment(source){
  let quoted=0;
  for(let index=0;index<source.length;index++){
    if('"\x27\x60'.includes(source[index]))quoted^=1;
    if(source[index]==='#'&&!quoted&&index>0&&source[index-1]===' ')return source.slice(0,index).trimEnd();
  }
  return source;
}
export function meaningFiles(root){
  const source=fs.readFileSync(path.join(root,'rust/src/seed/embedded_registry.rs'),'utf8');
  const files=new Map([...source.matchAll(/pub const (\w+): &str =\s*include_str!\("\.\.\/\.\.\/embedded\/(data\/seed\/[^"\r\n]+)"\)/gu)].map(match=>[match[1],match[2]]));
  const marker='pub const MEANING_FILES: &[&str] = &[';
  const start=source.indexOf(marker);
  if(start<0)throw Error('Missing MEANING_FILES registry');
  const end=source.indexOf('];',start+marker.length);
  if(end<0)throw Error('Unclosed MEANING_FILES registry');
  return [...source.slice(start+marker.length,end).matchAll(/\b(\w+_LINO)\b/gu)].map(match=>{
    const file=files.get(match[1]);
    if(!file)throw Error('Unbound meaning source: '+match[1]);
    return path.join(root,file);
  }).sort();
}
export function collect(root){
  const defined=new Set(),roles=new Set();
  for(const file of meaningFiles(root)){
    const stack=[];
    for(const raw of fs.readFileSync(file,'utf8').split('\n')){
      if(!raw.trim())continue;
      const indent=raw.length-raw.replace(/^ */u,'').length;
      const source=stripComment(raw.trim());
      if(!source)continue;
      while(stack.length&&stack.at(-1)[0]>=indent)stack.pop();
      let head=source.split(' ',1)[0];
      if(head.endsWith(':'))head=head.slice(0,-1);
      if(stack.at(-1)?.[1]==='meanings')defined.add(head);
      if(head==='role'&&source.includes(' ')){
        const value=source.slice(source.indexOf(' ')+1).trim().split(' ',1)[0];
        if(slug.test(value))roles.add(value);
      }
      stack.push([indent,head]);
    }
  }
  return {defined,roles};
}
export function render({defined,roles}){
  const output=[
    '# Canonical reserved-role registry (issue #398, PR #399).','#',
    '# Every \x60role <name>\x60 value used anywhere in data/seed/meanings*.lino is',
    '# declared here exactly once. A role is a reserved predicate over',
    '# meanings; this registry is its single definition so it can never',
    '# collide with — or drift from — the meaning graph. Regenerate with',
    '# \x60python3 scripts/generate-role-registry.py\x60; CI keeps it in lockstep.','#',
    '# kind meaning   -> the role name is also a defined meaning slug',
    '# kind predicate -> the role name is a reserved role-only identifier','roles'
  ];
  for(const role of [...roles].sort())output.push('  '+role,'    kind '+(defined.has(role)?'meaning':'predicate'));
  return output.join('\n')+'\n';
}
export function main(argv=process.argv.slice(2),root=defaultRoot){
  let check=false,output;
  for(let index=0;index<argv.length;index++){
    if(argv[index]==='--check')check=true;
    else if(argv[index]==='--output'){
      output=argv[++index];if(!output)throw Error('--output requires a path');
    }else if(argv[index].startsWith('-'))throw Error('Unknown role-registry option');
    else root=path.resolve(argv[index]);
  }
  const facts=collect(root),text=render(facts),target=output??path.join(root,'data/seed/roles.lino');
  if(check){
    if(!fs.existsSync(target)||fs.readFileSync(target,'utf8')!==text){console.error('role registry differs: '+target);return 1;}
  }else fs.writeFileSync(target,text);
  const both=[...facts.roles].filter(role=>facts.defined.has(role)).length;
  console.log('roles: '+facts.roles.size+' ('+both+' also meanings, '+(facts.roles.size-both)+' predicate-only)');
  console.log((check?'checked ':'wrote ')+target);return 0;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
  try{process.exitCode=main();}catch(error){console.error(error.message);process.exitCode=1;}
}
