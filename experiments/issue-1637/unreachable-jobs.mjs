// Issue #1637: list workflow jobs whose `if:` is false under every trigger context.
import {readFileSync,readdirSync} from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';
const dir='.github/workflows/';
const strip=e=>e.trim().replace(/^\$\{\{\s*/u,'').replace(/\s*\}\}$/u,'');
const any=new Proxy({},{get:(t,k)=>k===Symbol.toPrimitive?()=>'true':(k==='result'?'success':any)});
function ctx(event,ref,extra={}){
  const truthy=new Proxy({},{get:()=>'true'});
  return {github:{event_name:event,ref,run_attempt:1,repository:'link-assistant/formal-ai',actor:'x',event:{action:'synchronize',inputs:truthy,workflow_run:{head_branch:'main',conclusion:'success',event:'push'},pull_request:{draft:false,head:{repo:{full_name:'link-assistant/formal-ai'}}}},...extra},
    vars:new Proxy({},{get:()=> 'run'}),inputs:truthy,
    needs:new Proxy({},{get:()=>({result:'success',outputs:new Proxy({},{get:()=> 'true'})})})};
}
const events=[['push','refs/heads/main'],['pull_request','refs/pull/1/merge'],['workflow_dispatch','refs/heads/main'],['schedule','refs/heads/main'],['release','refs/tags/v1'],['workflow_run','refs/heads/main'],['workflow_call','refs/heads/main'],['push','refs/tags/v1']];
for(const f of readdirSync(dir).filter(f=>/\.ya?ml$/u.test(f))){
  const w=YAML.parse(readFileSync(dir+f,'utf8'));
  for(const [id,job] of Object.entries(w.jobs??{})){
    if(typeof job.if!=='string')continue;
    const code=strip(job.if).replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu,(_,k)=>'['+JSON.stringify(k)+']');
    let reachable=false,err=null;
    for(const [e,r] of events){try{if(new Script(code).runInNewContext({cancelled:()=>false,always:()=>true,success:()=>true,failure:()=>true,contains:(a,b)=>String(a).includes(b),startsWith:(a,b)=>String(a).startsWith(b),endsWith:(a,b)=>String(a).endsWith(b),fromJSON:x=>{try{return JSON.parse(x)}catch{return x}},format:(s,...a)=>s,...ctx(e,r)},{timeout:100})){reachable=true;break}}catch(x){err=x.message}}
    if(!reachable)console.log(f+'/'+id+(err?' (eval error: '+err+')':''));
  }
}
