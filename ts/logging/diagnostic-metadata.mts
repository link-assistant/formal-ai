import {createHash} from 'node:crypto';
import {encodeTyped} from './typed-codec.mjs';
// Mirrors bounded_diagnostic_metadata in rust/src/structured_log.rs. These are
// producer budgets, independent of the unchanged one-MiB diagnostic reader cap.
export const DIAGNOSTIC_FIELD_BYTES=4096;
export const DIAGNOSTIC_METADATA_BYTES=64*1024;
export const DIAGNOSTIC_PREVIEW_BYTES=256;
function summary(value,canonical) {
 const content=typeof value==='string'?value:canonical;
 const bytes=Buffer.from(content);let end=Math.min(bytes.length,DIAGNOSTIC_PREVIEW_BYTES);
 while(end>0&&end<bytes.length&&(bytes[end]&0xc0)===0x80)end--;
 return {diagnostic_summary:'post-redaction/v1',encoding:typeof value==='string'?'utf8':'formal-ai-log-v1',bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),preview:bytes.subarray(0,end).toString('utf8'),preview_truncated:end<bytes.length};
}
function ownField(object,key,value){Object.defineProperty(object,key,{value,enumerable:true,writable:true,configurable:true});}
function boundedValue(value) {
 // Raw UTF-8 size is a lower bound on encoded size; avoid parsing huge CST strings.
 if(typeof value==='string'&&Buffer.byteLength(value)>DIAGNOSTIC_FIELD_BYTES)return summary(value,'');
 const canonical=encodeTyped(value);
 return Buffer.byteLength(canonical)>DIAGNOSTIC_FIELD_BYTES?summary(value,canonical):value;
}
/** Mirrors `fn bounded_diagnostic_metadata` in rust/src/structured_log.rs; input must already be redacted. */
export function boundedDiagnosticMetadata(value) {
 if(!value||typeof value!=='object'||Array.isArray(value))return boundedValue(value);
 const candidates=Object.entries(value).map(([key,item])=>{
  const projected=boundedValue(item);
  const singleton={};ownField(singleton,key,projected);
  const cost=Buffer.byteLength(key)>DIAGNOSTIC_METADATA_BYTES?DIAGNOSTIC_METADATA_BYTES+1:Buffer.byteLength(encodeTyped(singleton));
  return {key,value:projected,cost};
 }).sort((left,right)=>left.cost-right.cost||Buffer.compare(Buffer.from(left.key),Buffer.from(right.key)));
 const fields={};let remaining=DIAGNOSTIC_METADATA_BYTES-2048,omitted=0;
 for(const candidate of candidates){
  if(candidate.cost>remaining){omitted++;continue;}
  ownField(fields,candidate.key,candidate.value);remaining-=candidate.cost;
 }
 // A full original encoding is needed only to authenticate actual aggregate omissions.
 return omitted?{diagnostic_metadata:'bounded-post-redaction/v1',fields,omitted_fields:omitted,omitted:summary(value,encodeTyped(value))}:fields;
}
