// Selected scratch verification of the proposed helper; real small Git objects only.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {validatePublishedStableSource,validateSelectedSourceAuthority} from '../../../scripts/native-release-trust.mjs';
const git=(cwd,...args)=>execFileSync('git',args,{cwd,encoding:'utf8',stdio:'pipe'}).trim();
function fixture(t){
 const cwd=mkdtempSync(join(tmpdir(),'release-trust-'));t.after(()=>rmSync(cwd,{recursive:true,force:true}));
 git(cwd,'init','--initial-branch=main');git(cwd,'config','user.name','Fixture');git(cwd,'config','user.email','fixture@example.invalid');
 writeFileSync(join(cwd,'source'),'old stable');git(cwd,'add','.');git(cwd,'commit','-m','stable');const releaseCommit=git(cwd,'rev-parse','HEAD');git(cwd,'tag','v1.2.3');
 writeFileSync(join(cwd,'source'),'current main');git(cwd,'commit','-am','main');const defaultCommit=git(cwd,'rev-parse','HEAD');
 return {cwd,options:{tag:'v1.2.3',release:{tag_name:'v1.2.3',draft:false,prerelease:false,published_at:'2026-10-01T00:00:00Z'},releaseCommit,defaultCommit}};
}
test('published old stable tag remains authorized when main moves forward',t=>{const f=fixture(t);assert.deepEqual(validatePublishedStableSource(f.cwd,f.options),{tag:f.options.tag,releaseCommit:f.options.releaseCommit,defaultCommit:f.options.defaultCommit});});
test('missing, draft, prerelease, mismatched or unpublished release metadata refuses',t=>{
 const f=fixture(t);for(const change of [{tag_name:'v9.9.9'},{draft:true},{prerelease:true},{published_at:null}])assert.throws(()=>validatePublishedStableSource(f.cwd,{...f.options,release:{...f.options.release,...change}}));
 assert.throws(()=>validatePublishedStableSource(f.cwd,{...f.options,tag:'v1.2.3-rc.1'}));assert.throws(()=>validatePublishedStableSource(f.cwd,{...f.options,releaseCommit:f.options.defaultCommit}));
});
test('published metadata does not authorize a tag outside default-branch ancestry',t=>{
 const f=fixture(t);git(f.cwd,'checkout','-b','foreign',f.options.releaseCommit);writeFileSync(join(f.cwd,'source'),'foreign code');git(f.cwd,'commit','-am','foreign');git(f.cwd,'tag','v1.2.4');
 const releaseCommit=git(f.cwd,'rev-parse','HEAD');assert.throws(()=>validatePublishedStableSource(f.cwd,{...f.options,tag:'v1.2.4',releaseCommit,release:{...f.options.release,tag_name:'v1.2.4'}}));
});

test('event-bound consumer authority refuses same-run metadata as stable authorization',t=>{
 const f=fixture(t),record={release_tag:f.options.tag,package_version:'1.2.3',source_commit:f.options.releaseCommit,selected_head:f.options.releaseCommit,bundle:null};
 const authority=validatePublishedStableSource(f.cwd,f.options);
 assert.equal(validateSelectedSourceAuthority(f.cwd,record,{event:'release',authority}),record);
 for(const change of [{package_version:'1.2.4'},{source_commit:f.options.defaultCommit},{selected_head:f.options.defaultCommit},{bundle:{name:'source.bundle'}},{release_tag:'v1.2.4'}])
  assert.throws(()=>validateSelectedSourceAuthority(f.cwd,{...record,...change},{event:'workflow_dispatch',authority}));
 assert.throws(()=>validateSelectedSourceAuthority(f.cwd,record,{event:'release'}));
 assert.throws(()=>validateSelectedSourceAuthority(f.cwd,record,{event:'pull_request'}));
 assert.throws(()=>validateSelectedSourceAuthority(f.cwd,record,{event:'unbound',authority}));
});
test('read-only PR source retains actual head and independently pinned base prerequisites',t=>{
 const f=fixture(t),record={release_tag:null,source_commit:f.options.defaultCommit,selected_head:f.options.releaseCommit,pinned_base:f.options.defaultCommit};
 assert.equal(validateSelectedSourceAuthority(f.cwd,record,{event:'pull_request'}),record);
 assert.throws(()=>validateSelectedSourceAuthority(f.cwd,{...record,pinned_base:'f'.repeat(40)},{event:'pull_request'}));
 assert.throws(()=>validateSelectedSourceAuthority(f.cwd,record,{event:'workflow_run'}));
});
