// Exercise the pinned PHP scripts, without package installation or remote Git IO.
// Disposable candidates do not claim that the upstream template has been repaired.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {cpSync,mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {TextDecoder} from 'node:util';
import test from 'node:test';

const fixture=fileURLToPath(new URL('./fixtures/php-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const raw=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${raw.length}\0`).update(raw).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(raw).digest('hex'),item.sha256);
}

function candidateVersion(source){
  const start=source.indexOf('    $data = json_decode($json, true);');
  const end=source.indexOf('\n}\n',start);
  assert.ok(start>0&&end>start);
  source=source.slice(0,start)+String.raw`    $data = json_decode($json, false, 512, JSON_THROW_ON_ERROR);
    if (!$data instanceof \stdClass) {
        throw new \RuntimeException('composer.json must be an object.');
    }
    if (!property_exists($data, 'version')) {
        return null;
    }
    if (!is_string($data->version)) {
        throw new \RuntimeException('version must be a string.');
    }
    return $data->version;`+source.slice(end);
  const fetch="Process::run(['git', 'fetch', '--no-tags', '--depth=1', 'origin', $baseRef], $root);";
  assert.ok(source.includes(fetch));
  source=source.replace(fetch,fetch.replace('Process::run','Process::mustRun'));
  const before=source.indexOf('// Read composer.json as it exists on the base branch.');
  const after=source.indexOf('\n$baseVersion =',before);
  assert.ok(before>0&&after>before);
  source=source.slice(0,before)+String.raw`// Resolve the fetched commit and prove file absence independently of git show.
  $baseCommit = trim(Process::mustRun(['git', 'rev-parse', '--verify', "origin/{$baseRef}^{commit}"], $root)->stdout);
  $basePath = Process::mustRun(['git', 'ls-tree', '--name-only', $baseCommit, '--', 'composer.json'], $root);
  if (trim($basePath->stdout) === '') {
      echo "composer.json is absent from the verified base commit; bootstrap allowed.\n";
      exit(0);
  }
  $baseFile = Process::mustRun(['git', 'show', "{$baseCommit}:composer.json"], $root);
`+source.slice(after);
  const comparison='$baseVersion !== null && $headVersion !== null && $baseVersion !== $headVersion';
  assert.ok(source.includes(comparison));
  return source.replace(comparison,'$baseVersion !== $headVersion');
}

function versionGuard(t,{kind,repaired=false}){
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-php-version-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  cpSync(join(fixture,'scripts'),join(cwd,'scripts'),{recursive:true});
  const env={...process.env};
  for(const name of Object.keys(env))if(name.startsWith('GIT_'))delete env[name];
  Object.assign(env,{GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_TERMINAL_PROMPT:'0',
    GIT_AUTHOR_NAME:'Fixture',GIT_AUTHOR_EMAIL:'fixture@example.invalid',
    GIT_COMMITTER_NAME:'Fixture',GIT_COMMITTER_EMAIL:'fixture@example.invalid',GITHUB_BASE_REF:'main'});
  const git=(...args)=>{
    const result=spawnSync('git',args,{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
    assert.equal(result.error,undefined,String(result.error));
    assert.equal(result.status,0,result.stderr);
    return result.stdout.trim();
  };
  const manifest=join(cwd,'composer.json');
  const original={name:'fixture/package',version:'1.2.3'};
  const withoutVersion={name:original.name};
  if(kind!=='new-file')writeFileSync(manifest,kind==='base-invalid'?'{invalid\n':JSON.stringify(
    kind==='added-version'?withoutVersion:original)+'\n');
  writeFileSync(join(cwd,'marker.txt'),'Finite local fixture\n');
  git('init','--quiet','--initial-branch=main');
  git('config','commit.gpgsign','false');git('config','core.hooksPath',join(cwd,'no-hooks'));
  git('add','.');git('commit','--quiet','-m','Version guard fixture');
  const missing=join(cwd,'missing-local-origin');
  git('remote','add','origin',kind==='unavailable-base'||kind==='stale-base'?missing:cwd);
  if(kind==='stale-base')git('update-ref','refs/remotes/origin/main','HEAD');
  const changed=['changed','unavailable-base','stale-base','added-version','new-file'].includes(kind);
  let head=changed?{...original,version:'9.9.9'}:original;
  if(kind==='removed')head=withoutVersion;
  if(kind==='null-version')head={...original,version:null};
  if(kind==='numeric-version')head={...original,version:123};
  writeFileSync(manifest,kind==='invalid'?'{invalid\n':JSON.stringify(head)+'\n');
  if(repaired){
    const path=join(cwd,'scripts/check-version-modification.php');
    writeFileSync(path,candidateVersion(readFileSync(path,'utf8')));
  }
  const result=spawnSync('php',['-d','memory_limit=32M','scripts/check-version-modification.php'],
    {cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
  assert.equal(result.error,undefined,String(result.error));
  return result;
}

const missingEvidence=['removed','invalid','base-invalid','unavailable-base','null-version','numeric-version','added-version'];
for(const kind of missingEvidence){
  test(`PHP version guard accepts ${kind} evidence; the disposable candidate rejects it`,{timeout:15000},t=>{
    const original=versionGuard(t,{kind});
    assert.equal(original.status,0,original.stderr);
    assert.match(original.stdout,/unchanged|new file/u);
    const repaired=versionGuard(t,{kind,repaired:true});
    assert.notEqual(repaired.status,0,repaired.stdout);
  });
}

test('PHP version guard candidates preserve unchanged, genuine first-file and changed-version controls',
  {timeout:20000},t=>{
    for(const repaired of [false,true])for(const [kind,status] of [['unchanged',0],['new-file',0],['changed',1]]){
      const result=versionGuard(t,{kind,repaired});
      assert.equal(result.status,status,`${kind}, candidate=${repaired}: ${result.stderr}`);
    }
  });

test('a stale base ref does not excuse a failed fetch in the candidate',{timeout:15000},t=>{
  const original=versionGuard(t,{kind:'stale-base'});
  assert.equal(original.status,1);
  assert.match(original.stdout+original.stderr,/changes the "version"/u);
  const repaired=versionGuard(t,{kind:'stale-base',repaired:true});
  assert.notEqual(repaired.status,0);
  assert.match(repaired.stderr,/Command failed.*git fetch/us);
});

const notice='\n\n---\n\n_Release notes truncated. See the full '+
  '[CHANGELOG.md](https://github.com/fixture/repo/blob/v1/CHANGELOG.md)._\n';
const decode=new TextDecoder('utf-8',{fatal:true});
const releaseSource=readFileSync(join(fixture,'scripts/src/ReleaseNotes.php'),'utf8');

function candidateNotes(source){
  const before=source.indexOf('        $budget = $max - \\strlen($notice);');
  const after=source.indexOf('\n        return $truncated . $notice;',before);
  assert.ok(before>0&&after>before);
  return source.slice(0,before)+String.raw`        if (\strlen($notice) > $max) {
            $notice = substr($notice, 0, max(0, $max));
        }
        $budget = max(0, $max - \strlen($notice));
        $truncated = substr($body, 0, $budget);
        while ($truncated !== '' && preg_match('//u', $truncated) !== 1) {
            $truncated = substr($truncated, 0, -1);
        }
`+source.slice(after);
}

function notes(t,{repaired=false,body,max}){
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-php-notes-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  const path=join(cwd,'ReleaseNotes.php');
  writeFileSync(path,repaired?candidateNotes(releaseSource):releaseSource);
  const expression=String.raw`require $argv[1];
    echo \LinkFoundation\Template\Pipeline\ReleaseNotes::capBytes(
      file_get_contents('php://stdin'), 'fixture/repo', 'v1', (int) $argv[2]);`;
  const result=spawnSync('php',['-d','memory_limit=32M','-r',expression,path,
    String(max)],{input:Buffer.from(body),timeout:5000,maxBuffer:131072});
  assert.equal(result.error,undefined,String(result.error));
  assert.equal(result.status,0,result.stderr.toString());
  return result.stdout;
}

for(const character of ['é','中','🙂']){
  test(`PHP release notes split ${character} even at complete UTF-8 boundaries; the candidate preserves validity`,
    {timeout:15000},t=>{
      const prefix='x'.repeat(200);
      const body=prefix+character+'y'.repeat(400);
      for(let cut=1;cut<=Buffer.byteLength(character);cut++){
        const max=Buffer.byteLength(notice)+prefix.length+cut;
        const original=notes(t,{body,max});
        assert.ok(original.length<=max);
        assert.throws(()=>decode.decode(original),TypeError,`original cut ${cut}`);
        const repaired=notes(t,{body,max,repaired:true});
        assert.ok(repaired.length<=max);
        const text=decode.decode(repaired);
        assert.equal(text,prefix+(cut===Buffer.byteLength(character)?character:'')+notice);
      }
    });
}

test('PHP release notes retain ASCII and untruncated Unicode controls',{timeout:15000},t=>{
  for(const repaired of [false,true]){
    assert.equal(decode.decode(notes(t,{body:'α🙂中',max:1000,repaired})),'α🙂中');
    const body='x'.repeat(1000),max=Buffer.byteLength(notice)+200;
    assert.equal(decode.decode(notes(t,{body,max,repaired})),'x'.repeat(200)+notice);
  }
});

test('PHP release notes exceed a cap smaller than their notice; the candidate honors the byte cap',
  {timeout:15000},t=>{
    for(const max of [0,1,7,Buffer.byteLength(notice)-1]){
      const body='x'.repeat(1000);
      assert.ok(notes(t,{body,max}).length>max);
      const repaired=notes(t,{body,max,repaired:true});
      assert.ok(repaired.length<=max);
      assert.doesNotThrow(()=>decode.decode(repaired));
    }
  });

test('the default 120000-byte release cap also produces invalid UTF-8 before the candidate',
  {timeout:15000},t=>{
    const max=120000,budget=max-Buffer.byteLength(notice);
    const prefix='x'.repeat(budget-Buffer.byteLength('é'));
    const body=prefix+'é'+'z'.repeat(1000);
    const original=notes(t,{body,max});
    assert.ok(original.length<=max);
    assert.throws(()=>decode.decode(original),TypeError);
    const repaired=notes(t,{body,max,repaired:true});
    assert.equal(repaired.length,max);
    assert.equal(decode.decode(repaired),prefix+'é'+notice);
  });
