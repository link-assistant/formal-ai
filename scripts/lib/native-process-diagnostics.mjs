// Render retained child diagnostics only; no native invocation or authority grant.
import {writeSync} from 'node:fs';
import {randomUUID} from 'node:crypto';
const chunkBytes=64*1024,jsonLineBytes=1024*1024;
function stderr(bytes){for(let offset=0;offset<bytes.length;){const count=writeSync(2,bytes,offset,bytes.length-offset);if(count<=0)throw Error('diagnostic_stderr_closed');offset+=count;}}
export function failedNativeProcess(result){return Boolean(result.error||result.signal||result.status!==0);}
export function reportNativeProcessFailure(stage,result,{cargoJson=false,write=stderr}={}){
 if(!['build','producer'].includes(stage))throw Error('invalid_diagnostic_stage');
 if(!failedNativeProcess(result))return false;
 const emit=value=>{const bytes=Buffer.isBuffer(value)?value:Buffer.from(value);for(let offset=0;offset<bytes.length;offset+=chunkBytes)write(bytes.subarray(offset,offset+chunkBytes));};
 const stdout=result.stdout??Buffer.alloc(0),stderrBytes=result.stderr??Buffer.alloc(0);
 if(!Buffer.isBuffer(stdout)||!Buffer.isBuffer(stderrBytes))throw Error('captured_diagnostic_bytes_required');
 const token=randomUUID();emit('::stop-commands::'+token+'\n');
 try{
  const errorCode=typeof result.error?.code==='string'&&/^[A-Z0-9_]{1,64}$/u.test(result.error.code)?result.error.code:result.error?'unknown':null;
  emit('Native '+stage+' failed: '+JSON.stringify({exitCode:result.status??null,signal:result.signal??null,errorCode,stdoutBytes:stdout.length,stderrBytes:stderrBytes.length})+'\n');
  emit('--- captured '+stage+' stdout'+(cargoJson?' (Cargo rendered diagnostics and unchanged other rows)':'')+' ---\n');
  if(cargoJson){let start=0;while(start<stdout.length){const newline=stdout.indexOf(10,start),end=newline<0?stdout.length:newline+1,line=stdout.subarray(start,end);let rendered=null;if(line.length<=jsonLineBytes){try{const row=JSON.parse(line.toString('utf8'));if(row.reason==='compiler-message'&&typeof row.message?.rendered==='string')rendered=row.message.rendered;}catch{/* Preserve malformed/non-JSON output below. */}}emit(rendered===null?line:rendered);start=end;}}
  else emit(stdout);
  emit('\n--- captured '+stage+' stderr ---\n');emit(stderrBytes);emit('\n--- end captured '+stage+' diagnostics; raw bytes and hashes retained in artifact ---\n');
 }finally{emit('::'+token+'::\n');}
 return true;
}
