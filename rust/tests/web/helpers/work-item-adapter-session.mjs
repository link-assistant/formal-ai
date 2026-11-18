// Issue #1483: drive the boundary-repair work item through the Claude
// (Anthropic Messages) and Codex (Responses) adapters without an LLM,
// executing each returned tool call against a scratch workspace.
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,chmodSync} from 'node:fs';
import {join,isAbsolute} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../../../js/server/worker-host.mjs';
import {handleAnthropicMessages} from '../../../../js/server/anthropic.mjs';
import {createResponse} from '../../../../js/server/responses.mjs';
import {parseResponsesRequest} from '../../../../js/server/responses-input.mjs';
import {createMemory} from '../../../../js/server/memory.mjs';
import {ensureNodeHost} from '../../../../js/server/node-host-install.mjs';

const source=`// Keep this comment.
export function accepts(value,minimum,maximum) {
  return value >= minimum && value < maximum;
}
export function untouched(value) { return value * 3; }
`;
export const boundaryTask='Repair the upper-bound check in src/ceiling.mjs so the maximum is included. Preserve the lower bound, comments and other functions.';
const object=(properties,required)=>({type:'object',properties,required,additionalProperties:false});
const text={type:'string'};
export function adapterTools(protocol){
  if(protocol==='claude')return[
    {name:'Read',input_schema:object({file_path:text},['file_path'])},
    {name:'Edit',input_schema:object({file_path:text,old_string:text,new_string:text},['file_path','old_string','new_string'])},
    {name:'Write',input_schema:object({file_path:text,content:text},['file_path','content'])},
    {name:'Bash',input_schema:object({command:text},['command'])}];
  return[{type:'function',name:'exec_command',parameters:object({cmd:text},['cmd'])},
    {type:'custom',name:'apply_patch',description:'Apply a source patch in the client workspace'}];
}
function resolveIn(directory,path){return isAbsolute(path)?path:join(directory,path);}
function shell(directory,command,env){
  const run=spawnSync('bash',['-c',command],{cwd:directory,encoding:'utf8',timeout:60000,env:env??process.env});
  return {status:run.status??1,stdout:run.stdout??'',stderr:run.stderr??''};
}
/** Applies one Codex `*** Update File`/`*** Add File` patch with exact hunks. */
function applyPatch(directory,input){
  const lines=input.replace(/\n$/u,'').split('\n');
  if(lines[0]!=='*** Begin Patch'||lines.at(-1)!=='*** End Patch')return {status:1,stdout:'',stderr:'malformed patch'};
  let index=1;const changed=[];
  while(index<lines.length-1){
    const header=lines[index++];
    const add=header.match(/^\*\*\* Add File: (.+)$/u),update=header.match(/^\*\*\* Update File: (.+)$/u);
    if(add){const body=[];while(index<lines.length-1&&!lines[index].startsWith('*** '))body.push(lines[index++].slice(1));
      writeFileSync(resolveIn(directory,add[1]),body.join('\n')+'\n');changed.push(add[1]);continue;}
    if(!update)return {status:1,stdout:'',stderr:'unsupported patch header: '+header};
    const file=resolveIn(directory,update[1]);let content=readFileSync(file,'utf8');
    while(index<lines.length-1&&!lines[index].startsWith('*** ')){
      if(lines[index].startsWith('@@')){index++;continue;}
      const before=[],after=[];
      while(index<lines.length-1&&!lines[index].startsWith('@@')&&!lines[index].startsWith('*** ')){
        const line=lines[index++];const body=line.slice(1);
        if(line[0]!=='+')before.push(body);if(line[0]!=='-')after.push(body);
      }
      const from=before.join('\n'),to=after.join('\n');
      if(!content.includes(from))return {status:1,stdout:'',stderr:'hunk does not apply'};
      content=content.replace(from,to);
    }
    writeFileSync(file,content);changed.push(update[1]);
  }
  return {status:0,stdout:'Done!\n'+changed.map(path=>'M '+path).join('\n')+'\n',stderr:''};
}
function execute(directory,call,env){
  const {name,args}=call;
  if(name==='Read'){try{return {status:0,stdout:readFileSync(resolveIn(directory,args.file_path),'utf8').split('\n').map((line,i)=>String(i+1).padStart(6)+'\t'+line).join('\n'),stderr:''};}catch(error){return {status:1,stdout:'',stderr:String(error.message)};}}
  if(name==='Edit'){const file=resolveIn(directory,args.file_path);const content=readFileSync(file,'utf8');
    if(!content.includes(args.old_string))return {status:1,stdout:'',stderr:'old_string not found'};
    writeFileSync(file,content.replace(args.old_string,args.new_string));return {status:0,stdout:`The file ${args.file_path} has been updated.`,stderr:''};}
  if(name==='Write'){writeFileSync(resolveIn(directory,args.file_path),args.content);return {status:0,stdout:`File created successfully at: ${args.file_path}`,stderr:''};}
  if(name==='Bash')return shell(directory,args.command,env);
  if(name==='exec_command')return shell(directory,args.cmd,env);
  if(name==='apply_patch')return applyPatch(directory,args.input);
  return {status:1,stdout:'',stderr:'unknown tool '+name};
}
async function next(context,protocol,history,tools,system){
  if(protocol==='claude'){
    const response=await handleAnthropicMessages(context,{body:JSON.stringify({model:'formal-ai',system,messages:history,tools,max_tokens:4096})});
    if(response.statusCode!==200)throw Error(response.body);
    const message=JSON.parse(response.body);
    return {final:message.content.filter(block=>block.type==='text').map(block=>block.text).join('\n'),
      calls:message.content.filter(block=>block.type==='tool_use').map(block=>({id:block.id,name:block.name,args:block.input})),wire:message};
  }
  const response=await createResponse(context,parseResponsesRequest(JSON.stringify({model:'formal-ai',instructions:system,input:history,tools})));
  return {final:response.output.filter(item=>item.type==='message').flatMap(item=>item.content).map(item=>item.text??'').join('\n'),
    calls:response.output.filter(item=>['function_call','custom_tool_call'].includes(item.type)).map(item=>({id:item.call_id,name:item.name,
      args:item.type==='custom_tool_call'?{input:item.input}:JSON.parse(item.arguments),custom:item.type==='custom_tool_call'})),wire:response.output};
}
function append(protocol,history,turn,results){
  if(protocol==='claude'){history.push({role:'assistant',content:turn.wire.content},{role:'user',content:results.map(({call,result})=>({type:'tool_result',tool_use_id:call.id,
    content:result.status===0?result.stdout:result.stdout+result.stderr+`\nExit code ${result.status}`,is_error:result.status!==0}))});return;}
  history.push(...turn.wire,...results.map(({call,result})=>({type:call.custom?'custom_tool_call_output':'function_call_output',call_id:call.id,
    output:call.custom?result.stdout+result.stderr:JSON.stringify({exit_code:result.status,stdout:result.stdout,stderr:result.stderr})})));
}
/** Runs one adapter session to its final message; returns the tool trail and the final text. */
export async function adapterSession(protocol,directory,task,{steps=12,tools:only=null,system:clientSystem='',env=null}={}){
  const context={worker:new WorkerHost(),agentMode:true,memory:createMemory({...process.env,FORMAL_AI_MEMORY_PATH:join(directory,'..','memory-'+protocol+'.lino')})};
  await ensureNodeHost(context,{sourceUsageRustGrammar:false});
  const tools=adapterTools(protocol).filter(tool=>only===null||only.includes(tool.name));
  // The client states its working directory in its system prompt, as Claude Code and Codex do.
  // A Hive system prompt (when given) precedes that block, as the client sends both.
  const system=(clientSystem?clientSystem+'\n\n':'')+`<env>\nWorking directory: ${directory}\n</env>`;
  const history=protocol==='claude'?[{role:'user',content:task}]:[{role:'user',content:[{type:'input_text',text:task}]}];
  const trail=[];
  for(let step=0;step<steps;step++){
    const turn=await next(context,protocol,history,tools,system);
    if(!turn.calls.length)return {final:turn.final,trail};
    const results=turn.calls.map(call=>({call,result:execute(directory,call,env)}));
    for(const {call,result} of results)trail.push({tool:call.name,status:result.status,args:call.args});
    append(protocol,history,turn,results);
  }
  return {final:null,trail};
}
export function boundaryWorkspace(){
  const root=mkdtempSync(join(tmpdir(),'formal-ai-adapter-'));const directory=join(root,'workspace');
  mkdirSync(join(directory,'src'),{recursive:true});writeFileSync(join(directory,'src/ceiling.mjs'),source);
  return {root,directory,dispose:()=>rmSync(root,{recursive:true,force:true})};
}
const hiveFixtures=new URL('../../../../data/benchmarks/fixtures/work-item-1483/',import.meta.url).pathname;
/** The ordinary Hive Mind system and user prompts of `protocol` (recorded from
 * hive-mind 2.39.10), filled for an issue whose text is not in the prompt. */
export function hivePrompts(protocol,values){
  const fill=text=>text.replace(/\{(issue_url|issue|pull_url|pull|branch|workspace_tmp|workspace|owner|repo)\}/gu,(_,key)=>String(values[key]));
  return {system:fill(readFileSync(join(hiveFixtures,`hive-${protocol}-system.txt`),'utf8')),user:fill(readFileSync(join(hiveFixtures,`hive-${protocol}-user.txt`),'utf8'))};
}
/** A `gh` on PATH that answers `gh issue view <url> --json <field> --jq .<field>`
 * from `issue`, as GitHub would, and logs each invocation; nothing leaves the host. */
export function ghShim(root,issue){
  const bin=join(root,'bin'),log=join(root,'gh.log');mkdirSync(bin,{recursive:true});
  writeFileSync(join(root,'issue.json'),JSON.stringify(issue));
  writeFileSync(join(bin,'gh'),`#!/usr/bin/env node
const fs=require('fs');const a=process.argv.slice(2);fs.appendFileSync(${JSON.stringify(log)},a.join(' ')+'\\n');
const issue=JSON.parse(fs.readFileSync(${JSON.stringify(join(root,'issue.json'))},'utf8'));
const at=a.indexOf('--json'),field=a[at+1];
if(a[0]!=='issue'||a[1]!=='view'||at<0||!(field in issue)){console.error('unsupported: gh '+a.join(' '));process.exit(1);}
console.log(issue[field]);
`);
  chmodSync(join(bin,'gh'),0o755);
  return {env:{...process.env,PATH:bin+':'+process.env.PATH},calls:()=>readFileSync(log,'utf8').trim().split('\n')};
}
