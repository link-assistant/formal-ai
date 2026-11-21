// Whole-tree inventory and workflow review aid. Findings require human review.
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const {parse} = createRequire(import.meta.url)('yaml');

const input=process.argv[2]??'/tmp/issue-1580-research/templates';
const output=process.argv[3]??'docs/case-studies/issue-1580/templates';
fs.mkdirSync(output,{recursive:true});
const local=execFileSync('git',['ls-files','-z'],{encoding:'utf8',maxBuffer:16*1024*1024}).split('\0').filter(Boolean);
const names=new Map();
for(const file of local){const name=path.basename(file);names.set(name,[...(names.get(name)??[]),file]);}
const sha=bytes=>createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
const manifests=JSON.parse(fs.readFileSync(path.join(input,'manifest.json')));
const summaries=[];
for(const source of manifests){
  const language=source.repo.split('/')[1].split('-')[0];
  const directory=path.join(input,language+'-complete');
  const inventory=JSON.parse(fs.readFileSync(path.join(input,language+'-complete-inventory.json')));
  const rows=['path\tgit_blob\tbytes\trole\tlocal_same_name\tcomparison'];
  const workflows=[];
  const candidates=[];
  for(const item of inventory){
    const file=item.path,bytes=fs.readFileSync(path.join(directory,file));
    if(sha(bytes)!==item.gitBlob)throw Error('source drift: '+file);
    const text=bytes.includes(0)?'':bytes.toString('utf8');
    let role;
    if(file.startsWith('.github/workflows/')&&/\.ya?ml$/u.test(file))role='workflow';
    else if(file.startsWith('.github/actions/'))role='composite-action';
    else if(/^(?:docs\/case-studies\/|dev\/log\/|experiments\/)/u.test(file))role='historical-evidence-or-experiment';
    else if(/(^|\/)(?:scripts|tests?|\.husky)\//u.test(file))role='script-or-test';
    else if(/(?:Dockerfile|Makefile|(?:package|composer|global)\.json|(?:Cargo|pyproject)\.toml|\.lock$|lock\.json$|\.config\.|\.rc$|\.yml$|\.yaml$|^\.github\/|^\.changeset\/|^changelog\.d\/|^\.prettier|^\.editorconfig)/u.test(file))role='build-or-policy-configuration';
    else if(/\.(?:md|rst|txt)$/u.test(file)||/LICENSE|COPYING/u.test(file))role='documentation-or-license';
    else role=text?'application-source-or-asset':'binary-asset';
    const matches=names.get(path.basename(file))??[];
    const equivalents=matches.filter(name=>!name.startsWith('dev/log/')&&!name.startsWith('docs/case-studies/')&&!name.startsWith('experiments/')).slice(0,6);
    const identical=equivalents.find(name=>sha(fs.readFileSync(name))===item.gitBlob);
    const comparison=identical?'identical:'+identical:equivalents.length?'different-implementation':'no-same-name-file';
    rows.push([file,item.gitBlob,item.bytes,role,equivalents.join(','),comparison].join('\t'));
    if(['workflow','composite-action','script-or-test','build-or-policy-configuration'].includes(role)){
      const topics=[...text.matchAll(/(?:id-token|trusted.?publish|concurrency|timeout|retry|preflight|continue-on-error|workflow_run|pull_request_target|arm64|aarch64|QEMU|actionlint|shellcheck|zizmor|changeset|changelog|GH006|GH013|dependabot|cancel-in-progress|npm publish|cargo publish|docker push)/giu)].map(match=>match[0].toLowerCase());
      candidates.push({path:file,role,topics:[...new Set(topics)].sort(),lines:text.split('\n').length});
    }
    if(role==='workflow'){
      const document=parse(text);
      workflows.push({path:file,gitBlob:item.gitBlob,document,name:document.name,events:document.on,permissions:document.permissions??null,concurrency:document.concurrency??null,jobs:Object.entries(document.jobs??{}).map(([id,job])=>({id,name:job.name,runsOn:job['runs-on'],timeout:job['timeout-minutes'],if:job.if,needs:job.needs,permissions:job.permissions,concurrency:job.concurrency,matrix:job.strategy?.matrix,steps:(job.steps??[]).map(step=>({name:step.name,uses:step.uses,if:step.if,timeout:step['timeout-minutes'],continueOnError:step['continue-on-error'],run:step.run})).filter(step=>step.run||step.uses)}))});
    }
  }
  fs.writeFileSync(path.join(output,language+'-files.tsv'),rows.join('\n')+'\n');
  // One workflow per JSON line. The complete parsed document preserves inputs,
  // outputs, environments, services, reusable bindings and other execution fields.
  // The remaining fields are a compact index, never the authoritative source.
  fs.writeFileSync(path.join(output,language+'-workflows.jsonl'),workflows.map(row=>JSON.stringify(row)).join('\n')+'\n');
  fs.writeFileSync(path.join(output,language+'-ci-files.jsonl'),candidates.map(row=>JSON.stringify(row)).join('\n')+'\n');
  summaries.push({language,repository:source.repo,commit:source.sha,blobCount:inventory.length,workflowCount:workflows.length,ciCandidateCount:candidates.length});
}
fs.writeFileSync(path.join(output,'sources.json'),JSON.stringify(summaries,null,2)+'\n');
console.log(JSON.stringify(summaries));
