// Canonical descriptor-bound byte reads for private runtime, memory and evidence files.
import fs from 'node:fs';
import path from 'node:path';
export function readBoundedFileBytes(directory,relative,maximum){
 const parts=relative.split('/');let filename=fs.realpathSync(directory);
 for(let index=0;index<parts.length;index++){
  filename=path.join(filename,parts[index]);const state=fs.lstatSync(filename);
  if(state.isSymbolicLink())throw Error('RuntimeArchiveSymlink');
  if(index<parts.length-1&&!state.isDirectory())throw Error('RuntimeArchiveDirectory');
 }
 if(typeof fs.constants.O_NOFOLLOW!=='number')throw Error('RuntimeArchiveNoFollowUnavailable');
 const descriptor=fs.openSync(filename,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
 try{
  const before=fs.fstatSync(descriptor,{bigint:true});
  if(!before.isFile()||before.size>BigInt(maximum)||before.size<0n)throw Error('RuntimeArchiveSizeOrType');
  const length=Number(before.size),bytes=Buffer.alloc(length);let offset=0;
  while(offset<length){
   const count=fs.readSync(descriptor,bytes,offset,length-offset,null);
   if(count===0)throw Error('RuntimeArchiveTruncated');offset+=count;
  }
  const after=fs.fstatSync(descriptor,{bigint:true}),current=fs.lstatSync(filename,{bigint:true});
  if(current.isSymbolicLink()||current.dev!==before.dev||current.ino!==before.ino
    ||after.size!==before.size||after.mtimeNs!==before.mtimeNs||after.ctimeNs!==before.ctimeNs
    ||current.size!==before.size||current.mtimeNs!==before.mtimeNs||current.ctimeNs!==before.ctimeNs)throw Error('RuntimeArchiveChanged');
  return bytes;
 }finally{fs.closeSync(descriptor);}
}
