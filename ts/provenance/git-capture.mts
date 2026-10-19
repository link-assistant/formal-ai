// Actual shell completion and Git object observations, never inferred edit ownership.
import fs from 'node:fs';
import path from 'node:path';
import {createHash}from 'node:crypto';
import {repositoryGit,repositoryText,observeRepositoryCommit,commitPrefix}from './git-evidence.mjs';
import {readBoundedFileBytes}from './bounded-file.mjs';
export function createGitToolCapture(options){
 const {workspace,append,retainBytes,currentOutput,latestRecord,editHashes}=options;
 let active=null;
 const text=value=>repositoryText(repositoryGit(workspace,value)).trimEnd();
 function observedHead(){
  try{
   if(fs.realpathSync(text(['rev-parse','--show-toplevel']))!==fs.realpathSync(workspace))return null;
   const head=text(['rev-parse','--verify','HEAD']);
   return /^[a-f0-9]{40}$/u.test(head)?head:null;
  }catch{return null;}
 }
 return {
  beforeTool(call){
   if(call.tool!=='bash')return null;
   if(active!==null)throw Error('provenance_tool_boundary_overlap');
   const messageOut=currentOutput();if(!messageOut)throw Error('provenance_tool_output_required');
   const before=observedHead(),callEvidence=retainBytes(Buffer.from(JSON.stringify(call)));
   const start=append('tool_start',{messageOut,call:callEvidence,beforeHead:before});
   const anchor=latestRecord();active={start,messageOut,before,anchor,process:null};return active;
  },
  toolEnvironment(){return active?.before?{...process.env,FORMAL_AI_CHAIN_PREFIX:commitPrefix(active.anchor)}:undefined;},
  processObservation(observed){
   if(active===null)return;
   if(active.process!==null)throw Error('provenance_duplicate_process_observation');
   const retainStream=stream=>{
    const bytes=readBoundedFileBytes(path.dirname(stream.path),path.basename(stream.path),64*1024*1024);
    if(bytes.length!==stream.bytes||createHash('sha256').update(bytes).digest('hex')!==stream.sha256)throw Error('provenance_process_stream_changed');
    return {bytes:stream.bytes,sha256:stream.sha256,evidence:retainBytes(bytes)};
   };
   active.process=append('tool_process',{toolStart:active.start,messageOut:active.messageOut,
    command:observed.command,cwd:observed.cwd,pid:observed.pid,status:observed.status,signal:observed.signal,error:observed.error,
    stdout:retainStream(observed.stdout),stderr:retainStream(observed.stderr),
    scope:'actual-shell-process-completion-and-snapshotted-streams'});
  },
  afterTool(boundary,call,receipt){
   if(boundary===null)return;
   if(active!==boundary)throw Error('provenance_tool_boundary_mismatch');
   try{
    append('tool_result',{toolStart:boundary.start,messageOut:boundary.messageOut,process:boundary.process,
     evidence:retainBytes(Buffer.from(JSON.stringify(receipt)))});
    const after=observedHead();if(boundary.before===null||after===null||after===boundary.before)return;
    let topology;
    try{
     repositoryGit(workspace,['merge-base','--is-ancestor',boundary.before,after]);
     topology=text(['rev-list','--reverse','--parents',boundary.before+'..'+after]).split('\n');
     if(topology.length>1000)throw Error('provenance_commit_range_limit');
    }catch{append('repository_gap',{toolStart:boundary.start,before:boundary.before,after,reason:'non-descendant-or-unavailable-range'});return;}
    const transition=append('repository_transition',{toolStart:boundary.start,process:boundary.process,
     before:boundary.before,after,topology});
    for(const row of topology){
     const [commit,...parents]=row.split(' ');
     try{
      const observation=observeRepositoryCommit(workspace,commit,{messageOut:boundary.messageOut,
       editHashes:[...editHashes],precommitAnchor:boundary.anchor});
      const recorded=append('commit',observation);
      append('commit_execution',{commitRecord:recorded,toolStart:boundary.start,process:boundary.process,transition,
       before:boundary.before,after,commit,parents});
     }catch{append('commit_observation_gap',{transition,commit,parents,reason:'unsupported-or-unavailable-commit'});}
    }
    editHashes.length=0;
   }finally{active=null;}
  }
 };
}
