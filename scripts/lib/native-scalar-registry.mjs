// Temporary source AST compiler for public primitive constants and unit enums.
// The maintained translator currently carries usize constants, enum and impl
// items. Retirement requires executable maintained twins for these exact shapes.
import {readFileSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {lex,topLevelItems} from '../self-translation/lexer.mjs';
import {tokenize} from './rust-specification-cases.mjs';
import {close,split} from './rust-specification-values.mjs';
import {nativeStringContract} from './native-string-contract.mjs';
const root=resolve(import.meta.dirname,'../..');
const hash=value=>createHash('sha256').update(value).digest('hex');
const ranges=new Map([['u8',255],['u16',65535],['u32',4294967295],['usize',65535]]);
const derives=new Set(['Debug','Clone','Copy','PartialEq','Eq','Default']);
function scalar(tokens,nativeType) {
  if(tokens.length!==1)throw Error('unknown scalar expression');
  if(nativeType==='bool'&&['true','false'].includes(tokens[0].text))return {value:tokens[0].text==='true',type:'boolean'};
  if(!ranges.has(nativeType)||tokens[0].kind!=='number'||!/^\d+$/u.test(tokens[0].text))throw Error('unknown native scalar type or literal');
  const value=Number(tokens[0].text);
  if(!Number.isSafeInteger(value)||value<0||value>ranges.get(nativeType))throw Error('native scalar exceeds qualified exact range');
  return {value,type:'number'};
}
function attributes(tokens,variant=false) {
  let cursor=0;const seenAttributes=new Set();
  while(tokens[cursor]?.text==='#'&&tokens[cursor+1]?.text==='[') {
    const end=close(tokens,cursor+1),body=tokens.slice(cursor+2,end);
    if(variant) {if(seenAttributes.has('default'))throw Error('duplicate default attribute');seenAttributes.add('default');if(body.map(t=>t.text).join('')!=='default')throw Error('unknown enum variant attribute');}
    else if(body[0]?.text==='derive'&&body[1]?.text==='('&&close(body,1)===body.length-1) {
      const names=split(body.slice(2,-1));
      for(const part of names){const name=part.map(token=>token.text).join('');if(seenAttributes.has(name))throw Error('duplicate derive');seenAttributes.add(name);}
      if(names.some(part=>part.length!==1||!derives.has(part[0].text)))throw Error('unknown enum derive');
    } else throw Error('unknown native item attribute');
    cursor=end+1;
  }
  return tokens.slice(cursor);
}
function unitEnum(raw) {
  nativeStringContract(raw);
  const tokens=attributes(tokenize(raw));
  if(tokens[0]?.text!=='pub'||tokens[1]?.text!=='enum'||tokens[2]?.kind!=='word'||tokens[3]?.text!=='{'||close(tokens,3)!==tokens.length-1)throw Error('unknown public unit enum shape');
  const defaultVariants=[];
  const variants=split(tokens.slice(4,-1)).map(part=> {
    if(part[0]?.text==='#')defaultVariants.push(part.at(-1)?.text);
    const bare=attributes(part,true);
    if(bare.length!==1||bare[0].kind!=='word')throw Error('unknown enum fields or discriminants');
    return bare[0].text;
  });
  if(!variants.length||new Set(variants).size!==variants.length)throw Error('empty or duplicate enum variants');
  const original=tokenize(raw),derived=new Set();let attributeAt=0;
  while(original[attributeAt]?.text==='#'&&original[attributeAt+1]?.text==='['){
    const end=close(original,attributeAt+1),body=original.slice(attributeAt+2,end);
    if(body[0]?.text==='derive')for(const name of split(body.slice(2,-1)))derived.add(name[0]?.text);
    attributeAt=end+1;
  }
  if(!['Copy','Clone','PartialEq'].every(name=>derived.has(name)))throw Error('non-Copy or unqualified owned enum derivation');
  const derivesDefault=derived.has('Default');
  if(derivesDefault?defaultVariants.length!==1:defaultVariants.length!==0)throw Error('unknown derived enum default');
  return {name:tokens[2].text,variants,defaultVariant:defaultVariants[0]};
}
function scalarMethod(tokens,owned) {
  const start=tokens.findIndex(token=>token.text==='pub');
  if(start<0)throw Error('native method is not public');
  if(tokens.slice(0,start).map(t=>t.text).join('')!=='#[must_use]'&&start!==0)throw Error('unknown method attribute');
  const declaration=tokens.slice(start);
  if(declaration.slice(0,
3).map(t=>t.text).join('')!=='pubconstfn'||declaration[3]?.kind!=='word'||declaration[4]?.text!=='('||declaration[5]?.text!=='self'||declaration[6]?.text!==')'||declaration[7]?.text!=='-'||declaration[8]?.text!=='>'||declaration[9]?.kind!=='word'||declaration[10]?.text!=='{'||close(declaration,






10)!==declaration.length-1)throw Error('unknown scalar method signature');

  const name=declaration[3].text,nativeType=declaration[9].text,body=declaration.slice(11,-1);
  if(nativeType==='bool'&&body[0]?.text==='matches'&&body[1]?.text==='!'&&body[2]?.text==='('&&close(body,2)===body.length-1){
    if(body[3]?.text!=='self'||body[4]?.text!==',')throw Error('unknown matches receiver');
    const selected=[];let cursor=5;while(cursor<body.length-1){
      if(body[cursor]?.text!=='Self'||body[cursor+1]?.text!==':'||body[cursor+2]?.text!==':'||!owned.variants.includes(body[cursor+3]?.text))throw Error('unknown matches enum pattern');
      selected.push(body[cursor+3].text);cursor+=4;
      if(cursor<body.length-1&&body[cursor++]?.text!=='|')throw Error('unknown matches pattern tail');
    }
    if(!selected.length||new Set(selected).size!==selected.length)throw Error('duplicate or empty matches patterns');
    return {name,nativeType,returnType:'boolean',cases:owned.variants.map(variant=>({variant,value:selected.includes(variant)}))};
  }
  if(body[0]?.text!=='match'||body[1]?.text!=='self'||body[2]?.text!=='{'||close(body,2)!==body.length-1)throw Error('unknown scalar method body or effects');
  const cases=[];
  for(const arm of split(body.slice(3,-1))) {
    const arrow=arm.findIndex((token,index)=>token.text==='='&&arm[index+1]?.text==='>');
    if(arrow<0)throw Error('scalar match arm absent');
    const patterns=[];let cursor=0;
    while(cursor<arrow) {
      if(arm[cursor]?.text!=='Self'||arm[cursor+1]?.text!==':'||arm[cursor+2]?.text!==':'||arm[cursor+3]?.kind!=='word'||!owned.variants.includes(arm[cursor+3].text))throw Error('unknown scalar enum pattern');
      patterns.push(arm[cursor+3].text);cursor+=4;
      if(cursor<arrow&&arm[cursor++]?.text!=='|')throw Error('unknown scalar pattern tail');
    }
    const result=scalar(arm.slice(arrow+2),nativeType);
    for(const variant of patterns)cases.push({variant,value:result.value});
  }
  if(cases.length!==owned.variants.length||new Set(cases.map(entry=>entry.variant)).size!==owned.variants.length)throw Error('nonexhaustive or duplicate scalar enum match');
  return {name,nativeType,returnType:nativeType==='bool'?'boolean':'number',cases};
}
export function generateScalars(source,path='rust/src/summarization/mod.rs',namespace='summarization',publicRoot=readFileSync(root+'/rust/src/lib.rs','utf8')) {
  // Literal syntax is qualified only inside owned enum/constant/method spans.
  const lexical=lex(source,'Rust').filter(t=>t.type!=='comment');
  if(lexical.some(t=>t.text==='trait'))throw Error('unknown native trait environment');
  if(lexical.some(t=>t.text==='use')&&(!existsSync(root+'/'+path)||source!==readFileSync(root+'/'+path,'utf8')))throw Error('unqualified scalar import environment');
  if(!topLevelItems(lex(publicRoot,'Rust')).some(item=>item.tokens.filter(t=>t.type!=='comment').map(t=>t.text).join('')==='pubmod'+namespace+';'))throw Error('unqualified public scalar module');
  if(lexical.some(token=>['macro_rules','macro_rules!','macro'].includes(token.text)))throw Error('unknown native macro environment');
  const importItems=topLevelItems(lex(source,'Rust')).filter(item=>item.tokens.some(token=>token.text==='use'));
  if(importItems.some(item=>item.tokens.some(token=>['Debug','Clone','Copy','PartialEq','Eq','Default','matches'].includes(token.text))))throw Error('shadowed builtin derive or matches macro');
  const items=topLevelItems(lex(source,'Rust')).map(item=>({raw:source.slice(item.start,item.end),span:{start:item.start,end:item.end},tokens:tokenize(source.slice(item.start,item.end))}));
  const sourceEnumNames=new Set();
  for(const item of items){const tokens=item.tokens,index=tokens.findIndex(token=>token.text==='enum');
    if(index>=0){const name=tokens[index+1]?.text;if(sourceEnumNames.has(name))throw Error('duplicate source enum');sourceEnumNames.add(name);}
  }
  const programs=[],refusals=[],enums=new Map();
  const witnesses=[{path,sha256:hash(source)},{path:'rust/src/lib.rs',sha256:hash(publicRoot)}];
  const binding=(item,properties)=>({...properties,nativeSourceQualified:true,sourceSpan:item.span,sourceWitnesses:witnesses});
  for(const item of items) {
    const bare=item.tokens[0]?.text==='#'?item.tokens.slice(item.tokens.findIndex(t=>t.text==='pub')):item.tokens;
    if(bare[0]?.text==='pub'&&bare[1]?.text==='enum') {
      if(enums.has(bare[2]?.text))throw Error('duplicate source enum');
      try {
        const owned=unitEnum(item.raw);
        if(enums.has(owned.name))throw Error('duplicate source enum');
        owned.path='formal_ai::'+namespace+'::'+owned.name;enums.set(owned.name,owned);
        if(owned.defaultVariant)programs.push(binding(item,{path:owned.path+'::default',kind:'nativeEnumDefault',enumPath:owned.path,variant:owned.defaultVariant,signature:[],returnType:'enum:'+owned.path}));
        for(const variant of owned.variants)programs.push(binding(item,{path:owned.path+'::'+variant,kind:'nativeEnumVariant',enumPath:owned.path,variant,returnType:'enum:'+owned.path}));
      } catch(error){refusals.push({item:'enum',reason:error.message});}
    }
    if(bare[0]?.text==='pub'&&bare[1]?.text==='const') {
      try {
        nativeStringContract(item.raw);
        if(item.tokens[0]?.text!=='pub'||bare.length!==8||bare[2]?.kind!=='word'||bare[3]?.text!==':'||bare[5]?.text!=='='||bare[7]?.text!==';')throw Error('unknown public scalar constant shape');
        const nativeType=bare[4].text,result=scalar([bare[6]],nativeType);
        programs.push(binding(item,{path:'formal_ai::'+namespace+'::'+bare[2].text,kind:'nativeConstant',nativeType,value:result.value,returnType:result.type}));
      }catch(error){refusals.push({item:'constant',reason:error.message});}
    }
  }
  for(const item of items) {
    const tokens=item.tokens;
    if(tokens[0]?.text==='#'&&tokens.some(token=>token.text==='impl')&&tokens.some(token=>enums.has(token.text)))throw Error('unknown owned impl attributes');
    if(tokens[0]?.text!=='impl')continue;
    const open=tokens.findIndex(t=>t.text==='{'),header=tokens.slice(0,open).map(t=>t.text);
    if(header.includes('for')&&header.some(name=>enums.has(name)))throw Error('custom trait implementation for owned enum');
    if(header.length!==2||!enums.has(header[1]))continue;
    const owned=enums.get(header[1]);owned.methodNames??=new Set();let cursor=open+1;
    while(cursor<tokens.length-1) {
      let body=cursor;while(body<tokens.length&&tokens[body].text!=='{')body++;
      if(body>=tokens.length)throw Error('native method body absent');
      const end=close(tokens,body),method=tokens.slice(cursor,end+1);cursor=end+1;
      const functionAt=method.findIndex(token=>token.text==='fn'),methodName=method[functionAt+1]?.text;
      if(functionAt<0||owned.methodNames.has(methodName))throw Error('absent or duplicate owned method');owned.methodNames.add(methodName);
      if(method.some((token,index)=>token.text==='fn'&&method[index+1]?.text==='default'))throw Error('inherent default shadows derived default');
      try {
        const compiled=scalarMethod(method,owned),methodPath=owned.path+'::'+compiled.name;
        if(programs.some(program=>program.path===methodPath))throw Error('duplicate source enum method');
        programs.push(binding(item,{path:methodPath,kind:'nativeEnumScalarMatch',enumPath:owned.path,signature:['enum:'+owned.path],...compiled}));
      }catch(error){refusals.push({item:'method',enum:owned.name,reason:error.message});}
    }
  }
  return {programs,refusals};
}
export const TEMPORARY_SCALAR_WORKAROUND={
  nativeShape:'Public bounded primitive literals; unit enums; exhaustive self-match methods returning bounded primitive literals',
  blockers:['a parameter or result without a JSDoc number, boolean or string type: usize','a Rust item outside portable-pure-v1'],
  sourceQualification:'Full producer and public-module SHA witnesses plus deterministic generation --check',
  retirement:'Remove when maintained translation emits executable twins for these exact constant, enum and impl shapes',
  upstreamIssue:null,
  unknownUsizeWidth:'Only 0..65535 qualifies; larger usize constants refuse'
};

function exportedNames(source) {
  const names=new Set();
  for(const item of topLevelItems(lex(source,'Rust'))) {
    const tokens=tokenize(source.slice(item.start,item.end)),publicAt=tokens.findIndex(t=>t.text==='pub');
    if(publicAt<0)continue;
    const declaration=tokens.slice(publicAt+1);
    if(declaration[0]?.text==='use'){for(const token of declaration.slice(1))if(token.kind==='word')names.add(token.text);}
    else if(['fn','enum','struct','const','type','mod'].includes(declaration[0]?.text)&&declaration[1]?.kind==='word')names.add(declaration[1].text);
  }
  return names;
}
export function scalarTestImportContract(source,ownerRoot,binding) {
  const importWitnesses=[];
  const namespace=binding.path.split('::')[1],rootExports=exportedNames(readFileSync(ownerRoot+'/rust/src/lib.rs','utf8'));
  const moduleExports=exportedNames(readFileSync(ownerRoot+'/'+binding.sourceWitnesses[0].path,'utf8'));
  function imported(tokens,prefix='') {
    const group=tokens.findIndex(t=>t.text==='{');
    if(group>=0){const base=prefix+tokens.slice(0,group).map(t=>t.text).join('');for(const part of split(tokens.slice(group+1,close(tokens,group))))imported(part,base);return;}
    const alias=tokens.findIndex(t=>t.text==='as'),path=prefix+(alias<0?tokens:tokens.slice(0,alias)).map(t=>t.text).join('');
    const parts=path.split('::');
    if(parts[0]!=='formal_ai'||parts.includes('*'))throw Error('unknown scalar test import');
    if(parts.length===2&&rootExports.has(parts[1]))return;
    if(parts[1]===namespace&&parts.length===3&&moduleExports.has(parts[2]))return;
    if(parts.length===3&&rootExports.has(parts[1])){
      const paths=['rust/src/'+parts[1]+'.rs','rust/src/'+parts[1]+'/mod.rs'].filter(path=>existsSync(ownerRoot+'/'+path));
      if(paths.length===1){const path=paths[0],body=readFileSync(ownerRoot+'/'+path,'utf8');
        if(exportedNames(body).has(parts[2])){importWitnesses.push({path,sha256:hash(body)});return;}
      }
    }
    throw Error('unknown scalar import path');
  }
  for(const item of topLevelItems(lex(source,'Rust'))) {
    const tokens=tokenize(source.slice(item.start,item.end));
    if(tokens.some(token=>['macro_rules','macro_rules!','macro'].includes(token.text)))throw Error('unknown scalar caller macro scope');
    if(tokens[0]?.text==='trait'||tokens[0]?.text==='extern'||(tokens[0]?.text==='impl'&&tokens.slice(0,tokens.findIndex(t=>t.text==='{')).some(t=>t.text==='for')))throw Error('unknown scalar test trait or crate scope');
    if(tokens[0]?.text==='use'){if(tokens.at(-1)?.text!==';')throw Error('unknown import tail');imported(tokens.slice(1,-1));}
  }
  return importWitnesses;
}

export function generateEnumPredicates(ownerRoot=root){
  const publicRoot=readFileSync(ownerRoot+'/rust/src/lib.rs','utf8'),programs=[],refusals=[];
  for(const item of topLevelItems(lex(publicRoot,'Rust'))){
    const tokens=tokenize(publicRoot.slice(item.start,item.end));
    if(tokens.length!==4||tokens[0].text!=='pub'||tokens[1].text!=='mod'||tokens[3].text!==';')continue;
    const namespace=tokens[2].text;
    if(namespace==='summarization')continue;
    const alternatives=['rust/src/'+namespace+'.rs','rust/src/'+namespace+'/mod.rs'];
    const paths=alternatives.filter(path=>existsSync(ownerRoot+'/'+path));if(paths.length!==1)continue;
    const path=paths[0],source=readFileSync(ownerRoot+'/'+path,'utf8');
    try{
      const generated=generateScalars(source,path,namespace,publicRoot);
      const owners=new Set(generated.programs.filter(program=>program.kind==='nativeEnumDefault').map(program=>program.enumPath));
      const predicates=generated.programs.filter(program=>program.kind==='nativeEnumScalarMatch'&&program.returnType==='boolean'&&owners.has(program.enumPath));
      const qualified=new Set(predicates.map(program=>program.enumPath));
      programs.push(...generated.programs.filter(program=>qualified.has(program.enumPath)));
    }catch(error){refusals.push({path,reason:error.message});}
  }
  return {programs,refusals};
}

export const TEMPORARY_ENUM_WORKAROUND={
  nativeShape:'Builtin derived unit enum Default with one default variant and builtin self matches boolean predicates',
  blocker:'a Rust item outside portable-pure-v1',
  sourceQualification:'Public root plus full producer and caller import DAG SHA witnesses; source AST regeneration before observations',
  retirement:'Remove when exact enum and impl shapes have executable maintained twins',
  upstreamIssue:null
};
