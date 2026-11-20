import assert from 'node:assert/strict';
import test from 'node:test';
import {gzipSync} from 'node:zlib';
import {mkdtempSync,writeFileSync,mkdirSync,rmSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {Readable} from 'node:stream';
import {checkFiles,filenames,scannerModule,createScanner} from '../../scripts/check-secrets-files.mjs';
import {readTarEntries,scanCompressed} from '../../scripts/lib/compressed-secrets.mjs';
function entry(name,content,type='0') {
  const data=Buffer.from(content),header=Buffer.alloc(512);
  header.write(name);header.write(data.length.toString(8).padStart(11,'0')+'\0',124);
  header.fill(32,148,156);header.write(type,156);
  header.write(header.reduce((sum,byte)=>sum+byte,0).toString(8).padStart(6,'0')+'\0 ',148);
  return Buffer.concat([header,data,Buffer.alloc((512-data.length%512)%512)]);
}
const tar=(...entries)=>Buffer.concat([...entries,Buffer.alloc(1024)]);
const credential='gh'+'p_'+'A'.repeat(36);
const reject=(_name,bytes)=>{if(bytes.includes(credential))throw Error('synthetic credential detected');};
for(const extension of ['log.gz','json.gz','tar.gz','tgz'])test('decompressed credential in '+extension+' is inspected',()=>{
  const content=/tar.gz|tgz/u.test(extension)?tar(entry('evidence.log',credential)):Buffer.from(credential);
  assert.throws(()=>scanCompressed('evidence.'+extension,gzipSync(content),reject),/synthetic credential/u);
});
test('nested gzip, GNU long names and PAX paths preserve content inspection',()=>{
  for(const metadata of [entry('././@LongLink','nested.log.gz\0','L'),entry('pax','22 path=nested.log.gz\n','x')]) {
    assert.throws(()=>scanCompressed('snapshot.tgz',gzipSync(tar(metadata,entry('short',gzipSync(credential)))),reject),/synthetic credential/u);
  }
  assert.throws(()=>scanCompressed('nested.gz.gz',gzipSync(gzipSync(credential)),reject),/synthetic credential/u);
});
test('tar names and link metadata are scanned even with empty content',()=>{
  for(const header of [entry(credential,'ordinary'),entry(credential,'','2')]) {
    assert.throws(()=>scanCompressed('names.tgz',gzipSync(tar(header)),reject),/synthetic credential/u);
  }
});
test('traversal member names are diagnostic only and never extracted',()=>{
  const seen=[];
  scanCompressed('snapshot.tgz',gzipSync(tar(entry('clean.txt','ok'),entry('../../outside.log','evidence'))),(name,bytes)=>seen.push([name,bytes.toString()]));
  assert.deepEqual(seen.filter(([name])=>!name.endsWith('.metadata.txt')),[['snapshot.tgz/clean.txt','ok'],['snapshot.tgz/../../outside.log','evidence']]);
});
test('corrupt gzip, invalid tar, unsupported metadata, truncation and orphan metadata fail closed',()=>{
  assert.throws(()=>scanCompressed('bad.gz',Buffer.from('not gzip'),()=>{}));
  assert.throws(()=>scanCompressed('bad.tgz',gzipSync('not tar'),()=>{}),/Truncated/u);
  const corrupt=tar(entry('ok','data'));corrupt[0]^=1;
  assert.throws(()=>readTarEntries(corrupt),/checksum/u);
  assert.throws(()=>readTarEntries(tar(entry('sparse','','S'))),/Unsupported/u);
  assert.throws(()=>readTarEntries(tar(entry('pax','10 size=1\n','x'),entry('ok','data'))),/Unsupported|Malformed/u);
  assert.throws(()=>readTarEntries(tar(entry('long','missing\0','L'))),/orphan/u);
  assert.throws(()=>readTarEntries(entry('ok','data')),/Truncated/u);
});
test('small finite controls enforce inflation, aggregate nesting and entry-count bounds',()=>{
  assert.throws(()=>scanCompressed('large.gz',gzipSync(Buffer.alloc(4096)),()=>{},{remaining:1024}));
  assert.throws(()=>scanCompressed('nested.gz',gzipSync('ok'),()=>{},{remaining:1024},4),/nesting/u);
  const nested=gzipSync(Buffer.alloc(64));
  const outer=tar(entry('nested.gz',nested));
  assert.throws(()=>scanCompressed('nested.tgz',gzipSync(outer),()=>{},{remaining:outer.length+32}));
  assert.throws(()=>readTarEntries(tar(...Array.from({length:10000},()=>entry('empty','')))),/count limit/u);
});
test('NUL inventory preserves spaces, quotes, UTF-8 and chunk boundaries without shell interpolation',async()=>{
  const names=["a file with 'quotes' 世界.txt",'-leading-dash.gz'];
  const bytes=Buffer.from(names.join('\0')+'\0'),cut=bytes.indexOf(Buffer.from('世界'))+1;
  const actual=[];for await(const name of filenames(Readable.from([bytes.subarray(0,cut),bytes.subarray(cut)])))actual.push(name);
  assert.deepEqual(actual,names);
  for(const input of [Buffer.alloc(0),Buffer.from('missing terminator'),Buffer.from('\0'),Buffer.alloc(32769,65)]) {
    await assert.rejects(async()=>{for await(const _ of filenames(Readable.from([input]))){}},/inventory|filename/u);
  }
});
test('one engine scans actual files and decompressed members sequentially and propagates findings',async t=>{
  const directory=mkdtempSync(join(tmpdir(),'secret-engine-fixture-'));t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const files=[join(directory,'a file.txt'),join(directory,'evidence.log.gz'),join(directory,'empty.gz')];
  writeFileSync(files[0],'ordinary');writeFileSync(files[1],gzipSync(credential));writeFileSync(files[2],gzipSync(''));
  let active=0,peak=0;const plain=[],content=[],output=[];
  const scan=async value=>{peak=Math.max(peak,++active);await Promise.resolve();active--;return {ok:!value.includes(credential),output:value.includes(credential)?'masked finding':''};};
  const engine={executeOnFiles:async({filePathList})=>{plain.push(filePathList);return scan('ordinary');},
    executeOnContent:async value=>{content.push(value);return scan(value.content);}};
  const result=await checkFiles(files,engine,value=>output.push(value));
  assert.deepEqual(result,{ok:false,files:3,compressedFiles:2});assert.equal(peak,1);
  assert.deepEqual(plain,[[files[0]]]);assert.equal(content.length,1);assert.equal(content[0].content,credential);
  assert.equal(output.join(''),'masked finding\n');
  await assert.rejects(()=>checkFiles([],engine),/Empty/u);
  writeFileSync(files[1],'not gzip');await assert.rejects(()=>checkFiles([files[1]],engine));
  await assert.rejects(()=>checkFiles([join(directory,'missing')],engine),/ENOENT/u);
});
test('engine identity and default-off profiling retain masking and rule execution',async t=>{
  const directory=mkdtempSync(join(tmpdir(),'secret-identity-'));t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const bin=join(directory,'bin');mkdirSync(bin);
  for(const name of ['secretlint','@secretlint/node','@secretlint/profiler','@secretlint/secretlint-rule-preset-recommend']) {
    const target=join(directory,'node_modules',name);mkdirSync(target,{recursive:true});
    writeFileSync(join(target,'package.json'),JSON.stringify({name,version:'13.0.5',main:'index.js',type:'module'}));
    writeFileSync(join(target,'index.js'),'');
  }
  const executable=join(directory,'node_modules/secretlint/bin/secretlint.js');mkdirSync(join(executable,'..'),{recursive:true});
  writeFileSync(executable,'#!/usr/bin/env node\n',{mode:0o755});symlinkSync(executable,join(bin,'secretlint'));
  writeFileSync(join(directory,'node_modules/@secretlint/profiler/index.js'),
    'export const secretLintProfiler={isEnabled:true,setEnabled(value){this.isEnabled=value;}};');
  writeFileSync(join(directory,'node_modules/@secretlint/node/index.js'),
    "import {secretLintProfiler} from '@secretlint/profiler';export const createEngine=config=>({config,profiling:secretLintProfiler.isEnabled});");
  assert(scannerModule('13.0.5',bin).endsWith('/@secretlint/node/index.js'));
  const scanner=await createScanner('13.0.5',bin);
  assert.equal(scanner.profiling,false);
  assert.deepEqual(scanner.config,{cwd:process.cwd(),formatter:'stylish',color:false,maskSecrets:true});
  const profilerManifest=join(directory,'node_modules/@secretlint/profiler/package.json');
  writeFileSync(profilerManifest,JSON.stringify({name:'@secretlint/profiler',version:'13.0.4',type:'module'}));
  await assert.rejects(()=>createScanner('13.0.5',bin),/profiler version mismatch/u);
  assert.throws(()=>scannerModule('13.0.4',bin),/version mismatch/u);
  assert.throws(()=>scannerModule('13.0.5',join(directory,'absent')),/unavailable/u);
});
