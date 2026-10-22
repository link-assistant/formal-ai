import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,readFileSync,mkdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {authenticateProjectionSource} from '../../../scripts/lib/native-session-source.mjs';
import {validateProjectionSelector} from '../../../scripts/lib/native-session-projections.mjs';
const workflow=new URL('../../../.github/workflows/evidence-check.yml',import.meta.url);
test('dispatch evidence uses a validated nonreserved base for both original checks',()=>{
 const source=readFileSync(workflow,'utf8');
 assert.equal((source.match(/FORMAL_AI_EVIDENCE_BASE_REF: \$\{\{ github.base_ref \|\| github.event.repository.default_branch \}\}/gu)??[]).length,2);
 assert.ok(!source.includes('GITHUB_BASE_REF:'));
 for(const command of ['self-hosting-metric.rs','check-formal-ai-contribution.rs'])assert.ok(source.includes(command));
 const dir=mkdtempSync(join(tmpdir(),'evidence-base-'));
 try {
 const git=args=>execFileSync('git',args,{cwd:dir,encoding:'utf8'}).trim();git(['init','-q']);git(['-c','user.name=Test','-c','user.email=test@example.com','commit','--allow-empty','-qm','fixture']);git(['update-ref','refs/remotes/origin/main','HEAD']);
 const block=source.split('        run: |')[1].split('          echo ')[0].replace(/^          /gmu,'');
 for(const base of ['main','','missing','../bad','-bad']) {
 const result=spawnSync('bash',['-c',block],{cwd:dir,env:{...process.env,GITHUB_BASE_REF:'',FORMAL_AI_EVIDENCE_BASE_REF:base},encoding:'utf8'});
 assert.equal(result.status===0,base==='main',base);
 assert.ok(!result.stderr.includes('origin/\''));
 }
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('authenticated dispatch binds API run, PR head/base/merge and physical tree; forged and stale selectors fail',()=>{
 const dir=mkdtempSync(join(tmpdir(),'projection-source-'));
 try {
 const git=args=>execFileSync('git',args,{cwd:dir,encoding:'utf8'}).trim();git(['init','-q']);git(['-c','user.name=Test','-c','user.email=test@example.com','commit','--allow-empty','-qm','fixture']);
 const sha=git(['rev-parse','HEAD']),tree=git(['rev-parse','HEAD^{tree}']),base='b'.repeat(40),merge='c'.repeat(40),repository='owner/repo';
 const run={id:123,run_attempt:1,event:'workflow_dispatch',path:'.github/workflows/regenerate-native-session-projections.yml',head_sha:sha,head_branch:'fix',repository:{full_name:repository}};
 const pr={number:1329,state:'open',head:{sha,ref:'fix',repo:{full_name:repository}},base:{sha:base,repo:{full_name:repository}},merge_commit_sha:merge};
 const responses={[`repos/${repository}/actions/runs/123`]:run,[`repos/${repository}/pulls/1329`]:pr,[`repos/${repository}/git/commits/${merge}`]:{sha:merge,parents:[{sha:base},{sha}]},[`repos/${repository}/git/commits/${sha}`]:{sha,tree:{sha:tree}}};
 const file=join(dir,'responses.json');const save=()=>writeFileSync(file,JSON.stringify(responses));save();
 const bin=join(dir,'bin');mkdirSync(bin);writeFileSync(join(bin,'gh'),'#!/usr/bin/env node\nconst fs=require("node:fs");process.stdout.write(JSON.stringify(JSON.parse(fs.readFileSync(process.env.FIXTURE_RESPONSES))[process.argv[3]]));\n',{mode:0o755});
 const env={...process.env,PATH:bin+':'+process.env.PATH,FIXTURE_RESPONSES:file,GITHUB_ACTIONS:'true',GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REPOSITORY:repository,GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'1',GITHUB_SHA:sha,GITHUB_REF:'refs/heads/fix',NATIVE_SESSION_REQUESTED_PULL_REQUEST:'1329'};
 assert.throws(()=>validateProjectionSelector(env,{requestedPullRequest:'1329'}),/unsupported without authenticated/);
 const proof=authenticateProjectionSource(env,dir);assert.equal(validateProjectionSelector(env,proof).tree,tree);
 assert.throws(()=>validateProjectionSelector(env,{...proof}),/unsupported without authenticated/);
 for(const [object,key,value] of [[run,'path','other.yml'],[run,'run_attempt',2],[pr,'state','closed'],[pr.head,'sha','d'.repeat(40)],[responses[`repos/${repository}/git/commits/${merge}`],'parents',[{sha:base}]], [responses[`repos/${repository}/git/commits/${sha}`].tree,'sha','e'.repeat(40)]]) {
 const old=object[key];object[key]=value;save();assert.throws(()=>authenticateProjectionSource(env,dir));object[key]=old;save();
 }
 }finally{rmSync(dir,{recursive:true,force:true});}
});
