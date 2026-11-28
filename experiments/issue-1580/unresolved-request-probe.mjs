// Supplied diagnostic: no independent Formal AI authorship credit.
// Replay the exact native Spanish sibling inputs in the production JS worker.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {gzipSync} from 'node:zlib';
import {REPO_ROOT, createWorkerContext, evaluate} from '../../js/server/worker-host.mjs';

const stage=process.argv[2]??'';
if(stage&&!/^[a-z0-9-]+$/.test(stage)) throw new Error('Use a simple stage name');
const out=path.join(REPO_ROOT,'experiments/issue-1580/evidence/checks/unresolved-request-probe',stage);
if(fs.existsSync(path.join(out,'answers.json.gz'))) throw new Error('Evidence already exists; supply a fresh stage name');

const sourcePath='rust/tests/unit/issue_706_any_language.rs';
const source=fs.readFileSync(path.join(REPO_ROOT,sourcePath),'utf8');
const fixture=source.slice(source.indexOf('fn a_language_without_localized_openers_reports_a_gap_not_english()'));
const prompts=[...fixture.matchAll(/\.answer\("([^"\n]+)"\)/gu)].map(match=>match[1]);
if(prompts.length!==2) throw new Error('Expected the two unchanged native sibling inputs');
const worker=createWorkerContext();
await evaluate(worker,'loadSeed()');
const attemptedFetches=[];
worker.fetch=async input=>{
  attemptedFetches.push(String(input));
  throw new Error('Network disabled for the bounded unresolved-request probe');
};
const answers=[];
for(const prompt of prompts){
  const answer=await worker.solve(prompt,[],{},{},[],{});
  answers.push({prompt,answer});
  console.log(JSON.stringify({prompt,intent:answer.intent,content:answer.content}));
}
const raw=Buffer.from(JSON.stringify({
  source:{path:sourcePath,sha256:createHash('sha256').update(source).digest('hex')},
  answers,attemptedFetches,actualNetworkRequests:0,nativeBehaviorVerified:false,
  autonomousAuthorshipCredit:0,
}));
fs.mkdirSync(out,{recursive:true});
fs.writeFileSync(path.join(out,'answers.json.gz'),gzipSync(raw,{mtime:0}));
console.log(JSON.stringify({rawBytes:raw.length,sha256:createHash('sha256').update(raw).digest('hex'),
  attemptedFetches:attemptedFetches.length,actualNetworkRequests:0,nativeBehaviorVerified:false}));
