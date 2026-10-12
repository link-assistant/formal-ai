import { readFileSync, statSync } from 'node:fs';
import { tokenize } from '../../scripts/lib/translation-blockers.mjs';
import { digest } from './cohort-runner.mjs';

export function readBoundSource(binding) {
  if (!binding || typeof binding.path !== 'string' || !/^[a-f0-9]{64}$/u.test(binding.sha256)) throw new Error('source identity required');
  if (statSync(binding.path).size > 1048576) throw new Error('source bound');
  const bytes = readFileSync(binding.path);
  if (bytes.length > 1048576 || digest(bytes) !== binding.sha256) throw new Error('source drift');
  const source = bytes.toString('utf8');
  if (!Buffer.from(source).equals(bytes)) throw new Error('source must be UTF8');
  return source;
}
// Deliberately bounded static literal subset. Never eval/import observed modules.
function stringValue(token) {
  if (token?.kind !== 'string' || token.value.length < 2 || token.value.at(-1) !== token.value[0]) throw new Error('literal string required');
  if (token.value[0] === '"') return JSON.parse(token.value);
  const value = token.value.slice(1, -1);
  if (value.includes('\\')) throw new Error('unsupported escaped single-quoted contract');
  return value;
}
export function extractExportedLiteral(binding, exportName) {
  const source = readBoundSource(binding);
  const tokens = tokenize(source).filter(token => token.kind !== 'comment');
  const matches = [];
  for (let index = 0; index < tokens.length - 4; index++) {
    if (tokens.slice(index,index+4).map(token=>token.value).join(' ') === 'export const ' + exportName + ' =') matches.push(index+4);
  }
  if (matches.length !== 1) throw new Error('one explicit exported literal contract required');
  let at = matches[0], nodes = 0;
  const start = tokens[at].start;
  const expect = value => { if (tokens[at++]?.value !== value) throw new Error('unsupported literal syntax'); };
  function value(depth) {
    if (depth > 24 || ++nodes > 1000) throw new Error('literal contract bound');
    const token = tokens[at];
    if (token?.kind === 'string') { at++; return stringValue(token); }
    if (token?.kind === 'number' && /^\d+$/u.test(token.value)) { at++; const number=Number(token.value);if(!Number.isSafeInteger(number))throw new Error('numeric contract bound');return number; }
    if (['true','false','null'].includes(token?.value)) {at++;return token.value==='null'?null:token.value==='true';}
    if (token?.value === '[') {
      at++; const result=[];
      while(tokens[at]?.value!==']') {result.push(value(depth+1));if(tokens[at]?.value!==']')expect(',');}
      at++;return result;
    }
    if (token?.value === '{') {
      at++; const result=Object.create(null);
      while(tokens[at]?.value!=='}') {
        const key=tokens[at++];const name=key?.kind==='string'?stringValue(key):key?.kind==='word'?key.value:null;
        if(!name || ['__proto__','prototype','constructor'].includes(name) || Object.hasOwn(result,name))throw new Error('duplicate or unsafe literal key');
        expect(':');result[name]=value(depth+1);if(tokens[at]?.value!=='}')expect(',');
      }
      at++;return result;
    }
    throw new Error('nonliteral contract requires missing capability');
  }
  const result=value(0);
  if(tokens[at]?.value!==';')throw new Error('literal declaration terminator required');
  return { value:result,source:binding,span:{start,end:tokens[at].start},exportName };
}
export function extractArithmeticAssertions(binding, testName) {
  const source=readBoundSource(binding),tokens=tokenize(source).filter(token=>token.kind!=='comment');
  const matches=tokens.map((token,index)=>({token,index})).filter(({token,index})=>tokens[index-2]?.value==='test' && tokens[index-1]?.value==='(' && token.kind==='string' && stringValue(token)===testName);
  if(matches.length!==1)throw new Error('one maintained arithmetic fixture required');
  const start=matches[0].index;
  let end=tokens.findIndex((token,index)=>index>start && token.value==='test' && tokens[index+1]?.value==='(');
  if(end<0)end=tokens.length;
  const assertions=[];
  for(let index=start;index<end;index++) {
    if(tokens.slice(index,index+6).map(token=>token.value).join(' ')!=='assert . equal ( module .'
        || tokens[index+6]?.kind!=='word' || tokens[index+7]?.value!=='(')continue;
    const exportName=tokens[index+6].value;
    let at=index+8;
    const integer=()=>{let sign=1;if(tokens[at]?.value==='-'){sign=-1;at++;}const token=tokens[at++];if(token?.kind!=='number'||!/^\d+$/u.test(token.value))throw new Error('numeric assertion required');return sign*Number(token.value);};
    const expect=value=>{if(tokens[at++]?.value!==value)throw new Error('arithmetic assertion shape drift');};
    const first=integer();expect(',');const second=integer();expect(')');expect(',');const expected=integer();expect(')');
    if(![first,second,expected].every(Number.isSafeInteger))throw new Error('numeric assertion bound');
    assertions.push({exportName,first,second,expected,span:{start:tokens[index].start,end:tokens[at-1].end}});
  }
  if(assertions.length<2 || !assertions.some(item=>item.first<0||item.second<0))throw new Error('insufficient independent arithmetic fixture');
  const names=new Set(assertions.map(item=>item.exportName));
  if(names.size!==1)throw new Error('ambiguous callable contract');
  return {exportName:assertions[0].exportName,assertions,source:binding,testName};
}
export function validateHeldOutContract(value) {
  if(!Array.isArray(value)||value.length>20)throw new Error('heldout array bound');
  const ids=new Set();
  for(const item of value) {
    if(!item || typeof item.id!=='string'||!item.id||ids.has(item.id))throw new Error('heldout identities required');ids.add(item.id);
    if(typeof item.destination!=='string'||item.destination.startsWith('/')||/[\\:\u0000-\u001F]/u.test(item.destination)
        ||item.destination.split('/').some(part=>part==='..'||part==='.'||!part))throw new Error('heldout relative destination required');
    if(!item.sourceBindings || typeof item.sourceBindings!=='object' || Array.isArray(item.sourceBindings) || Object.keys(item.sourceBindings).length===0)throw new Error('heldout source bindings required');
    const identifiers=[item.exportName,item.parameterName,...Object.values(item.sourceBindings)];
    for(const name of identifiers)if(typeof name!=='string'||!/^[A-Za-z][A-Za-z0-9]*$/u.test(name))throw new Error('heldout identifier required');
    if(new Set(Object.values(item.sourceBindings)).size!==Object.values(item.sourceBindings).length)throw new Error('heldout binding collision');
    if(!Array.isArray(item.fixtureNames)||!item.fixtureNames.every(name=>typeof name==='string'&&name))throw new Error('heldout fixture names required');
  }
  return value;
}
