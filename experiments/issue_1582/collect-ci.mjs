// Supplied evidence collection recipe; zero autonomous authorship credit.
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const root='docs/case-studies/issue-1582/raw-data';
const records=JSON.parse(fs.readFileSync(path.join(root,'branch-runs.json'),'utf8'));
const runs=(Array.isArray(records)?records.flatMap(page=>page.workflow_runs??page):records.workflow_runs);
const directory=path.join(root,'ci');fs.mkdirSync(directory,{recursive:true});
const inventory=[];
for(const run of runs){
  const id=run.id??run.databaseId;
  const log=spawnSync('gh',['run','view',String(id),'--repo','link-assistant/formal-ai','--log'],{encoding:'utf8',maxBuffer:32*1024*1024});
  fs.writeFileSync(path.join(directory,id+'.log'),log.stdout??'');
  fs.writeFileSync(path.join(directory,id+'.log-status.json'),JSON.stringify({id,sha:run.head_sha??run.headSha,created_at:run.created_at??run.createdAt,conclusion:run.conclusion,exit:log.status,stderr:log.stderr,error:log.error?.message??null},null,2)+'\n');
  inventory.push({id,sha:run.head_sha??run.headSha,conclusion:run.conclusion,logAvailable:log.status===0});
}
fs.writeFileSync(path.join(directory,'inventory.json'),JSON.stringify(inventory,null,2)+'\n');
console.log(JSON.stringify({runs:inventory.length,logs:inventory.filter(row=>row.logAvailable).length}));
