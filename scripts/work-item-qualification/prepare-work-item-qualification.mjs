#!/usr/bin/env node
// DRAFT: reuse maintained bounded source/CI admission before any hosted fixture effect.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {taskNames} from './fixtures.mjs';
import {workItemIdentity} from './policy.mjs';
const root=process.env.GITHUB_WORKSPACE, shared=async name=>import(pathToFileURL(path.join(root,name)));
const {githubReader}=await shared('scripts/admit-hive-original-delivery.mjs');
const {observePublishedAdmission}=await shared('scripts/lib/hive-delivery-release.mjs');
const sha=value=>createHash('sha256').update(value).digest('hex');
assert.equal(process.env.GITHUB_ACTIONS,'true');assert.equal(process.env.GITHUB_REF,'refs/heads/main');
assert.equal(process.env.GITHUB_RUN_ATTEMPT,'1','explicit fresh owner required after uncertainty');
assert.equal(process.env.WORK_ITEM_REAL_EFFECTS,'true','explicit qualification grant required');
assert.equal(process.env.GITHUB_REPOSITORY,'link-assistant/formal-ai');
const repository=process.env.GITHUB_REPOSITORY,reader=githubReader({repository});
const source=(await reader.read('repos/'+repository+'/commits/main')).sha;
assert.equal(source,process.env.GITHUB_SHA,'main moved before selection');
const pullRequest=Number(process.env.WORK_ITEM_PULL_REQUEST);assert.equal(pullRequest,1495,'single issue/PR scope');
const pr=await reader.read('repos/'+repository+'/pulls/'+pullRequest);
assert.equal(pr.merged,true,'native/client qualification follows merge, never PR compilation');
const ancestry=await reader.read('repos/'+repository+'/compare/'+pr.merge_commit_sha+'...'+source);
assert(['identical','ahead'].includes(ancestry.status));assert.equal(ancestry.merge_base_commit.sha,pr.merge_commit_sha);
// Reuse the existing independently authenticated publication-source/CI evaluator.
// A prepared version commit need not have its own push pipeline: its producer and exact metadata delta do.
const verification=await reader.read('repos/'+repository+'/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
assert.equal(verification.workflow_runs.length,1);
const published=await observePublishedAdmission({repository,reader,verifiedRunId:verification.workflow_runs[0].id});
assert.equal(published.admitted,true,'current release/CI qualification pending: '+JSON.stringify({reason:published.reason}));
assert.equal(published.selected.source,source,'current main is not the qualified published source; never fall back to older code');
const hiveSource=execFileSync('gh',['api','repos/link-assistant/hive-mind/commits/main','--jq','.sha'],
  {encoding:'utf8',timeout:30000,maxBuffer:65536}).trim();assert.match(hiveSource,/^[a-f0-9]{40}$/u);
const {suite,inputHash,policyHash}=await workItemIdentity({read:name=>fs.readFileSync(path.join(root,name))});
const selection={schema:'formal-ai-work-item-admission/v1',admitted:true,source,hiveSource,pullRequest,
  ci:published.ci,publishedObservedCi:published.publishedObservedCi,publishedSourceProof:published,
  publishedRuntime:published.publishedRuntime,producer:{run:process.env.GITHUB_RUN_ID,attempt:process.env.GITHUB_RUN_ATTEMPT},
  inputHash,policyHash,suite};
const owner={schema:'formal-ai-hive-effect-owner/v1',repository,run:Number(process.env.GITHUB_RUN_ID),
  attempt:1,protocolCommit:process.env.GITHUB_SHA,selectedSource:source,scope:'pr-'+pullRequest,
  tools:suite.clients,taskMode:'work-item-1483',inputHash,policyHash};
const output=path.join(process.env.RUNNER_TEMP,'work-item-admission');fs.mkdirSync(output,{recursive:true});
fs.writeFileSync(output+'/selection.json',JSON.stringify(selection,null,2)+'\n');
fs.writeFileSync(output+'/owner.json',JSON.stringify(owner,null,2)+'\n');
fs.writeFileSync(output+'/suite.json',JSON.stringify(suite,null,2)+'\n');
fs.appendFileSync(process.env.GITHUB_OUTPUT,'source='+source+'\nhive_source='+hiveSource+'\ndirectory='+output+'\n');
