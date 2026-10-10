#!/usr/bin/env node
// Immutable Actions artifact download: authenticated ID metadata and full ZIP digest.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {inflateRawSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const limit=16*1024*1024;
const crc=bytes=>{let value=0xffffffff;for(const byte of bytes){value^=byte;for(let bit=0;bit<8;bit++)value=(value>>>1)^((value&1)?0xedb88320:0);}return (value^0xffffffff)>>>0;};
/** Conservative single-disk ZIP32 reader. Unknown formats refuse before any write. */
export function readArtifactZip(bytes) {
 assert.ok(Buffer.isBuffer(bytes));assert.ok(bytes.length>=22&&bytes.length<=limit);
 let end=-1;
 for(let offset=bytes.length-22;offset>=Math.max(0,bytes.length-65557);offset--)
  if(bytes.readUInt32LE(offset)===0x06054b50&&offset+22+bytes.readUInt16LE(offset+20)===bytes.length){end=offset;break;}
 assert.ok(end>=0,'missing ZIP end');
 assert.equal(bytes.readUInt16LE(end+4),0);assert.equal(bytes.readUInt16LE(end+6),0);
 const count=bytes.readUInt16LE(end+10);assert.equal(bytes.readUInt16LE(end+8),count);
 assert.ok(count>0&&count<=256,'ZIP entry limit');
 const size=bytes.readUInt32LE(end+12),start=bytes.readUInt32LE(end+16);
 assert.equal(start+size,end,'ZIP central boundary');
 const entries=[],names=new Set(),ranges=[];let offset=start,total=0;
 for(let index=0;index<count;index++) {
  assert.ok(offset+46<=end);assert.equal(bytes.readUInt32LE(offset),0x02014b50);
  const flags=bytes.readUInt16LE(offset+8),method=bytes.readUInt16LE(offset+10);
  assert.equal(flags&~0x0808,0,'encrypted or unknown ZIP flags');assert.ok([0,8].includes(method));
  const checksum=bytes.readUInt32LE(offset+16),compressed=bytes.readUInt32LE(offset+20),length=bytes.readUInt32LE(offset+24);
  const nameLength=bytes.readUInt16LE(offset+28),extra=bytes.readUInt16LE(offset+30),comment=bytes.readUInt16LE(offset+32);
  assert.equal(bytes.readUInt16LE(offset+34),0);assert.ok(offset+46+nameLength+extra+comment<=end);
  const mode=bytes.readUInt32LE(offset+38)>>>16;assert.ok([0,0x8000,0x4000].includes(mode&0xf000),'ZIP symlink or special file');
  const nameBytes=bytes.subarray(offset+46,offset+46+nameLength);
  const name=new TextDecoder('utf-8',{fatal:true}).decode(nameBytes);
  assert.match(name,/^[A-Za-z0-9_./-]+$/u,'portable artifact filename required');
  assert.ok(name.length>0&&name.length<=512&&!/[\\\x00-\x1f:]/u.test(name));
  assert.ok(!name.startsWith('/')&&name===name.normalize('NFC'),'absolute or noncanonical ZIP name');
  const directory=name.endsWith('/'),parts=(directory?name.slice(0,-1):name).split('/');
  assert.ok(parts.every(part=>part!==''&&part!=='.'&&part!=='..'&&!/[ .]$/u.test(part)),'unsafe ZIP path');
  const key=parts.join('/').toLowerCase();assert.ok(!names.has(key),'ZIP duplicate/case collision');names.add(key);
  const local=bytes.readUInt32LE(offset+42);assert.ok(local+30<=start);assert.equal(bytes.readUInt32LE(local),0x04034b50);
  assert.equal(bytes.readUInt16LE(local+6),flags);assert.equal(bytes.readUInt16LE(local+8),method);
  const localName=bytes.readUInt16LE(local+26),localExtra=bytes.readUInt16LE(local+28);
  assert.equal(localName,nameLength);assert.ok(bytes.subarray(local+30,local+30+localName).equals(nameBytes));
  const begin=local+30+localName+localExtra,finish=begin+compressed;assert.ok(finish<=start);
  if(!(flags&8)){assert.equal(bytes.readUInt32LE(local+14),checksum);assert.equal(bytes.readUInt32LE(local+18),compressed);assert.equal(bytes.readUInt32LE(local+22),length);}
  assert.ok(length<=limit&&compressed<=limit);total+=length;assert.ok(total<=limit,'ZIP inflated limit');
  const content=method===0?bytes.subarray(begin,finish):inflateRawSync(bytes.subarray(begin,finish),{maxOutputLength:limit});
  assert.equal(content.length,length);assert.equal(crc(content),checksum,'ZIP CRC mismatch');
  if(directory)assert.equal(length,0);
  let recordEnd=finish;
  if(flags&8){
   const signed=bytes.readUInt32LE(finish)===0x08074b50;
   const descriptor=finish+(signed?4:0);
   assert.ok(descriptor+12<=start);
   assert.equal(bytes.readUInt32LE(descriptor),checksum);
   assert.equal(bytes.readUInt32LE(descriptor+4),compressed);
   assert.equal(bytes.readUInt32LE(descriptor+8),length);
   recordEnd=descriptor+12;
  }
  ranges.push([local,recordEnd]);entries.push({name:parts.join('/'),directory,bytes:content});
  offset+=46+nameLength+extra+comment;
 }
 assert.equal(offset,end);
 ranges.sort((a,b)=>a[0]-b[0]);for(let i=1;i<ranges.length;i++)assert.ok(ranges[i-1][1]<=ranges[i][0],'overlapping ZIP entries');
 for(const entry of entries)for(const other of entries)
  if(other.name.toLowerCase().startsWith(entry.name.toLowerCase()+'/'))assert.ok(entry.directory,'ZIP file/directory collision');
 return entries;
}
export function verifyArtifactArchive(expected,metadata,archive) {
 assert.match(expected.repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
 for(const key of ['id','run'])assert.match(String(expected[key]),/^[1-9][0-9]*$/u);
 assert.match(expected.head,/^[a-f0-9]{40}$/u);assert.ok(expected.name.length>0);
 assert.equal(String(metadata.id),String(expected.id));assert.equal(metadata.name,expected.name);
 assert.equal(metadata.expired,false);assert.equal(String(metadata.workflow_run.id),String(expected.run));
 assert.equal(metadata.workflow_run.head_sha,expected.head);
 assert.equal(metadata.size_in_bytes,archive.length,'actual archive byte length differs');
 assert.equal(metadata.digest,'sha256:'+sha(archive),'full ZIP digest mismatch');
 return readArtifactZip(archive);
}
export function downloadArtifact(expected,destination,run=spawnSync) {
 assert.match(expected.repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
 for(const key of ['id','run'])assert.match(String(expected[key]),/^[1-9][0-9]*$/u);
 assert.match(expected.head,/^[a-f0-9]{40}$/u);assert.ok(typeof expected.name==='string'&&expected.name.length>0);
 const base='repos/'+expected.repository+'/actions/artifacts/'+expected.id;
 const deadline=Date.now()+60000;
 const invoke=(endpoint,maxBuffer)=>{
  const remaining=deadline-Date.now();assert.ok(remaining>0,'artifact transfer deadline expired');
  const result=run('gh',['api','--hostname','github.com',endpoint],{timeout:Math.min(30000,remaining),maxBuffer});
  if(result.error)throw result.error;assert.equal(result.status,0,result.stderr?.toString());assert.equal(result.signal,null);
  return result.stdout;
 };
 const metadata=JSON.parse(invoke(base,1024*1024));
 const archive=invoke(base+'/zip',limit+1);assert.ok(archive.length<=limit);
 const entries=verifyArtifactArchive(expected,metadata,archive);
 const repeated=JSON.parse(invoke(base,1024*1024));assert.deepEqual(repeated,metadata,'artifact metadata changed during transfer');
 const target=path.resolve(destination);assert.equal(fs.existsSync(target),false,'fresh artifact destination required');
 let parent=path.dirname(target);
 while(parent!==path.dirname(parent)){if(fs.existsSync(parent))assert.ok(fs.lstatSync(parent).isDirectory()&&!fs.lstatSync(parent).isSymbolicLink());parent=path.dirname(parent);}
 fs.mkdirSync(target,{recursive:true});
 for(const entry of entries){const file=path.join(target,entry.name);assert.ok(file.startsWith(target+path.sep));
  if(entry.directory)fs.mkdirSync(file,{recursive:true});else{fs.mkdirSync(path.dirname(file),{recursive:true});fs.writeFileSync(file,entry.bytes,{flag:'wx',mode:0o600});}}
 return {artifactId:metadata.id,zipSha256:sha(archive),files:entries.filter(e=>!e.directory).length};
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const expected={repository:process.env.GITHUB_REPOSITORY,id:process.env.ARTIFACT_ID,run:process.env.GITHUB_RUN_ID,
  head:process.env.FIXTURE_EVENT_HEAD,name:process.env.ARTIFACT_NAME};
 console.log(JSON.stringify(downloadArtifact(expected,process.env.ARTIFACT_DESTINATION)));
}
