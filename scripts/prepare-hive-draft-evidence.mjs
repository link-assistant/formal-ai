// Stage only this CI session's complete text files; never follow links or inspect the home directory.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {makeRedactor} from '../js/server/structured-log.mjs';
import {parseJson,toCompactJson} from '../js/server/json.mjs';
import {encodeDialogRecord,decodeDialogRecord} from '../js/server/dialog-codec.mjs';
import {redact as redactCredentialsAndPersonalData} from './collect-ci-failures.mjs';
export function prepareDraftEvidence({source,destination,env=process.env,maxFiles=2048,maxFileBytes=67108864,maxBytes=134217728}) {
 assert(path.isAbsolute(source)&&path.isAbsolute(destination));
 const relative=path.relative(source,destination);
 assert(relative&&relative.startsWith('..'+path.sep),'staging must be outside the session directory');
 for(const [value,maximum] of [[maxFiles,2048],[maxFileBytes,67108864],[maxBytes,134217728]])
  assert(Number.isSafeInteger(value)&&value>0&&value<=maximum);
 fs.mkdirSync(destination,{recursive:false,mode:0o700});
 const scrub=makeRedactor(env,{preserveJsonNumbers:true,transformText:redactCredentialsAndPersonalData}),files=[],unavailable=[],pending=[''];let bytes=0,visited=0;
 const safe=text=>scrub(text);
 const safeContent=(name,text)=>{
  const parsed=input=>toCompactJson(scrub(parseJson(input,{unboundedIntegers:true})));
  const recover=(input,convert)=>{
   try{return convert(input);}catch{
    if(!unavailable.some(row=>row.path===safe(name)&&row.reason==='malformed_structured_capture'))
     unavailable.push({path:safe(name),reason:'malformed_structured_capture'});
    return safe(input);
   }
  };
  if(name.endsWith('.json'))return recover(text,parsed)+'\n';
  if(name.endsWith('.jsonl')||(name.endsWith('.log')&&text.startsWith('{"stream":')))return text.split('\n').map(line=>line?recover(line,parsed):'').join('\n');
  if(name.endsWith('.lino')&&text.startsWith('(formal-ai-log-v1 '))
   return text.split('\n').map(line=>line?recover(line,value=>encodeDialogRecord(scrub(decodeDialogRecord(value)))):'').join('\n');
  return safe(text);
 };
 const receipt={schema:'hive-draft-session-capture/v1',runId:env.GITHUB_RUN_ID??null,
  runAttempt:env.GITHUB_RUN_ATTEMPT??null,workflowSource:env.GITHUB_SHA??null,
  buildOutcome:env.DRAFT_BUILD_OUTCOME??null,attemptOutcome:env.DRAFT_ATTEMPT_OUTCOME??null,
  files,unavailable,redaction:'known environment credentials, credential patterns and personal-data patterns',
  rawPrivateFilesIncluded:false,taskAcceptance:false};
 try {
  assert(!fs.lstatSync(source).isSymbolicLink()&&fs.realpathSync(source)===path.resolve(source),'aliased session directory');
  while(pending.length){
   const name=pending.pop(),absolute=path.join(source,name);
   if(++visited>maxFiles){unavailable.push({path:safe(name),reason:'entry_budget_exhausted'});break;}
   const stat=fs.lstatSync(absolute,{bigint:true});
   if(stat.isDirectory()){
    for(const entry of fs.readdirSync(absolute))pending.push(path.join(name,entry));
    continue;
   }
   if(!stat.isFile()||stat.nlink!==1n){unavailable.push({path:safe(name),reason:'non_regular_or_linked_file'});continue;}
   if(stat.size>BigInt(maxFileBytes)||bytes+Number(stat.size)>maxBytes){unavailable.push({path:safe(name),reason:'explicit_byte_budget_exhausted'});continue;}
   let fd;
   try{
    fd=fs.openSync(absolute,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
    const before=fs.fstatSync(fd,{bigint:true});
    assert.equal(before.ino,stat.ino);assert.equal(before.dev,stat.dev);
    const content=fs.readFileSync(fd),after=fs.fstatSync(fd,{bigint:true});
    for(const key of ['dev','ino','size','mtimeNs','ctimeNs'])assert.equal(before[key],after[key],'session changed while reading');
    assert(content.length<=maxFileBytes&&bytes+content.length<=maxBytes,'session grew beyond budget');bytes+=content.length;
    const text=content.toString('utf8');assert(!content.includes(0)&&Buffer.from(text).equals(content),'non UTF-8 session member');
    const safeName=name==='capture-manifest.json'?'build-capture-manifest.json':safe(name),output=path.join(destination,safeName);
    assert(!path.relative(destination,output).startsWith('..'),'invalid staged path');
    fs.mkdirSync(path.dirname(output),{recursive:true,mode:0o700});
    const retained=Buffer.from(safeContent(name,text));fs.writeFileSync(output,retained,{flag:'wx',mode:0o600});
    files.push({path:safeName,bytes:retained.length,sha256:createHash('sha256').update(retained).digest('hex'),complete:true});
   }catch{unavailable.push({path:safe(name),reason:'member_capture_failed'});}finally{if(fd!==undefined)fs.closeSync(fd);}
  }
 }catch{unavailable.push({path:'.',reason:'session_directory_unavailable'});}
 const required=[];
 if(env.DRAFT_BUILD_OUTCOME&&env.DRAFT_BUILD_OUTCOME!=='skipped')required.push('build.log','build-outcome.json');
 if(env.DRAFT_ATTEMPT_OUTCOME&&env.DRAFT_ATTEMPT_OUTCOME!=='skipped')required.push('attempt.log','attempt-outcome.json','draft-diagnostics-adapter.json');
 const paths=new Set(files.map(file=>file.path));
 receipt.missingRequired=required.filter(name=>!paths.has(name));
 receipt.completeAvailableFileCapture=unavailable.length===0;
 receipt.nativeResponsesObserved=files.some(file=>file.path.startsWith('formal-ai-native/')&&file.bytes>0);
 receipt.dialogExchangesObserved=files.some(file=>file.path.startsWith('formal-ai-dialog/')&&file.bytes>0);
 receipt.fullModelEvidence=receipt.nativeResponsesObserved&&receipt.dialogExchangesObserved&&receipt.completeAvailableFileCapture;
 receipt.incompleteProcesses=[];
 for(const name of ['build-outcome.json','attempt-outcome.json'].filter(name=>paths.has(name))){
  try{const outcome=JSON.parse(fs.readFileSync(path.join(destination,name),'utf8'));
   if(outcome.state!=='finished'||outcome.captureExitCode!==0||outcome.completeConsoleCapture!==true)
    receipt.incompleteProcesses.push(name);
  }catch{receipt.incompleteProcesses.push(name);}
 }
 receipt.complete=receipt.completeAvailableFileCapture&&receipt.missingRequired.length===0&&receipt.incompleteProcesses.length===0;
 fs.writeFileSync(path.join(destination,'capture-manifest.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx',mode:0o600});
 return receipt;
}
if(process.argv[1]&&fs.realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
 assert.equal(process.argv.length,4);
 const receipt=prepareDraftEvidence({source:process.argv[2],destination:process.argv[3]});
 console.log(JSON.stringify({complete:receipt.complete,files:receipt.files.length,unavailable:receipt.unavailable.length,
  missingRequired:receipt.missingRequired,fullModelEvidence:receipt.fullModelEvidence}));
 if(!receipt.complete)process.exitCode=1;
}
