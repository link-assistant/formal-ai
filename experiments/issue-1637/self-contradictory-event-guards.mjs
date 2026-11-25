// Issue #1637: report jobs whose `if:` both refuses and requires the same
// github.event_name, and which no trigger context can start.
import {readFileSync,readdirSync} from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';
const dir='.github/workflows/';
export function selfContradictoryJobs(){
  const found=[];
  for(const f of readdirSync(dir).filter(f=>/\.ya?ml$/u.test(f)).sort()){
    const w=YAML.parse(readFileSync(dir+f,'utf8'));
    for(const [id,job] of Object.entries(w.jobs??{})){
      if(typeof job.if!=='string')continue;
      const events=[...job.if.matchAll(/github\.event_name != '([a-z_]+)'/gu)].map(m=>m[1])
        .filter(e=>job.if.includes(`github.event_name == '${e}'`));
      if(!events.length)continue;
      const code=job.if.trim().replace(/^\$\{\{\s*/u,'').replace(/\s*\}\}$/u,'')
        .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu,(_,k)=>'['+JSON.stringify(k)+']');
      const deep=new Proxy({},{get:(_,k)=>k==='result'?'success':k===Symbol.toPrimitive?()=> 'true':deep});
      const reachable=[...new Set([...events,'push'])].some(event=>Boolean(new Script(code).runInNewContext({
        cancelled:()=>false,always:()=>true,success:()=>true,
        github:{event_name:event,ref:'refs/heads/main',run_attempt:1,event:{action:'synchronize',workflow_run:{head_branch:'main'}}},
        vars:new Proxy({},{get:()=> 'run'}),inputs:deep,needs:new Proxy({},{get:()=>({result:'success',outputs:new Proxy({},{get:()=> 'true'})})}),
      },{timeout:100})));
      if(!reachable)found.push(f+'/'+id);
    }
  }
  return found;
}
if(import.meta.url===`file://${process.argv[1]}`)console.log(selfContradictoryJobs());
