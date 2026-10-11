import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {parseIncludeSite,includeSites,classify,run} from '../../../scripts/mirror-package-data.mjs';
test('original Rust literal and self-read parser vectors',()=>{
 assert.deepEqual(parseIncludeSite('"../../embedded/data/seed/x.lino"'),{literal:'../../embedded/data/seed/x.lino',selfRead:false});
 assert.equal(parseIncludeSite('concat!(env!("CARGO_MANIFEST_DIR"), "/", file!())').selfRead,true);
 assert.throws(()=>parseIncludeSite('concat!(env!("CARGO_MANIFEST_DIR"), "/../data/x")'));
});
test('original Rust include order and package-boundary vectors',()=>{
 const sites=includeSites('const A: &str = include_str!("../embedded/data/a.lino");\nconst B: &str = include_str!(\n  "../embedded/js/b.js"\n);\n');
 assert.equal(sites.length,2);assert.equal(sites[0][0],1);assert.equal(sites[1][1].literal,'../embedded/js/b.js');
 assert.deepEqual(classify('/repo/rust/src/x.rs','/repo/rust','/repo'),{kind:'in-package'});
 assert.deepEqual(classify('/repo/rust/embedded/data/seed/x.lino','/repo/rust','/repo'),{kind:'mirror',repoRelative:'data/seed/x.lino'});
 for(const target of ['/repo/data/seed/x.lino','/repo/rust/../data/x.lino'])assert.throws(()=>classify(target,'/repo/rust','/repo'));
 assert.throws(()=>includeSites('include_str!("x"'),/unterminated/);
});
test('generic demand producer discovers additions preserves bytes and removes obsolete mirrors',()=>{
 const root=fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()),'mirror-twin-'));
 const put=(relative,bytes)=>{const file=path.join(root,relative);fs.mkdirSync(path.dirname(file),{recursive:true});fs.writeFileSync(file,bytes);};
 try{
  put('rust/src/lib.rs','const A:&str=include_str!("../embedded/data/a.lino");\nconst SELF:&str=include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/",file!()));');
  const bytes=Buffer.from([0xef,0xbb,0xbf,0x61,0x0d,0x0a,0xff]);put('data/a.lino',bytes);put('rust/embedded/data/obsolete.lino','old');
  assert.throws(()=>run(['--check'],root),/stale/);assert.match(run(['--write'],root),/1 refreshed/);
  assert.deepEqual(fs.readFileSync(path.join(root,'rust/embedded/data/a.lino')),bytes);assert.equal(fs.existsSync(path.join(root,'rust/embedded/data/obsolete.lino')),false);assert.match(run(['--check'],root),/all byte-equal/);
  put('rust/src/new.rs','const B:&str=include_str!("../embedded/docs/new.txt");');put('docs/new.txt','new\n{body}');
  assert.throws(()=>run(['--check'],root),/docs[/\\]new.txt/);run(['--write'],root);assert.equal(fs.readFileSync(path.join(root,'rust/embedded/docs/new.txt'),'utf8'),'new\n{body}');
  put('rust/src/new.rs','const B:&str=include_str!(concat!(env!("OTHER"),"/outside"));');assert.throws(()=>run(['--write'],root),/neither a plain literal/);
 }finally{fs.rmSync(root,{recursive:true,force:true});}
});
