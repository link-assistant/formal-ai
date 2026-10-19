import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {openMemoryCustody}from '../../../js/server/provenance-memory.mjs';
import {parseLinksNotation,writeAtomic}from '../../../js/server/memory-store.mjs';
import {blake3Hex}from '../../../js/crypto/blake3.mjs';
test('actual canonical memory records exact message bytes with private atomic revisions',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'logical-memory-'));
 try{
  const workspace=path.join(directory,'workspace');fileSystem.mkdirSync(workspace);
  const file=path.join(directory,'memory.lino'),taskURL='https://example.invalid/tasks/declared-fixture';
  const custody=openMemoryCustody({file,workspace,sessionId:'declared-session',taskURL});
  const initial=custody.snapshot();assert.equal(initial.memoryHash,blake3Hex(fileSystem.readFileSync(file)));
  const input=Buffer.from('{"prompt":"unseen 语言"}'),output=Buffer.from('{"answer":"independent 47"}');
  const first=custody.advance('message_in',input),second=custody.advance('message_out',output);
  assert.equal(first.previousMemoryHash,initial.memoryHash);assert.equal(second.previousMemoryHash,first.memoryHash);
  assert.equal(first.revision,1);assert.equal(second.revision,2);
  assert.equal(second.memoryHash,blake3Hex(fileSystem.readFileSync(file)));
  const events=parseLinksNotation(fileSystem.readFileSync(file,'utf8'));
  assert.equal(events.length,2);assert.equal(events[0].inputs,'message_in');assert.equal(events[1].inputs,'message_out');
  assert.equal(events[0].conversation_id,'declared-session');
  assert.throws(()=>openMemoryCustody({file,workspace,sessionId:'other',taskURL}),/EEXIST/);
  custody.close();
  const reopened=openMemoryCustody({file,workspace,sessionId:'next',taskURL});
  assert.equal(reopened.snapshot().revision,2);reopened.close();
  if(process.platform!=='win32')assert.equal(fileSystem.statSync(file).mode&0o077,0);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
test('an actual one-byte external memory edit refuses at the next boundary and releases its owner',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'logical-memory-tamper-'));
 try{
  const workspace=path.join(directory,'workspace');fileSystem.mkdirSync(workspace);
  const file=path.join(directory,'memory.lino'),taskURL='https://example.invalid/tasks/tamper-fixture';
  const custody=openMemoryCustody({file,workspace,sessionId:'tamper-session',taskURL});
  custody.advance('message_in',Buffer.from('actual input'));
  const bytes=fileSystem.readFileSync(file);bytes[0]^=1;fileSystem.writeFileSync(file,bytes);
  assert.throws(()=>custody.advance('message_out',Buffer.from('must not be persisted')),/MemoryOutOfBandEdit/);
  assert.throws(()=>custody.close(),/MemoryOutOfBandEdit/);
  assert.equal(fileSystem.existsSync(file+'.provenance-owner.lino'),false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
test('inside-workspace configuration and symlink memory files refuse without recording',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'logical-memory-paths-'));
 try{
  const workspace=path.join(directory,'workspace');fileSystem.mkdirSync(workspace);
  const taskURL='https://example.invalid/tasks/path-fixture';
  assert.throws(()=>openMemoryCustody({file:path.join(workspace,'absent','memory.lino'),workspace,sessionId:'path',taskURL}),/MemoryInsideWorkspace/);
  assert.deepEqual(fileSystem.readdirSync(workspace),[]);
  const target=path.join(directory,'target');fileSystem.writeFileSync(target,'unchanged',{mode:0o600});
  const alias=path.join(directory,'memory.lino');fileSystem.symlinkSync(target,alias);
  assert.throws(()=>openMemoryCustody({file:alias,workspace,sessionId:'path',taskURL}),/MemoryCustodyFileNotPrivate/);
  assert.equal(fileSystem.readFileSync(target,'utf8'),'unchanged');
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
test('private atomic writes reject a stale temporary file and preserve default writer behavior',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'logical-memory-atomic-'));
 try{
  const filename=path.join(directory,'memory.lino');writeAtomic(filename,'original');
  const temporary=filename+'.tmp.'+process.pid;fileSystem.writeFileSync(temporary,'foreign');
  assert.throws(()=>writeAtomic(filename,'replacement',{privateMode:true}),/EEXIST/);
  assert.equal(fileSystem.readFileSync(filename,'utf8'),'original');
  assert.equal(fileSystem.readFileSync(temporary,'utf8'),'foreign');
  writeAtomic(filename,'legacy');assert.equal(fileSystem.readFileSync(filename,'utf8'),'legacy');
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});

test('external durable-revision mutation refuses the next observed memory boundary',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'logical-memory-revision-'));
 try{
  const workspace=path.join(directory,'workspace');fileSystem.mkdirSync(workspace);
  const file=path.join(directory,'memory.lino'),taskURL='https://example.invalid/tasks/revision-fixture';
  const custody=openMemoryCustody({file,workspace,sessionId:'revision-session',taskURL});
  custody.advance('message_in',Buffer.from('actual revision input'));
  const revision=file+'.provenance-revision.lino',bytes=fileSystem.readFileSync(revision);
  bytes[0]^=1;fileSystem.writeFileSync(revision,bytes);
  assert.throws(()=>custody.snapshot(),/MemoryRevisionOutOfBand/);
  assert.throws(()=>custody.close(),/MemoryRevisionOutOfBand/);
  assert.equal(fileSystem.existsSync(file+'.provenance-owner.lino'),false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
