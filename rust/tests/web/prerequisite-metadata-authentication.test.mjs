import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

test('actual source metadata requests scope read credentials and strip redirect headers without installation or network',()=>{
  const source=fileURLToPath(new URL('../../../data/meta/prerequisite-installer.py',import.meta.url));
  const program=String.raw`
import importlib.util,sys,urllib.request
spec=importlib.util.spec_from_file_location('source_installer',sys.argv[1])
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
base='https://api.github.com/repos/vendor/compiler/releases/'
for suffix in ['latest','tags/v1.2.3']:
 request=module.discovery_request(base+suffix,{'GH_TOKEN':'fixture-read-token'})
 assert request.get_method()=='GET' and request.data is None
 assert request.get_header('Authorization')=='Bearer fixture-read-token'
 assert 'Authorization' not in request.headers
 for url in ['https://release-assets.githubusercontent.com/artifact.zip',base+'tags/v2']:
  redirected=urllib.request.HTTPRedirectHandler().redirect_request(request,None,302,'redirect',{},url)
  assert not redirected.has_header('Authorization')
for url in [
 'https://github.com/vendor/compiler/releases/download/v1/compiler.zip',
 'https://publisher.invalid/docs',
 'http://api.github.com/repos/vendor/compiler/releases/latest',
 'https://api.github.com.evil.invalid/repos/vendor/compiler/releases/latest',
 'https://api.github.com:444/repos/vendor/compiler/releases/latest',
 'https://user@api.github.com/repos/vendor/compiler/releases/latest',
 'https://api.github.com/repos/vendor/compiler/contents/private',
 base+'latest?redirect=elsewhere',base+'latest#fragment']:
 assert not module.discovery_request(url,{'GH_TOKEN':'fixture-read-token'}).has_header('Authorization'),url
assert not module.discovery_request(base+'latest',{}).has_header('Authorization')
assert module.discovery_request(base+'latest',{'GITHUB_TOKEN':'fixture-fallback'}).get_header('Authorization')=='Bearer fixture-fallback'
for token in ['invalid\nheader','invalid\rheader',True]:
 try: module.discovery_request(base+'latest',{'GH_TOKEN':token})
 except ValueError as error: assert str(error)=='invalid_github_read_token'
 else: raise AssertionError('invalid credential accepted')
print('source metadata credential and redirect controls passed')
`;
  const result=spawnSync('python3',['-B','-c',program,source],{encoding:'utf8',timeout:15000,maxBuffer:32768});
  assert.equal(result.status,0,(result.stdout+result.stderr).slice(-4000));
  assert.equal(result.stdout.trim(),'source metadata credential and redirect controls passed');
});
