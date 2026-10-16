import fs from 'node:fs';
import {convertLog,makeRedactor} from './structured-log.mjs';
import {decodeDialogRecord,encodeDialogRecord} from './dialog-codec.mjs';
import {parseJson,toPrettyJson,toCompactJson} from './json.mjs';
export function runLogsCli(argv,{readFile=file=>fs.readFileSync(file,'utf8'),env=process.env}={}){
 try{
  let file=null,to='lino',from=null;
  for(let i=0;i<argv.length;i++){
   if(argv[i]==='--format'){to=argv[++i];continue;}
   if(argv[i]==='--from'){from=argv[++i];continue;}
   if(argv[i].startsWith('-')||file!==null)throw Error('invalid_arguments');file=argv[i];
  }
  if(!file||!['lino','json','pretty'].includes(to))throw Error('invalid_arguments');
  const input=readFile(file);from??=file.endsWith('.jsonl')?'jsonl':'lino';
  if(typeof input!=='string'||Buffer.byteLength(input)>16*1024*1024||!['lino','jsonl'].includes(from))throw Error('invalid_input');
  if(from==='jsonl'||input.startsWith('(formal-ai-log-v1 ')){
   if(input!==''&&!input.endsWith('\n'))throw Error('unterminated_log');
   const records=input===''?[]:input.slice(0,-1).split('\n').map(from==='jsonl'?parseJson:decodeDialogRecord);
   const redact=makeRedactor(env,{preserveJsonNumbers:true}),safe=records.map(x=>redact(x));
   const encoded=safe.map(encodeDialogRecord);
   if(encoded.some(x=>Buffer.byteLength(x)>16*1024*1024))throw Error('record_size_limit');
   return {code:0,stdout:to==='lino'?encoded.map(x=>x+'\n').join(''):(to==='pretty'?toPrettyJson(safe):toCompactJson(safe))+'\n',stderr:''};
  }
  return {code:0,stdout:convertLog(input,{from,to,env}),stderr:''};
 }catch{return {code:2,stdout:'',stderr:'log_conversion_refused\n'};}
}
