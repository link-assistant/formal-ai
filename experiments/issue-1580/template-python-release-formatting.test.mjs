// Execute the complete pinned formatter with an offline GitHub CLI boundary.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {templatePythonAvailable} from './template-python-runtime.mjs';

const fixture=fileURLToPath(new URL('./fixtures/python-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
const names=['scripts/format_release_notes.py','scripts/release_naming.py'];
const sources=new Map(names.map(name=>{
  const item=provenance.files.find(row=>row.path===name);assert(item);
  const bytes=readFileSync(join(fixture,name));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
  return [name,bytes.toString('utf8')];
}));
const notes=String.raw`Preserve the café examples.

~~~python
record = {"message": "hello\nworld", "quote": "\"", "path": "C:\repo"}
~~~`;
const runBounded=String.raw`
import resource, runpy, sys
resource.setrlimit(resource.RLIMIT_AS, (134217728, 134217728))
resource.setrlimit(resource.RLIMIT_CPU, (3, 3))
sys.argv = sys.argv[1:]
runpy.run_path(sys.argv[0], run_name='__main__')
`;

function formatterFixture(t,{candidate=false}={}){
  if(!templatePythonAvailable(t))return;
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-python-release-notes-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  const bin=join(cwd,'bin');mkdirSync(bin);mkdirSync(join(cwd,'scripts'));
  for(const [name,original] of sources){
    let source=original;
    if(candidate&&name==='scripts/format_release_notes.py'){
      const boundary=String.raw`    # Fix escaped newlines and special characters
    cleaned_body = cleaned_body.replace("\\n", "\n")
    cleaned_body = cleaned_body.replace("\\r", "")
    cleaned_body = cleaned_body.replace('\\"', '"')
`;
      assert.equal(source.split(boundary).length,2,'one escape-rewriting boundary');
      source=source.replace(boundary,'    # The API body is already decoded; preserve literal examples.\n');
    }
    writeFileSync(join(cwd,name),source);
  }
  writeFileSync(join(bin,'gh'),String.raw`#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
args = sys.argv[1:]
with Path(os.environ['NOTES_FIXTURE_CALLS']).open('a') as log:
    log.write(json.dumps(args) + '\n')
endpoint = 'repos/example/release/releases/42'
if args == ['api', endpoint, '--jq', '.body']:
    if os.environ['NOTES_FIXTURE_FAILURE'] == 'get':
        print('fixture read refused', file=sys.stderr)
        sys.exit(1)
    print(Path(os.environ['NOTES_FIXTURE_INPUT']).read_text())
elif len(args) == 6 and args[:5] == ['api', '-X', 'PATCH', endpoint, '-f'] and args[5].startswith('body='):
    if os.environ['NOTES_FIXTURE_FAILURE'] == 'patch':
        print('fixture write refused', file=sys.stderr)
        sys.exit(1)
    Path(os.environ['NOTES_FIXTURE_OUTPUT']).write_text(args[5][5:])
    print('{}')
else:
    print('unexpected offline GitHub invocation', file=sys.stderr)
    sys.exit(2)
`,{mode:0o755});
  const input=join(cwd,'input'),output=join(cwd,'output'),calls=join(cwd,'calls');
  const invoke=(body,{failure=''}={})=>{
    writeFileSync(input,body);writeFileSync(output,'');writeFileSync(calls,'');
    const result=spawnSync('python3',['-B','-c',runBounded,join(cwd,names[0]),
      '--release-id','42','--version','1.2.3','--repository','example/release','--package-name','example-package'],{
      cwd,env:{...process.env,PATH:bin+':'+process.env.PATH,NOTES_FIXTURE_INPUT:input,
        NOTES_FIXTURE_OUTPUT:output,NOTES_FIXTURE_CALLS:calls,NOTES_FIXTURE_FAILURE:failure},
      encoding:'utf8',timeout:5000,maxBuffer:65536,
    });
    assert.ifError(result.error);assert.equal(result.signal,null);
    return {...result,body:readFileSync(output,'utf8'),calls:readFileSync(calls,'utf8').split('\n').filter(Boolean).map(line=>JSON.parse(line))};
  };
  return {invoke};
}

test('Python pinned release formatter rewrites literal code escapes and publishes the corrupted body',{timeout:10000},t=>{
  const f=formatterFixture(t);if(!f)return;
  const result=f.invoke(notes);assert.equal(result.status,0,result.stderr);
  assert.match(result.stdout,/Release notes updated successfully/u);
  assert.equal(result.calls.length,2);
  assert.match(result.body,/hello\nworld/u);
  assert.match(result.body,/"quote": """/u);
  assert.match(result.body,/"path": "C:epo"/u);
  assert.ok(!result.body.endsWith(notes));
});

test('Python candidate preserves literal escapes and Unicode while adding the version badge',{timeout:10000},t=>{
  const f=formatterFixture(t,{candidate:true});if(!f)return;
  const result=f.invoke(notes);assert.equal(result.status,0,result.stderr);
  assert.match(result.body,/^\[!\[PyPI version\]/u);
  assert.ok(result.body.endsWith('\n\n'+notes),JSON.stringify(result.body));
  assert.equal(result.calls.length,2);
});

test('Python release formatting retains an already formatted body without a second write',{timeout:10000},t=>{
  for(const candidate of [false,true]){
    const f=formatterFixture(t,{candidate});if(!f)return;
    const body='[![PyPI](https://img.shields.io/badge/pypi-1.2.3-blue.svg)](https://pypi.org/project/example-package/1.2.3/)\n\n'+notes;
    const result=f.invoke(body);assert.equal(result.status,0,result.stderr);
    assert.equal(result.calls.length,1);assert.equal(result.body,'');
    assert.match(result.stdout,/No changes needed/u);
  }
});

test('Python release formatting keeps read and write failures fatal before and after the candidate',{timeout:15000},t=>{
  for(const candidate of [false,true]){
    const f=formatterFixture(t,{candidate});if(!f)return;
    for(const failure of ['get','patch']){
      const result=f.invoke(notes,{failure});assert.equal(result.status,1,result.stdout);
      assert.equal(result.body,'');assert.equal(result.calls.length,failure==='get'?1:2);
      assert.match(result.stderr,/fixture (read|write) refused/u);
      assert.doesNotMatch(result.stdout,/Release notes updated successfully/u);
    }
  }
});
