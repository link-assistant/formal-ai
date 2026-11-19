// Exact native Hive inputs; compiler output and network/CI responses are mocks.
// Source observations and optional delivery commands use a bounded real workspace.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {planChatStepResolved} from '../../js/agentic/planner.mjs';
import {appendDefinition,projectAppendContracts} from '../../js/agentic/append_contract.mjs';
import {applyPatchInput} from '../../js/agentic/crate/protocol_responses_apply_patch.mjs';
import {canonicalCreation} from '../../js/agentic/qualified_tool_observation.mjs';

const native=fs.readFileSync('rust/tests/unit/issue_1133_hive_mind_three_runs.rs','utf8');
const rustString=value=>JSON.parse(value.replace(/\\\r?\n[ \t]*/gu,'').replace(/\r?\n/gu,'\\n'));
const constant=name=>rustString(native.match(new RegExp('const '+name+': &str =\\s*("(?:\\\\.|[^"\\\\])*")'))[1]);
let prompt=rustString(native.match(/fn prompt\(\) -> String \{[\s\S]*?format!\(\s*("(?:\\.|[^"\\])*")/u)[1]);
for(const name of ['ISSUE','PR','BRANCH'])prompt=prompt.replaceAll('{'+name+'}',constant(name));
const pageLiteral=rustString(native.match(/fn page\(language: &str\) -> String \{[\s\S]*?format!\(\s*("(?:\\.|[^"\\])*")/u)[1]);
const mode=process.argv[2]||'Scala';
const physicalEdits=process.argv.includes('--physical-edits');
const projectEdits=process.argv.includes('--project-edits');
const physicalDelivery=process.argv.includes('--physical-delivery');
const agentCliReceipts=process.argv.includes('--agent-cli-receipts');
const requestedSteps=process.argv.find(arg=>arg.startsWith('--steps='))?.slice('--steps='.length);
const ceiling=requestedSteps===undefined
  ?(process.argv.includes('--expanded-steps')?32:(mode==='Scala'?8:10)):Number(requestedSteps);
if(!Number.isInteger(ceiling)||ceiling<1||ceiling>32)throw Error('fixture steps must be between 1 and 32');
const tools=mode==='Scala'
  ?['bash','batch','codesearch','edit','glob','grep','list','read','task','todoread','todowrite','webfetch','websearch','write']
  :['shell','apply_patch','update_plan','mcp__codex_apps__github_fetch','mcp__codex_apps__github_search'];
await installNodeHost(new WorkerHost());
const root=fs.mkdtempSync(path.join(os.tmpdir(),'hive-source-fixture-'));
const messages=[{role:'user',content:prompt.replaceAll('Scala',mode)}];
const definitions=tools.filter(name=>['Write','write'].includes(name)).map(appendDefinition);
const steps=[];
const toolEnv={PATH:process.env.PATH,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null'};
function execute(command){
  const result=spawnSync('sh',['-c',command],{
    cwd:root,encoding:'utf8',timeout:2000,maxBuffer:1024*1024,env:toolEnv,
  });
  return {stdout:result.stdout,stderr:result.stderr,exit_code:result.status,
    timed_out:result.error?.code==='ETIMEDOUT',complete:!result.error};
}
function receipt(observed){
  if(!agentCliReceipts)return JSON.stringify(observed);
  return `Output: ${observed.stdout}${observed.stderr?'\nError: '+observed.stderr:''}\nExit Code: ${observed.exit_code}`;
}
function git(...args){
  const result=spawnSync('git',args,{cwd:root,encoding:'utf8',timeout:2000,maxBuffer:1024*1024,env:toolEnv});
  if(result.error||result.status!==0)throw Error('fixture git failed: '+result.stderr);
  return result.stdout.trim();
}
function setupDelivery(){
  git('init','-q','-b',constant('BRANCH'));
  git('config','user.name','Offline fixture');
  git('config','user.email','fixture@example.invalid');
  git('commit','--allow-empty','-qm','Initial');
  git('init','--bare','-q','.git/origin.git');
  git('remote','add','origin','.git/origin.git');
  git('push','-qu','origin','HEAD');
  const bin=path.join(root,'.git','fixture-bin');fs.mkdirSync(bin);
  fs.copyFileSync('rust/tests/fixtures/hive-mind/fake-gh.cjs',path.join(bin,'gh'));
  fs.chmodSync(path.join(bin,'gh'),0o755);
  toolEnv.PATH=bin+path.delimiter+toolEnv.PATH;
  toolEnv.FAKE_GH_STATE=path.join(root,'.git','gh-state.json');
  fs.writeFileSync(toolEnv.FAKE_GH_STATE,JSON.stringify({calls:[],
    issue:{title:'Implement Hello World in '+mode,body:pageLiteral.replaceAll('{language}',mode)},
    branch:constant('BRANCH'),pr:{title:'[WIP] Implement Hello World',body:'Work in progress',isDraft:true,url:constant('PR')},
  }));
}
try {
  if(physicalDelivery)setupDelivery();
  for(let turn=0;turn<ceiling;turn++){
    const started=Date.now();
    const plan=await planChatStepResolved(projectAppendContracts(messages,definitions),tools);
    if(plan?.kind!=='tool_calls'){
      steps.push({turn,elapsedMs:Date.now()-started,plan});break;
    }
    const call=plan.calls[0],args=JSON.parse(call.arguments),id='c'+turn;
    const wireArguments=projectEdits&&call.tool==='apply_patch'
      ?(applyPatchInput(call.arguments)??call.arguments):call.arguments;
    const creation=projectEdits&&call.tool==='apply_patch'?canonicalCreation(wireArguments):null;
    let content='',observed;
    if(args.command?.includes('workspace-source-v1')){
      observed=execute(args.command);content=receipt(observed);
    } else if((['Write','write'].includes(call.tool)
      ||physicalEdits&&call.tool==='apply_patch'&&typeof args.content==='string')
      &&args.append_mode!=='atomic_record_append'){
      const source=creation??args;
      const target=path.resolve(root,source.path);
      if(!target.startsWith(root+path.sep))throw Error('fixture path escapes workspace');
      fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,source.content);
      if(fs.readFileSync(target,'utf8')!==source.content)throw Error('fixture write was not observed');
      content=JSON.stringify({success:true,path:source.path,bytes:Buffer.byteLength(source.content)});
    } else if(call.tool==='webfetch'||args.command?.startsWith('gh issue view ')){
      content=pageLiteral.replaceAll('{language}',mode);
    } else if(mode==='Scala'&&call.tool==='bash'){
      content='/bin/sh: 1: scalac: not found\n';
    } else if(physicalDelivery&&['bash','shell'].includes(call.tool)&&args.command!=='sh tests/verify-output.sh'){
      observed=execute(args.command);content=receipt(observed);
    } else if(mode!=='Scala'&&call.tool==='shell'){
      content=args.command?.includes('git push')?'0123abcd\n':'Hello, World!\n';
    }
    steps.push({turn,elapsedMs:Date.now()-started,call,wireArguments,content});
    messages.push({role:'assistant',content:'',tool_calls:[
      {id,type:'function',function:{name:call.tool,arguments:wireArguments}},
    ]},{role:'tool',name:call.tool,tool_call_id:id,content,is_error:observed?observed.exit_code!==0:false});
  }
  const outcome=physicalDelivery?{
    status:git('status','--porcelain'),head:git('rev-parse','HEAD'),
    pushedHead:git('--git-dir=.git/origin.git','rev-parse',constant('BRANCH')),
    pushed:git('--git-dir=.git/origin.git','ls-tree','-r','--name-only',constant('BRANCH')).split('\n'),
    commits:Number(git('rev-list','--count','HEAD')),
    state:JSON.parse(fs.readFileSync(toolEnv.FAKE_GH_STATE,'utf8')),
  }:null;
  console.log(JSON.stringify({mode,physicalEdits,projectEdits,physicalDelivery,agentCliReceipts,ceiling,
    steps,messages,outcome,compilerAndNetworkAreMocks:true,fullIssueComplete:false},null,2));
}finally{fs.rmSync(root,{recursive:true,force:true});}
