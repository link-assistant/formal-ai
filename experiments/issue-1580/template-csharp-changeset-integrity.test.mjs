// Execute unchanged pinned helpers and a disposable candidate repair. No .NET
// build, Git, provider API, package publication or network is used.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {cpSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, utimesSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const fixture=fileURLToPath(new URL('./fixtures/csharp-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}

function mergeFixture(t,{collision=false,repaired=false,malformed=false}={}){
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-csharp-changesets-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  cpSync(join(fixture,'scripts'),join(cwd,'scripts'),{recursive:true});
  const directory=join(cwd,'.changeset');mkdirSync(directory);
  const first=collision?'merged-bright-apple.md':'first.md';
  const originals={
    [first]:"---\n'MyPackage': patch\n---\n\nPreserve the earlier patch description.\n",
    'other.md':malformed?'invalid front matter\n':"---\n'MyPackage': major\n---\n\nPreserve the later major description.\n",
  };
  let seconds=1700000000;
  for(const [name,content] of Object.entries(originals)){
    const path=join(directory,name);writeFileSync(path,content);utimesSync(path,seconds,seconds++);
  }
  const script=join(cwd,'scripts/merge-changesets.mjs');
  if(repaired){
    const source=readFileSync(script,'utf8');
    const boundary='writeFileSync(mergedFilePath, mergedContent);';
    assert.equal(source.split(boundary).length,2,'one merged output creation boundary');
    writeFileSync(script,source.replace(boundary,"writeFileSync(mergedFilePath, mergedContent, {flag: 'wx'});"));
  }
  const env={...process.env,GITHUB_ACTIONS:''};
  delete env.NODE_OPTIONS;
  const result=spawnSync(process.execPath,[
    '--max-old-space-size=64','--import','data:text/javascript,Math.random%3D()%3D%3E0',script,
  ],{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
  assert.equal(result.error,undefined,String(result.error));
  return {directory,originals,result,files:readdirSync(directory).sort()};
}

test('C# pinned changeset merger reports success after deleting a colliding output',{timeout:10000},t=>{
  const f=mergeFixture(t,{collision:true});
  assert.equal(f.result.status,0,f.result.stderr);
  assert.match(f.result.stdout,/Changeset merge completed successfully/u);
  assert.deepEqual(f.files,[],'both valid input descriptions and their major bump have vanished');
});

test('C# candidate preserves every input and rejects an output filename collision',{timeout:10000},t=>{
  const f=mergeFixture(t,{collision:true,repaired:true});
  assert.equal(f.result.status,1,f.result.stdout);
  assert.match(f.result.stderr,/EEXIST/u);
  assert.deepEqual(f.files,Object.keys(f.originals).sort());
  for(const [name,content] of Object.entries(f.originals))assert.equal(readFileSync(join(f.directory,name),'utf8'),content);
  assert.doesNotMatch(f.result.stdout,/Changeset merge completed successfully/u);
});

test('C# noncolliding merge retains the highest bump and ordered descriptions before and after repair',{timeout:10000},t=>{
  for(const repaired of [false,true]){
    const f=mergeFixture(t,{repaired});
    assert.equal(f.result.status,0,f.result.stderr);
    assert.deepEqual(f.files,['merged-bright-apple.md']);
    assert.equal(readFileSync(join(f.directory,f.files[0]),'utf8'),"---\n'MyPackage': major\n---\n\nPreserve the earlier patch description.\n\nPreserve the later major description.\n");
  }
});

test('C# malformed input remains a fatal, nonmutating error before and after repair',{timeout:10000},t=>{
  for(const repaired of [false,true]){
    const f=mergeFixture(t,{repaired,malformed:true});
    assert.equal(f.result.status,1,f.result.stdout);
    assert.deepEqual(f.files,Object.keys(f.originals).sort());
    for(const [name,content] of Object.entries(f.originals))assert.equal(readFileSync(join(f.directory,name),'utf8'),content);
  }
});
