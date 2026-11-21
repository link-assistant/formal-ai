// Actual pinned change detector; Git comparisons use finite local repositories.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {templatePythonAvailable} from './template-python-runtime.mjs';
const base=fileURLToPath(new URL('./fixtures/python-current-detector/',import.meta.url));
const original=readFileSync(join(base,'detect_code_changes.py'),'utf8');
const identity=JSON.parse(readFileSync(join(base,'source.json'),'utf8')).files[0];
assert.equal(createHash('sha256').update(original).digest('hex'),identity.sha256);
assert.equal(createHash('sha1').update(`blob ${Buffer.byteLength(original)}\0`).update(original).digest('hex'),identity.gitBlob);
const declaration='CODE_EXTENSIONS = (".py", ".sh", ".json", ".toml", ".yml", ".yaml")';
assert.equal(original.split(declaration).length,2);
const candidate=original.replace(declaration,declaration.slice(0,-1)+', ".js", ".mjs", ".cjs")');
const probe=String.raw`
import importlib.util, json, os, resource, subprocess, sys
from pathlib import Path
resource.setrlimit(resource.RLIMIT_AS,(134217728,134217728))
resource.setrlimit(resource.RLIMIT_CPU,(6,6))
script=Path(sys.argv[1]);mode=sys.argv[2]
spec=importlib.util.spec_from_file_location('detector',script)
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
events=['push','pull_request']
if mode=='classify':
    paths=['scripts/check-deprecated-packages.mjs','.github/actions/artifact/index.mjs',
           '.github/actions/deploy-pages/deploy.mjs','scripts/helper.js','scripts/helper.cjs',
           'scripts/check.py','scripts/check.sh','config.json','config.toml',
           '.github/workflows/check.yml']
    result={path:{event:module.detect_change_types([path],event_name=event) for event in events} for path in paths}
elif mode=='excluded':
    paths=['docs/helper.mjs','python/docs/helper.js','examples/helper.cjs','experiments/helper.mjs',
           'dev/log/helper.js','changelog.d/helper.mjs','README.md']
    result={path:{event:module.detect_change_types([path],event_name=event) for event in events} for path in paths}
else:
    def git(*arguments):
        return subprocess.run(['git',*arguments],check=True,capture_output=True,text=True).stdout.strip()
    git('init','--quiet','--initial-branch=main');git('config','user.name','Detector fixture')
    git('config','user.email','fixture@example.invalid');git('config','commit.gpgsign','false')
    Path('README.md').write_text('fixture\n');git('add','README.md');git('commit','--quiet','-m','base')
    base=git('rev-parse','HEAD');result={}
    for path in ['scripts/check-deprecated-packages.mjs','scripts/helper.js','scripts/helper.cjs']:
        git('reset','--hard',base)
        target=Path(path);target.parent.mkdir(exist_ok=True);target.write_text('// changed production helper\n')
        git('add',path);git('commit','--quiet','-m','change helper');head=git('rev-parse','HEAD')
        result[path]={}
        for event in events:
            output=Path('output');output.write_text('')
            environment={**os.environ,'GITHUB_EVENT_NAME':event,'GITHUB_BEFORE_SHA':base,
                         'GITHUB_BASE_SHA':base,'GITHUB_HEAD_SHA':head,'GITHUB_OUTPUT':str(output)}
            run=subprocess.run([sys.executable,'-B',str(script)],env=environment,capture_output=True,text=True,check=True)
            result[path][event]={'outputs':dict(line.split('=',1) for line in output.read_text().splitlines()),
                                 'listedChangedHelper':path in run.stdout}
print(json.dumps(result))
`;
function observe(t,mode,repaired){
  if(!templatePythonAvailable(t))return;
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-python-detector-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  const script=join(cwd,'detector.py');writeFileSync(script,repaired?candidate:original);
  const result=spawnSync('python3',['-B','-c',probe,script,mode],{cwd,encoding:'utf8',timeout:10000,maxBuffer:65536});
  assert.equal(result.error,undefined,String(result.error));assert.equal(result.status,0,result.stderr);
  return JSON.parse(result.stdout);
}
test('Python detector misses JavaScript production helpers for push and pull request',{timeout:15000},t=>{
  const old=observe(t,'classify',false);if(!old)return;
  const repaired=observe(t,'classify',true);
  for(const [path,events] of Object.entries(old))for(const event of ['push','pull_request']){
    const omitted=/\.(?:mjs|cjs|js)$/.test(path);
    assert.equal(events[event]['any-code-changed'],!omitted,path);
    assert.equal(repaired[path][event]['any-code-changed'],true,path);
    assert.equal(repaired[path][event]['docs-changed'],false,path);
  }
});
test('Python detector actual Git entrypoint reports false no-code outputs for JavaScript',{timeout:25000},t=>{
  const old=observe(t,'git',false);if(!old)return;
  const repaired=observe(t,'git',true);
  for(const [path,events] of Object.entries(old))for(const event of ['push','pull_request']){
    assert.equal(events[event].listedChangedHelper,true,path);
    assert.deepEqual(events[event].outputs,{'any-code-changed':'false','docs-changed':'false'},path);
    assert.deepEqual(repaired[path][event].outputs,{'any-code-changed':'true','docs-changed':'false'},path);
  }
});
test('Python detector candidate preserves documentation and experimental exclusions',{timeout:15000},t=>{
  const old=observe(t,'excluded',false);if(!old)return;
  const repaired=observe(t,'excluded',true);assert.deepEqual(repaired,old);
  for(const events of Object.values(repaired))for(const result of Object.values(events)){
    assert.equal(result['any-code-changed'],false);
  }
  assert.equal(repaired['docs/helper.mjs'].push['docs-changed'],true);
  assert.equal(repaired['README.md'].pull_request['docs-changed'],true);
});
