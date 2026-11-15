// Issue #1483: the Hive Mind `--tool agent --model formal-ai` leg run locally.
//
// What is real: Hive Mind's own prompt builders (agent.prompts.lib.mjs), its
// model mapping, its `buildAgentArgs`, and its `resolveFormalAiToolExecution`
// (client home and environment pointed at a Formal AI server through
// HIVE_MIND_FORMAL_AI_BASE_URL); the native `agent` CLI; and this checkout's
// JavaScript Formal AI server, which must already be listening at --base-url.
// What is replaced: `solve`'s GitHub orchestration (clone, branch, draft PR,
// commit, push, comments). The workspace is a local clone checked out at
// --at (with --apply, a prior run's diff committed on top, to test that a
// repeat is a verified no-op); `gh issue view` is answered from a recorded public issue fixture, so
// nothing is written to GitHub. No completed solution is in the prompt.
//
// Usage:
//   node experiments/issue-1483/hive-agent-local.mjs --base-url http://127.0.0.1:18483 \
//     --at 3c592625d --fixture data/benchmarks/fixtures/work-item-1455/failure.json \
//     --issue 1455 --pull 1456 --out /tmp/fa-1483/run-1455 [--apply <diff>] [--keep] [--hive <hive-mind src>]
import {mkdirSync,writeFileSync,readFileSync,chmodSync,rmSync,mkdtempSync} from 'node:fs';
import {join,dirname,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {execFileSync,spawn} from 'node:child_process';

const args=process.argv.slice(2);
const option=(name,fallback=null)=>{const at=args.indexOf(name);return at>=0?args[at+1]:fallback;};
const hive=option('--hive','/home/box/.bun/install/global/node_modules/@link-assistant/hive-mind/src/');
const baseUrl=option('--base-url'),at=option('--at'),issueNumber=Number(option('--issue')),pull=Number(option('--pull'));
const fixture=JSON.parse(readFileSync(option('--fixture'),'utf8')),out=resolve(option('--out'));
const repository='link-assistant/formal-ai',[owner,repo]=repository.split('/');
const root=new URL('../../',import.meta.url).pathname;
mkdirSync(out,{recursive:true});

const parent=mkdtempSync(join(tmpdir(),'formal-ai-hive-agent-'));
const workspace=join(parent,'formal-ai');
// A clone, not a worktree: the agent CLI expects `.git` to be a directory.
execFileSync('git',['clone','--quiet','--no-checkout',root,workspace]);
const branchName=`issue-${issueNumber}-local-${process.pid}`;
execFileSync('git',['checkout','--quiet','-b',branchName,at],{cwd:workspace});
if(option('--apply')){
  execFileSync('git',['apply',resolve(option('--apply'))],{cwd:workspace});
  execFileSync('git',['-c','user.name=local','-c','user.email=local@localhost','commit','--quiet','-am','prior run'],{cwd:workspace});
}

// gh answers only `issue view <url> --json title|body --jq .<field>` from the fixture.
const bin=join(parent,'bin');mkdirSync(bin);
const issueUrl=`https://github.com/${repository}/issues/${issueNumber}`;
writeFileSync(join(parent,'issue.json'),JSON.stringify({url:issueUrl,title:fixture.title,body:fixture.objective}));
writeFileSync(join(bin,'gh'),`#!/usr/bin/env node
const fs=require('fs');const a=process.argv.slice(2);fs.appendFileSync(${JSON.stringify(join(out,'gh.log'))},a.join(' ')+'\\n');
const issue=JSON.parse(fs.readFileSync(${JSON.stringify(join(parent,'issue.json'))},'utf8'));
const at=a.indexOf('--json'),field=a[at+1];
if(a[0]!=='issue'||a[1]!=='view'||a[2]!==issue.url||at<0||!['title','body'].includes(field)){console.error('gh is not available in this local run: gh '+a.join(' '));process.exit(1);}
console.log(issue[field]);
`);
chmodSync(join(bin,'gh'),0o755);

const prompts=await import(join(hive,'agent.prompts.lib.mjs'));
const {buildAgentArgs}=await import(join(hive,'agent-command.lib.mjs'));
const {mapModelToId}=await import(join(hive,'agent.lib.mjs'));
const {resolveFormalAiToolExecution}=await import(join(hive,'formal-ai.lib.mjs'));
const argv={model:'formal-ai',tool:'agent',verbose:false};
const params={issueUrl,issueNumber,prNumber:pull,prUrl:`https://github.com/${repository}/pull/${pull}`,branchName,tempDir:workspace,
  workspaceTmpDir:join(parent,'tmp'),isContinueMode:false,forkedRepo:null,feedbackLines:[],owner,repo,argv,contributingGuidelines:''};
const systemPrompt=await prompts.buildSystemPrompt(params),prompt=await prompts.buildUserPrompt(params);
const combinedPrompt=systemPrompt?`${systemPrompt}\n\n${prompt}`:prompt;
if(combinedPrompt.includes(fixture.objective.slice(0,80)))throw Error('the prompt must not carry the task text');
writeFileSync(join(out,'prompt.txt'),combinedPrompt.replaceAll(parent,'<parent>'));

// Hive's runtime checks `gh auth status` itself; only the agent process sees the fixture gh.
const env={...process.env,HIVE_MIND_FORMAL_AI_BASE_URL:baseUrl,HIVE_MIND_FORMAL_AI_HOME_ROOT:join(parent,'home')};
const invocation=await resolveFormalAiToolExecution({tool:'agent',model:argv.model,toolPath:'agent',workdir:workspace,
  log:async line=>writeFileSync(join(out,'hive-runtime.log'),String(line)+'\n',{flag:'a'}),env});
const agentEnv={...env,...invocation.env};
agentEnv.PATH=bin+':'+(agentEnv.PATH??process.env.PATH);
const agentArgs=buildAgentArgs({model:mapModelToId(argv.model),verbose:false});
writeFileSync(join(out,'command.json'),JSON.stringify({command:invocation.command,args:agentArgs,envNames:Object.keys(invocation.env??{}).sort(),backend:invocation.backend??null,
  formalAiVersion:invocation.formalAiVersion??null,at,branchName},null,1));

const child=spawn(invocation.command,agentArgs,{cwd:workspace,env:agentEnv,stdio:['pipe','pipe','pipe']});
const stdout=[],stderr=[];
child.stdout.on('data',chunk=>stdout.push(chunk));child.stderr.on('data',chunk=>stderr.push(chunk));
child.stdin.end(combinedPrompt);
const status=await new Promise(done=>child.on('close',(code,signal)=>done({code,signal})));
writeFileSync(join(out,'agent.stdout.ndjson'),Buffer.concat(stdout).toString('utf8').replaceAll(parent,'<parent>'));
writeFileSync(join(out,'agent.stderr.log'),Buffer.concat(stderr).toString('utf8').replaceAll(parent,'<parent>'));
const diff=execFileSync('git',['diff'],{cwd:workspace,encoding:'utf8'});
const changed=execFileSync('git',['status','--porcelain','--untracked-files=all'],{cwd:workspace,encoding:'utf8'});
writeFileSync(join(out,'workspace.diff'),diff);
writeFileSync(join(out,'workspace.status'),changed);
writeFileSync(join(out,'result.json'),JSON.stringify({exit:status,changed:changed.trim().split('\n').filter(Boolean)},null,1));
console.log(JSON.stringify({exit:status,changed:changed.trim().split('\n').filter(Boolean)}));
await invocation.stop?.();
if(!args.includes('--keep'))rmSync(parent,{recursive:true,force:true});
else console.log('kept',workspace);
