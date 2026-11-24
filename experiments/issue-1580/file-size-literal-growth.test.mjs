import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync,renameSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {growingPathsFromNumstat} from '../../scripts/check-file-size.mjs';

test('NUL numstat preserves literal filenames and selects the destination of a growing rename',()=>{
 const paths=['src/line\nbreak.rs','src/tab\tname.rs','src/λ.rs','src/back\\slash.rs'];
 const inventory=paths.map(path=>'1\t0\t'+path+'\0').join('')
  +'3\t3\tunchanged.rs\0'+'1\t2\tshrinking.rs\0'+'-\t-\tbinary.png\0'
  +'2\t1\t\0old\nname.rs\0new\tname.rs\0';
 assert.deepEqual(growingPathsFromNumstat(inventory),[...paths,'new\tname.rs']);
});
test('malformed or incomplete NUL numstat cannot certify an empty growth selection',()=>{
 for(const inventory of ['1\t0\tfile.rs','1\t0\0','1\t0\t\0old.rs\0',
   '1\t0\t\0\0new.rs\0','?\t0\tfile.rs\0','-\t0\tfile.rs\0','1\t0\tfile.rs\0garbage\0','9007199254740992\t0\tfile.rs\0']){
  assert.equal(growingPathsFromNumstat(inventory),null,JSON.stringify(inventory));
 }
 assert.deepEqual(growingPathsFromNumstat(''),[]);
});
test('actual Git growth produces exactly one escaped annotation per literal path',{timeout:20000},()=>{
 const directory=mkdtempSync(join(tmpdir(),'size-warning-git-paths-'));
 const env={...process.env,GIT_CONFIG_GLOBAL:'/dev/null',GIT_CONFIG_NOSYSTEM:'1',FILE_SIZE_WARNING_BASE:''};
 const run=(command,args,extra={})=>{
  const result=spawnSync(command,args,{cwd:directory,env:{...env,...extra},encoding:'utf8',timeout:8000});
  assert.equal(result.error,undefined,String(result.error));assert.equal(result.status,0,result.stdout+result.stderr);
  return result.stdout;
 };
 const git=args=>run('git',['-c','user.name=Size fixture','-c','user.email=size-fixture@example.invalid',...args]);
 const grow=['src/plain.rs','src/line\n::warning::spoof.rs','src/tab\tname.rs','src/λ.rs','src/percent%,colon:.rs'];
 const text=count=>Array.from({length:count},(_,i)=>'// line '+i+'\n').join('');
 try{
  mkdirSync(join(directory,'src'));git(['init','-q']);
  for(const name of [...grow,'src/shrinking.rs','src/old-name.rs'])writeFileSync(join(directory,name),text(name.endsWith('shrinking.rs')?902:900));
  writeFileSync(join(directory,'src/unchanged.rs'),text(901));
  git(['add','--all']);git(['commit','-qm','baseline']);const base=git(['rev-parse','HEAD']).trim();
  for(const name of grow)writeFileSync(join(directory,name),text(901));
  writeFileSync(join(directory,'src/shrinking.rs'),text(901));
  renameSync(join(directory,'src/old-name.rs'),join(directory,'src/renamed\nfile.rs'));
  writeFileSync(join(directory,'src/renamed\nfile.rs'),text(901));
  git(['add','--all']);git(['commit','-qm','growth']);
  const output=run(process.execPath,[resolve('scripts/check-file-size.mjs')],{FILE_SIZE_WARNING_BASE:base});
  const annotations=output.split('\n').filter(line=>line.startsWith('::warning '));
  const escape=value=>value.replaceAll('%','%25').replaceAll('\r','%0D').replaceAll('\n','%0A').replaceAll(':','%3A').replaceAll(',','%2C');
  const expected=[...grow,'src/renamed\nfile.rs'].map(name=>'::warning file='+escape(name)+'::');
  assert.equal(annotations.length,expected.length,output);
  for(const prefix of expected)assert.equal(annotations.filter(line=>line.startsWith(prefix)).length,1,prefix);
  assert(!output.includes('\n::warning::spoof'),output);
 }finally{rmSync(directory,{recursive:true,force:true});}
});

test('an incomplete Git growth inventory reports every warning with default-off diagnostics',{timeout:10000},()=>{
 const directory=mkdtempSync(join(tmpdir(),'size-warning-partial-inventory-'));
 try{
  const bin=join(directory,'bin');mkdirSync(bin);mkdirSync(join(directory,'src'));
  writeFileSync(join(directory,'src/warning.rs'),'// line\n'.repeat(901));
  const body="process.stdout.write(process.argv[2]==='ls-files'?'src/warning.rs\\0':'1\\t0\\tpartial.rs\\0garbage\\0');";
  writeFileSync(join(bin,'git'),'#!'+process.execPath+'\n'+body,{mode:0o755});
  for(const debug of ['0','1']){
   const result=spawnSync(process.execPath,[resolve('scripts/check-file-size.mjs')],{
    cwd:directory,encoding:'utf8',timeout:4000,env:{...process.env,PATH:bin+':'+process.env.PATH,
     FILE_SIZE_WARNING_BASE:'fixture-base',FORMAL_AI_FILE_SIZE_DEBUG:debug}});
   assert.equal(result.error,undefined);assert.equal(result.status,0,result.stdout+result.stderr);
   assert.equal(result.stdout.split('\n').filter(line=>line.startsWith('::warning file=src/warning.rs::')).length,1);
   assert.equal(result.stderr,debug==='0'?'':'Incomplete NUL warning-growth inventory; reporting every size warning.\n');
  }
 }finally{rmSync(directory,{recursive:true,force:true});}
});
