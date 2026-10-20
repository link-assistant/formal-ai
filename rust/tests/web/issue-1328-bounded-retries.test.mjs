import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {performance} from 'node:perf_hooks';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {repeatedActionStop} from '../../../js/agentic/repetition_guard.mjs';
import {Progress} from '../../../js/agentic/progress.mjs';
import {Capability} from '../../../js/agentic/capability.mjs';
import {normalizePrompt} from '../../../js/agentic/crate/engine.mjs';
import {responseArgumentsForTool} from '../../../js/agentic/crate/protocol_responses.mjs';
import {invalidArguments} from '../../../js/agentic/crate/tool_call_validation.mjs';
import {chatCompletionFromPlan, responseFromPlan} from '../../../js/server/agentic.mjs';
import {responsesInputMessages} from '../../../js/server/responses-input.mjs';
import {repairStep, failedStep} from '../../../js/agentic/repair_loop.mjs';
import {CURRENT_BRANCH_PUSH} from '../../../js/agentic/git_push_recovery.mjs';
import {composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {targetOf, recipeCommitCommand} from '../../../js/agentic/git_commit.mjs';
import {createLogger, decodeTyped} from '../../../js/server/structured-log.mjs';
import {downloadWithRetry} from '../../../desktop/scripts/prefetch-builder-toolsets.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
const namespace = 'mcp__codex_apps__search_service';
const name = '_web_run';
const tool = `${namespace}__${name}`;
const adapterError = {content:[{type:'text', text:'InvalidActionArgumentsError: Provide either an input string or top-level Search commands, without combining them.'}], structuredContent:{error:'InvalidActionArgumentsError',error_code:'INVALID_ARGUMENT',error_data:{reason:'missing_required_argument',failure_stage:'argument_binding'}}, isError:true};
const user = {role:'user', content:'Fix the failing coding task.'};
const schema = {type:'object', properties:{input:{type:'string'}}, required:['input'], additionalProperties:false};
const tools = [{type:'namespace',name:namespace,tools:[{type:'function',name,parameters:schema}]}];
function replay(count, {unknown=false, missing=false}={}) {
  const items = [{type:'message', ...user}];
  for (let i=0;i<count;i++) {
    items.push({type:'function_call',call_id:'same-client-id',name:unknown?'unknown':name,namespace,arguments:i%2?' { } ':'{}'});
    if (!missing) items.push({type:'function_call_output',call_id:'same-client-id',output:adapterError});
  }
  return responsesInputMessages(items);
}

test('Responses round trip retains the namespace and recognizes the real MCP binding error', () => {
  const messages = replay(1);
  assert.equal(messages[1].tool_calls[0].function.name,tool);
  assert.equal(messages[2].name,tool);
  assert.equal(Progress.scan(messages).done(Capability.Search),true);
  assert.equal(Progress.scan(messages).search_output,null);
});

test('same call ID, unknown error shape, missing receipts and JSON layout cannot reopen the action budget', async () => {
  for (const options of [{},{unknown:true},{missing:true},{unknown:true,missing:true}]) {
    assert.equal(repeatedActionStop(replay(2,options)),null);
    const messages = replay(3,options);
    assert.match(repeatedActionStop(messages),/after 3 attempts/);
    const planned = await planChatStep(messages,['bash',tool]);
    assert.equal(planned.kind,'final');
    assert.match(planned.answer,/task is incomplete/);
  }
});

test('canonical key order counts as one action, distinct tools and operands keep their own budget', () => {
  const messages=[user];
  for (let i=0;i<3;i++) messages.push({role:'assistant',content:'',tool_calls:[{id:String(i),function:{name:'unknown',arguments:i%2?'{"b":2,"a":1}':'{"a":1,"b":2}'}}]});
  assert.match(repeatedActionStop(messages),/after 3 attempts/);
  messages[3].tool_calls[0].function.arguments='{"a":2,"b":2}';
  assert.equal(repeatedActionStop(messages),null);
});

test('continuation preserves exhaustion; a new explicit source read opens its proper window', () => {
  const messages=replay(3);
  assert.match(repeatedActionStop([...messages,{role:'user',content:'continue'}]),/Repeated action stopped/);
  assert.equal(repeatedActionStop([...messages,{role:'assistant',content:'The attempt failed.'},{role:'user',content:'Read the current contents of fresh.txt.'}]),null);
});

test('the captured 161-call shape with 277 KB fixed context terminates before expensive planning', async () => {
  const messages=replay(161);messages[0]={role:'user',content:'Fix the failing coding task. '+ 'stable harness context '.repeat(14000)};
  const start=performance.now(),plan=await planChatStep(messages,['bash',tool]);
  assert.equal(plan.kind,'final');assert.match(plan.answer,/after 161 attempts/);
  assert.ok(performance.now()-start<5000,'bounded replay must stop promptly');
});

test('search query is bound to the advertised input property on both wire surfaces', () => {
  const args=JSON.stringify({query:'rust src refspec so does not match any'});
  assert.deepEqual(JSON.parse(responseArgumentsForTool(tools,tool,args,'',null)),{input:'rust src refspec so does not match any'});
  const request={messages:[user],tools}, plan={kind:'tool_calls',calls:[{tool,arguments:args}]};
  const chat=chatCompletionFromPlan({},request,user.content,plan,0);
  assert.deepEqual(JSON.parse(chat.choices[0].message.tool_calls[0].function.arguments),{input:'rust src refspec so does not match any'});
  const response=responseFromPlan({},request,request,user.content,plan,0);
  const call=response.output.find(item=>item.type==='function_call');
  assert.equal(call.namespace,namespace);assert.equal(call.name,name);
  assert.deepEqual(JSON.parse(call.arguments),{input:'rust src refspec so does not match any'});
});

test('unbound nested required values produce a terminal failure, no empty call, and actionable diagnostics', () => {
  const events=[],ctx={logger:{log:(...args)=>events.push(args)}};
  const nested={type:'object',properties:{options:{type:'object',properties:{owner:{type:'string'}},required:['owner']}},required:['options']};
  const request={messages:[user],tools:[{type:'function',name:'unknown',parameters:nested}]};
  const plan={kind:'tool_calls',calls:[{tool:'unknown',arguments:'{"options":{}}'}]};
  const chat=chatCompletionFromPlan(ctx,request,user.content,plan,0);
  assert.equal(chat.choices[0].finish_reason,'stop');assert.deepEqual(chat.choices[0].message.tool_calls,[]);
  assert.match(chat.choices[0].message.content,/No call was sent/);
  const response=responseFromPlan(ctx,request,request,user.content,plan,0);
  assert.ok(!response.output.some(item=>item.type==='function_call'));
  assert.ok(events.some(event=>event[1]==='tool_argument_binding_failed'&&event[2].schema===nested));
});

test('schema types, combinations, bounds and unresolved references are checked after binding', () => {
  for(const [value,contract]of [[{},schema],[{input:''},schema],[{input:4},schema],['x',{enum:['y']}],['x',{const:'y'}],['x',{pattern:'^y$'}],[0,{minimum:1}],[[],{minItems:1}],[{},{$ref:'#/$defs/unknown'}],[{input:'ok'},{oneOf:[schema,schema]}]]) assert.ok(invalidArguments(value,contract));
  assert.equal(invalidArguments({input:'ok'},{anyOf:[schema,{type:'number'}]}),null);
});

test('failed and empty search receipts consume the repair phase rather than repeating it', () => {
  const failure=failedStep('rust','error[E0425]: cannot find value `missing` in this scope\n --> main.rs:1:1',{failed_command:'rustc main.rs',artifact_path:'main.rs'});
  assert.equal(repairStep([user],['web_search','bash'],failure,0,3).kind,'search');
  for(const result of [adapterError,'']) {
    const messages=responsesInputMessages([{type:'message',...user},{type:'function_call',call_id:'1',name:'web_search',arguments:'{"query":"error"}'},{type:'function_call_output',call_id:'1',output:result}]);
    assert.deepEqual(repairStep(messages,['web_search','bash'],failure,0,3),{kind:'stop',reason:'no_match'});
  }
});

test('local refspec failure re-reads Git branch state before considering web research', () => {
  const failure=failedStep('rust','error: src refspec so does not match any',{failed_command:"git push -q origin 'so'"});
  const result=repairStep([user],['bash',tool],failure,0,3);
  assert.equal(result.kind,'retry');assert.equal(result.plan.calls[0].tool,'bash');
  assert.equal(JSON.parse(result.plan.calls[0].arguments).command,CURRENT_BRANCH_PUSH);
  assert.match(CURRENT_BRANCH_PUSH,/git branch --show-current/);
});

test('Git recovery pushes checked-out content and refuses detached HEAD', () => {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-1328-git-'));
  try {
    const remote=path.join(dir,'remote.git'),work=path.join(dir,'work');fs.mkdirSync(work);
    const git=(cwd,...args)=>execFileSync('git',args,{cwd,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
    git(dir,'init','--bare',remote);git(work,'init','-b','issue-1-current');
    git(work,'config','user.name','Formal AI fixture');git(work,'config','user.email','fixture@example.invalid');
    fs.writeFileSync(path.join(work,'main.txt'),'hello\n');git(work,'add','main.txt');git(work,'-c','commit.gpgsign=false','commit','-m','fixture');git(work,'remote','add','origin',remote);
    assert.throws(()=>git(work,'push','origin','so'));
    execFileSync('/bin/sh',['-c',CURRENT_BRANCH_PUSH],{cwd:work,stdio:'pipe'});
    assert.equal(git(dir,'--git-dir='+remote,'rev-parse','refs/heads/issue-1-current'),git(work,'rev-parse','HEAD'));
    git(work,'checkout','--detach');assert.throws(()=>execFileSync('/bin/sh',['-c',CURRENT_BRANCH_PUSH],{cwd:work,stdio:'pipe'}));
  } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('default commit template observes the branch and prose branch so never becomes a ref', () => {
  const target=targetOf('Solve https://github.com/example/project/issues/1. Commit each atomic step to the pull request branch so interrupted work remains preserved.');
  assert.ok(target);assert.equal(target.branch,null);
  const command=recipeCommitCommand({path:'main.rs',source:'',supporting_files:[]},target);
  assert.match(command,/git branch --show-current/);assert.ok(!command.includes("origin 'so'"));
});

test('normalization preserves multilingual content and common contractions across cached requests', () => {
  const prompt="You're coding C++ and C#. Привет दुनिया 你好!";
  const expected='you are coding cpp and csharp привет दुनिया 你好';
  assert.equal(normalizePrompt(prompt),expected);
  for(let i=0;i<40;i++) normalizePrompt(`another prompt ${i}`);
  assert.equal(normalizePrompt(prompt),expected);
});

test('unwritable logs report their real cause once, expose health and preserve stderr diagnostics', () => {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-1328-log-')),output=[];let opens=0;
  try {
    const error=Object.assign(new Error('Permission denied: secret-value'),{code:'EACCES'});
    const logger=createLogger({directory:dir,env:{API_TOKEN:'secret-value'},openFile:()=>{opens++;throw error;},stderrWrite:bytes=>{output.push(bytes.toString());return bytes.length;}});
    for(let i=0;i<8;i++)logger.log('debug','still_observable',{attempt:i});
    assert.equal(opens,1);assert.equal(logger.healthStatus().status,'unwritable');
    assert.equal(logger.healthStatus().error_kind,'EACCES');assert.match(logger.healthStatus().error,/Permission denied/);
    const records=output.map(line=>decodeTyped(line.trim()));
    assert.equal(records.filter(record=>record.kind==='logging_failure').length,1);
    assert.equal(records.filter(record=>record.kind==='still_observable').length,8);
    assert.ok(!output.join('').includes('secret-value'));
  } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('non-finite retry configuration is rejected before downloading', async () => {
  for(const options of [{attempts:Infinity},{attempts:NaN},{attempts:0},{attempts:1.5},{attempts:9},{attempts:Number.MAX_SAFE_INTEGER},{retryDelayMs:60001},{totalTimeoutMs:900001},{totalTimeoutMs:Infinity},{stallTimeoutMs:0},{retryDelayMs:-1}]) {
    await assert.rejects(downloadWithRetry('https://invalid.example/test','unused',{...options,fetchImpl:()=>assert.fail('IO must not start')}),RangeError);
  }
});

test('wire binding refuses invalid explicit values and ungrounded required numbers rather than changing their meaning', () => {
  for(const [parameters,args] of [
    [{type:'object',properties:{input:{type:'string',enum:['safe']}},required:['input']},'{"input":"destructive"}'],
    [{type:'object',properties:{input:{type:'array',minItems:1,items:{type:'string'}}},required:['input']},'{"input":[]}'],
    [{type:'object',properties:{unboundCount:{type:'integer'}},required:['unboundCount']},'{}'],
  ]) {
    const request={messages:[user],tools:[{type:'function',name:'unknown',parameters}]};
    const response=chatCompletionFromPlan({},request,user.content,{kind:'tool_calls',calls:[{tool:'unknown',arguments:args}]},0);
    assert.equal(response.choices[0].finish_reason,'stop');
  }
});

test('a bug report cannot turn its quoted reproduction and later output discussion into a completed file task', () => {
  for (const text of [
    "0.354.1 loops for hours\n\nA Hive Mind run, `solve example/issues/1`, did nothing.\n\nThe output failed. Write a fix; code references are at v0.354.1.",
    "A run `solve example/issues/1` failed. The output shows write failures at v0.354.1.",
    "An execution failed. Run 'printf observed' and write its stdout to reports/observed.txt.",
    "Execute the task.\n\nRun 'printf observed' and write its stdout to reports/observed.txt.",
    "Execute the task.\r\n\r\nRun 'printf observed' and write its stdout to reports/observed.txt.",
    "Execute the task.\n \n  Run 'printf observed' and write its stdout to reports/observed.txt.",
    "Run 'printf observed' to reproduce.\r\n\r\nOther context: write its output to unrelated.txt.",
    "Run 'printf observed' to reproduce.\n \nOther context: write its output to unrelated.txt.",
    "Run `printf previous` to reproduce. Run 'printf observed' and write its stdout to reports/observed.txt.",
    "Run `printf observed` to reproduce a failure.\n\nOther context: write its output to unrelated.txt.",
  ]) {
    const plan=composeGeneralChangePlan(text);
    assert.notEqual(plan?.mode,'command_output');
    assert.notEqual(plan?.target,'v0.354.1');
  }
  const legitimate=composeGeneralChangePlan("Please run 'printf observed' and write its stdout to reports/observed.txt");
  assert.equal(legitimate?.mode,'command_output');
  for(const prefix of ['Execute the auto-learning task. ', 'Please execute the requested task. ', 'Execute the requested task.\n  ']) {
    const request=prefix+"Run 'printf observed' and write its exact stdout to reports/observed.txt";
    const owned=composeGeneralChangePlan(request);
    assert.equal(owned?.mode,'command_output');assert.equal(owned?.target,'reports/observed.txt');
    assert.equal(owned.content,'');assert.equal(owned.goal,request);
  }
});

test('distinct-call cycles and unbounded driving options are also finite', async () => {
  const messages=[user];
  for(let i=0;i<128;i++)messages.push({role:'assistant',content:'',tool_calls:[{id:String(i),function:{name:'unknown',arguments:JSON.stringify({index:i})}}]});
  assert.match(repeatedActionStop(messages),/Action budget exhausted/);
  for(const steps of [Infinity,NaN,-1,1.5])await assert.rejects(drive(()=>assert.fail('planner must not run'),'/private/tmp/unused','task',{steps}),RangeError);
});


test('schema combinations share one predicate and string lengths count Unicode characters', () => {
  assert.equal(invalidArguments('😀',{type:'string',minLength:1,maxLength:1}),null);
  assert(invalidArguments('😀',{type:'string',minLength:2}));
  assert.equal(invalidArguments('x',{allOf:[{type:'string'},{const:'x'}],anyOf:[{const:'x'},{const:'y'}],oneOf:[{const:'x'},{type:'number'}]}),null);
  assert(invalidArguments('x',{allOf:[{const:'y'}]}));
  assert(invalidArguments('x',{not:{const:'x'}}));
});


test('refspec recovery preserves explicit branch and remote intent', () => {
  const failure=failedStep('rust','error: src refspec missing does not match any',{failed_command:'git push -q origin missing'});
  const bound=repairStep([{role:'user',content:'Push to branch intended-destination.'}],['bash'],failure,0,3);
  assert.equal(bound.kind,'retry');assert.equal(bound.plan.kind,'final');assert.match(bound.plan.answer,/explicit branch binding/);
  const other=repairStep([user],['bash'],{...failure,failed_command:'git push upstream missing'},0,3);
  assert.equal(other.kind,'retry');assert.equal(other.plan.kind,'final');assert.match(other.plan.answer,/originally requested remote/);
});


test('the actual generated recipe commit pushes plain named and current refs without escaped quote bytes', () => {
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-recipe-ref-'));
  try {
    const remote=path.join(directory,'remote.git'),work=path.join(directory,'work');fs.mkdirSync(work);
    const git=(args,cwd=work)=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
    git(['init','--bare','-q',remote],directory);git(['init','-q','-b','task/quoted-ref']);
    git(['config','user.name','Test']);git(['config','user.email','test@example.com']);
    fs.writeFileSync(path.join(work,'README.md'),'seed\n');git(['add','README.md']);git(['commit','-qm','seed']);git(['remote','add','origin',remote]);
    const file='generated source.txt';
    for(const [index,branch]of [null,'task/quoted-ref'].entries()) {
      const source='verified fixture '+index+'\n';fs.writeFileSync(path.join(work,file),source);
      const recipe={path:file,source,language:'text',supporting_files:[],commands:[]};
      const command=recipeCommitCommand(recipe,{branch,reference:'https://github.com/example/repo/issues/17',bindingRefused:false});
      execFileSync('sh',['-c',command],{cwd:work,encoding:'utf8'});
      assert.equal(git(['rev-parse','HEAD']),git(['rev-parse','refs/heads/task/quoted-ref'],remote));
      assert.equal(git(['show','refs/heads/task/quoted-ref:'+file],remote),source.trim());
    }
    assert.deepEqual(git(['for-each-ref','--format=%(refname)'],remote).split('\n'),['refs/heads/task/quoted-ref']);
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('provider-owned uppercase schema type names retain values and refuse actual type mismatches', () => {
  const schema={type:'OBJECT',properties:{command:{type:'STRING'}},required:['command']};
  assert.equal(invalidArguments({command:'mytool --flag'},schema),null);
  assert.match(invalidArguments({command:7},schema),/type mismatch/);
  assert.match(invalidArguments({},schema),/required value is unbound/);
});

test('canonical schema equality ignores object order, preserves reserved keys and retains array order', () => {
  const left=JSON.parse('{"__proto__":{"payload":1},"nested":{"second":2,"first":1}}');
  const right=JSON.parse('{"nested":{"first":1,"second":2},"__proto__":{"payload":1}}');
  assert.equal(invalidArguments(left,{enum:[right]}),null);
  assert.equal(invalidArguments(left,{const:right}),null);
  assert.match(invalidArguments({nested:right.nested},{const:right}),/const mismatch/);
  assert.match(invalidArguments([2,1],{const:[1,2]}),/const mismatch/);
  const original=[user];
  for(let index=0;index<3;index++)original.push({role:'assistant',content:'',tool_calls:[{id:String(index),function:{name:'opaque',arguments:JSON.stringify(index===1?right:left)}}]});
  assert.match(repeatedActionStop(original),/after 3 attempts/);
  const distinct=[user];
  for(let index=0;index<3;index++)distinct.push({role:'assistant',content:'',tool_calls:[{id:String(index),function:{name:'opaque',arguments:'{"__proto__":{"payload":'+index+'}}'}}]});
  assert.equal(repeatedActionStop(distinct),null);
});

test('object property schemas use declared keys rather than inherited prototype members', () => {
  for(const key of ['__proto__','constructor','toString']) {
    const value=JSON.parse('{"'+key+'":1}');
    assert.match(invalidArguments(value,{type:'object',properties:{},additionalProperties:false}),/forbidden value/);
    const properties=JSON.parse('{"'+key+'":{"type":"number"}}');
    assert.equal(invalidArguments(value,{type:'object',properties,additionalProperties:false}),null);
  }
});
