// Source-qualified borrowed ASCII constants through the maintained translator.
// Unconsumed producer bodies have no executable permissions from this registry.
import {readFileSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {isDeepStrictEqual} from 'node:util';
import {lex,topLevelItems} from '../self-translation/lexer.mjs';
import {parseRustItem} from '../self-translation/frontends.mjs';
import {check,toLino,fromLino} from '../self-translation/ir.mjs';
import {emitJavaScript} from '../self-translation/emitters.mjs';
import {selfTranslate} from '../self-translation/envelope.mjs';
import {tokenize} from './rust-specification-cases.mjs';
import {close,split} from './rust-specification-values.mjs';
import {nativeStringContract} from './native-string-contract.mjs';
import {scalarTestImportContract} from './native-scalar-registry.mjs';
const root=resolve(import.meta.dirname,'../..');
const hash=value=>createHash('sha256').update(value).digest('hex');
const word=(token,text)=>token?.kind==='word'&&(text===undefined||token.text===text);
const punct=(token,text)=>token?.kind==='punct'&&token.text===text;
function ownedSyntax(tokens){
  if(tokens.some(token=>!['word','punct'].includes(token.kind)||
    (token.kind==='word'&&token.text.startsWith('r#'))))throw Error('unsupported constant namespace lexical kind');
}
function declarations(source){
  return topLevelItems(lex(source,'Rust')).map(item=>({raw:source.slice(item.start,item.end),
    tokens:tokenize(source.slice(item.start,item.end)),lexical:item.tokens,
    span:{start:item.start,end:item.end}}));
}
function header(tokens){
  let cursor=0;
  while(punct(tokens[cursor],'#')&&punct(tokens[cursor+1],'['))cursor=close(tokens,cursor+1)+1;
  return tokens.slice(cursor);
}
function visibilityAt(tokens){
  if(!word(tokens[0],'pub'))return 0;
  return punct(tokens[1],'(')?close(tokens,1)+1:1;
}
function leaves(tokens,prefix=[]){
  ownedSyntax(tokens);
  if(tokens.some(token=>token.text==='*'))throw Error('unknown wildcard constant namespace');
  const group=tokens.findIndex(token=>punct(token,'{'));
  if(group>=0){
    if(close(tokens,group)!==tokens.length-1)throw Error('unknown constant import group tail');
    const head=tokens.slice(0,group),parts=[];
    for(let index=0;index<head.length;index+=3){
      if(!word(head[index])||!punct(head[index+1],':')||!punct(head[index+2],':'))throw Error('unknown constant group namespace');
      parts.push(head[index].text);
    }
    return split(tokens.slice(group+1,-1)).flatMap(part=>leaves(part,[...prefix,...parts]));
  }
  const alias=tokens.findIndex(token=>word(token,'as'));
  const path=alias<0?tokens:tokens.slice(0,alias),parts=[];
  for(let index=0;index<path.length;index+=3){
    if(!word(path[index]))throw Error('unknown constant namespace word');
    parts.push(path[index].text);
    if(index+1<path.length&&(!punct(path[index+1],':')||!punct(path[index+2],':')))throw Error('unknown constant namespace separator');
  }
  if(!parts.length||(alias>=0&&(tokens.length!==alias+2||!word(tokens[alias+1]))))throw Error('unknown constant alias');
  return [{parts:[...prefix,...parts],name:alias<0?parts.at(-1):tokens[alias+1].text}];
}
function namespace(entries,name){
  const matches=[];
  for(const entry of entries){
    const bare=header(entry.tokens),at=visibilityAt(bare);
    if(['mod','struct','enum','type','const','static','fn'].some(kind=>word(bare[at],kind))&&bare[at+1]?.text===name){
      if(!word(bare[at+1]))throw Error('counterfeit constant module name');matches.push(entry);
    }
    if(word(bare[at],'use')&&leaves(bare.slice(at+1,-1)).some(leaf=>leaf.name===name))throw Error('shadowed constant namespace');
  }
  if(matches.length!==1)throw Error('ambiguous public constant module');
  const tokens=matches[0].tokens;ownedSyntax(tokens);
  if(tokens.length!==4||!word(tokens[0],'pub')||!word(tokens[1],'mod')||!word(tokens[2],name)||!punct(tokens[3],';'))throw Error('unknown public constant module header');
}
function exports(entries,namespaceName,name){
  const output=[];
  for(const entry of entries){
    const bare=header(entry.tokens);
    if(!word(bare[0],'pub')||!word(bare[1],'use'))continue;
    const bindings=leaves(bare.slice(2,-1));
    for(const binding of bindings){
      const parts=binding.parts[0]==='crate'?binding.parts.slice(1):binding.parts;
      if(parts.length!==2||parts[0]!==namespaceName||parts[1]!==name)continue;
      if(entry.tokens!==bare&&entry.tokens.length!==bare.length)throw Error('conditional constant reexport');
      if(!punct(bare.at(-1),';'))throw Error('unknown constant reexport tail');
      const collision=entries.filter(other=>{
        const tokens=header(other.tokens),at=visibilityAt(tokens);
        return word(tokens[at+1],binding.name)&&['mod','struct','enum','type','const','static','fn'].some(kind=>word(tokens[at],kind));
      });
      const all=entries.filter(other=>word(header(other.tokens)[0],'pub')&&word(header(other.tokens)[1],'use'))
        .flatMap(other=>leaves(header(other.tokens).slice(2,-1))).filter(other=>other.name===binding.name);
      if(collision.length||all.length!==1)throw Error('ambiguous constant public reexport');
      output.push('formal_ai::'+binding.name);
    }
  }
  return output;
}
export function generateBorrowedConstants(source,path,namespaceName,publicRoot){
  const items=declarations(source),publicItems=declarations(publicRoot);
  namespace(publicItems,namespaceName);
  for(const entry of [...items,...publicItems]){
    const tokens=entry.tokens,bare=header(tokens),at=visibilityAt(bare);
    if(punct(tokens[0],'#')&&punct(tokens[1],'!'))throw Error('unknown constant module inner attribute');
    if(word(bare[at],'macro_rules')||word(bare[at],'macro')||
      (word(bare[at])&&punct(bare[at+1],'!')))throw Error('unknown constant module expansion');
    if(word(bare[at],'use')&&leaves(bare.slice(at+1,-1)).some(leaf=>leaf.name==='str'))throw Error('shadowed native str type');
    if(['type','struct','enum','mod'].some(kind=>word(bare[at],kind))&&word(bare[at+1],'str'))throw Error('shadowed native str type');
  }
  const programs=[],refusals=[],witnesses=[{path,sha256:hash(source)},{path:'rust/src/lib.rs',sha256:hash(publicRoot)}];
  for(const item of items){
    const tokens=item.tokens,bare=header(tokens);
    if(!word(bare[0],'pub')||!word(bare[1],'const'))continue;
    try{
      if(tokens.length!==9||!word(tokens[0],'pub')||!word(tokens[1],'const')||!word(tokens[2])||
        !punct(tokens[3],':')||!punct(tokens[4],'&')||!word(tokens[5],'str')||!punct(tokens[6],'=')||
        tokens[7]?.kind!=='string'||!punct(tokens[8],';'))throw Error('unknown public borrowed literal constant shape');
      nativeStringContract(item.raw);
      if(!/^[\x00-\x7f]*$/u.test(tokens[7].text))throw Error('non-ASCII borrowed constant domain');
      const name=tokens[2].text;
      if(items.filter(other=>{const raw=header(other.tokens),at=visibilityAt(raw);
        return word(raw[at+1],name)&&['const','static','fn','struct','enum','type','mod'].some(kind=>word(raw[at],kind));}).length!==1)
        throw Error('ambiguous source constant declaration');
      const ir=check(parseRustItem(item.lexical),new Map()),lino=toLino(ir);
      if(ir.kind!=='constant'||ir.value.kind!=='string'||ir.value.value!==tokens[7].text)throw Error('maintained constant IR disagrees with native literal');
      const restored=fromLino(lino)[0];
      if(!isDeepStrictEqual(toLino(restored),lino))throw Error('maintained constant meta inverse mismatch');
      const translated=selfTranslate(item.raw,'Rust','JavaScript');
      if(translated.items.length!==1||translated.items[0].status!=='translated'||
        selfTranslate(translated.code,'JavaScript','Rust').code!==
          '// formal-ai:self-translation:v1 source=JavaScript target=Rust sha256='+hash(translated.code)+
          ' bytes='+Buffer.byteLength(translated.code,'utf8')+'\n\n'+item.raw+'\n')throw Error('maintained constant translation inverse mismatch');
      const canonicalPath='formal_ai::'+namespaceName+'::'+name;
      for(const publicPath of [canonicalPath,...exports(publicItems,namespaceName,name)])programs.push({
        path:publicPath,canonicalPath,kind:'nativeConstant',nativeType:'&str',returnType:'string',value:ir.value.value,
        nativeSourceQualified:true,nativeBorrowedConstantQualified:true,sourceSpan:item.span,sourceWitnesses:witnesses,
        maintainedMeta:lino,maintainedJavaScript:emitJavaScript(ir),maintainedTranslationSha256:hash(translated.code)});
    }catch(error){refusals.push({item:'constant',name:bare[2]?.text,reason:error.message});}
  }
  return {programs,refusals};
}
export function borrowedConstantTestImportContract(source,ownerRoot,binding){
  const reserved=new Set(['formal_ai','str','assert','assert_eq','assert_ne']);
  for(const item of declarations(source)){
    const tokens=item.tokens,bare=header(tokens),at=visibilityAt(bare);
    if(punct(tokens[0],'#')&&punct(tokens[1],'!'))throw Error('unknown borrowed constant caller inner attribute');
    if(word(bare[at],'use')){
      if(tokens.length!==bare.length)throw Error('conditional borrowed constant caller import');
      for(const leaf of leaves(bare.slice(at+1,-1)))if(reserved.has(leaf.name))throw Error('shadowed borrowed constant caller syntax');
    }
    if(tokens.length!==bare.length){
      const prefix=tokens.slice(0,tokens.length-bare.length);
      if(prefix.length!==4||!punct(prefix[0],'#')||!punct(prefix[1],'[')||!word(prefix[2],'test')||!punct(prefix[3],']'))
        throw Error('unknown borrowed constant caller item attribute');
    }
    if(['fn','mod','const','static','type','struct','enum'].some(kind=>word(bare[at],kind))&&reserved.has(bare[at+1]?.text))
      throw Error('shadowed borrowed constant caller name');
  }
  return scalarTestImportContract(source,ownerRoot,{...binding,path:binding.canonicalPath});
}
export function generateNativeBorrowedConstants(ownerRoot=root){
  const publicRoot=readFileSync(ownerRoot+'/rust/src/lib.rs','utf8'),programs=[],refusals=[];
  for(const item of declarations(publicRoot)){
    const tokens=item.tokens;
    if(tokens.length!==4||!word(tokens[0],'pub')||!word(tokens[1],'mod')||!word(tokens[2])||!punct(tokens[3],';'))continue;
    const namespaceName=tokens[2].text,paths=['rust/src/'+namespaceName+'.rs','rust/src/'+namespaceName+'/mod.rs']
      .filter(path=>existsSync(ownerRoot+'/'+path));
    if(paths.length!==1){refusals.push({namespace:namespaceName,reason:'ambiguous physical constant module'});continue;}
    try{const output=generateBorrowedConstants(readFileSync(ownerRoot+'/'+paths[0],'utf8'),paths[0],namespaceName,publicRoot);
      programs.push(...output.programs);refusals.push(...output.refusals.map(refusal=>({path:paths[0],...refusal})));}
    catch(error){refusals.push({path:paths[0],reason:error.message});}
  }
  return {programs,refusals};
}
