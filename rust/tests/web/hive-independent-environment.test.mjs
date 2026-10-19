import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

test('independent compiler environments authorize only owned metadata and retain bounded failure diagnostics',()=>{
  const source=fileURLToPath(new URL('../../../scripts/hive-original-delivery-oracle.py',import.meta.url));
  const program=String.raw`
import base64,importlib.util,json,os,pathlib,sys
spec=importlib.util.spec_from_file_location('independent_oracle',sys.argv[1])
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
workspace=pathlib.Path('/fixture/isolated-workspace')
inherited={'PATH':'fixture-path','JAVA_HOME':'fixture-java','LANG':'fixture-lang',
 'LC_ALL':'fixture-locale','TMPDIR':'fixture-temp','HOME':'unrelated-home',
 'GH_TOKEN':'fixture-read-token','GITHUB_TOKEN':'fixture-fallback-token',
 'OPENAI_API_KEY':'unrelated-fixture-secret','AWS_SECRET_ACCESS_KEY':'unrelated-fixture-secret'}
original=dict(inherited)
expected={key:inherited[key] for key in ('PATH','JAVA_HOME','LANG','LC_ALL','TMPDIR')}
expected['HOME']=str(workspace/'home')
assert module.independent_environment(workspace,inherited)==expected
metadata=module.independent_environment(workspace,inherited,metadata_read=True)
assert metadata==dict(expected,GH_TOKEN='fixture-read-token',GITHUB_TOKEN='fixture-fallback-token')
assert inherited==original
assert module.independent_environment(workspace,{})=={'HOME':str(workspace/'home')}
assert module.independent_environment(workspace,{},metadata_read=True)=={'HOME':str(workspace/'home')}
def result(code,stdout,stderr,signal=None):
 return {'exitCode':code,'signal':signal,'stdoutBase64':base64.b64encode(stdout).decode(),
  'stderrBase64':base64.b64encode(stderr).decode()}
assert module.require_process_success(result(0,b'compiler output',b''),'compilation') is None
for code,stdout,stderr,signal in [(1,b'HTTP Error 403: rate limit exceeded',b'',None),
 (2,b'',b'compiler error',None),(-9,b'prefix'*1000,b'tail'*2000,'SIGKILL')]:
 try: module.require_process_success(result(code,stdout,stderr,signal),'compiler discovery')
 except AssertionError as error:
  diagnostic=json.loads(str(error))
  assert diagnostic=={'phase':'compiler discovery','exitCode':code,'signal':signal,
   'stdoutTail':stdout.decode()[-4096:],'stderrTail':stderr.decode()[-4096:]}
 else: raise AssertionError('unsuccessful process accepted')
os.environ['GITHUB_ACTIONS']='false'
try: module.main()
except SystemExit as error: assert str(error)=='hosted-only: no local native/compiler execution'
else: raise AssertionError('hosted-only execution guard was lost')
print('independent compiler credential boundary, failure diagnostics and local refusal passed')
`;
  const result=spawnSync('python3',['-B','-c',program,source],{encoding:'utf8',timeout:15000,maxBuffer:32768});
  assert.equal(result.status,0,(result.stdout+result.stderr).slice(-4000));
  assert.equal(result.stdout.trim(),'independent compiler credential boundary, failure diagnostics and local refusal passed');
});
