// Complete bounded streaming capture; stdout/stderr keep independent UTF-8 decoders.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {StringDecoder} from 'node:string_decoder';
import {constants} from 'node:os';
import {fileURLToPath} from 'node:url';
import {boundedCommand} from '../js/server/bounded-command.mjs';
export async function captureDraftCommand({seconds,phase,command,args=[],directory,env=process.env,maxBytes=134217728,mirror=true}) {
 assert(Number.isSafeInteger(seconds)&&seconds>=0&&seconds<=1800);
 assert(/^[a-z][a-z-]*$/.test(phase)&&path.isAbsolute(directory));
 assert(Number.isSafeInteger(maxBytes)&&maxBytes>0&&maxBytes<=134217728);
 const grace=Number(env.FORMAL_AI_DEADLINE_GRACE_SECONDS??10);
 assert(Number.isSafeInteger(grace)&&grace>=0&&grace<=30);
 fs.mkdirSync(directory,{recursive:true});
 const receiptFile=path.join(directory,phase+'-outcome.json');
 const receipt={schema:'draft-command-outcome/v1',state:'running',budgetSeconds:seconds,
  startedAt:new Date().toISOString(),consoleFormat:'JSONL stream chunks; separate exact stdout/stderr files',maxBytes};
 fs.writeFileSync(receiptFile,JSON.stringify(receipt)+'\n',{flag:'wx',mode:0o600});
 const combined=fs.openSync(path.join(directory,phase+'.log'),'wx',0o600),handles=[],decoders=new Map();
 let bytes=0,sequence=0,cancellationSignal=null;
 const cancellation=new AbortController();
 const onSignal=signal=>{
  cancellationSignal=signal;
  cancellation.abort();
 };
 const onInt=()=>onSignal('SIGINT'),onTerm=()=>onSignal('SIGTERM');
 process.on('SIGINT',onInt);process.on('SIGTERM',onTerm);
 const record=(stream,content)=>{if(content)fs.writeFileSync(combined,JSON.stringify({stream,sequence:++sequence,content})+'\n');};
 const observer=(stream,descriptor)=>{
  const fd=fs.openSync(path.join(directory,phase+'.'+stream+'.log'),'wx',0o600),decoder=new StringDecoder('utf8');
  handles.push(fd);decoders.set(stream,decoder);
  return chunk=>{
   if(bytes+chunk.length>maxBytes)throw Object.assign(Error('console_byte_budget_exhausted'),{code:'ELOGSIZE'});
   bytes+=chunk.length;fs.writeFileSync(fd,chunk);record(stream,decoder.write(chunk));
   if(mirror)fs.writeFileSync(descriptor,chunk);
  };
 };
 let result;
 try{
  const onStdout=observer('stdout',1),onStderr=observer('stderr',2);
  result=await boundedCommand(command,args,{
   timeout:Math.max(1,seconds*1000),terminationGraceMs:grace*1000,signal:cancellation.signal,
   stdoutLimitBytes:0,stderrLimitBytes:0,env,onStdout,onStderr});
 }finally{
  process.off('SIGINT',onInt);process.off('SIGTERM',onTerm);
  for(const [stream,decoder]of decoders)record(stream,decoder.end());
  for(const fd of handles)fs.closeSync(fd);fs.closeSync(combined);
 }
 const complete=result.cleanup_complete&&(!result.error||['ETIMEDOUT','ECANCELLED'].includes(result.error.code));
 const exitCode=result.timed_out?124:result.status??(result.signal?128+(constants.signals[result.signal]??0):1);
 const finished={...receipt,state:'finished',finishedAt:new Date().toISOString(),exitCode,
  signal:result.signal,cancellationSignal,deadlineExpired:result.timed_out,
  commandExitCode:result.status,error:result.error?.code??null,
  outerDeadlineExpired:result.timed_out,cleanupComplete:result.cleanup_complete,bytes,
  captureExitCode:complete?0:1,completeConsoleCapture:complete};
 fs.writeFileSync(receiptFile,JSON.stringify(finished)+'\n',{mode:0o600});
 return finished;
}
if(process.argv[1]&&fs.realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
 const [seconds,phase,command,...args]=process.argv.slice(2);
 const result=await captureDraftCommand({seconds:Number(seconds),phase,command,args,directory:process.env.FORMAL_AI_DRAFT_LOG_DIR});
 process.exitCode=result.completeConsoleCapture?(result.exitCode??1):1;
}
