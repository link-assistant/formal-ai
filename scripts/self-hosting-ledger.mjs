// JavaScript twin of self-hosting-ledger.rs; preserves complete source records.
import assert from 'node:assert/strict';
import {readFileSync,lstatSync,openSync,writeFileSync,closeSync,fsyncSync,mkdirSync,renameSync} from 'node:fs';
import {dirname,join} from 'node:path';
const LIMIT=1500,MAX_PARTS=128,PREFIX='  archive "self-hosting-ledger-parts/';
const count=text=>text===''?0:text.split('\n').length-Number(text.endsWith('\n'));
function readText(file) {
  const meta=lstatSync(file);
  assert.ok(meta.isFile()&&meta.size<=1048576,'invalid ledger file');
  return new TextDecoder('utf-8',{fatal:true}).decode(readFileSync(file));
}
function parts(text) {
  const names=[];
  for(const line of text.split('\n').filter(line=>line.startsWith('  archive'))) {
    const name=String(names.length+1).padStart(4,'0')+'.lino';
    assert.ok(line===PREFIX+name+'"'&&names.length<MAX_PARTS,'noncanonical or excessive ledger archives');
    names.push(name);
  }
  return names;
}
export function readLedger(ledger) {
  const text=readText(ledger);
  let result='';
  const names=parts(text);
  if(names.length)assert.ok(lstatSync(join(dirname(ledger),'self-hosting-ledger-parts')).isDirectory(),'ledger archive directory must not be a symlink');
  for(const name of names) {
    const part=readText(join(dirname(ledger),'self-hosting-ledger-parts',name));
    assert.ok(part.startsWith('self-hosting-ledger-archive\n')&&parts(part).length===0&&count(part)<=LIMIT,'invalid or nested ledger archive');
    result+=part;
  }
  return result+text;
}
function create(file,text) {
  const fd=openSync(file,'wx');
  try {writeFileSync(fd,text);fsyncSync(fd);}finally{closeSync(fd);}
}
export function appendLedger(ledger,record) {
  readLedger(ledger);
  const text=readText(ledger);
  if(count(text+record)<=LIMIT) {writeFileSync(ledger,record,{flag:'a'});return;}
  const offset=text.indexOf('  release\n');
  assert.ok(offset>=0,'ledger has no release record');
  const header=text.slice(0,offset),rows=text.slice(offset).split('  release\n').slice(1);
  const latest=rows.pop(),names=parts(header),chunks=[];
  let chunk='self-hosting-ledger-archive\n';
  for(const row of rows) {
    const record='  release\n'+row;
    assert.ok(count(record)+1<=LIMIT,'single release record exceeds ledger limit');
    if(count(chunk)+count(record)>LIMIT) {chunks.push(chunk);chunk='self-hosting-ledger-archive\n';}
    chunk+=record;
  }
  if(chunk!=='self-hosting-ledger-archive\n')chunks.push(chunk);
  assert.ok(names.length+chunks.length<=MAX_PARTS,'ledger archive bound exhausted');
  let next=header;
  for(let index=0;index<chunks.length;index++)next+=PREFIX+String(names.length+index+1).padStart(4,'0')+'.lino"\n';
  next+='  release\n'+latest+record;
  assert.ok(count(next)<=LIMIT,'release or archive index exceeds ledger limit');
  const directory=join(dirname(ledger),'self-hosting-ledger-parts');
  mkdirSync(directory,{recursive:true});assert.ok(lstatSync(directory).isDirectory(),'ledger archive directory must not be a symlink');
  chunks.forEach((part,index)=>create(join(directory,String(names.length+index+1).padStart(4,'0')+'.lino'),part));
  const pending=ledger.replace(/\.[^/.]+$/u,'')+'.lino.pending';
  create(pending,next);renameSync(pending,ledger);
}
