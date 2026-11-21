import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,mkdirSync,rmSync,symlinkSync,unlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {CANONICAL_SOURCE,CANONICAL_PREFIX,CANONICAL_FILES,CANONICAL_PARTS,partitionCanonicalSource,assembleCanonicalSource,readCanonicalSource} from '../../scripts/canonical-release-source.mjs';
import {registeredSourceProfile} from '../../scripts/governed-github-command-provider.mjs';
import {publicationContract} from '../../scripts/self-authored-publication-source.mjs';
const original=gunzipSync(readFileSync(new URL('./fixtures/canonical-release-source/original-source.yml.gz',import.meta.url))).toString('utf8');
const sha=text=>createHash('sha256').update(text).digest('hex');
test('live canonical partition removes the reproduced physical size warning without changing any byte',()=>{
 const bounded=files=>files.forEach(file=>assert(file.text.trimEnd().split('\n').length<=1000,'physical source warning: '+file.path));
 assert.equal(sha(original),'9ff11df305ad9857b69df846559370ae977d583b94b2ee101ca7ed1299eefd0b');
 assert.throws(()=>bounded([{path:CANONICAL_SOURCE,text:original}]),/physical source warning/);
 const files=partitionCanonicalSource(original);bounded(files);assert.equal(assembleCanonicalSource(files),original);
 assert.deepEqual(files.map(file=>file.path),CANONICAL_FILES);
 const current=readCanonicalSource();bounded(current.files);
 const packet=JSON.parse(readFileSync(CANONICAL_PREFIX+'stage-source-coverage.json','utf8'));
 assert.equal(sha(current.text),packet.workflowSha256);
});
test('canonical assembly refuses incomplete, duplicate, foreign, reordered and altered physical declarations',()=>{
 const files=partitionCanonicalSource(original);
 for(const mutate of [v=>v.pop(),v=>v.reverse(),v=>v[2]=structuredClone(v[1]),v=>v[1].text+='x',v=>v[0].text+='x',v=>v[1].path='../foreign']){
  const copy=structuredClone(files);mutate(copy);assert.throws(()=>assembleCanonicalSource(copy));
 }
 const unknown=structuredClone(files);unknown[0].text=unknown[0].text.replace('/v1','/v2');assert.throws(()=>assembleCanonicalSource(unknown));
 assert.equal(assembleCanonicalSource([{path:CANONICAL_SOURCE,text:original}]),original,'explicit legacy single-file fixture remains readable');
});
test('physical canonical reader rejects missing or redirected source parts',t=>{
 const root=mkdtempSync(join(tmpdir(),'canonical-release-source-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
 for(const file of partitionCanonicalSource(original)){mkdirSync(dirname(join(root,file.path)),{recursive:true});writeFileSync(join(root,file.path),file.text);}
 assert.equal(readCanonicalSource(root).text,original);
 const target=join(root,CANONICAL_PARTS[1]),saved=readFileSync(target);unlinkSync(target);assert.throws(()=>readCanonicalSource(root));
 const alternate=join(root,'alternate');writeFileSync(alternate,saved);symlinkSync(alternate,target);assert.throws(()=>readCanonicalSource(root),/regular canonical source|redirects/);
});
test('authenticated publication and command profiles bind the index and both actual fragments',()=>{
 const physical=readCanonicalSource(),contract=publicationContract(process.cwd());
 assert.equal(contract.routes.length,52);assert.equal(contract.bindings,104);assert.equal(contract.sources.length,6);
 for(const file of physical.files)assert.equal(contract.sources.find(source=>source.path===file.path).sha256,sha(file.text));
 assert.notEqual(contract.sources.find(source=>source.path===CANONICAL_SOURCE).sha256,sha(physical.text),'physical index digest is never substituted with assembled workflow digest');
 const profile=registeredSourceProfile(contract.sources.map(file=>file.path));assert.equal(profile.requiredRequests,19);assert.equal(profile.maximumRequests,19);
 for(const names of [contract.sources.map(file=>file.path).filter(file=>file!==CANONICAL_PARTS[1]),contract.sources.map(file=>file.path).map(file=>file===CANONICAL_PARTS[1]?CANONICAL_PREFIX+'foreign.txt':file)])assert.throws(()=>registeredSourceProfile(names));
});
