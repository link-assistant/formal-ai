// Semantic evidence from real server diagnostics and optional actual exchanges.
// Raw files are immutable inputs. The maintained codecs retain their own limits.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {decodeTyped} from '../../js/server/structured-log.mjs';
import {decodeDialogRecord} from '../../js/server/dialog-codec.mjs';
const read=file=>new TextDecoder('utf-8',{fatal:true}).decode(fs.readFileSync(file));
export function projectServerLog(raw,{completionPath='/api/openai/v1/chat/completions'}={}){
 if(!['/api/openai/v1/chat/completions','/v1/chat/completions','*'].includes(completionPath))throw Error('unsupported_completion_path');
 const messages=[],diagnostics=[];let records=0;
 for(const[index,line]of raw.split('\n').entries()){
  if(/^\s*\(?formal-ai-log-v1\b/u.test(line)){
   let record;try{record=decodeTyped(line);}catch(error){throw Error(`invalid typed log at line ${index+1}: ${error.message}`);}
   if(record?.schema!=='formal-ai-diagnostic-v1'||typeof record.message!=='string')throw Error(`invalid diagnostic record at line ${index+1}`);
   let message=record.message;
   if(message==='large_native_diagnostic'&&record.metadata){
    const{http_method:method,http_path:requestPath,body_bytes:bytes}=record.metadata;
    if(typeof method==='string'&&/^[A-Z]+$/u.test(method)&&typeof requestPath==='string'&&/^\/[^\s\p{Cc}]*$/u.test(requestPath)&&Number.isSafeInteger(bytes)&&bytes>=0)message=`[trace] ${method} ${requestPath} (${bytes} byte body)`;
   }
   diagnostics.push(message);messages.push(message);records++;
  }else if(line.trim())messages.push(line);
 }
 if(records===0)throw Error('canonical typed server records missing');
 const posts=diagnostics.filter(message=>{
  const header=/^\[trace\] POST (\/[^\s\p{Cc}]*) \([0-9]+ byte body\)(?:\n|$)/u.exec(message);
  return Boolean(header)&&(completionPath==='*'||header[1]===completionPath);
 }).length;
 return{semantic:messages.join('\n'),diagnostics,posts};
}
/** Full response bytes are evidence for emitted tool arguments, never POST counts. */
export function exchangeResponses(directory,{minimumPosts=1}={}){
 const responses=[];let posts=0;
 for(const name of fs.readdirSync(directory).sort()){
  if(!name.endsWith('.lino'))continue;
  const raw=read(path.join(directory,name));if(!raw.endsWith('\n'))throw Error('unterminated_dialog_record');
  for(const line of raw.slice(0,-1).split('\n')){
   const record=decodeDialogRecord(line);
   if(typeof record?.method!=='string'||typeof record?.path!=='string')continue;
   if(typeof record.response_body!=='string')throw Error('complete_dialog_response_missing');
   responses.push(record.response_body);if(record.method==='POST')posts++;
  }
 }
 if(responses.length===0)throw Error('actual_dialog_exchange_records_missing');
 if(posts<Math.max(1,minimumPosts))throw Error('missing_completed_dialog_exchanges');
 return responses.join('\n');
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const[input,output,completionPath='/api/openai/v1/chat/completions',dialogDirectory]=process.argv.slice(2);if(!input||!output)throw Error('raw input and semantic output required');
 const result=projectServerLog(read(input),{completionPath});
 const responses=dialogDirectory?exchangeResponses(dialogDirectory,{minimumPosts:result.posts}):'';
 fs.writeFileSync(output,result.semantic+'\n'+(responses?'[actual persisted server responses]\n'+responses+'\n':''));
 console.log(result.posts);
}
