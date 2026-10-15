// Private issue #1220 candidate. Supplied recipe, zero autonomous authorship credit.
// Reuses the maintained Links Notation grammar/escaping; no JSON wrapped as Lino.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {isFloat,parseJson,toCompactJson} from './json.mjs';
import {encodeWord,decodeWord,parseLinks,writeLink} from '../logging/lino-codec.mjs';
const levels=Object.freeze({silent:-1,error:0,warn:1,info:2,debug:3,trace:4});
const utf8Order=(a,b)=>Buffer.compare(Buffer.from(a),Buffer.from(b));
function text(value){if(typeof value!=='string'||!value.isWellFormed())throw Error('unsupported_text');return value;}
function node(value,depth=0){
 if(depth>64)throw Error('depth_limit');
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
  const items=[];for(let i=0;i<value.length;i++){if(!Object.hasOwn(d[i]??{},'value'))throw Error('unsupported_array');items.push(node(d[i].value,depth+1));}
  return ['array',...items];
 }
 if(value&&Object.getPrototypeOf(value)===Object.prototype){
  const d=Object.getOwnPropertyDescriptors(value),keys=Reflect.ownKeys(d);
  if(keys.some(k=>typeof k!=='string'||!Object.hasOwn(d[k],'value')))throw Error('unsupported_object');
  return ['object',...keys.sort(utf8Order).map(k=>['field',encodeWord(text(k)),node(d[k].value,depth+1)])];
 }
 throw Error('unsupported_value');
}
function value(n,depth=0){
 if(depth>64||!Array.isArray(n))throw Error('invalid_node');
 const [tag,...args]=n;
 if(tag==='null'&&args.length===0)return null;
 if(tag==='string'&&args.length===1&&typeof args[0]==='string')return decodeWord(args[0]);
 if(tag==='boolean'&&args.length===1&&['true','false'].includes(args[0]))return args[0]==='true';
 if(tag==='number'&&args.length===1&&typeof args[0]==='string'){const x=Number(args[0]);if(writeLink(node(x))!==writeLink(n))throw Error('noncanonical_number');return x;}
 if(tag==='array')return args.map(x=>value(x,depth+1));
 if(tag==='object'){
  const out={};for(const f of args){if(!Array.isArray(f)||f.length!==3||f[0]!=='field'||typeof f[1]!=='string')throw Error('invalid_field');
   const key=decodeWord(f[1]);if(Object.hasOwn(out,key))throw Error('duplicate_field');Object.defineProperty(out,key,{value:value(f[2],depth+1),enumerable:true,writable:true,configurable:true});}
  return out;
 }
 throw Error('invalid_node');
}
export function encodeTyped(value){return writeLink(['formal-ai-log-v1',node(value)]);}
export const MAX_LOG_RECORD_BYTES=1024*1024;
export function decodeTyped(line){
 if(typeof line!=='string'||Buffer.byteLength(line)>MAX_LOG_RECORD_BYTES)throw Error('record_size_limit');
 if(typeof line!=='string'||line.includes('\n')||line.includes('\r'))throw Error('one_line_required');
 const parsed=parseLinks(line);if(parsed.length!==1||parsed[0].length!==2||parsed[0][0]!=='formal-ai-log-v1')throw Error('invalid_record');
 const decoded=value(parsed[0][1]);if(encodeTyped(decoded)!==line)throw Error('noncanonical_record');return decoded;
}
const normalizeKey=k=>k.toLowerCase().split('_').join('').split('-').join('');
const secretKeys=new Set(['authorization','proxyauthorization','password','passwd','secret','clientsecret','token','accesstoken','refreshtoken','apikey','formalaiapikey','cookie','setcookie','privatekey','xapikey','xauthtoken','xgoogapikey','anthropicapikey','ghtoken','githubtoken']);
export function makeRedactor(env={}, {preserveJsonNumbers=false}={}){
 const secrets=Object.entries(env).filter(([k,v])=>typeof v==='string'&&v.length>0&&(secretKeys.has(normalizeKey(k))||normalizeKey(k).endsWith('token')||normalizeKey(k).endsWith('secret')||normalizeKey(k).endsWith('apikey'))).map(([,v])=>v).sort((a,b)=>b.length-a.length);
 const scrub=s=>{text(s);for(const secret of secrets)s=s.split(secret).join('[REDACTED]');return s;};
 const walk=(v,depth=0,seen=new Set())=>{
  if(depth>64)throw Error('redaction_depth_limit');
  if(typeof v==='string')return scrub(v);
  if(v===null||['number','boolean'].includes(typeof v))return v;
  if(preserveJsonNumbers&&typeof v==='bigint')return v;
  if(!v||typeof v!=='object'||seen.has(v))throw Error('unsupported_redaction_value');
  seen.add(v);try{
   // V8 may format lazy Error.stack while retrieving its descriptor, invoking message getters.
   // Never materialize stack; other own data fields and nested causes remain available.
   if(v instanceof Error){
  const out={
  type:'Error',stack_omitted:true}
  ;
  const keys=Reflect.ownKeys(v);
  if(keys.some(k=>typeof k!=='string'||k==='type'||k==='stack_omitted'))throw Error('unsupported_error_property');
  for(const k of keys){
  if(k==='stack')continue;
  const d=Object.getOwnPropertyDescriptor(v,k);
  if(!Object.hasOwn(d??{
  }
  ,'value'))throw Error('unsupported_error_property');
  const key=scrub(k);
  if(Object.hasOwn(out,key))throw Error('redacted_key_collision');
  const x=secretKeys.has(normalizeKey(k))?'[REDACTED]':walk(d.value,depth+1,seen);
  Object.defineProperty(out,key,{
  value:typeof x==='string'?x.slice(0,8192):x,enumerable:true,writable:true,configurable:true}
  );
  }
  return out;
  }
  
   const d=Object.getOwnPropertyDescriptors(v);
   if(preserveJsonNumbers&&Object.getPrototypeOf(v)===Object.prototype){
    const keys=Reflect.ownKeys(d);
    const data=keys.every(k=>Object.hasOwn(d[k],'value'));
    if(data&&keys.length===3&&keys.filter(k=>typeof k==='symbol').length===1&&isFloat(v)){
     if(typeof d.value?.value!=='number'||![32,64].includes(d.bits?.value))throw Error('invalid_float_wrapper');
     return v;
    }
   }
   if(Array.isArray(v)){if(Reflect.ownKeys(d).length!==v.length+1)throw Error('unsupported_array_property');const out=[];for(let i=0;i<v.length;i++){if(!Object.hasOwn(d[i]??{},'value'))throw Error('accessor_array');out.push(walk(d[i].value,depth+1,seen));}return out;}
   if(Object.getPrototypeOf(v)!==Object.prototype)throw Error('unsupported_redaction_object');
   const out={
  }
  ;
  for(const k of Reflect.ownKeys(d)){
  if(typeof k!=='string'||!Object.hasOwn(d[k],'value'))throw Error('accessor_object');
  const key=scrub(k);
  if(Object.hasOwn(out,key))throw Error('redacted_key_collision');
  Object.defineProperty(out,key,{
  value:secretKeys.has(normalizeKey(k))?'[REDACTED]':walk(d[k].value,depth+1,seen),enumerable:true,writable:true,configurable:true}
  );
  }
  return out;
  
  }finally{seen.delete(v);}
 };
 const redact=v=>walk(v);
 redact.addSecret=value=>{
  if(typeof value!=='string'||value.length===0)throw Error('invalid_secret');
  text(value);
  if(!secrets.includes(value)){secrets.push(value);secrets.sort((a,b)=>b.length-a.length);}
 };
 return redact;
}
function writeAll(write,bytes){let offset=0;while(offset<bytes.length){const count=write(bytes.subarray(offset));if(!Number.isSafeInteger(count)||count<=0||count>bytes.length-offset)throw Error('sink_progress');offset+=count;}}
// Best-effort diagnostics: return typed fixed failure codes, never raw IO error text.
// This logger makes no durability/provenance guarantee and is not a chain writer.
export function createLogger({level='debug',directory,env=process.env,pid=process.pid,clock=()=>new Date().toISOString(),stderrWrite=b=>fs.writeSync(process.stderr.fd,b),openFile=file=>fs.openSync(file,'a',0o600),fileWrite=(fd,b)=>fs.writeSync(fd,b),closeFile=fd=>fs.closeSync(fd)}={}){
 if(!Object.hasOwn(levels,level))throw Error('invalid_log_level');
 const redact=makeRedactor(env),failures=[],sinks=[];let sequence=0,fd=null,file=null;
 if(level!=='silent'){
  sinks.push({name:'stderr',write:stderrWrite,enabled:true});
  if(directory){try{fs.mkdirSync(directory,{recursive:true,mode:0o700});file=path.join(directory,'serve-'+pid+'.lino');fd=openFile(file);sinks.push({name:'file',write:b=>fileWrite(fd,b),enabled:true});}catch{failures.push({sink:'file',reason:'open_failed'});}}
 }
 return {
  file,failures,level,
  addSecret(value){redact.addSecret(value);},
  log(eventLevel,kind,metadata={},payload){
   if(!Object.hasOwn(levels,eventLevel)||eventLevel==='silent')throw Error('invalid_event_level');
   if(levels[eventLevel]>levels[level])return {emitted:false,reason:'disabled'};
   let line;
   try{
    const safeMetadata=redact(metadata);
    const record=redact({schema:'formal-ai-diagnostic-v1',timestamp:clock(),timestamp_precision:'milliseconds',pid,sequence:++sequence,level:eventLevel,component:safeMetadata.component??'server',dialog_id:safeMetadata.dialog_id??null,message:kind,kind,metadata:safeMetadata});
    if(payload!==undefined){
  const content=redact(typeof payload==='function'?payload():payload),canonical=encodeTyped(content);
  record.payload={
  algorithm:'sha256',bytes:Buffer.byteLength(canonical),digest:createHash('sha256').update(canonical).digest('hex')}
  ;
  if(level==='trace'){
    if(Buffer.byteLength(canonical)<=MAX_LOG_RECORD_BYTES/2)record.payload.content=content;
    else record.payload.content_omitted='record_size_limit';
  }
  }
  
    line=encodeTyped(record)+'\n';
   }catch{return {emitted:false,reason:'serialization_failed'};}
   const bytes=Buffer.from(line),delivered=[];
   if(bytes.length>MAX_LOG_RECORD_BYTES)return {emitted:false,reason:'record_size_limit'};
   for(const sink of sinks){if(!sink.enabled)continue;try{writeAll(sink.write,bytes);delivered.push(sink.name);}catch{sink.enabled=false;failures.push({sink:sink.name,reason:'write_failed'});}}
   return {emitted:delivered.length>0,delivered};
  },
  close(){if(fd!==null){try{closeFile(fd);}catch{failures.push({sink:'file',reason:'close_failed'});}fd=null;}}
 };
}

// Explicit format conversion API. Conversation endpoint migration is separate.
// Reject unterminated input so a failed/partial append is never silently complete.
export function convertLog(input,{from='lino',to='lino',env=process.env}={}){
 if(!['lino','jsonl'].includes(from)||!['lino','json','jsonl','pretty'].includes(to))throw Error('invalid_format');
 if(typeof input!=='string'||Buffer.byteLength(input)>16*MAX_LOG_RECORD_BYTES)throw Error('log_size_limit');
 if(input!==''&&!input.endsWith('\n'))throw Error('unterminated_log');
 const redact=makeRedactor(env),records=input===''?[]:input.slice(0,-1).split('\n').map(line=>redact(from==='lino'?decodeTyped(line):JSON.parse(line)));
 // Validate the same supported typed model for every export format.
 const encoded=records.map(encodeTyped);
 if(encoded.some(line=>Buffer.byteLength(line)>MAX_LOG_RECORD_BYTES))throw Error('record_size_limit');
 if(to==='lino')return encoded.map(x=>x+'\n').join('');
 if(to==='jsonl')return records.map(x=>JSON.stringify(x)+'\n').join('');
 return JSON.stringify(records,null,to==='pretty'?2:0)+'\n';
}

const truth=v=>['1','true','yes','on'].includes(String(v??'').trim().toLowerCase());
export function createServeLogger(options={}){
 const env=options.env??process.env,silent=options.silent===true||truth(env.FORMAL_AI_SILENT);
 const configured=options.logLevel??env.FORMAL_AI_LOG_LEVEL??'debug';
 if(!silent&&configured==='trace'&&options.trace!==true)throw Error('trace_requires_explicit_flag');
 const level=silent?'silent':options.trace===true?'trace':configured;
 const directory=options.logDirectory??env.FORMAL_AI_LOG_DIR??path.join(env.HOME||os.homedir(),'.formal-ai','logs');
 return createLogger({env,level,directory});
}
export function diagnosticBody(body){try{return JSON.parse(body);}catch{return String(body??'');}}
// Legacy serde-compatible bodies preserve Float markers/BigInt through redaction.
// Diagnostic encodeTyped deliberately keeps its narrower finite/safe-number model.
export function redactedBody(body,env=process.env){
 let value;
 try{value=parseJson(body);}catch{return makeRedactor(env)(String(body??''));}
 return toCompactJson(makeRedactor(env,{preserveJsonNumbers:true})(value));
}
