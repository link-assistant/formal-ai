import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import test from 'node:test';

test('canonical catalog release metadata binds repository and stable tag without HTML asset recommendations',()=>{
 const result=spawnSync('python3',['-c',String.raw`import importlib.util,json,hashlib
s=importlib.util.spec_from_file_location('installer','data/meta/prerequisite-installer.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
url='https://api.github.com/repos/owner/compiler/releases/latest'
release={'draft':False,'prerelease':False,'tag_name':'v4.2.0','html_url':'https://github.com/owner/compiler/releases/tag/v4.2.0','assets':[{'name':'compiler.zip'},{'name':'compiler-x86_64-pc-linux.zip'},{'name':'compiler-aarch64-pc-linux.zip'}]}
calls=[]
def fetch(url,limit):
 assert limit==2097152
 calls.append(url);raw=json.dumps(release).encode();return raw,hashlib.sha256(raw).hexdigest()
r=m.github_release_document(url,fetch,'api.github.com')
assert calls==[url] and r[:3]==('owner/compiler','latest',release)
assert r[3]==hashlib.sha256(json.dumps(release).encode()).hexdigest()
assert m.select_release_archive(r[2]['assets'],'','linux','x86_64')['name']=='compiler-x86_64-pc-linux.zip'
assert m.github_release_document('https://publisher.test/docs',fetch,'publisher.test') is None
assert calls==[url]
def refused(url=url,host='api.github.com'):
 try:m.github_release_document(url,fetch,host)
 except (ValueError,TypeError):return
 raise AssertionError('untrusted or unstable metadata accepted')
for bad in [url+'?query=value',url+'#fragment',url.replace('https:','http:'),url.replace('/latest','/latest/extra')]:refused(bad)
refused(host='publisher.test')
for field,value in [('draft',True),('prerelease',True),('html_url','https://github.com/foreign/compiler/releases/tag/v4.2.0'),('tag_name','')]:
 old=release[field];release[field]=value;refused();release[field]=old
refused(url.replace('/latest','/tags/different'))
r=m.github_release_document(url.replace('/latest','/tags/v4.2.0'),fetch,'api.github.com');assert r[1]=='tags/v4.2.0'
seed=open('data/seed/setup-publishers.lino').read().split('setup_publisher_scalac\n',1)[1].split('setup_publisher_kotlinc',1)[0]
assert 'host "api.github.com"' in seed and 'documentation "https://api.github.com/repos/scala/scala3/releases/latest"' in seed
print('PURE_RELEASE_DOCUMENTATION_PASSED')`],{encoding:'utf8',timeout:15000});
 assert.equal(result.status,0,result.stderr);
 assert.match(result.stdout,/PURE_RELEASE_DOCUMENTATION_PASSED/);
});
