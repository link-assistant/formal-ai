// Shared pure typed Links Notation codec. Surfaces add transport and redaction separately.
import {isFloat,f32,f64,renderFloat} from './json-numbers.mjs';
import {encodeWord,decodeWord,parseLinks,writeLink} from './lino-codec.mjs';
const utf8Order=(a,b)=>Buffer.compare(Buffer.from(a),Buffer.from(b));
function text(value){if(typeof value!=='string'||!value.isWellFormed())throw Error('unsupported_text');return value;}
function node(value,depth=0,preserveJsonNumbers=false,dialogRecord=false){
 if(!dialogRecord&&depth>64)throw Error('depth_limit');
 if(preserveJsonNumbers&&typeof value==='bigint')return ['integer',String(value)];
 if(preserveJsonNumbers&&isFloat(value)){if(!Number.isFinite(value.value)||![32,64].includes(value.bits))throw Error('unsupported_float');return ['float',String(value.bits),renderFloat(value.value,value.bits)];}
 if(value===null)return ['null'];
 if(typeof value==='string')return ['string',encodeWord(text(value))];
 if(typeof value==='boolean')return ['boolean',String(value)];
 if(typeof value==='number'){
  if(!Number.isFinite(value)||Object.is(value,-0)||(Number.isInteger(value)&&!Number.isSafeInteger(value)))throw Error('unsupported_number');
  return ['number',String(value)];
 }
 if(Array.isArray(value)){
  const d=Object.getOwnPropertyDescriptors(value);
  if(Reflect.ownKeys(d).length!==value.length+1)throw Error('unsupported_array');
  const items=[];for(let i=0;i<value.length;i++){if(!Object.hasOwn(d[i]??{},'value'))throw Error('unsupported_array');items.push(node(d[i].value,depth+1,preserveJsonNumbers,dialogRecord));}
  return ['array',...items];
 }
 if(value&&Object.getPrototypeOf(value)===Object.prototype){
  const d=Object.getOwnPropertyDescriptors(value),keys=Reflect.ownKeys(d);
  if(keys.some(k=>typeof k!=='string'||!Object.hasOwn(d[k],'value')))throw Error('unsupported_object');
  return ['object',...keys.sort(utf8Order).map(k=>['field',encodeWord(text(k)),node(d[k].value,depth+1,preserveJsonNumbers,dialogRecord)])];
 }
 throw Error('unsupported_value');
}
function value(n,depth=0,preserveJsonNumbers=false,dialogRecord=false){
 if((!dialogRecord&&depth>64)||!Array.isArray(n))throw Error('invalid_node');
 const [tag,...args]=n;
 if(preserveJsonNumbers&&tag==='integer'&&args.length===1&&typeof args[0]==='string'&&/^-?(0|[1-9][0-9]*)$/u.test(args[0]))return BigInt(args[0]);
 if(preserveJsonNumbers&&tag==='float'&&args.length===2&&['32','64'].includes(args[0])&&typeof args[1]==='string'&&Number.isFinite(Number(args[1])))return args[0]==='32'?f32(Number(args[1])):f64(Number(args[1]));
 if(tag==='null'&&args.length===0)return null;
 if(tag==='string'&&args.length===1&&typeof args[0]==='string')return decodeWord(args[0]);
 if(tag==='boolean'&&args.length===1&&['true','false'].includes(args[0]))return args[0]==='true';
 if(tag==='number'&&args.length===1&&typeof args[0]==='string'){const x=Number(args[0]);if(writeLink(node(x))!==writeLink(n))throw Error('noncanonical_number');return x;}
 if(tag==='array')return args.map(x=>value(x,depth+1,preserveJsonNumbers,dialogRecord));
 if(tag==='object'){
  const out={};for(const f of args){if(!Array.isArray(f)||f.length!==3||f[0]!=='field'||typeof f[1]!=='string')throw Error('invalid_field');
   const key=decodeWord(f[1]);if(Object.hasOwn(out,key))throw Error('duplicate_field');Object.defineProperty(out,key,{value:value(f[2],depth+1,preserveJsonNumbers,dialogRecord),enumerable:true,writable:true,configurable:true});}
  return out;
 }
 throw Error('invalid_node');
}
export function encodeTyped(value,{preserveJsonNumbers=false}={}){return writeLink(['formal-ai-log-v1',node(value,0,preserveJsonNumbers)]);}
export const MAXIMUM_LOG_RECORD_BYTES=1024*1024;
export {MAXIMUM_LOG_RECORD_BYTES as MAX_LOG_RECORD_BYTES};
function decodeWithPolicy(line,{preserveJsonNumbers=false,maximumRecordBytes=MAXIMUM_LOG_RECORD_BYTES,dialogRecord=false}={}){
 if(!Number.isSafeInteger(maximumRecordBytes)||maximumRecordBytes<0)throw Error('invalid_record_size_limit');
 if(typeof line!=='string'||Buffer.byteLength(line)>maximumRecordBytes)throw Error('record_size_limit');
 if(typeof line!=='string'||line.includes('\n')||line.includes('\r'))throw Error('one_line_required');
 const parsed=parseLinks(line);if(parsed.length!==1||parsed[0].length!==2||parsed[0][0]!=='formal-ai-log-v1')throw Error('invalid_record');
 let tree=parsed[0][1];
 if(dialogRecord&&Array.isArray(tree)&&tree[0]==='dialog-tree'){
  if(tree.length!==2||typeof tree[1]!=='string')throw Error('invalid_dialog_tree');
  const expanded=parseLinks(decodeWord(tree[1]));if(expanded.length!==1)throw Error('invalid_dialog_tree');tree=expanded[0];
 }
 const decoded=value(tree,0,preserveJsonNumbers,dialogRecord);if((dialogRecord?encodeDialogTyped(decoded):encodeTyped(decoded,{preserveJsonNumbers}))!==line)throw Error('noncanonical_record');return decoded;
}
export {text as validateTypedText};

export function decodeTyped(line,options={}){return decodeWithPolicy(line,{...options,dialogRecord:false});}
export function encodeDialogTyped(record){
 const tree=node(record,0,true,true),body=writeLink(tree);let depth=1,maximum=1;
 for(const character of body){if(character==='('){depth+=1;maximum=Math.max(maximum,depth);}if(character===')')depth-=1;}
 return writeLink(['formal-ai-log-v1',maximum>64?['dialog-tree',encodeWord(body)]:tree]);
}
export function decodeDialogTyped(line){return decodeWithPolicy(line,{preserveJsonNumbers:true,maximumRecordBytes:typeof line==='string'?Buffer.byteLength(line):0,dialogRecord:true});}
