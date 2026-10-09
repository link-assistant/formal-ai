// Scratch-only source registration audit. Reuses the unchanged production Rust lexer.
import assert from 'node:assert/strict';
import {readFileSync,existsSync,lstatSync} from 'node:fs';
import {dirname,basename,resolve,relative,join} from 'node:path';
import {createHash} from 'node:crypto';
import {tokenize} from './rust-specification-cases.mjs';
import {close} from './rust-specification-values.mjs';
const sha=value=>createHash('sha256').update(value).digest('hex');
export function cargoTestTargets(root){
 const file=join(root,'rust/Cargo.toml'),text=readFileSync(file,'utf8'),targets=[];
 for(const block of text.split(/^\[\[test\]\]\s*$/mu).slice(1)){
  const body=block.split(/^\[/mu)[0],name=/^name\s*=\s*"([A-Za-z_][A-Za-z_0-9-]*)"\s*$/mu.exec(body)?.[1],path=/^path\s*=\s*"([^"\\\r\n]+)"\s*$/mu.exec(body)?.[1];
  assert.ok(name&&path,'unsupported explicit Cargo test target');targets.push({name,path:resolve(root,'rust',path),cargoSHA:sha(text)});
 }
 assert.ok(targets.length>0);assert.equal(new Set(targets.map(t=>t.name)).size,targets.length);return targets;
}
export function readNativeTestModuleGraph(root){
 root=resolve(root);const functions=[],unsupported=[],sources=new Map();
 function file(path,target,prefix,trace,active){
  if(active.has(path)){unsupported.push({path,reason:'module cycle',trace});return;}
  if(!existsSync(path)){unsupported.push({path,reason:'missing module file',trace});return;}
  assert.equal(lstatSync(path).isSymbolicLink(),false,'symlink module source');
  assert.ok(relative(root,path)!=='..'&&!relative(root,path).startsWith('../'),'module leaves source checkout');
  const text=readFileSync(path,'utf8'),sourceSHA=sha(text);sources.set(path,sourceSHA);
  const tokens=tokenize(text.replace(/^#!(?!\[)[^\r\n]*(?:\r?\n|$)/u,'')),directory=(trace.length===1||basename(path)==='mod.rs')?dirname(path):join(dirname(path),basename(path,'.rs'));
  scope(tokens,path,directory,target,prefix,trace,new Set([...active,path]),sourceSHA,false);
 }
 function scope(tokens,path,directory,target,prefix,trace,active,sourceSHA,inline){
  let attrs=[];const declared=[];let opaque=false;const startingFunction=functions.length;
  for(let index=0;index<tokens.length;index++){
   const token=tokens[index];
   if(token.text==='#'){
    const open=tokens[index+1]?.text==='!'?index+2:index+1;assert.equal(tokens[open]?.text,'[','unsupported attribute token in '+relative(root,path)+' at '+index);const end=close(tokens,open);assert.ok(end<tokens.length,'unclosed Rust attribute');
    if(open===index+1)attrs.push(tokens.slice(open+1,end));index=end;continue;
   }
   if(token.kind==='word'&&token.text==='mod'&&tokens[index+1]?.kind==='word'){
    const name=tokens[index+1].text,next=tokens[index+2],pathAttrs=attrs.filter(a=>a[0]?.text==='path'),conditionalPath=attrs.some(a=>a[0]?.text==='cfg_attr'&&a.some(t=>t.text==='path'));
    const registration={parent:relative(root,path),parentSHA:sourceSHA,module:name,attributes:attrs.map(a=>a.map(t=>t.text).join(' '))};const childTrace=[...trace,registration];
    if(next?.text==='{'){
     const end=close(tokens,index+2);assert.ok(end<tokens.length);scope(tokens.slice(index+3,end),path,join(directory,name),target,[...prefix,name],childTrace,active,sourceSHA,true);index=end;
    }else if(next?.text===';'){
     index+=2;
     if(pathAttrs.length>1||conditionalPath||(inline&&pathAttrs.length)){unsupported.push({path:relative(root,path),reason:'ambiguous or unsupported conditional/inline path attribute',trace:childTrace});attrs=[];continue;}
     let candidates;
     if(pathAttrs.length){const a=pathAttrs[0];if(a.length!==3||a[1].text!=='='||a[2].kind!=='string'){unsupported.push({path,reason:'nonliteral path attribute',trace:childTrace});attrs=[];continue;}candidates=[resolve(dirname(path),a[2].text)];registration.kind='path';}
     else{candidates=[join(directory,name+'.rs'),join(directory,name,'mod.rs')].filter(existsSync);registration.kind='default';}
     if(candidates.length!==1){unsupported.push({path,reason:candidates.length?'ambiguous module filenames':'unmapped module filenames',trace:childTrace,candidates});attrs=[];continue;}
     file(candidates[0],target,[...prefix,name],childTrace,active);
    }else{unsupported.push({path,reason:'unsupported module declaration',trace:childTrace});}
    attrs=[];continue;
   }
   if(token.kind==='word'&&token.text==='fn'&&tokens[index+1]?.kind==='word'){
    const name=tokens[index+1].text;let body=index+2;while(body<tokens.length&&tokens[body].text!=='{'&&tokens[body].text!==';')body++;
    const isTest=attrs.some(a=>a.length===1&&a[0].text==='test'),ignored=attrs.some(a=>a[0]?.text==='ignore');
    if(body<tokens.length&&tokens[body].text==='{'){
     const end=close(tokens,body);assert.ok(end<tokens.length);if(isTest&&!ignored){const row={target:target.name,source:relative(root,path),function:name,caller:[...prefix,name].join('::'),sourceSHA,cargoSHA:target.cargoSHA,trace,attributes:attrs.map(a=>a.map(t=>t.text).join(' '))};functions.push(row);declared.push(row);}index=end;
    }else if(isTest){unsupported.push({path,reason:'test declaration without supported body',trace});index=body;}
    attrs=[];continue;
   }
   if(token.kind==='word'&&['const','static','use'].includes(token.text)){
    let end=index+1;for(;end<tokens.length;end++){if(['{','[','('].includes(tokens[end].text))end=close(tokens,end);else if(tokens[end].text===';')break;}index=end;attrs=[];continue;
   }
   if(token.kind==='word'&&tokens[index+1]?.text==='!'){
    opaque=true;unsupported.push({path,reason:'opaque item macro may declare additional tests or modules',macro:token.text,trace});let open=index+2;if(['{','[','('].includes(tokens[open]?.text))index=close(tokens,open);attrs=[];continue;
   }
   if(['{','[','('].includes(token.text)){index=close(tokens,index);attrs=[];}
   else if(token.text===';')attrs=[];
  }
  if(opaque)for(const row of functions.slice(startingFunction))row.opaqueSource=true;
 }
 const targets=cargoTestTargets(root);for(const target of targets)file(target.path,target,[],[{kind:'cargo-test-target',target:target.name,path:relative(root,target.path),cargoSHA:target.cargoSHA}],new Set());
 return {functions,unsupported,sources:[...sources].map(([path,sourceSHA])=>({path:relative(root,path),sourceSHA})),targets:targets.map(t=>({...t,path:relative(root,t.path)}))};
}
export function bindOriginalSourceCase(graph,id,actualListing=null){
 const split=id.lastIndexOf('::');assert.ok(split>0);const source=id.slice(0,split),name=id.slice(split+2);
 const matches=graph.functions.filter(row=>row.source===source&&row.function===name);assert.equal(matches.length,1,'source test has no unique registration: '+id);
 const row=matches[0];assert.notEqual(row.opaqueSource,true,'opaque item macro prevents source proof');
 const full=graph.functions.filter(other=>other.target===row.target&&other.caller===row.caller);assert.equal(full.length,1,'ambiguous full native caller');
 const runtimeKey=row.target+'\t'+row.caller;
 if(actualListing!==null)assert.equal(actualListing.filter(key=>key===runtimeKey).length,1,'actual runtime listing does not uniquely confirm registered caller');
 return {...row,runtimeKey,suppliedRuntimeListingMatches:actualListing!==null};
}
