import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
export async function actualTraceOracle({root,directory,noop=false,unsupported=false,terminalAdapter}){
  const shared=async name=>import(pathToFileURL(path.join(root,name)));
  const {WorkerHost}=await shared('js/server/worker-host.mjs');
  const {installNodeHost}=await shared('js/agentic/node-host.mjs');
  const {loadConversationContext}=await shared('js/server/conversations.mjs');
  const {canonicalCapturedMessages,auditCapturedActions}=await shared('scripts/lib/issue-1364-replay-contract.mjs');
  const {plainText}=await shared('js/agentic/content.mjs');
  await installNodeHost(new WorkerHost());
  const location=path.join(directory,'formal-ai-dialog');assert(fs.statSync(location).isDirectory());
  const files=fs.readdirSync(location);assert(files.length>0&&files.length<=256);
  const ids=new Set(files.map(name=>name.replace(/(?:\.conversation)?\.(?:lino|jsonl)$/u,'')));
  const groups=[],terminals=[];let bytes=0;
  for(const name of files){const stat=fs.lstatSync(path.join(location,name));assert(stat.isFile()&&!stat.isSymbolicLink());bytes+=stat.size;assert(stat.size<=32*1024*1024&&bytes<=64*1024*1024);}
  for(const id of ids){
    assert(/^[A-Za-z0-9_-]+$/u.test(id));
    const context=loadConversationContext(id,{FORMAL_AI_DIALOG_LOG_DIR:location});
    if(!context.server_logs.some(row=>row.method==='POST'&&row.request_body))continue;
    const messages=canonicalCapturedMessages(context);
    if(messages.some(row=>row.role==='assistant'&&row.tool_calls?.length))groups.push(auditCapturedActions(messages));
    // Final response text is taken from actual captured wire response, never solver console speculation.
    const final=context.server_logs.at(-1);
    const finalMessage=[...context.messages].reverse().find(row=>row.role==='assistant'&&!row.tool_calls?.length);
    const text=finalMessage?plainText(finalMessage.content):null;
    terminals.push(terminalAdapter?await terminalAdapter({context,final,messages,text}):explicitUnknownTerminal(text));
  }
  assert(groups.length>0,'actual client tool execution trace required');
  const result={schema:'formal-ai-work-item-actual-trace/v1',groups:groups.length,
    calls:groups.reduce((sum,row)=>sum+row.calls,0),writes:groups.reduce((sum,row)=>sum+row.writes,0),terminals};
  if(noop)assert.equal(result.writes,0,'no transient high-level write/edit/subagent action on repeat');
  if(unsupported)assert(terminals.some(row=>row.disposition==='gap'&&row.verified===false
    &&row.unresolvedEvidence===true),'unsupported task requires explicit unknown evidence terminal from actual response');
  return result;
}
export function explicitUnknownTerminal(text){
  let record;try{record=JSON.parse(text);}catch{return {disposition:'unknown',verified:null,unresolvedEvidence:false};}
  if(!record||typeof record!=='object')return {disposition:'unknown',verified:null,unresolvedEvidence:false};
  const state=record.state&&typeof record.state==='object'?record.state:record;
  const verified=state.verified??state.taskVerified??record.verified??record.taskVerified??null;
  if(record.schema==='repository-work-item-evidence/v1'&&record.verified===false&&record.authored===false
    &&['EvidenceCollected','MissingReadTool','ReadPolicyRefused','IncompleteSourceObservation'].includes(record.reason)){
    const unbound=Array.isArray(record.requirements)?record.requirements.filter(row=>row.status==='unbound'):[];
    return {schema:record.schema,disposition:'gap',verified:false,unresolvedEvidence:unbound.length>0,
      terminal:record.reason,missingEvidenceCount:unbound.length,evidenceKind:'legacy-unbound-requirements',attachmentContentsInferred:false};
  }
  const terminal=state.terminalState??state.terminal_state??state.status??record.reason;
  const gaps=['gap','unresolved','unsupported','evidence_unavailable','MissingEvidence','UnavailableEvidence',
    'IncompleteSourceObservation','MissingReadTool','unsupported_evidence'];
  const missing=[...(Array.isArray(state.missingEvidence)?state.missingEvidence:[]),
    ...(Array.isArray(state.unresolvedEvidence)?state.unresolvedEvidence:[]),
    ...(Array.isArray(state.evidence)?state.evidence.filter(row=>['unavailable','missing','unknown','unresolved'].includes(row.status)):[])];
  return {schema:record.schema??null,disposition:gaps.includes(terminal)?'gap':'unknown',verified,
    unresolvedEvidence:missing.length>0,terminal,missingEvidenceCount:missing.length};
}

/** An explicit unsupported answer is evidence only after the actual client
 * terminates cleanly. Exit5 is the configured no-change outcome, not a crash. */
export function cleanQualificationPass(pass,{unsupported=false}={}){
  return pass?.cleanupComplete===true&&pass.signal===null&&pass.timedOut===false&&pass.error===null
    &&(unsupported?[0,5].includes(pass.status):pass.status===0);
}
