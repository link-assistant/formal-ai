import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,readFileSync,readdirSync,mkdirSync,rmSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {readLedger,appendLedger} from '../../../scripts/self-hosting-ledger.mjs';
const row=index=>'  release\n    tag "v'+index+'"\n'+Array.from({length:12},(_,field)=>'    field_'+field+' "'+index+'"\n').join('');
function fixture(t,text) {
  const dir=mkdtempSync(join(tmpdir(),'formal-ai-ledger-'));
  t.after(()=>rmSync(dir,{recursive:true,force:true}));
  const file=join(dir,'self-hosting-ledger.lino');writeFileSync(file,text);
  return {dir,file,parts:join(dir,'self-hosting-ledger-parts')};
}
const records=text=>text.split('\n').filter(line=>line==='  release'||line.startsWith('    '));
test('actual oversized release ledger splits losslessly and keeps the latest status in the root',t=>{
  const before='self_hosting_ledger\n  policy "preserved"\n'+Array.from({length:108},(_,i)=>row(i)).join('');
  const f=fixture(t,before);appendLedger(f.file,'');
  assert.deepEqual(records(readLedger(f.file)),records(before));
  assert.match(readFileSync(f.file,'utf8'),/tag "v107"/u);
  assert.ok(!readFileSync(f.file,'utf8').includes('tag "v0"'));
  for(const file of [f.file,...readdirSync(f.parts).map(name=>join(f.parts,name))])assert.ok(readFileSync(file,'utf8').trimEnd().split('\n').length<=1500);
});
test('hundreds of future appends rotate repeatedly without changing prior records',t=>{
  const f=fixture(t,'self_hosting_ledger\n'+row(0));let expected=row(0);
  for(let index=1;index<=330;index++) {const next=row(index);appendLedger(f.file,next);expected+=next;}
  assert.deepEqual(records(readLedger(f.file)),records(expected));
  assert.ok(readdirSync(f.parts).length>=3);
  assert.match(readFileSync(f.file,'utf8'),/tag "v330"/u);
});
test('rotated records retain their explicit inherited defaults in every archive',t=>{
  const shared='  release-defaults\n    metric_version "3"\n    target_percentage_basis_points "389"\n';
  const before='self_hosting_ledger\n'+shared+Array.from({length:108},(_,i)=>row(i)).join('');
  const f=fixture(t,before);appendLedger(f.file,'');
  assert.deepEqual(records(readLedger(f.file).replaceAll(shared,'')),records(before.replace(shared,'')));
  for(const name of readdirSync(f.parts))
    assert(readFileSync(join(f.parts,name),'utf8').startsWith('self-hosting-ledger-archive\n'+shared));
  assert(readFileSync(f.file,'utf8').includes(shared));
});
test('duplicate, nested, traversal, missing and excessive archives fail before writes',t=>{
  for(const directive of ['../outside','0002.lino','0001.lino"\n  archive "self-hosting-ledger-parts/0001.lino','0001.lino']) {
    const text='self_hosting_ledger\n  archive "self-hosting-ledger-parts/'+directive+'"\n'+row(0);
    const f=fixture(t,text);assert.throws(()=>appendLedger(f.file,row(1)));assert.equal(readFileSync(f.file,'utf8'),text);
  }
  const f=fixture(t,'self_hosting_ledger\n  archive "self-hosting-ledger-parts/0001.lino"\n'+row(1));mkdirSync(f.parts);
  writeFileSync(join(f.parts,'0001.lino'),'self-hosting-ledger-archive\n  archive "self-hosting-ledger-parts/0001.lino"\n'+row(0));
  assert.throws(()=>readLedger(f.file),/nested/u);
});
test('interrupted publication retains history and refuses to overwrite uncommitted archive bytes',t=>{
  const before='self_hosting_ledger\n'+Array.from({length:108},(_,i)=>row(i)).join('');
  const f=fixture(t,before);mkdirSync(f.parts);const part=join(f.parts,'0001.lino');writeFileSync(part,'prior interrupted write\n');
  assert.throws(()=>appendLedger(f.file,''),/EEXIST/u);
  assert.equal(readFileSync(f.file,'utf8'),before);assert.equal(readFileSync(part,'utf8'),'prior interrupted write\n');
});
test('symlink archives cannot substitute evidence or redirect writes',t=>{
  const text='self_hosting_ledger\n  archive "self-hosting-ledger-parts/0001.lino"\n'+row(1);
  const f=fixture(t,text);mkdirSync(f.parts);writeFileSync(join(f.dir,'outside'),'self-hosting-ledger-archive\n'+row(0));symlinkSync(join(f.dir,'outside'),join(f.parts,'0001.lino'));
  assert.throws(()=>readLedger(f.file),/invalid ledger/u);
});
test('archive and single-record bounds fail before rewriting the root',t=>{
  const excessive='self_hosting_ledger\n'+Array.from({length:129},(_,i)=>'  archive "self-hosting-ledger-parts/'+String(i+1).padStart(4,'0')+'.lino"\n').join('')+row(0);
  const f=fixture(t,excessive);assert.throws(()=>appendLedger(f.file,row(1)),/excessive/u);assert.equal(readFileSync(f.file,'utf8'),excessive);
  const g=fixture(t,'self_hosting_ledger\n'+row(0));const before=readFileSync(g.file,'utf8');
  assert.throws(()=>appendLedger(g.file,'  release\n'+'    value "large"\n'.repeat(1500)),/limit/u);
  assert.equal(readFileSync(g.file,'utf8'),before);
});
