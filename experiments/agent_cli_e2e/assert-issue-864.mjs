// Issue #864 agent-mode execution policy, superseded by #1205/R1207-2:
// failures retain diagnostics without asking an unattended client a question.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {projectServerLog as semanticServerLog} from './server-log-evidence.mjs';
export const COMMAND='issue_864_command_that_does_not_exist';
export const TASK=`Run ${COMMAND}`;

export {semanticServerLog};

export function assertAgentFailure(final){
 if(!final.startsWith('The command failed:'))throw Error('final command-failure diagnostic missing');
 if(!/\bExit code 127\b/u.test(final))throw Error('final exit code 127 missing');
 if(!new RegExp(COMMAND+': (?:command )?not found','u').test(final))throw Error('final missing-command diagnostic missing');
 if(/report\s+issue|would you like|reply\s+[`"']?report/iu.test(final))throw Error('interactive report invitation in unattended agent final');
}

/** Assert evidence from the actual script's Agent CLI and server artifacts. */
export function assertFailureEvidence({task,final,serverRaw,agentStream}){
 if(task.trimEnd()!==TASK)throw Error('original failure task changed');
 assertAgentFailure(final);
 const {semantic,diagnostics,posts}=semanticServerLog(serverRaw);
 const messages=agentStream.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line));
 const assistant=messages.filter(event=>event.type==='message'&&event.role==='assistant').at(-1);
 if(!assistant||!Array.isArray(assistant.content))throw Error('real Agent CLI final assistant event missing');
 const actual=assistant.content.filter(part=>part.type==='text').map(part=>part.text).join('');
 if(actual.trimEnd()!==final.trimEnd())throw Error('extracted final does not match Agent CLI final');
 assertAgentFailure(actual);
 const planned=diagnostics.filter(line=>line.includes('agentic_outcome: planned ToolCalls'));
 if(!planned.some(line=>{
  const match=/arguments: ("(?:\\.|[^"\\])*")/u.exec(line);if(!match)return false;
  try{return JSON.parse(JSON.parse(match[1])).command===COMMAND;}catch{return false;}
 }))throw Error('actual planned command arguments missing or changed');
 if(!diagnostics.some(line=>line.includes('agentic_outcome: planned Final("The command failed:')))throw Error('actual planned final failure missing');
 if(/gh\s+issue\s+create/iu.test(semantic+'\n'+agentStream+'\n'+final))throw Error('issue auto-creation is forbidden');
 if(posts<2)throw Error(`expected a tool-call round trip, got ${posts} chat completion(s)`);
 return {posts,semantic};
}
const read=file=>new TextDecoder('utf-8',{fatal:true}).decode(fs.readFileSync(file));
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const [directory]=process.argv.slice(2);if(!directory)throw Error('evidence directory required');
 const result=assertFailureEvidence({task:read(path.join(directory,'task.txt')),final:read(path.join(directory,'final-answer.txt')),serverRaw:read(path.join(directory,'formal-ai.log')),agentStream:read(path.join(directory,'agent-stream.jsonl'))});
 fs.writeFileSync(path.join(directory,'formal-ai.semantic.log'),result.semantic+'\n');
 console.log(`issue 864 Agent CLI E2E passed: diagnostic failure without interactive invitation (${result.posts} rounds)`);
}
