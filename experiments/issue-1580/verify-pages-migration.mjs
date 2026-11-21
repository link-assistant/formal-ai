import {readCanonicalSource,CANONICAL_SOURCE} from '../../scripts/canonical-release-source.mjs';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
const root=fileURLToPath(new URL('../../', import.meta.url));
const baseline='f967e4efb247e1697f5226bb43bbac21e517e090';
const old=file=>execFileSync('git',['show',baseline+':'+file],{cwd:root,encoding:'utf8',maxBuffer:2*1024*1024});
const current=file=>file===CANONICAL_SOURCE?readCanonicalSource(root).text:readFileSync(root+file,'utf8');
const rows=[];
for(const file of ['.github/workflows/release.yml','experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/original-release-workflow.yml']){
 const before=YAML.parse(old(file)),after=YAML.parse(current(file)),normalized=structuredClone(after);
 const previous=before.jobs['deploy-pages'],job=normalized.jobs['deploy-pages'];
 const checkout=job.steps.shift();
 assert.equal(checkout.name,'Checkout Pages deployment protocol');
 assert.deepEqual(checkout,{name:'Checkout Pages deployment protocol',uses:'actions/checkout@v7',with:{ref:'${{ github.workflow_sha }}',path:'.formal-ai-pages-protocol','persist-credentials':false,'fetch-depth':1,filter:'blob:none','sparse-checkout-cone-mode':false,'sparse-checkout':'/.github/actions/deploy-pages/'}});
 const step=job.steps.find(x=>x.id==='deployment');
 assert.equal(step.uses,'./.formal-ai-pages-protocol/.github/actions/deploy-pages');
 assert.deepEqual(step.with,{artifact_id:'${{ needs.build-pages.outputs.artifact_id }}',source_sha:'${{ needs.build-pages.outputs.pages_sha }}',timeout:600000});
 assert.deepEqual(step.env,{NODE_OPTIONS:'--throw-deprecation'});
 const prior=previous.steps.find(x=>x.id==='deployment');
 assert.equal(prior.uses,'actions/deploy-pages@v5');
 Object.assign(step,{uses:prior.uses,with:prior.with,env:prior.env});
 assert.deepEqual(normalized,before,file+' all other workflow fields and steps preserved');
 rows.push({file,beforeSha256:createHash('sha256').update(old(file)).digest('hex'),afterSha256:createHash('sha256').update(current(file)).digest('hex')});
}
{
 const file='.github/workflows/pages-artifact.yml',before=YAML.parse(old(file)),after=YAML.parse(current(file)),normalized=structuredClone(after);
 assert.deepEqual(normalized.on.workflow_call.outputs.artifact_id,{description:'Immutable ID of the Pages artifact uploaded by this build.',value:'${{ jobs.build.outputs.artifact_id }}'});
 assert.equal(normalized.jobs.build.outputs.artifact_id,'${{ steps.pages_artifact.outputs.artifact_id }}');
 const upload=normalized.jobs.build.steps.find(x=>x.name==='Upload GitHub Pages artifact');assert.equal(upload.id,'pages_artifact');
 delete upload.id;delete normalized.on.workflow_call.outputs.artifact_id;delete normalized.jobs.build.outputs.artifact_id;
 assert.deepEqual(normalized,before,'Every existing build command, ref check, permission, timeout and output is preserved');
 rows.push({file,beforeSha256:createHash('sha256').update(old(file)).digest('hex'),afterSha256:createHash('sha256').update(current(file)).digest('hex')});
}
const proof={baseline,wholeWorkflowComparisons:rows,transport:'Only one exact upload artifact ID crosses the reusable workflow boundary.',preserved:'Every original workflow/job field and unrelated step, including source checks, trust predicates, concurrency, permissions, deadlines, output source SHA and live E2E commands.',hostedDeployment:false,independentAuthorshipCredit:0};
console.log(JSON.stringify(proof,null,2));
