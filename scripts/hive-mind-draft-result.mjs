#!/usr/bin/env node
// #1210: trust only this attempt's exact branch in this repository, created during this run.
import {execFileSync} from 'node:child_process';
import {appendFileSync, readFileSync, writeFileSync} from 'node:fs';
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
const evidenceOnly = file => file.split('/').at(-1)==='.gitkeep'
  || file.startsWith('dev/log/') || file==='.formal-ai/general-change-plan.lino';
/** Source/config/docs changes make a candidate eligible; only original behavioral oracles can prove a fix. */
export function usefulDraftChanges(files) {
  if (!Array.isArray(files)) throw Error('invalid file census');
  return files.filter(file => {
    if(typeof file.filename!=='string'||!['added','modified','removed','renamed','copied','changed'].includes(file.status)) throw Error('invalid file record');
    return (!evidenceOnly(file.filename)||(file.previous_filename&&!evidenceOnly(file.previous_filename)))
      && (file.changes>0 || ['added','removed','renamed','copied'].includes(file.status));
  }).map(file=>file.filename);
}
/** Complete bounded diff and stable identity: metadata-only PRs never fan out dozens of CI jobs. */
export function inspectAttemptDraft(gh, repository, selected) {
  const endpoint='repos/'+repository+'/pulls/'+selected.number;
  const before=JSON.parse(gh(['api',endpoint]));
  const same=pr=>pr.state==='open'&&pr.head?.ref===selected.headRefName
    && pr.head?.repo?.full_name?.toLowerCase()===repository.toLowerCase()
    && /^[a-f0-9]{40}$/.test(pr.head?.sha??'')&&pr.head.sha===selected.headRefOid;
  if(!same(before)||!Number.isInteger(before.changed_files)||before.changed_files<0||before.changed_files>1000)
    throw Error('unverified or oversized candidate diff');
  const files=[];
  for(let page=1;files.length<before.changed_files&&page<=10;page++){
    const next=JSON.parse(gh(['api',endpoint+'/files?per_page=100&page='+page]));
    if(!Array.isArray(next)||next.length===0||next.length>100) throw Error('incomplete diff census');
    files.push(...next);
  }
  if(files.length!==before.changed_files||new Set(files.map(file=>file.filename)).size!==files.length)
    throw Error('incomplete or duplicate diff census');
  const after=JSON.parse(gh(['api',endpoint]));
  if(!same(after)||after.base?.sha!==before.base?.sha||after.changed_files!==before.changed_files)
    throw Error('candidate changed during inspection');
  const changed=usefulDraftChanges(files);
  return {schema:'formal-ai-draft-selection/v1',pullRequest:selected.number,head:before.head.sha,
    base:before.base.sha,files:files.map(file=>({path:file.filename,status:file.status,changes:file.changes})),
    usefulChanges:changed,eligibleForChecks:changed.length>0,taskVerified:false,
    disposition:changed.length>0?'candidate-awaiting-original-oracles':'gap-no-implementation'};
}
export function main(env = process.env) {
  const gh = args => execFileSync('gh', args, {encoding:'utf8',timeout:20000,maxBuffer:4194304}).trim();
  const repository = env.GITHUB_REPOSITORY;
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository || '')) throw Error('invalid repository');
  if (process.argv.includes('--select')) {
    let result;
    try {
      const rows = JSON.parse(gh(['pr','list','--repo',repository,'--state','open','--head',env.DRAFT_HEAD_BRANCH,'--limit','100','--json','number,headRefName,headRefOid,headRepository,state,createdAt']));
      if(!Array.isArray(rows)||rows.length>=100)throw Error('incomplete candidate census');
      const selected = selectAttemptPullRequest(rows, {branch:env.DRAFT_HEAD_BRANCH,repository,startedAt:readFileSync(env.RUNNER_TEMP+'/draft-started-at','utf8').trim()});
      result=selected?inspectAttemptDraft(gh,repository,selected):{schema:'formal-ai-draft-selection/v1',eligibleForChecks:false,taskVerified:false,disposition:'gap-no-unique-candidate'};
    } catch {
      result={schema:'formal-ai-draft-selection/v1',eligibleForChecks:false,taskVerified:false,disposition:'gap-unavailable-complete-diff'};
    }
    if(env.FORMAL_AI_DRAFT_LOG_DIR)writeFileSync(env.FORMAL_AI_DRAFT_LOG_DIR+'/draft-selection.json',JSON.stringify(result,null,2)+'\n',{flag:'wx'});
    if(result.eligibleForChecks)appendFileSync(env.GITHUB_OUTPUT,'pull_request_number='+result.pullRequest+'\n');
    else {console.log('No verified useful candidate: '+result.disposition+'. Checks were not dispatched.');process.exitCode=1;}
  } else if (process.argv.includes('--failure')) {
    if (!/^[1-9][0-9]*$/.test(env.DRAFT_ISSUE_NUMBER || '')) throw Error('invalid issue number');
    const runUrl=(env.GITHUB_SERVER_URL || 'https://github.com')+'/'+repository+'/actions/runs/'+env.GITHUB_RUN_ID;
    gh(['issue','comment',env.DRAFT_ISSUE_NUMBER,'--repo',repository,'--body',failureComment({runUrl,artifactUrl:env.DRAFT_ARTIFACT_URL})]);
  } else throw Error('expected --select or --failure');
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
