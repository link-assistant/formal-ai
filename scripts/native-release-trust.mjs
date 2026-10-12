// Captured from the triggering workflow before any selected-source checkout.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
const git=(cwd,args)=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
const stableTag=/^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;
const commit=value=>assert.match(value,/^[a-f0-9]{40}$/u);
export function validatePublishedStableSource(cwd,{tag,release,defaultCommit,releaseCommit}) {
 assert.match(tag,stableTag,'only an exact stable semver tag can authorize a release source');
 assert.equal(release.tag_name,tag);assert.equal(release.draft,false);assert.equal(release.prerelease,false);
 assert.ok(typeof release.published_at==='string'&&Number.isFinite(Date.parse(release.published_at)));
 commit(defaultCommit);commit(releaseCommit);
 assert.equal(git(cwd,['rev-parse','refs/tags/'+tag+'^{commit}']),releaseCommit,'release API commit and fetched tag must agree');
 execFileSync('git',['merge-base','--is-ancestor',releaseCommit,defaultCommit],{cwd,stdio:'pipe'});
 return {tag,releaseCommit,defaultCommit};
}
export function authorizePublishedStableSource(cwd,{tag,repository}) {
 assert.match(tag,stableTag);assert.match(repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
 const api=path=>JSON.parse(execFileSync('gh',['api','repos/'+repository+'/'+path],{encoding:'utf8',maxBuffer:1024*1024}));
 const release=api('releases/tags/'+encodeURIComponent(tag));
 const branch=api('').default_branch;assert.ok(typeof branch==='string'&&branch.length>0);
 const defaultCommit=api('commits/'+encodeURIComponent(branch)).sha;
 const releaseCommit=api('commits/'+encodeURIComponent(tag)).sha;
 commit(defaultCommit);commit(releaseCommit);
 execFileSync('git',['fetch','--no-tags','origin','refs/tags/'+tag+':refs/tags/'+tag,defaultCommit],{cwd,stdio:'pipe'});
 return validatePublishedStableSource(cwd,{tag,release,defaultCommit,releaseCommit});
}

/** Event authority is independent of the downloaded same-run source receipt. */
export function validateSelectedSourceAuthority(cwd,record,{event,authority=null,purpose=event==='pull_request'?'validation':'publication'}) {
 assert.ok(['pull_request','release','workflow_run','workflow_dispatch'].includes(event),'unrecognized release event');
 assert.ok(['validation','publication'].includes(purpose),'unrecognized source purpose');
 if(purpose==='validation') {
  assert.ok(['pull_request','workflow_dispatch'].includes(event),'read-only validation requires an actual PR or explicit dispatch');
  assert.equal(authority,null,'read-only validation cannot acquire publication authority');
  assert.equal(record.release_tag,null,'read-only validation cannot acquire stable publication authority');
  commit(record.selected_head);commit(record.pinned_base);
  for(const prerequisite of [record.selected_head,record.pinned_base])
   execFileSync('git',['merge-base','--is-ancestor',prerequisite,record.source_commit],{cwd,stdio:'pipe'});
 } else {
  assert.notEqual(event,'pull_request','PR cannot publish a stable release');
  assert.ok(authority,'independent published-release authority required');
  assert.match(authority.tag,stableTag);commit(authority.releaseCommit);commit(authority.defaultCommit);
  assert.equal(record.release_tag,authority.tag);
  assert.equal(record.package_version,authority.tag.slice(1),'actual package version must match published tag');
  assert.equal(record.source_commit,authority.releaseCommit);
  assert.equal(record.selected_head,authority.releaseCommit);
  assert.equal(record.bundle,null,'stable release cannot substitute a synthetic bundle');
  assert.equal(git(cwd,['rev-parse','refs/tags/'+authority.tag+'^{commit}']),authority.releaseCommit);
  execFileSync('git',['merge-base','--is-ancestor',record.source_commit,authority.defaultCommit],{cwd,stdio:'pipe'});
 }
 return record;
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const result=authorizePublishedStableSource(process.cwd(),{tag:process.env.NATIVE_RELEASE_TAG,repository:process.env.NATIVE_REPOSITORY});
 assert.ok(process.env.GITHUB_ENV,'workflow environment receipt required');
 writeFileSync(process.env.GITHUB_ENV,
  'NATIVE_AUTHORIZED_RELEASE_COMMIT='+result.releaseCommit+'\nNATIVE_AUTHORIZED_DEFAULT_COMMIT='+result.defaultCommit+'\n',{flag:'a'});
 console.log(JSON.stringify(result));
}
