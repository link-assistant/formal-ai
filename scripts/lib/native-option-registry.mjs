// Temporary source AST compiler; maintained translation carries impl_item.
// This compiler proves finite ASCII literal operations, not Unicode/std profiles.
import {readFileSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {lex,topLevelItems} from '../self-translation/lexer.mjs';
import {tokenize} from './rust-specification-cases.mjs';
import {close,split} from './rust-specification-values.mjs';
import {generateScalars,scalarTestImportContract} from './native-scalar-registry.mjs';
import {nativeStringContract} from './native-string-contract.mjs';
const ownerRoot=resolve(import.meta.dirname,'../..');
const hash=value=>createHash('sha256').update(value).digest('hex');
const protectedNames=new Set(['Option','Some','None','str','String','std','core','Copy','Clone','PartialEq','Eq','Default']);
const keywords=new Set(('as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn abstract become box do final macro override priv typeof unsized virtual yield try').split(' '));
function text(tokens){return tokens.map(token=>token.text).join('');}
function identifier(token){return token?.kind==='word'&&token.text!=='_'&&!token.text.startsWith('r#')&&!keywords.has(token.text);}
function syntax(tokens){
  if(tokens.some(token=>!['word','punct','number'].includes(token.kind)||(token.kind==='word'&&token.text.startsWith('r#'))))throw Error('unknown owned syntax token kind');
}
function importedNames(tokens){
  if(tokens.some(token=>token.text==='*'))throw Error('unknown wildcard import');
  const open=tokens.findIndex(token=>token.text==='{');
  if(open>=0){if(close(tokens,open)!==tokens.length-1)throw Error('unknown import tail');return split(tokens.slice(open+1,-1)).flatMap(importedNames);}
  const alias=tokens.findIndex(token=>token.text==='as');
  if(alias>=0){if(alias!==tokens.length-2||!identifier(tokens.at(-1)))throw Error('unknown import alias');return [tokens.at(-1).text];}
  if(!identifier(tokens.at(-1)))throw Error('unknown imported binding');return [tokens.at(-1).text];
}
function items(source){return topLevelItems(lex(source,'Rust')).map(item=>({raw:source.slice(item.start,item.end),span:{start:item.start,end:item.end},tokens:tokenize(source.slice(item.start,item.end))}));}
function asciiLiteral(tokens){
  if(tokens.length!==1||tokens[0].kind!=='string'||Array.from(tokens[0].text).some(character=>character.charCodeAt(0)>127))throw Error('unknown ASCII string literal');
  return tokens[0].text;
}
function arrow(arm){const index=arm.findIndex((token,offset)=>token.text==='='&&arm[offset+1]?.text==='>');if(index<0)throw Error('match arm arrow absent');return index;}
function variant(tokens,variants){
  syntax(tokens);
  if(tokens.length!==4||text(tokens.slice(0,3))!=='Self::'||!identifier(tokens[3])||!variants.includes(tokens[3].text))throw Error('unknown owned enum pattern');
  return tokens[3].text;
}
function method(tokens,owned){
  let offset=0;
  if(tokens[0]?.text==='#'){
    if(text(tokens.slice(0,4))!=='#[must_use]')throw Error('unknown method attribute');syntax(tokens.slice(0,4));offset=4;
  }
  const declaration=tokens.slice(offset);let cursor=0;
  if(declaration[cursor++]?.text!=='pub')throw Error('nonpublic method');
  const constant=declaration[cursor]?.text==='const';if(constant)cursor++;
  if(declaration[cursor++]?.text!=='fn'||!identifier(declaration[cursor]))throw Error('unknown function declaration');
  const name=declaration[cursor++].text;
  if(declaration[cursor]?.text!=='(')throw Error('parameter list absent');
  const parameterEnd=close(declaration,cursor),parameters=declaration.slice(cursor+1,parameterEnd);cursor=parameterEnd+1;
  const bodyAt=declaration.findIndex((token,index)=>index>=cursor&&token.text==='{');
  if(bodyAt<0||close(declaration,bodyAt)!==declaration.length-1)throw Error('unknown method body tail');
  const returns=declaration.slice(cursor,bodyAt);syntax(declaration.slice(0,bodyAt));
  const body=declaration.slice(bodyAt+1,-1);
  if(body[0]?.text!=='match')throw Error('unknown method effect/body');
  const matchAt=body.findIndex(token=>token.text==='{');
  if(matchAt<0||close(body,matchAt)!==body.length-1)throw Error('unknown match body effects');
  syntax(body.slice(0,matchAt));
  const arms=split(body.slice(matchAt+1,-1));
  if(text(parameters)==='self'&&text(returns)==="->&'staticstr"&&text(body.slice(1,matchAt))==='self'){
    const cases=arms.map(arm=>{const index=arrow(arm);return {variant:variant(arm.slice(0,index),owned.variants),value:asciiLiteral(arm.slice(index+2))};});
    if(cases.length!==owned.variants.length||new Set(cases.map(entry=>entry.variant)).size!==owned.variants.length)throw Error('nonexhaustive/duplicate string producer');
    return {name,kind:'nativeEnumStringMatch',signature:['enum:'+owned.path],returnType:'string',cases};
  }
  if(constant||parameters.length!==4||!identifier(parameters[0])||protectedNames.has(parameters[0].text)||text(parameters.slice(1))!==':&str'||text(returns)!=='->Option<Self>')throw Error('unknown Option parser signature');
  const parameter=parameters[0].text,input=text(body.slice(1,matchAt));
  const normalized=input===parameter+'.trim().to_ascii_lowercase().as_str()';
  if(input!==parameter&&!normalized)throw Error('unknown string normalization/effects');
  const cases=[];let fallback=false;
  for(const [position,arm] of arms.entries()){
    const index=arrow(arm),pattern=arm.slice(0,index),value=arm.slice(index+2);
    if(pattern.length===1&&pattern[0].kind==='word'&&pattern[0].text==='_'){
      syntax(value);if(fallback||position!==arms.length-1||text(value)!=='None')throw Error('unknown Option fallback');fallback=true;continue;
    }
    const input=asciiLiteral(pattern);syntax(value);
    if(value[0]?.text!=='Some'||value[1]?.text!=='('||close(value,1)!==value.length-1)throw Error('unknown Option constructor');
    cases.push({input,variant:variant(value.slice(2,-1),owned.variants)});
  }
  if(!fallback||!cases.length||new Set(cases.map(entry=>entry.input)).size!==cases.length)throw Error('missing fallback/duplicate string pattern');
  return {name,kind:'nativeOptionStringParse',signature:['string'],returnType:'option:enum:'+owned.path,normalized,cases};
}
export function generateOptions(source,path,namespace,publicRoot){
  nativeStringContract(source);
  const declarations=items(source),roots=items(publicRoot);
  const matching=roots.filter(item=>text(item.tokens)==='pubmod'+namespace+';');
  if(matching.length!==1)throw Error('ambiguous public Option module');syntax(matching[0].tokens);
  for(const item of roots){
    if(item===matching[0])continue;
    const tokens=item.tokens,bodyAt=tokens.findIndex(token=>token.text==='{'),header=tokens.slice(0,bodyAt<0?tokens.length:bodyAt);
    if(header.some(token=>['no_implicit_prelude','macro','macro_rules','macro_rules!','macro_use'].includes(token.text)))throw Error('unknown root Option prelude/macro scope');
    const declared=header.findIndex(token=>['mod','type','enum','struct','const','static','fn','extern'].includes(token.text));
    if(declared>=0&&(header[declared+1]?.text===namespace||protectedNames.has(header[declared+1]?.text)))throw Error('ambiguous root Option namespace');
    if(header.some(token=>token.text==='use')){
      const position=tokens.findIndex(token=>token.text==='use');
      for(const name of importedNames(tokens.slice(position+1,-1)))if(name===namespace||protectedNames.has(name))throw Error('shadowed root Option import');
    }
  }
  for(const item of declarations){
    const bodyAt=item.tokens.findIndex(token=>token.text==='{'),header=item.tokens.slice(0,bodyAt<0?item.tokens.length:bodyAt);
    if(header.some(token=>['macro_rules','macro_rules!','macro','trait','extern'].includes(token.text)))throw Error('unknown producer macro/trait/crate scope');
    if(header[0]?.text==='#'&&header[1]?.text==='!')throw Error('unknown owned inner attribute/prelude');
    const useAt=header.findIndex(token=>token.kind==='word'&&token.text==='use');
    if(useAt>=0){
      if(useAt!==0&&!(useAt===1&&header[0]?.kind==='word'&&header[0]?.text==='pub'))throw Error('unknown owned import attribute');
      syntax(item.tokens);
      const names=importedNames(item.tokens.slice(useAt+1,-1));
      if(new Set(names).size!==names.length)throw Error('duplicate imported Option binding');
      for(const name of names)if(protectedNames.has(name))throw Error('unknown builtin Option import scope');
    }
    const declarationAt=header.findIndex(token=>['type','struct','enum','mod','const','static','fn'].includes(token.text));
    if(declarationAt>=0&&protectedNames.has(header[declarationAt+1]?.text))throw Error('shadowed builtin Option type/constructor');
  }
  const scalars=generateScalars(source,path,namespace,publicRoot),owners=new Map();
  for(const program of scalars.programs)if(program.kind==='nativeEnumVariant'){
    if(!identifier({kind:'word',text:program.enumPath.split('::').at(-1)})||!identifier({kind:'word',text:program.variant}))throw Error('unknown Rust enum identifier');
    if(!owners.has(program.enumPath))owners.set(program.enumPath,{path:program.enumPath,variants:[]});owners.get(program.enumPath).variants.push(program.variant);
  }
  const programs=[],refusals=[],compiled=new Map(),seen=new Map();
  for(const item of declarations){
    const tokens=item.tokens,bodyAt=tokens.findIndex(token=>token.text==='{');
    if(tokens[0]?.text!=='impl'||bodyAt<0)continue;
    const enumPath='formal_ai::'+namespace+'::'+tokens[1]?.text,owned=owners.get(enumPath);
    if(!owned)continue;syntax(tokens.slice(0,bodyAt));
    if(bodyAt!==2||close(tokens,bodyAt)!==tokens.length-1)throw Error('unknown owned impl/header');
    if(!seen.has(enumPath))seen.set(enumPath,new Set());
    let cursor=bodyAt+1;
    while(cursor<tokens.length-1){
      const endAt=tokens.findIndex((token,index)=>index>=cursor&&token.text==='{');
      if(endAt<0)throw Error('owned method body absent');const end=close(tokens,endAt),raw=tokens.slice(cursor,end+1);cursor=end+1;
      const functionAt=raw.findIndex(token=>token.kind==='word'&&token.text==='fn'),name=raw[functionAt+1]?.text;
      if(functionAt<0||!identifier(raw[functionAt+1])||seen.get(enumPath).has(name))throw Error('duplicate/unknown owned method');seen.get(enumPath).add(name);
      try{
        const result=method(raw,owned),program={path:enumPath+'::'+result.name,...result,enumPath,nativeSourceQualified:true,nativeOptionQualified:true,
          sourceSpan:item.span,sourceWitnesses:[{path,sha256:hash(source)},{path:'rust/src/lib.rs',sha256:hash(publicRoot)}]};
        if(!compiled.has(enumPath))compiled.set(enumPath,[]);compiled.get(enumPath).push(program);
      }catch(error){refusals.push({path,enumPath,method:name,reason:error.message});}
    }
  }
  for(const [enumPath,methods] of compiled){
    const producers=methods.filter(program=>program.kind==='nativeEnumStringMatch'),parsers=methods.filter(program=>program.kind==='nativeOptionStringParse');
    const pairedParsers=parsers.filter(parser=>producers.some(producer=>producer.cases.every(entry=>parser.cases.some(candidate=>candidate.input===entry.value&&candidate.variant===entry.variant))));
    if(!pairedParsers.length)continue;
    programs.push(...scalars.programs.filter(program=>program.enumPath===enumPath),...producers,...pairedParsers);
  }
  return {programs,refusals};
}
export function generateNativeOptions(root=ownerRoot){
  const publicRoot=readFileSync(root+'/rust/src/lib.rs','utf8'),programs=[],refusals=[];
  for(const item of items(publicRoot)){
    const tokens=item.tokens;if(tokens.length!==4||text(tokens.slice(0,2))!=='pubmod'||tokens[3]?.text!==';'||!identifier(tokens[2]))continue;
    syntax(tokens);const namespace=tokens[2].text,paths=['rust/src/'+namespace+'.rs','rust/src/'+namespace+'/mod.rs'].filter(path=>existsSync(root+'/'+path));
    if(paths.length!==1)continue;const path=paths[0];
    try{const generated=generateOptions(readFileSync(root+'/'+path,'utf8'),path,namespace,publicRoot);programs.push(...generated.programs);refusals.push(...generated.refusals);}
    catch(error){refusals.push({path,reason:error.message});}
  }
  return {programs,refusals};
}
export const TEMPORARY_OPTION_WORKAROUND={
  nativeShape:'Copy/PartialEq unit enums; exhaustive static ASCII string self-match; literal ASCII string match returning builtin Option<Self>',
  blocker:'a Rust item outside portable-pure-v1: impl_item carried, not executable',
  sourceQualification:'Exact producer/public module SHA, owned AST regeneration and paired producer/parser inverse',
  retirement:'Remove when maintained translator emits executable twins for these exact typed enum and impl shapes',
  upstreamIssue:null,unicodeContract:'Only ASCII literals/input; all non-ASCII input refuses, no Unicode/std compiler equivalence claim'
};

export function optionTestImportContract(source,root,binding){
  const witnesses=scalarTestImportContract(source,root,binding);
  const callerItems=items(source);
  for(const item of callerItems){
    const tokens=item.tokens,bodyAt=tokens.findIndex(token=>token.text==='{'),header=tokens.slice(0,bodyAt<0?tokens.length:bodyAt);
    if(header.some(token=>['macro','macro_rules','macro_rules!','extern','trait'].includes(token.text)))throw Error('unknown Option caller namespace');
    if(header[0]?.text==='#'&&header[1]?.text==='!')throw Error('unknown owned inner attribute/prelude');
    const useAt=header.findIndex(token=>token.kind==='word'&&token.text==='use');
    if(useAt>=0){
      if(useAt!==0&&!(useAt===1&&header[0]?.kind==='word'&&header[0]?.text==='pub'))throw Error('unknown owned import attribute');
      syntax(item.tokens);
      const names=importedNames(item.tokens.slice(useAt+1,-1));
      if(new Set(names).size!==names.length)throw Error('duplicate imported Option binding');
      for(const name of names)if(protectedNames.has(name))throw Error('shadowed Option caller import');
    }
    const position=header.findIndex(token=>['mod','type','enum','struct','const','static','fn'].includes(token.text));
    if(position>=0&&protectedNames.has(header[position+1]?.text))throw Error('shadowed Option caller declaration');
  }
  for(const path of ['rust/Cargo.toml','rust/tests/unit/mod.rs','rust/tests/unit/specification/mod.rs']){
    const absolute=root+'/'+path;if(!existsSync(absolute))throw Error('Option caller parent source missing');
    const parent=readFileSync(absolute,'utf8');
    if(path!=='rust/Cargo.toml')for(const item of items(parent)){
      const tokens=item.tokens;
      if(tokens.some(token=>['macro','macro_rules','macro_rules!','no_implicit_prelude','macro_use','cfg'].includes(token.text)))throw Error('unknown parent Option prelude/macro scope');
      if(tokens[0]?.text==='use'||tokens[0]?.text==='extern')throw Error('unknown parent Option import scope');
    }
    witnesses.push({path,sha256:hash(parent)});
  }
  return witnesses;
}
