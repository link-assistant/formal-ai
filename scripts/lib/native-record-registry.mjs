import {scalarTestImportContract} from './native-scalar-registry.mjs';
import {readFileSync,existsSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {lex,topLevelItems} from '../self-translation/lexer.mjs';
import {tokenize} from './rust-specification-cases.mjs';
import {close,split} from './rust-specification-values.mjs';
const root=resolve(import.meta.dirname,'../..');
const hash=source=>createHash('sha256').update(source).digest('hex');
const knownDerives=new Set(['Debug','Clone','Copy','PartialEq','Eq','Default']);
function qualifyRecordTokens(tokens){
  if(tokens.some(token=>!['word','number','punct'].includes(token.kind)||
    (token.kind==='word'&&token.text.startsWith('r#'))))throw Error('unsupported owned record token kind');
}
function attributes(tokens){
  let cursor=0;const derives=new Set();
  while(tokens[cursor]?.text==='#'){
    if(tokens[cursor+1]?.text!=='[')throw Error('unknown record attribute');
    const end=close(tokens,cursor+1),body=tokens.slice(cursor+2,end);
    if(body[0]?.text!=='derive'||body[1]?.text!=='('||close(body,1)!==body.length-1)throw Error('unknown record derive');
    for(const part of split(body.slice(2,-1))){
      if(part.length!==1||part[0].kind!=='word'||!knownDerives.has(part[0].text)||derives.has(part[0].text))throw Error('unknown or duplicate record derive');
      derives.add(part[0].text);
    }
    cursor=end+1;
  }
  return {tokens:tokens.slice(cursor),derives};
}
function typeOf(tokens,owners){
  if(tokens.length===1&&tokens[0].kind==='word'){
    const name=tokens[0].text;
    if(['u8','u16','u32','u64','usize'].includes(name))return {kind:'integer',native:name,type:'number',copy:true};
    if(name==='bool')return {kind:'boolean',type:'boolean',copy:true};
    if(name==='String')return {kind:'string',type:'string',copy:false};
    if(owners.has(name))return {kind:'record',name,type:'record:'+owners.get(name).path,copy:owners.get(name).copy};
  }
  if(tokens[0]?.text==='('&&close(tokens,0)===tokens.length-1){
    const elements=split(tokens.slice(1,-1)).map(part=>typeOf(part,owners));
    if(!elements.length||elements.some(type=>!type.copy||type.kind==='record'))throw Error('unknown record tuple ownership');
    return {kind:'tuple',elements,type:'tuple:'+JSON.stringify(elements.map(type=>type.type)),copy:true};
  }
  throw Error('unknown record field type');
}
function expression(tokens,fields){
  let comparisonCount=0,comparisonDepth=0;
  for(let index=0;index<tokens.length;index++){
    if(tokens[index].kind==='punct'&&'([{'.includes(tokens[index].text))comparisonDepth++;
    if(tokens[index].kind==='punct'&&')]}'.includes(tokens[index].text))comparisonDepth--;
    if(!comparisonDepth&&['==','!=','>','<'].some(operator=>tokens.slice(index,index+operator.length).map(token=>token.text).join('')===operator))comparisonCount++;
    if(!comparisonDepth&&['&&','||'].some(operator=>tokens.slice(index,index+operator.length).map(token=>token.text).join('')===operator))comparisonCount=0;
    if(comparisonCount>1)throw Error('chained native comparison remains unsupported');
  }
  for(const operator of ['||','&&','==','!=','>','<']){
    let depth=0;
    for(let index=0;index<tokens.length;index++){
      if(['(','[','{'].includes(tokens[index].text))depth++;
      if([')',']','}'].includes(tokens[index].text))depth--;
      const width=operator.length;
      if(!depth&&tokens.slice(index,index+width).map(token=>token.text).join('')===operator){
        const left=expression(tokens.slice(0,index),fields),right=expression(tokens.slice(index+width),fields);
        if(left.type!==right.type||(['&&','||'].includes(operator)?left.type!=='boolean':left.type!=='number'&&left.type!=='boolean'))throw Error('unknown borrowed primitive comparison');
        if(left.type==='number'){
          if(left.native&&right.native&&left.native!==right.native)throw Error('unknown native numeric type coercion');
          const native=left.native??right.native;
          for(const value of [left,right])if(value.kind==='literal'&&native==='u8'&&value.value>255)throw Error('literal outside inferred native integer type');
        }
        return {kind:'binary',operator,left,right,type:'boolean'};
      }
    }
  }
  if(tokens.length===1&&tokens[0].kind==='number'&&/^\d+$/u.test(tokens[0].text)){
    const value=Number(tokens[0].text);if(!Number.isInteger(value)||value>65535)throw Error('unknown native integer width');
    return {kind:'literal',value,type:'number'};
  }
  if(tokens.length===1&&tokens[0].kind==='word'&&['true','false'].includes(tokens[0].text))return {kind:'literal',value:tokens[0].text==='true',type:'boolean'};
  if(tokens[0]?.kind!=='word'||tokens[0]?.text!=='self')throw Error('unknown borrowed method effect');
  const path=[];let type={kind:'owner',fields};
  for(let cursor=1;cursor<tokens.length;cursor+=2){
    if(tokens[cursor]?.text!=='.')throw Error('unknown borrowed projection');
    const key=tokens[cursor+1]?.text;
    if(type.kind==='owner'&&tokens[cursor+1]?.kind==='word'&&Object.hasOwn(type.fields,key))type=type.fields[key];
    else if(type.kind==='tuple'&&tokens[cursor+1]?.kind==='number'&&/^(0|[1-9]\d*)$/u.test(key))type=type.elements[Number(key)];
    else throw Error('unknown borrowed projection type');
    if(!type)throw Error('unknown borrowed field');path.push(key);
  }
  if(!path.length||!type.copy||!['integer','boolean'].includes(type.kind))throw Error('non-Copy borrowed field escape');
  return {kind:'projection',path,type:type.type,...(type.kind==='integer'?{native:type.native}:{})};
}
function rootBinding(publicRoot, name, expected) {
  const declarations = [];
  function imported(tokens) {
    const group = tokens.findIndex(token => token.kind === 'punct' && token.text === '{');
    if (group >= 0) return split(tokens.slice(group + 1, close(tokens, group))).flatMap(imported);
    if (tokens.some(token => token.text === '*')) throw Error('unknown root wildcard namespace');
    const alias = tokens.findIndex(token => token.text === 'as');
    return [alias >= 0 ? tokens[alias + 1]?.text : tokens.at(-1)?.text];
  }
  for (const item of topLevelItems(lex(publicRoot, 'Rust'))) {
    const tokens = tokenize(publicRoot.slice(item.start, item.end));
    let cursor = 0;
    while (tokens[cursor]?.text === '#') {
      if (tokens[cursor + 1]?.text !== '[') throw Error('unknown root attribute');
      const end = close(tokens, cursor + 1), attribute = tokens[cursor + 2];
      if (attribute?.kind !== 'word' || !['cfg', 'doc'].includes(attribute.text)) throw Error('unknown root attribute effect');
      cursor = end + 1;
    }
    const headerStart = cursor;
    if (tokens[cursor]?.text === 'pub') {
      cursor += 1;
      if (tokens[cursor]?.text === '(') cursor = close(tokens, cursor) + 1;
    }
    const keyword = tokens[cursor], declared = tokens[cursor + (keyword?.text === 'extern' ? 2 : 1)];
    if (keyword?.kind === 'word' && tokens[cursor + 1]?.text === '!') throw Error('unknown root macro expansion');
    let matching = ['mod', 'type', 'struct', 'enum', 'fn', 'const', 'static', 'extern'].includes(keyword?.text)
      && [name, 'r#' + name].includes(declared?.text);
    if (keyword?.text === 'extern' && tokens.some((token, index) => token.text === 'as' && tokens[index + 1]?.text === name)) matching = true;
    if (keyword?.text === 'use' && imported(tokens.slice(cursor + 1, -1)).includes(name)) matching = true;
    if (['mod', 'extern', 'type', 'struct', 'enum', 'use'].includes(keyword?.text)) {
      const bodyAt = keyword.text === 'use' ? -1
        : tokens.findIndex((token, index) => index >= cursor && token.text === '{');
      const header = tokens.slice(headerStart, bodyAt < 0 ? tokens.length : bodyAt);
      qualifyRecordTokens(header);
    }
    if (matching) {
      qualifyRecordTokens(tokens);
      declarations.push(tokens.map(token => token.text).join(''));
    }
  }
  if (declarations.length !== 1 || declarations[0] !== expected) throw Error('ambiguous or unknown root namespace binding');
}

function producerImportNames(tokens) {
  qualifyRecordTokens(tokens);
  if (tokens.some(token => token.text === '*')) throw Error('unknown producer wildcard binding');
  const group = tokens.findIndex(token => token.text === '{');
  if (group >= 0) {
    if (close(tokens, group) !== tokens.length - 1) throw Error('unknown producer import group');
    return split(tokens.slice(group + 1, -1)).flatMap(producerImportNames);
  }
  const alias = tokens.findIndex(token => token.text === 'as');
  if (alias >= 0 && (alias !== tokens.length - 2 || tokens[alias + 1]?.kind !== 'word')) throw Error('unknown producer import alias');
  const name = tokens.at(-1);
  if (name?.kind !== 'word' || name.text === 'self') throw Error('unknown producer import binding');
  return [name.text];
}

export function generateRecords(source,path,namespace,publicRoot){
  rootBinding(publicRoot, namespace, 'pubmod' + namespace + ';');
  const items=topLevelItems(lex(source,'Rust')),owners=new Map(),refusals=[],programs=[];
  const lexical=lex(source,'Rust').filter(token=>token.type!=='comment');
  const importedNames = new Set();
  for(const item of items){const tokens=tokenize(source.slice(item.start,item.end));
    if(tokens[0]?.text==='static'||(tokens[0]?.kind==='word'&&tokens[1]?.text==='!'))throw Error('unknown record module initialization or expansion');
    let attributeAt=0;
    while(tokens[attributeAt]?.text==='#'){
      const end=close(tokens,attributeAt+1),body=tokens.slice(attributeAt+2,end);
      if(body.some(token=>!['word','number','punct'].includes(token.kind)))return {programs:[],refusals:[{reason:'unsupported record module attribute token kind'}]};
      qualifyRecordTokens(body);
      if(body.map(token=>token.text).join('')!=='must_use'&&body[0]?.text!=='derive')throw Error('unknown record module attribute effect');
      if(body[0]?.text==='derive')attributes(tokens.slice(attributeAt,end+1));
      attributeAt=end+1;
    }
    if (tokens[0]?.text === 'use') {
      if (tokens[0].kind !== 'word' || tokens.at(-1)?.text !== ';') throw Error('unknown producer import declaration');
      for (const name of producerImportNames(tokens.slice(1, -1))) {
        if (importedNames.has(name)) throw Error('duplicate producer import binding');
        importedNames.add(name);
      }
    }
    if(tokens[0]?.text==='use'||tokens[0]?.text==='extern'){
      for(let index=0;index<tokens.length-1;index++)if(tokens[index].text==='as'&&['alloc','core','String'].includes(tokens[index+1].text))throw Error('shadowed record builtin namespace');
      if(tokens.at(-2)?.kind==='word'&&['alloc','core'].includes(tokens.at(-2).text)&&tokens[0].text==='use')throw Error('shadowed record builtin namespace');
    }
    if(tokens[0]?.text==='include'&&tokens[1]?.text==='!')throw Error('unknown expanded record owner scope');
    if(tokens[0]?.text==='use'&&tokens.some(token=>knownDerives.has(token.text)))throw Error('shadowed builtin record derive');
    if(tokens[0]?.text==='use'&&tokens.some(token=>['String','bool','u8','u16','u32','u64','usize'].includes(token.text))&&tokens.map(token=>token.text).join('')!=='usealloc::string::{String,ToString};')return {programs:[],refusals:[{reason:'shadowed builtin record field type'}]};
    for(let index=0;index<tokens.length-1;index++)if(['type','struct','enum','fn','const','static','mod'].includes(tokens[index].text)&&['String','bool','u8','u16','u32','u64','usize','alloc','core'].includes(tokens[index+1].text))throw Error('shadowed builtin record type');
  }

  if(lexical.some(token=>['macro_rules!','macro_rules','macro'].includes(token.text)))throw Error('unknown record macro or trait scope');
  for(const item of items){
    const raw=source.slice(item.start,item.end),all=tokenize(raw),structAt=all.findIndex(token=>token.text==='struct');
    if(structAt<0||all.slice(0,structAt).some(token=>token.text==='fn'))continue;
    const name=all[structAt+1]?.text;if(items.filter(item=>tokenize(source.slice(item.start,item.end)).some((token,index,tokens)=>token.text==='struct'&&tokens[index+1]?.text===name)).length!==1)throw Error('ambiguous record declaration');if(owners.has(name))throw Error('duplicate record owner');
    try{
      qualifyRecordTokens(all);const {tokens,derives}=attributes(all);
      if(tokens[0]?.text!=='pub'||tokens[1]?.text!=='struct'||tokens[2]?.kind!=='word'||tokens[3]?.text!=='{'||close(tokens,3)!==tokens.length-1)throw Error('unknown owned record shape');
      const fields={},nativeFields={};
      for(const field of split(tokens.slice(4,-1))){
        if(field[0]?.text!=='pub'||field[1]?.kind!=='word'||field[2]?.text!==':'||field[1].text==='__proto__'||Object.hasOwn(fields,field[1].text))throw Error('unknown or duplicate record field');
        const type=typeOf(field.slice(3),owners);
        if (type.kind === 'string') rootBinding(publicRoot, 'alloc', 'externcratealloc;');
        if(type.kind==='string'&&!items.some(item=>source.slice(item.start,item.end).replace(/\s+/gu,'')==='usealloc::string::{String,ToString};'))throw Error('unknown owned String import');
        fields[field[1].text]=type.type;nativeFields[field[1].text]=type;
      }
      if(!Object.keys(fields).length)throw Error('empty record');
      const copy=derives.has('Copy')&&derives.has('Clone')&&Object.values(nativeFields).every(type=>type.copy);
      if(derives.has('Copy')&&!copy)throw Error('invalid record Copy capability');
      const owner={path:'formal_ai::'+namespace+'::'+name,fields,nativeFields,copy,derives,span:{start:item.start,end:item.end}};
      owners.set(name,owner);
    }catch(error){refusals.push({name,reason:error.message});}
  }
  const sourceWitnesses=[{path,sha256:hash(source)},{path:'rust/src/lib.rs',sha256:hash(publicRoot)}];
  for(const [name,owner]of owners){
    const common={recordPath:owner.path,fields:owner.fields,nativeFields:owner.nativeFields,copy:owner.copy,sourceWitnesses,nativeSourceQualified:true,nativeRecordQualified:true};
    programs.push({...common,path:owner.path,kind:'nativeRecordConstructor',returnType:'record:'+owner.path,sourceSpan:owner.span});
    if(owner.derives.has('Default')&&Object.values(owner.nativeFields).every(type=>['integer','boolean'].includes(type.kind))){
      programs.push({...common,path:owner.path+'::default',kind:'nativeRecordDefault',signature:[],returnType:'record:'+owner.path,
        defaults:Object.fromEntries(Object.entries(owner.nativeFields).map(([field,type])=>[field,type.kind==='boolean'?false:0])),sourceSpan:owner.span});
    }
    const methods=new Set();
    for(const item of items){
      const tokens=tokenize(source.slice(item.start,item.end));
      const implementationAt=tokens.findIndex(token=>token.text==='impl');
      if(implementationAt<0)continue;
      if(implementationAt!==0&&tokens.slice(implementationAt,tokens.findIndex(token=>token.text==='{')).some(token=>token.text===name))throw Error('unknown attributed record impl');
      if(tokens[0]?.text!=='impl')continue;
      const open=tokens.findIndex(token=>token.text==='{');
      if(tokens.slice(1,open).some(token=>token.text==='for')&&tokens[open-1]?.text===name)throw Error('custom record trait implementation');
      if(open!==2||tokens[1]?.text!==name)continue;qualifyRecordTokens(tokens.slice(0,open));
      const inner=source.slice(item.start,item.end),brace=inner.indexOf('{');
      for(const method of topLevelItems(lex(inner.slice(brace+1,-1),'Rust'))){
        const methodTokens=tokenize(inner.slice(brace+1,-1).slice(method.start,method.end));
        const functionAt=methodTokens.findIndex(token=>token.text==='fn');if(functionAt<0)continue;
        const methodName=methodTokens[functionAt+1]?.text;if(methods.has(methodName))throw Error('duplicate record method');methods.add(methodName);
        if(methodName==='default')throw Error('custom record default');
        try{
          qualifyRecordTokens(methodTokens);
          const openBody=methodTokens.findIndex(token=>token.text==='{');
          const header=methodTokens.slice(0,openBody).map(token=>token.text).join('');
          const borrowed=['pubconstfn'+methodName+'(&self)->bool','#[must_use]pubconstfn'+methodName+'(&self)->bool'].includes(header);
          const copied=owner.copy&&['pubconstfn'+methodName+'(self)->bool','#[must_use]pubconstfn'+methodName+'(self)->bool'].includes(header);
          if((!borrowed&&!copied)||close(methodTokens,openBody)!==methodTokens.length-1)throw Error('unknown record receiver or Copy capability');
          const body=expression(methodTokens.slice(openBody+1,-1),owner.nativeFields);if(body.type!=='boolean')throw Error('unknown record return');
          programs.push({...common,path:owner.path+'::'+methodName,kind:copied?'nativeRecordCopyRead':'nativeRecordBorrow',signature:['record:'+owner.path],returnType:'boolean',body,
            sourceSpan:{start:item.start+brace+1+method.start,end:item.start+brace+1+method.end}});
        }catch(error){refusals.push({name:methodName,reason:error.message});}
      }
    }
  }
  const ownedPaths=new Set(programs.filter(program=>['nativeRecordBorrow','nativeRecordCopyRead'].includes(program.kind)).map(program=>program.recordPath));
  let expanded=true;while(expanded){expanded=false;for(const program of programs)if(ownedPaths.has(program.recordPath))for(const type of Object.values(program.nativeFields)){if(type.kind==='record'&&!ownedPaths.has(type.type.slice(7))){ownedPaths.add(type.type.slice(7));expanded=true;}}}
  return {programs:programs.filter(program=>ownedPaths.has(program.recordPath)),refusals};
}
export function generateNativeRecords(ownerRoot=root){
  const publicRoot=readFileSync(ownerRoot+'/rust/src/lib.rs','utf8'),programs=[],refusals=[];
  for(const item of topLevelItems(lex(publicRoot,'Rust'))){
    const tokens=tokenize(publicRoot.slice(item.start,item.end));
    if(tokens.length!==4||tokens[0].text!=='pub'||tokens[1].text!=='mod'||tokens[3].text!==';')continue;
    const namespace=tokens[2].text,paths=['rust/src/'+namespace+'.rs','rust/src/'+namespace+'/mod.rs'].filter(path=>existsSync(ownerRoot+'/'+path));
    if(paths.length!==1)continue;const path=paths[0];
    try{programs.push(...generateRecords(readFileSync(ownerRoot+'/'+path,'utf8'),path,namespace,publicRoot).programs);}catch(error){refusals.push({path,reason:error.message});}
  }
  return {programs,refusals};
}

export const TEMPORARY_RECORD_WORKAROUND={
  nativeShape:'Public source-owned typed structs, builtin primitive Default, and primitive Copy projections in const methods borrowing &self or consuming source-proved Copy self',
  blocker:'Maintained Rust to meta-language translation carries the consumed impl_item as unsupported; struct_item syntax alone is not executable parity',
  sourceQualification:'Full public module and producer SHA witnesses; deterministic AST regeneration before runtime observations',
  retirement:'Remove when maintained translation produces executable typed constructor, default and source-proved receiver-method twins for this exact source shape',
  upstreamIssue:null
};

export function recordTestImportContract(source,ownerRoot,binding,body){
  const used=new Set(body.filter(token=>token.kind==='word').map(token=>token.text));
  const aliases=new Set(),replacements=[],witnesses=[];
  function imports(tokens,prefix=''){
    qualifyRecordTokens(tokens);
    const group=tokens.findIndex(token=>token.text==='{');
    if(group>=0){
      const base=prefix+tokens.slice(0,group).map(token=>token.text).join('');
      return split(tokens.slice(group+1,close(tokens,group))).flatMap(part=>imports(part,base));
    }
    const aliasAt=tokens.findIndex(token=>token.text==='as');
    const path=prefix+(aliasAt<0?tokens:tokens.slice(0,aliasAt)).map(token=>token.text).join('');
    if(path.includes('*'))throw Error('wildcard record caller import');
    const name=aliasAt<0?path.split('::').at(-1):tokens[aliasAt+1]?.text;
    if(['formal_ai','std','Self','self','Default','Copy','Clone','PartialEq','Eq','String','ToOwned','ToString'].includes(name))throw Error('shadowed record crate, type or derive root');
    if(!name||aliases.has(name))throw Error('ambiguous record caller import');
    aliases.add(name);return [{path,name}];
  }
  let nominalWitnesses;
  function nominalAttributes(tokens){
    while(tokens.slice(0,4).map(token=>token.text).join('')==='#[must_use]'){
      qualifyRecordTokens(tokens.slice(0,4));tokens=tokens.slice(4);
    }
    return attributes(tokens);
  }
  function nominalBare(tokens){
    let cursor=0;
    while(tokens[cursor]?.kind==='punct'&&tokens[cursor].text==='#'){
      if(tokens[cursor+1]?.text!=='[')throw Error('unknown nominal outer attribute');
      cursor=close(tokens,cursor+1)+1;
    }
    return tokens.slice(cursor);
  }
  function publicLeaves(tokens,prefix=[]){
    qualifyRecordTokens(tokens);
    if(tokens.some(token=>token.text==='*'))throw Error('wildcard nominal reexport');
    const group=tokens.findIndex(token=>token.text==='{');
    if(group>=0){
      const end=close(tokens,group);
      if(end!==tokens.length-1)throw Error('unknown nominal reexport group tail');
      const head=tokens.slice(0,group);
      if(head.length&&(head.length%3!==0||head.some((token,index)=>index%3?token.kind!=='punct'||token.text!==':':token.kind!=='word')))throw Error('unknown nominal reexport prefix');
      const path=head.filter(token=>token.text!==':');
      if(path.some(token=>token.kind!=='word'))throw Error('unknown nominal reexport prefix');
      return split(tokens.slice(group+1,end)).flatMap(part=>publicLeaves(part,[...prefix,...path.map(token=>token.text)]));
    }
    const alias=tokens.findIndex(token=>token.kind==='word'&&token.text==='as');
    const path=alias<0?tokens:tokens.slice(0,alias);
    if(path.length%3!==1||path.some((token,index)=>index%3?token.kind!=='punct'||token.text!==':':token.kind!=='word')){
      throw Error('unknown nominal reexport path');
    }
    if(alias>=0&&(tokens.length!==alias+2||tokens[alias+1].kind!=='word'))throw Error('unknown nominal reexport alias');
    const parts=[...prefix,...path.filter((_,index)=>index%3===0).map(token=>token.text)];
    return [{parts,name:alias<0?parts.at(-1):tokens[alias+1].text}];
  }
  function nominalModule(path){
    const native=readFileSync(ownerRoot+'/'+path,'utf8'),items=topLevelItems(lex(native,'Rust'));
    const entries=items.map(item=>({item,tokens:tokenize(native.slice(item.start,item.end))}));
    for(const {tokens} of entries){
      const bare=nominalBare(tokens),useAt=bare[0]?.kind==='word'&&bare[0].text==='pub'?1:0;
      if(bare[useAt]?.kind==='word'&&bare[useAt].text==='use'){
        if(bare.at(-1)?.text!==';')throw Error('unknown nominal import tail');
        const leaves=publicLeaves(bare.slice(useAt+1,-1));
        if(leaves.some(leaf=>knownDerives.has(leaf.name)||['must_use','derive'].includes(leaf.name))){
          throw Error('shadowed unused nominal builtin derive or attribute');
        }
      }
      if(tokens.some(token=>['macro','macro_rules','macro_rules!'].includes(token.text)))throw Error('unknown unused nominal macro scope');
      if(tokens[0]?.text==='#'&&tokens[1]?.text==='!')throw Error('unknown nominal module inner attribute');
      if(tokens[0]?.kind==='word'&&tokens[0].text==='static')throw Error('unknown nominal module initializer');
      if(tokens[0]?.kind==='word'&&tokens[1]?.text==='!')throw Error('unknown nominal module expansion');
    }
    if(path!=='rust/src/lib.rs')nominalWitnesses.set(path,{path,sha256:hash(native)});return entries;
  }
  function nominalChild(entries,path,name,requirePublic){
    const declarations=entries.filter(({tokens})=>{
      const bare=nominalBare(tokens);
      let at=bare[0]?.kind==='word'&&bare[0].text==='pub'?1:0;
      if(at&&bare[at]?.text==='(')at=close(bare,at)+1;
      return bare[at]?.kind==='word'&&bare[at].text==='mod'&&bare[at+1]?.kind==='word'&&bare[at+1].text===name;
    });
    for(const {tokens} of entries){
      const bare=nominalBare(tokens),at=bare[0]?.text==='pub'?1:0;
      if(bare[at]?.kind==='word'&&bare[at].text==='use'){
        if(publicLeaves(bare.slice(at+1,-1)).some(leaf=>leaf.name===name))throw Error('shadowed nominal module binding');
      }else if(bare[at]?.kind==='word'&&['struct','enum','trait','type','const','static','fn'].includes(bare[at].text)&&bare[at+1]?.text===name){
        throw Error('shadowed nominal module binding');
      }
    }
    if(declarations.length!==1)throw Error('unknown or ambiguous nominal module binding');
    const tokens=nominalAttributes(declarations[0].tokens).tokens;
    qualifyRecordTokens(tokens);
    const header=tokens.map(token=>token.text).join('');
    if(!['pubmod'+name+';',...(requirePublic?[]:['mod'+name+';','pub(crate)mod'+name+';'])].includes(header)){
      throw Error('unknown nominal external module declaration');
    }
    const directory=path.endsWith('/lib.rs')||path.endsWith('/mod.rs')?path.slice(0,path.lastIndexOf('/')):path.slice(0,-3);
    const paths=[directory+'/'+name+'.rs',directory+'/'+name+'/mod.rs'].filter(candidate=>existsSync(ownerRoot+'/'+candidate));
    if(paths.length!==1)throw Error('ambiguous nominal physical module');return paths[0];
  }
  function nominalExport(path,name,visiting=new Set()){
    const identity=path+'::'+name;
    if(visiting.has(identity)||visiting.size>=32)throw Error('cyclic nominal public reexport');
    const next=new Set([...visiting,identity]),entries=nominalModule(path),declarations=[],exports=[];
    for(const entry of entries){
      const bare=nominalBare(entry.tokens);
      if(bare[0]?.kind!=='word'||bare[0].text!=='pub')continue;
      if(bare[1]?.kind==='word'&&bare[1].text==='use'){
        nominalAttributes(entry.tokens);qualifyRecordTokens(entry.tokens);
        if(bare.at(-1)?.text!==';')throw Error('unknown nominal public reexport tail');
        exports.push(...publicLeaves(bare.slice(2,-1)).filter(leaf=>leaf.name===name));
      }else if(bare[1]?.kind==='word'&&['trait','struct','enum','fn','const','mod'].includes(bare[1].text)&&bare[2]?.kind==='word'&&bare[2].text===name){
        declarations.push(entry);
      }
    }
    if(declarations.length+exports.length!==1)throw Error('unknown or ambiguous unused nominal declaration');
    if(exports.length){
      let parts=exports[0].parts,target=path;
      if(parts[0]==='self')parts=parts.slice(1);
      else if(parts[0]==='crate'){target='rust/src/lib.rs';parts=parts.slice(1);}
      if(parts.length<2||parts.some(part=>['super','Self','std','formal_ai'].includes(part)))throw Error('unknown nominal reexport namespace');
      for(const part of parts.slice(0,-1))target=nominalChild(nominalModule(target),target,part,false);
      return nominalExport(target,parts.at(-1),next);
    }
    const entry=declarations[0],tokens=entry.tokens,nominal=nominalAttributes(tokens);
    const boundary=tokens.findIndex(token=>token.kind==='punct'&&['{','=',';'].includes(token.text));
    if(boundary<0)throw Error('unknown unused nominal declaration header');
    qualifyRecordTokens(tokens.slice(0,boundary));
    if(nominal.tokens[1]?.text==='trait'){
      const bodyAt=nominal.tokens.findIndex(token=>token.kind==='punct'&&token.text==='{');
      if(nominal.derives.size||bodyAt!==3||nominal.tokens[2]?.text!==name)throw Error('unknown imported trait dispatch closure');
      const members=nominal.tokens.slice(bodyAt+1,-1);
      if(members.some(token=>!['word','number','punct'].includes(token.kind)||token.text==='#'||(token.kind==='word'&&token.text.startsWith('r#')))){
        throw Error('unknown imported trait dispatch closure');
      }
      if(members.some((token,index)=>token.kind==='word'&&members[index+1]?.text==='!')||
        members.some((token,index)=>token.kind==='word'&&token.text==='fn'&&['to_owned','to_string'].includes(members[index+1]?.text))){
        throw Error('unknown imported trait dispatch closure');
      }
    }
  }
  function unusedNominal(leaf){
    const parts=leaf.path.split('::');
    if(parts.length!==3||parts[0]!=='formal_ai')throw Error('unknown unused record import namespace');
    nominalWitnesses=new Map();
    const path=nominalChild(nominalModule('rust/src/lib.rs'),'rust/src/lib.rs',parts[1],true);
    nominalExport(path,parts[2]);witnesses.push(...nominalWitnesses.values());
  }
  for(const item of topLevelItems(lex(source,'Rust'))){
    const tokens=tokenize(source.slice(item.start,item.end));
    if(tokens[0]?.text==='#'){
      const allowed=tokens.slice(0,4).map(token=>token.text).join('')==='#[test]'&&tokens[4]?.kind==='word'&&tokens[4].text==='fn';
      if(!allowed)throw Error('unknown record caller item attribute');
      qualifyRecordTokens(tokens.slice(0,5));
    }
    if(tokens.some((token,index)=>['mod','extern'].includes(token.text)&&['std','formal_ai'].includes(tokens[index+1]?.text)))throw Error('shadowed record caller crate');
    if(tokens[0]?.text==='static'||(tokens[0]?.kind==='word'&&tokens[1]?.text==='!'))throw Error('unknown module initialization or expansion');
    if (tokens.some(token => token.kind === 'word' && ['trait', 'impl'].includes(token.text))) {
      throw Error('unknown caller trait or implementation dispatch');
    }
    if(tokens[0]?.text!=='use')continue;
    qualifyRecordTokens(tokens);
    if(tokens.at(-1)?.text!==';')throw Error('unknown record import tail');
    const leaves=imports(tokens.slice(1,-1)),retained=[];
    for(const leaf of leaves){
      if(leaf.path.startsWith('std::')){
        if(used.has(leaf.name))throw Error('executed std import remains unsupported');
      }else if(leaf.path.startsWith('formal_ai::')&&!used.has(leaf.name))unusedNominal(leaf);
      else retained.push('use '+leaf.path+(leaf.name===leaf.path.split('::').at(-1)?'':' as '+leaf.name)+';');
    }
    replacements.push({start:item.start,end:item.end,text:retained.join(' ')});
  }
  let filtered=source;
  for(const item of replacements.reverse())filtered=filtered.slice(0,item.start)+item.text+filtered.slice(item.end);
  return [...witnesses,...scalarTestImportContract(filtered,ownerRoot,binding)];
}
