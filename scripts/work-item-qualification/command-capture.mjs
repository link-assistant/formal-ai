// Complete bounded preparatory/transport process output, before public redaction.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
export function createCommandCapture(directory,{maximumBytes=256*1024*1024,maximumCommands=2048}={}) {
  if(!Number.isSafeInteger(maximumBytes)||maximumBytes<0||maximumBytes>256*1024*1024
      ||!Number.isSafeInteger(maximumCommands)||maximumCommands<1||maximumCommands>2048)throw Error('InvalidCommandCaptureBounds');
  const destination=path.join(directory,'commands');
  fs.mkdirSync(destination,{recursive:true,mode:0o700});
  let bytes=0,commands=0;
  return (metadata,stdout,stderr,input=null)=>{
    const streams=[['stdout',stdout],['stderr',stderr],['stdin',input]]
      .filter(([,value])=>value!==null&&value!==undefined).map(([name,value])=>[name,Buffer.from(value)]);
    const amount=streams.reduce((total,[,value])=>total+value.length,0);
    if(commands>=maximumCommands||bytes+amount>maximumBytes){
      fs.writeFileSync(path.join(destination,'capture-incomplete.json'),JSON.stringify({
        reason:'CommandCaptureBudgetExceeded',commands,bytes,maximumCommands,maximumBytes,complete:false}));
      throw Error('CommandCaptureBudgetExceeded');
    }
    bytes+=amount;commands+=1;
    const prefix=String(commands).padStart(6,'0');
    const members=[];
    for(const [name,value]of streams){
      const member=prefix+'.'+name+'.log';
      fs.writeFileSync(path.join(destination,member),value,{flag:'wx',mode:0o600});
      members.push({stream:name,path:member,bytes:value.length,
        sha256:createHash('sha256').update(value).digest('hex')});
    }
    fs.writeFileSync(path.join(destination,prefix+'.json'),JSON.stringify({
      ...metadata,members,capturedBytes:amount,complete:!metadata.error},null,2)+'\n',{flag:'wx',mode:0o600});
  };
}
