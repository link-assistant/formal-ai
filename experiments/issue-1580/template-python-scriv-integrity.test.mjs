// Actual current Python selector; the finite Scriv producer is mocked.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {templatePythonAvailable} from './template-python-runtime.mjs';
const base=fileURLToPath(new URL('./fixtures/python-current-changeset/',import.meta.url));
const original=readFileSync(join(base,'scripts/create_manual_changeset.py'),'utf8');
const identity=JSON.parse(readFileSync(join(base,'source.json'),'utf8')).files[0];
assert.equal(createHash('sha256').update(original).digest('hex'),identity.sha256);
assert.equal(createHash('sha1').update(`blob ${Buffer.byteLength(original)}\0`).update(original).digest('hex'),identity.gitBlob);
const first=original.indexOf('def create_with_scriv(');
const end=original.indexOf('\ndef create_manual_fragment(',first);
const before=original.slice(first,end);
assert.equal(before.split('        result = subprocess.run(').length,2);
assert.equal(before.split('if f.name != "README.md"').length,2);
const after=before.replace('        result = subprocess.run(',
  '        changelog_dir = Path("changelog.d")\n        previous = set(changelog_dir.glob("*.md"))\n        result = subprocess.run(')
  .replace('if f.name != "README.md"','if f not in previous and f.name != "README.md"')
  .replace('            if fragments:', '            if len(fragments) != 1:\n                return 1\n            if fragments:');
const candidate=original.slice(0,first)+after+original.slice(end);
const probe=String.raw`
import contextlib, importlib.util, io, json, os, resource, subprocess, sys
from pathlib import Path
from unittest.mock import patch
resource.setrlimit(resource.RLIMIT_AS, (134217728,134217728))
resource.setrlimit(resource.RLIMIT_CPU, (3,3))
spec=importlib.util.spec_from_file_location('creator',sys.argv[1])
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
directory=Path('changelog.d');directory.mkdir()
mode=sys.argv[2]
if mode!='empty':
    old=directory/'earlier.md';old.write_text('### Added\n\n- Earlier feature.\n')
    os.utime(old,(1900000000 if mode=='future' else 1600000000,)*2)
def scriv(args,**kwargs):
    assert args==['scriv','create']
    names=[] if mode=='none' else ['new.md','concurrent.md'] if mode=='multiple' else ['new.md']
    for name in names:
        fragment=directory/name;fragment.write_text('Scriv draft '+name+'\n');os.utime(fragment,(1700000000,)*2)
    return subprocess.CompletedProcess(args,0,'Created new.md\n','')
log=io.StringIO()
with patch.object(module.subprocess,'run',scriv),contextlib.redirect_stdout(log):
    status=module.create_with_scriv('patch','Later patch.')
print(json.dumps({'status':status,'fragments':{p.name:p.read_text() for p in sorted(directory.glob('*.md'))},'log':log.getvalue()}))
`;
function observe(t,mode,repaired){
  if(!templatePythonAvailable(t))return;
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-python-scriv-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  const script=join(cwd,'creator.py');writeFileSync(script,repaired?candidate:original);
  const result=spawnSync('python3',['-B','-c',probe,script,mode],{cwd,encoding:'utf8',timeout:5000,maxBuffer:65536});
  assert.equal(result.error,undefined,String(result.error));
  assert.equal(result.status,0,result.stderr);
  return JSON.parse(result.stdout);
}
test('Python Scriv overwrites an unrelated future-dated fragment',{timeout:15000},t=>{
  const old=observe(t,'future',false);if(!old)return;
  assert.equal(old.status,0);
  assert.equal(old.fragments['earlier.md'],'### Fixed\n\n- Later patch.\n');
  assert.equal(old.fragments['new.md'],'Scriv draft new.md\n');
  const repaired=observe(t,'future',true);
  assert.equal(repaired.status,0);
  assert.equal(repaired.fragments['earlier.md'],'### Added\n\n- Earlier feature.\n');
  assert.equal(repaired.fragments['new.md'],'### Fixed\n\n- Later patch.\n');
});
test('Python Scriv claims success and overwrites history when no file is created',{timeout:15000},t=>{
  const old=observe(t,'none',false);if(!old)return;
  assert.equal(old.status,0);
  assert.equal(old.fragments['earlier.md'],'### Fixed\n\n- Later patch.\n');
  const repaired=observe(t,'none',true);
  assert.equal(repaired.status,1);
  assert.deepEqual(repaired.fragments,{'earlier.md':'### Added\n\n- Earlier feature.\n'});
});
test('Python Scriv candidate refuses ambiguous concurrent fragment selection',{timeout:15000},t=>{
  const repaired=observe(t,'multiple',true);if(!repaired)return;
  assert.equal(repaired.status,1);
  assert.deepEqual(repaired.fragments,{'concurrent.md':'Scriv draft concurrent.md\n','earlier.md':'### Added\n\n- Earlier feature.\n','new.md':'Scriv draft new.md\n'});
});
test('Python Scriv retains ordinary single-new-file behavior',{timeout:15000},t=>{
  for(const mode of ['empty','older'])for(const repaired of [false,true]){
    const result=observe(t,mode,repaired);if(!result)return;
    assert.equal(result.status,0);
    assert.equal(result.fragments['new.md'],'### Fixed\n\n- Later patch.\n');
    if(mode==='older')assert.equal(result.fragments['earlier.md'],'### Added\n\n- Earlier feature.\n');
  }
});
