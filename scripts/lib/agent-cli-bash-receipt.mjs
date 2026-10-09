import {spawn} from 'node:child_process';
import {StringDecoder} from 'node:string_decoder';

// Producer-owned status: only `close` supplies the exit and stream witness.
export async function executeShellReceipt({command,cwd,timeout,abort,maxOutputLength=30000,onOutput=()=>{}}) {
 const proc=spawn(command,{shell:true,cwd,env:{...process.env},stdio:['ignore','pipe','pipe'],detached:process.platform!=='win32'});
 const decoders={stdout:new StringDecoder('utf8'),stderr:new StringDecoder('utf8')};
 const captured={stdout:'',stderr:''};
 const bytes={stdout:0,stderr:0};
 let retained=0, truncated=false, timedOut=false, aborted=false, closed=false, observedExit=null, observedSignal=null;
 const capture=(channel,text)=>{
  let take=Math.min(text.length,Math.max(0,maxOutputLength-retained));
  // Do not split a Unicode scalar at the retained boundary.
  if(take>0&&take<text.length&&/[\uD800-\uDBFF]/u.test(text[take-1])) take--;
  if(take<text.length) truncated=true;
  captured[channel]+=text.slice(0,take); retained+=take;
  onOutput(captured.stdout+captured.stderr);
 };
 for(const channel of ['stdout','stderr']) {
  proc[channel].on('data',chunk=>{bytes[channel]+=chunk.length;capture(channel,decoders[channel].write(chunk));});
  proc[channel].once('end',()=>capture(channel,decoders[channel].end()));
 }
 const killTree=()=>{
  if(closed||!proc.pid)return;
  if(process.platform==='win32') {
   const killer=spawn('taskkill',['/pid',String(proc.pid),'/f','/t'],{stdio:'ignore'});
   killer.on('error',()=>proc.kill('SIGTERM'));
  } else {
   try{process.kill(-proc.pid,'SIGTERM');}catch{proc.kill('SIGTERM');}
   const force=setTimeout(()=>{if(!closed){try{process.kill(-proc.pid,'SIGKILL');}catch{proc.kill('SIGKILL');}}},200);
   force.unref();
  }
 };
 const abortHandler=()=>{aborted=true;killTree();};
 abort?.addEventListener('abort',abortHandler,{once:true});
 const timer=setTimeout(()=>{timedOut=true;killTree();},timeout);
 try {
  const completion=new Promise((resolve,reject)=>{
   proc.once('close',(code,signal)=>{closed=true;observedExit=code;observedSignal=signal;resolve();});
   proc.once('error',reject);
  });
  if(abort?.aborted)abortHandler();
  await completion;
 } finally {
  clearTimeout(timer);abort?.removeEventListener('abort',abortHandler);
 }
 return {schema:'bash-execution-receipt/v1',command,cwd,exit_code:observedExit,signal:observedSignal,
  stream_complete:closed,complete:closed&&!truncated&&!timedOut&&!aborted,
  truncated,timed_out:timedOut,aborted,stdout:captured.stdout,stderr:captured.stderr,
  stdout_bytes:bytes.stdout,stderr_bytes:bytes.stderr,retained_code_units:retained,output_limit_code_units:maxOutputLength};
}

export function encodeExecutionReceipt(receipt) {return JSON.stringify(receipt);}
