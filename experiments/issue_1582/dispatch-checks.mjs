// Reuse the repository's closed checks-only dispatch contract; no release mode.
import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {checksDispatchContract} from '../../scripts/lib/checks-dispatch-contract.mjs';
const branch='issue-1582-79aa864de807',repo='link-assistant/formal-ai';
const results=[];
for(const workflow of fs.readdirSync('.github/workflows').filter(name=>name.endsWith('.yml'))){
 const source=fs.readFileSync('.github/workflows/'+workflow,'utf8');
 if(!checksDispatchContract(source))continue;
 try{
  execFileSync('gh',['workflow','run',workflow,'--repo',repo,'--ref',branch,'-f','mode=checks','-f','pull-request=1583'],{encoding:'utf8',stdio:['ignore','pipe','pipe']});
  results.push({workflow,dispatched:true});
 }catch(error){results.push({workflow,dispatched:false,error:String(error.stderr)});}
}
fs.writeFileSync('/tmp/issue-1582-dispatch.json',JSON.stringify(results,null,2)+'\n');
console.log(JSON.stringify(results));
if(results.some(result=>!result.dispatched))process.exitCode=1;
