import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
const authenticated=new WeakSet();
export function isAuthenticatedProjectionSource(value) { return authenticated.has(value); }
/** Select the exact dispatched PR head, with current API-authenticated base/merge witnesses. */
export function authenticateProjectionSource(environment,cwd) {
 const requested=environment.NATIVE_SESSION_REQUESTED_PULL_REQUEST;
 assert.match(requested??'',/^[1-9]\d{0,9}$/u,'finite PR number required');
 assert.equal(environment.GITHUB_ACTIONS,'true');
 assert.equal(environment.GITHUB_EVENT_NAME,'workflow_dispatch');
 const repository=environment.GITHUB_REPOSITORY;
 assert.match(repository??'',/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
 for(const key of ['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT'])assert.match(environment[key]??'',/^[1-9]\d*$/u);
 const api=path=>JSON.parse(execFileSync('gh',['api',`repos/${repository}/${path}`],{cwd,env:environment,encoding:'utf8',timeout:30000,maxBuffer:1024*1024}));
 const run=api(`actions/runs/${environment.GITHUB_RUN_ID}`);
 assert.equal(run.repository.full_name,repository);
 assert.equal(run.id.toString(),environment.GITHUB_RUN_ID);
 assert.equal(run.run_attempt.toString(),environment.GITHUB_RUN_ATTEMPT);
 assert.equal(run.event,'workflow_dispatch');
 assert.equal(run.path,'.github/workflows/regenerate-native-session-projections.yml');
 assert.equal(run.head_sha,environment.GITHUB_SHA);
 const pr=api(`pulls/${requested}`);
 assert.equal(pr.number.toString(),requested);
 assert.equal(pr.state,'open');
 assert.equal(pr.base.repo.full_name,repository);
 assert.equal(pr.head.repo.full_name,repository,'dispatch must run in the PR head repository');
 assert.equal(pr.head.sha,run.head_sha,'workflow is stale relative to current PR head');
 assert.equal(pr.head.ref,run.head_branch);
 assert.equal(environment.GITHUB_REF,`refs/heads/${pr.head.ref}`);
 for(const sha of [pr.head.sha,pr.base.sha,pr.merge_commit_sha])assert.match(sha??'',/^[a-f0-9]{40}$/u,'current head/base/merge identity required');
 const merge=api(`git/commits/${pr.merge_commit_sha}`);
 assert.equal(merge.sha,pr.merge_commit_sha);
 assert.deepEqual(merge.parents.map(parent=>parent.sha),[pr.base.sha,pr.head.sha],'current PR merge parents differ');
 const git=args=>execFileSync('git',args,{cwd,encoding:'utf8',timeout:30000,maxBuffer:1024*1024}).trim();
 const commit=git(['rev-parse','HEAD']),tree=git(['rev-parse','HEAD^{tree}']);
 assert.equal(commit,pr.head.sha,'physical checkout differs from authenticated PR head');
 const head=api(`git/commits/${pr.head.sha}`);
 assert.equal(head.sha,commit);assert.equal(head.tree.sha,tree);
 assert.equal(git(['status','--porcelain','--untracked-files=no']),'','tracked checkout must be clean');
 const proof=Object.freeze({requestedPullRequest:requested,selection:'authenticated current PR head',repository,commit,tree,base:pr.base.sha,merge:merge.sha,run:environment.GITHUB_RUN_ID,attempt:environment.GITHUB_RUN_ATTEMPT,workflow:run.path});
 authenticated.add(proof);return proof;
}
