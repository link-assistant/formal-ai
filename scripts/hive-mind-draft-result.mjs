#!/usr/bin/env node
// #1210: trust only this attempt's exact branch in this repository, created during this run.
import {execFileSync} from 'node:child_process';
import {appendFileSync, readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
export function selectAttemptPullRequest(rows, {branch, repository, startedAt}) {
  const since = Date.parse(startedAt);
  if (!branch || !repository || !Number.isFinite(since)) return null;
  const matches = rows.filter(pr => pr.headRefName === branch && pr.headRepository?.nameWithOwner?.toLowerCase() === repository.toLowerCase()
    && pr.state === 'OPEN' && Number.isInteger(pr.number) && pr.number > 0 && Date.parse(pr.createdAt) >= since);
  return matches.length === 1 ? matches[0] : null;
}
export function failureComment({runUrl, artifactUrl}) {
  return 'Formal AI through Hive Mind did not produce a verified new draft PR in this attempt. '
    + (artifactUrl ? '[Complete session evidence](' + artifactUrl + '). ' : 'The session artifact was unavailable; inspect the upload step. ')
    + '[Workflow run and build/attempt logs](' + runUrl + '). No draft was hand-fixed or merged.';
}
export function main(env = process.env) {
  const gh = args => execFileSync('gh', args, {encoding:'utf8'}).trim();
  const repository = env.GITHUB_REPOSITORY;
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository || '')) throw Error('invalid repository');
  if (process.argv.includes('--select')) {
    const rows = JSON.parse(gh(['pr','list','--repo',repository,'--state','open','--head',env.DRAFT_HEAD_BRANCH,'--limit','100','--json','number,headRefName,headRepository,state,createdAt']));
    const selected = selectAttemptPullRequest(rows, {branch:env.DRAFT_HEAD_BRANCH,repository,startedAt:readFileSync(env.RUNNER_TEMP+'/draft-started-at','utf8').trim()});
    if (selected) appendFileSync(env.GITHUB_OUTPUT, 'pull_request_number='+selected.number+'\n');
    else console.log('No unique new same-repository PR matches the attempt head; checks were not dispatched.');
  } else if (process.argv.includes('--failure')) {
    if (!/^[1-9][0-9]*$/.test(env.DRAFT_ISSUE_NUMBER || '')) throw Error('invalid issue number');
    const runUrl=(env.GITHUB_SERVER_URL || 'https://github.com')+'/'+repository+'/actions/runs/'+env.GITHUB_RUN_ID;
    gh(['issue','comment',env.DRAFT_ISSUE_NUMBER,'--repo',repository,'--body',failureComment({runUrl,artifactUrl:env.DRAFT_ARTIFACT_URL})]);
  } else throw Error('expected --select or --failure');
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
