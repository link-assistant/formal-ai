import {readFileSync,readdirSync,realpathSync,lstatSync,mkdtempSync,mkdirSync,writeFileSync,symlinkSync,rmSync} from 'node:fs';
import {resolve,dirname,join,relative,extname} from 'node:path';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const hash=b=>createHash('sha256').update(b).digest('hex');
function sourceTokens(source){
 const tokens=[],delimiters=[];let index=0;
 while(index<source.length){
  if(/\s/u.test(source[index])){index++;continue;}
  if(source.startsWith('//',index)){let n=source.indexOf('\n',index);index=n<0?source.length:n+1;continue;}
  if(source.startsWith('/*',index)){index+=2;let comments=1;while(comments>0){if(index>=source.length)return null;if(source.startsWith('/*',index)){comments++;index+=2;}else if(source.startsWith('*/',index)){comments--;index+=2;}else index+=String.fromCodePoint(source.codePointAt(index)).length;}continue;}
  const start=index,depth=delimiters.length,rawStart=source.startsWith('br',index)?index+2:source[index]==='r'?index+1:index;
  let rawQuote=rawStart;while(source[rawQuote]==='#')rawQuote++;
  if(rawStart>index&&source[rawQuote]==='"'){const closing='"'+'#'.repeat(rawQuote-rawStart);index=rawQuote+1;let end=source.indexOf(closing,index);if(end<0)return null;index=end+closing.length;}
  else if(source[index]==='"'){index++;while(true){if(index>=source.length)return null;if(source[index]==='\\'){index++;if(index>=source.length)return null;index+=String.fromCodePoint(source.codePointAt(index)).length;}else if(source[index]==='"'){index++;break;}else index+=String.fromCodePoint(source.codePointAt(index)).length;}}
  else if(source[index]==="'"){let end=index+1;if(source[end]==='\\'){end++;if(source[end]==='u'&&source[end+1]==='{'){const n=source.indexOf('}',end);if(n<0)return null;end=n+1;}else if(source[end]==='x')end+=3;else {if(end>=source.length)return null;end+=String.fromCodePoint(source.codePointAt(end)).length;}}else {if(end>=source.length)return null;end+=String.fromCodePoint(source.codePointAt(end)).length;}index=source[end]==="'"?end+1:index+1;}
  else {const first=String.fromCodePoint(source.codePointAt(index));index+=first.length;if(first==='_'||/[\p{L}\p{N}]/u.test(first)){while(index<source.length){const c=String.fromCodePoint(source.codePointAt(index));if(c!=='_'&&!/[\p{L}\p{N}]/u.test(c))break;index+=c.length;}}else if('([{'.includes(first))delimiters.push(first);else if(')]}'.includes(first)){const expected={')':'(',']':'[','}':'{'}[first];if(delimiters.pop()!==expected)return null;}}
  tokens.push({text:source.slice(start,index),start,end:index,depth});
 }return delimiters.length===0?tokens:null;
}
function externalFile(crateRoot,sourcePath,literal){try{
 if(!literal.startsWith('"')||!literal.endsWith('"'))return null;const rel=literal.slice(1,-1);
 if(!rel||rel.includes('\\')||rel.split('/').some(p=>p===''||p==='.'))return null;
 crateRoot=realpathSync(crateRoot);const testsRoot=join(crateRoot,'tests');let target=realpathSync(dirname(sourcePath)),entered=false;
 if(relative(crateRoot,target).startsWith('..'))return null;
 for(const part of rel.split('/')){if(part==='..'&&!entered&&target!==crateRoot)target=dirname(target);else if(part!=='..'){entered=true;target=join(target,part);if(lstatSync(target).isSymbolicLink())return null;}else return null;}
 if(!target.startsWith(testsRoot+'/')||extname(target)!=='.rs'||!lstatSync(testsRoot).isDirectory()||lstatSync(testsRoot).isSymbolicLink()||!lstatSync(target).isFile()||realpathSync(target)!==target)return null;
 const sha256=hash(readFileSync(target));if(hash(readFileSync(target))!==sha256)return null;return {path:target,sha256};
 }catch{return null;}}
const fixed=[[0,'#'],[1,'['],[2,'cfg'],[3,'('],[4,'test'],[5,')'],[6,']'],[7,'#'],[8,'['],[9,'path'],[10,'='],[12,']'],[13,'mod'],[15,';']];
function registrations(root,path,source){const tokens=sourceTokens(source);if(!tokens)return [];const found=[];for(let index=0;index<tokens.length;index++){const token=tokens[index];if(token.depth!==0||(index>0&&![';','}'].includes(tokens[index-1].text)))continue;const item=tokens.slice(index,index+16);if(item.length!==16||fixed.some(([n,s])=>item[n].text!==s))continue;if(!/^[_\p{L}][_\p{L}\p{N}]*$/u.test(item[14].text))continue;const file=externalFile(root,path,item[11].text);if(file)found.push({start:token.start,end:item[15].end,...file});}return found;}
function marker(trimmed){if(trimmed.startsWith('#[cfg(test)]')||trimmed.startsWith('#[cfg(test)')||trimmed.startsWith('mod tests'))return true;if(trimmed.startsWith('#[')){const attr=trimmed.slice(2).trimStart();if(attr.startsWith('test]')||attr.startsWith('test('))return true;const n=attr.indexOf('::');if(n>=0&&(attr.slice(n+2).startsWith('test]')||attr.slice(n+2).startsWith('test(')))return true;}return false;}
function violations(root,path,source){const reg=registrations(root,path,source),out=[];let offset=0;for(const [i,chunk]of(source.match(/[^\n]*\n|[^\n]+$/gu)??[]).entries()){const line=chunk.replace(/[\r\n]+$/u,'');if(marker(line.trimStart())&&!reg.some(r=>offset+line.length>r.start&&offset<r.end))out.push({line:i+1,text:line.trim()});offset+=chunk.length;}return out;}
let checks=0;const check=fn=>{fn();checks++;};
const root=mkdtempSync('/private/tmp/pr1188-external-scan-');mkdirSync(join(root,'src/nested'),{recursive:true});mkdirSync(join(root,'tests'));const path=join(root,'src/nested/module.rs'),fixture='#[test]\nfn physical_assertion() { assert_eq!(2 + 2, 4); }\n';
try{
 for(const name of ['renamed-alpha.rs','renamed-beta.rs']){writeFileSync(join(root,'tests',name),fixture);const declaration=`#[cfg(test)]\n#[path = "../../tests/${name}"]\nmod arbitrary_name;`;
 check(()=>{const r=registrations(root,path,declaration);assert.equal(r.length,1);assert.equal(readFileSync(r[0].path,'utf8'),fixture);assert.equal(r[0].sha256,hash(fixture));assert.equal(violations(root,path,declaration).length,0);});
 check(()=>assert.equal(registrations(root,path,`const TEXT: &str = r###"{ #[cfg(test)] }"###;\n/* { nested /* } */ } */\n${declaration}`).length,1));
 check(()=>assert.equal(violations(root,path,declaration.replaceAll('\n','\r\n')).length,0));
 }
 const draft=readFileSync('/private/tmp/pr1188-ci-T4077/draft.rs','utf8');
 const corpus=draft.split('for source in [')[1].split('\n    ]')[0];const negatives=[...corpus.matchAll(/^\s*("(?:[^"\\]|\\.)*"),$/gmu)].map(m=>JSON.parse(m[1]));assert.equal(negatives.length,14);
 for(const source of negatives)check(()=>{assert.equal(registrations(root,path,source).length,0,source);assert.ok(violations(root,path,source).length>0,source);});
 for(const suffix of ['#[unknown]','#[cfg(any(test))]','#[path = "../../tests/renamed-alpha.rs"]'])check(()=>assert.equal(registrations(root,path,`${suffix}\n#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod hidden;`).length,0));
 symlinkSync(join(root,'tests/renamed-alpha.rs'),join(root,'tests/alias.rs'));
 check(()=>{const s='#[cfg(test)]\n#[path = "../../tests/alias.rs"]\nmod alias;';assert.equal(registrations(root,path,s).length,0);assert.ok(violations(root,path,s).length>0);});
 check(()=>assert.equal(registrations(root,path,'#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod αβ;').length,1));
 check(()=>assert.equal(violations(root,path,'#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod arbitrary_name;\n#[test]\nfn unrelated_inline() {}').length,1));
}finally{rmSync(root,{recursive:true,force:true});}
const crateRoot=resolve('rust'),physical=[];let sourceCount=0;const walk=p=>readdirSync(p,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(join(p,e.name)):e.isFile()&&e.name.endsWith('.rs')?[join(p,e.name)]:[]);
for(const path of walk(join(crateRoot,'src'))){sourceCount++;const source=readFileSync(path,'utf8');const reg=registrations(crateRoot,path,source),v=violations(crateRoot,path,source);assert.deepEqual(v,[],path);physical.push(...reg.map(r=>({source:relative(crateRoot,path),fixture:relative(crateRoot,r.path),sha256:r.sha256})));}
assert.equal(physical.length,2);for(const r of physical){const frozen=readFileSync('/private/tmp/pr1188-ci-T4076/'+r.fixture.split('/').at(-1));assert.equal(hash(frozen),r.sha256);checks++;}
const original=readFileSync('/private/tmp/pr1188-ci-T4076/original.rs','utf8'),draft=readFileSync('/private/tmp/pr1188-ci-T4077/draft.rs','utf8');
for(const name of ['src_rust_paths','is_inline_test_marker']){const from=original.indexOf('fn '+name),to=original.indexOf('\n}\n',from)+3;assert.ok(draft.includes(original.slice(from,to)),name+' original body retained');checks++;}
const a=original.slice(original.lastIndexOf('    assert!('));assert.ok(draft.includes(a.trimEnd()));checks++;
console.log(JSON.stringify({checks,sourceCount,physical,zeroSkip:true,nativeExecution:false,structuralTwin:true,originalAssertionIntact:true}));
