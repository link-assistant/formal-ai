// Execute the complete, unchanged pinned Python entrypoint and a disposable guard.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {cpSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {templatePythonAvailable} from './template-python-runtime.mjs';

const fixture=fileURLToPath(new URL('./fixtures/python-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}

const freezeClock=String.raw`
import datetime, os, resource, runpy, sys
resource.setrlimit(resource.RLIMIT_AS, (134217728, 134217728))
resource.setrlimit(resource.RLIMIT_CPU, (3, 3))
class FixedDateTime(datetime.datetime):
    @classmethod
    def now(cls, tz=None):
        value = cls.fromisoformat(os.environ['FRAGMENT_FIXTURE_TIME'])
        return value if tz is None else value.replace(tzinfo=datetime.timezone.utc).astimezone(tz)
datetime.datetime = FixedDateTime
sys.argv = sys.argv[1:]
runpy.run_path(sys.argv[0], run_name='__main__')
`;

function fragmentFixture(t,{candidate=false}={}){
  if(!templatePythonAvailable(t))return;
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-python-fragments-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  cpSync(join(fixture,'scripts'),join(cwd,'scripts'),{recursive:true});
  const env={...process.env,GIT_CONFIG_COUNT:'1',GIT_CONFIG_KEY_0:'user.name',GIT_CONFIG_VALUE_0:'Fragment Fixture'};
  for(const name of ['GIT_DIR','GIT_COMMON_DIR','GIT_INDEX_FILE','GIT_WORK_TREE','GITHUB_OUTPUT'])delete env[name];
  const initialized=spawnSync('git',['init','--quiet','--initial-branch=main'],{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:65536});
  assert.equal(initialized.error,undefined,String(initialized.error));
  assert.equal(initialized.status,0,initialized.stderr);
  const script=join(cwd,'scripts/create_manual_changeset.py');
  if(candidate){
    const source=readFileSync(script,'utf8');
    const boundary='    fragment_path.write_text(content)';
    assert.equal(source.split(boundary).length,2,'one manual fragment write boundary');
    writeFileSync(script,source.replace(boundary,'    with fragment_path.open("x") as fragment:\n        fragment.write(content)'));
  }
  const directory=join(cwd,'changelog.d');mkdirSync(directory);
  const invoke=(description,{time='2026-11-17T14:00:00',bump='patch'}={})=>{
    const result=spawnSync('python3',['-B','-c',freezeClock,script,bump,'--no-scriv','--description',description],{
      cwd,env:{...env,FRAGMENT_FIXTURE_TIME:time},encoding:'utf8',timeout:5000,maxBuffer:65536,
    });
    assert.equal(result.error,undefined,String(result.error));
    return result;
  };
  const fragments=()=>Object.fromEntries(readdirSync(directory).sort().map(name=>[name,readFileSync(join(directory,name),'utf8')]));
  return {directory,invoke,fragments};
}

test('Python pinned manual fragment creation silently replaces an earlier same-second change',{timeout:15000},t=>{
  const f=fragmentFixture(t);if(!f)return;
  const first=f.invoke('Preserve the earlier feature.',{bump:'minor'});
  assert.equal(first.status,0,first.stderr);
  const before=f.fragments();assert.equal(Object.keys(before).length,1);
  assert.match(Object.values(before)[0],/### Added\n\n- Preserve the earlier feature\./u);
  const second=f.invoke('Later patch description.');
  assert.equal(second.status,0,second.stderr);
  assert.match(first.stdout,/Created changelog fragment:/u);
  assert.match(second.stdout,/Created changelog fragment:/u);
  assert.deepEqual(Object.keys(f.fragments()),Object.keys(before));
  assert.deepEqual(Object.values(f.fragments()),['### Fixed\n\n- Later patch description.\n']);
});

test('Python candidate rejects a same-second collision and preserves the earlier bytes',{timeout:15000},t=>{
  const f=fragmentFixture(t,{candidate:true});if(!f)return;
  const first=f.invoke('Keep this breaking change.',{bump:'major'});
  assert.equal(first.status,0,first.stderr);
  const before=f.fragments();
  const second=f.invoke('A later patch cannot erase the breaking change.');
  assert.equal(second.status,1,second.stdout);
  assert.match(second.stderr,/FileExistsError/u);
  assert.doesNotMatch(second.stdout,/Created changelog fragment:/u);
  assert.deepEqual(f.fragments(),before);
});

test('Python separate timestamps retain both descriptions before and after the guard',{timeout:15000},t=>{
  for(const candidate of [false,true]){
    const f=fragmentFixture(t,{candidate});if(!f)return;
    assert.equal(f.invoke('First patch.').status,0);
    assert.equal(f.invoke('Second feature.',{time:'2026-11-17T14:00:01',bump:'minor'}).status,0);
    assert.deepEqual(Object.values(f.fragments()),['### Fixed\n\n- First patch.\n','### Added\n\n- Second feature.\n']);
  }
});

test('Python invalid bump remains a fatal nonmutating argument error before and after the guard',{timeout:15000},t=>{
  for(const candidate of [false,true]){
    const f=fragmentFixture(t,{candidate});if(!f)return;
    writeFileSync(join(f.directory,'existing.md'),'Retain the pending release evidence.\n');
    const before=f.fragments();
    const result=f.invoke('Invalid request.',{bump:'invalid'});
    assert.equal(result.status,2,result.stdout);
    assert.match(result.stderr,/invalid choice/u);
    assert.deepEqual(f.fragments(),before);
  }
});
