// Summarises the `agent` CLI event stream (pretty-printed JSON objects, one
// after another) saved by hive-agent-local.mjs: each completed tool call with
// its exit status, and every assistant text.
//   node experiments/issue-1483/agent-session-summary.mjs <agent.stdout.ndjson>
import {readFileSync} from 'node:fs';
const text=readFileSync(process.argv[2],'utf8'),events=[];
let depth=0,start=-1,quoted=false,escaped=false;
for(let i=0;i<text.length;i++){
  const c=text[i];
  if(quoted){if(escaped)escaped=false;else if(c==='\\')escaped=true;else if(c==='"')quoted=false;continue;}
  if(c==='"')quoted=true;else if(c==='{'){if(depth++===0)start=i;}else if(c==='}'&&--depth===0)events.push(JSON.parse(text.slice(start,i+1)));
}
for(const event of events){
  if(event.type==='tool_use'&&event.part.state.status==='completed'){
    const {input,metadata}=event.part.state;
    const shown=input.command??input.filePath??JSON.stringify(input);
    console.log(`tool ${event.part.tool}: ${String(shown).split('\n')[0].slice(0,140)}${metadata?.exit!==undefined?` (exit ${metadata.exit})`:''}`);
  }
  if(event.type==='text')console.log(`text: ${event.part.text}`);
}
