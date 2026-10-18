// Generated CLI debug credentials: never send token bytes through diagnostics.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

const same=(a,b)=>a.dev===b.dev&&a.ino===b.ino;

/** Only the generated CLI path writes a credential file; embedders retain ctx. */
export function createGeneratedDebugHandoff(session,{env=process.env,url}={}){
  if(!session.generated)throw Error('generated_session_required');
  const parent=path.join(env.HOME||os.homedir(),'.formal-ai');
  fs.mkdirSync(parent,{recursive:true,mode:0o700});
  const directory=fs.mkdtempSync(path.join(parent,'debug-session-'));
  fs.chmodSync(directory,0o700);
  const directoryIdentity=fs.lstatSync(directory);
  const file=path.join(directory,'credential.json');
  let fd=null,fileIdentity=null;
  try{
    fd=fs.openSync(file,'wx',0o600);
    fs.fchmodSync(fd,0o600);
    fileIdentity=fs.fstatSync(fd);
    const uid=typeof process.getuid==='function'?process.getuid():fileIdentity.uid;
    if(!fileIdentity.isFile()||fileIdentity.uid!==uid||(fileIdentity.mode&0o777)!==0o600){
      throw Error('credential_file_custody');
    }
    if(!directoryIdentity.isDirectory()||(directoryIdentity.mode&0o777)!==0o700){
      throw Error('credential_directory_custody');
    }
    fs.writeFileSync(fd,JSON.stringify({
      schema:'formal-ai-debug-session-credential-v1',pid:process.pid,
      session_id:session.id,url,token:session.token,
    })+'\n');
    fs.fsyncSync(fd);
    fs.closeSync(fd);fd=null;
  }catch{
    if(fd!==null)try{fs.closeSync(fd);}catch{}
    if(fileIdentity){
      try{if(same(fs.lstatSync(file),fileIdentity))fs.unlinkSync(file);}catch{}
    }
    try{fs.rmdirSync(directory);}catch{}
    throw Error('debug_credential_handoff_failed');
  }
  let active=true;
  return {
    file,
    cleanup(){
      if(!active)return {cleaned:true};
      try{
        const currentDirectory=fs.lstatSync(directory);
        if(!same(currentDirectory,directoryIdentity)||!currentDirectory.isDirectory()){
          return {cleaned:false,reason:'directory_custody_changed'};
        }
        let currentFile;
        try{currentFile=fs.lstatSync(file);}catch(error){if(error.code!=='ENOENT')throw error;}
        if(currentFile){
          if(!same(currentFile,fileIdentity)||!currentFile.isFile()){
            return {cleaned:false,reason:'credential_custody_changed'};
          }
          fs.unlinkSync(file);
        }
        fs.rmdirSync(directory);active=false;
        return {cleaned:true};
      }catch{return {cleaned:false,reason:'cleanup_failed'};}
    },
  };
}
