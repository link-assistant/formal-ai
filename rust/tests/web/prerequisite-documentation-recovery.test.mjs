import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
test('documentation recovery is grant-bound, finite, and preserves the selected document identity',()=>{
 const result=spawnSync('python3',['-c',String.raw`import importlib.util,hashlib,urllib.error,ssl
s=importlib.util.spec_from_file_location('installer','data/meta/prerequisite-installer.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
a='https://www.publisher.test/download/';b='https://publisher.test/download/'
hosts=['www.publisher.test','publisher.test']
body=b'<a href="next.html">Current version</a>'
leaf=b'<a href="https://github.com/example/compiler/releases/latest">download</a>'
count=0
def run(errors,initial=a,grant=hosts,document=leaf):
 calls=[]
 def fetch(url,limit):
  assert limit==2097152
  calls.append(url)
  if url in errors: raise errors[url]
  raw=body if document==body and url.endswith('/download/') else leaf
  return raw,hashlib.sha256(raw).hexdigest()
 try:return m.publisher_document(initial,fetch,'publisher.test',grant),calls,None
 except Exception as e:return None,calls,e
http=lambda code:urllib.error.HTTPError(a,code,'unavailable',{},None)
r,c,e=run({a:http(522)});assert e is None and c==[a,b] and r[0]==b and r[3]==hashlib.sha256(leaf).hexdigest()
r,c,e=run({});assert c==[a] and r[0]==a
for code in [401,403,404]:
 r,c,e=run({a:http(code)});assert e is not None and c==[a]
for error in [ValueError('download_limit_exceeded'),ssl.SSLError('certificate'),urllib.error.URLError(ssl.SSLError('certificate'))]:
 r,c,e=run({a:error});assert e is not None and c==[a]
r,c,e=run({a:http(522)},grant=[hosts[0]]);assert e is not None and c==[a]
r,c,e=run({a:http(522),b:http(522)});assert e is not None and c==[a,b]
r,c,e=run({a+'next.html':http(522)},document=body);assert e is None and c==[a,a+'next.html',b+'next.html'] and r[0]==b+'next.html'
assert m.documentation_alias(a,'foreign.test',hosts) is None
assert m.documentation_alias('http://www.publisher.test/download/','publisher.test',hosts) is None
assert m.diagnostic_url('https://user:secret@publisher.test/x?token=secret#private')=='https://publisher.test/x'
print('PURE_DOCUMENTATION_CONTROLS_PASSED')`],{encoding:'utf8',timeout:15000});
 assert.equal(result.status,0,result.stderr);assert.match(result.stdout,/PURE_DOCUMENTATION_CONTROLS_PASSED/);
});
