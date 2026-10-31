// R1183: semantic regeneration belongs to the hosted pinned compiler boundary.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const workflow=YAML.parse(fs.readFileSync(root+'/.github/workflows/layered-ci.yml','utf8'));
const command='node scripts/generate-coding-research-program.mjs --check --meta-language .meta-language';
const pin='eb0574d5528babd3b4aadece81590d8e2230b99b';
const eligibility=String.fromCharCode(36)+"{{ (github.event_name != 'pull_request' && github.event_name != 'pull_request_target' && (github.ref == 'refs/heads/main' || github.event_name == 'release' || (github.event_name == 'workflow_run' && github.event.workflow_run.head_branch == 'main'))) && (!cancelled() && (needs.js.result == 'success' || needs.js.result == 'skipped')) }}";
function issues(document,registeredCompilerGate=false) {
  const job=document.jobs['js-rust'], steps=job.steps;
  const findings=[];
  if(job.if!==eligibility) findings.push('CompilerJobEligibility');
  const checkout=steps.findIndex(step=>step.with?.repository==='link-foundation/meta-language'
    && step.with?.ref===pin && step.with?.path==='.meta-language');
  const dependencies=steps.findIndex(step=>step.run===
    'npm ci --prefix .meta-language/js --omit=dev --ignore-scripts --no-audit --no-fund');
  const rust=steps.findIndex(step=>step.uses==='dtolnay/rust-toolchain@stable');
  const checks=steps.map((step,index)=>({step,index})).filter(item=>item.step.run===command);
  if(checkout<0 || dependencies<=checkout || rust<=dependencies
      || checks.length!==1 || checks[0].index<=rust) findings.push('CompilerPrerequisiteOrder');
  for(const {step} of checks) {
    if('if' in step || 'continue-on-error' in step) findings.push('OptionalSemanticCheck');
  }
  if(registeredCompilerGate) findings.push('CompilerGateInGenericStage');
  return findings;
}
test('actual semantic check is mandatory after pinned checkout, dependencies and Rust setup',()=>{
  const generic=fs.existsSync(root+'/data/meta/ci-gates/check-coding-research-program.lino');
  assert.deepEqual(issues(workflow,generic),[]);
});
test('missing or reordered compiler dependencies cannot satisfy the workflow contract',()=>{
  for(const change of ['pin','dependencies','rust','command']) {
    const mutated=structuredClone(workflow),steps=mutated.jobs['js-rust'].steps;
    if(change==='pin') steps.find(step=>step.with?.repository==='link-foundation/meta-language').with.ref='different';
    if(change==='dependencies') steps.splice(steps.findIndex(step=>step.run?.startsWith('npm ci --prefix .meta-language/')),1);
    if(change==='rust') steps.splice(steps.findIndex(step=>step.uses==='dtolnay/rust-toolchain@stable'),1);
    if(change==='command') steps.find(step=>step.run===command).run=command+' || true';
    assert(issues(mutated).includes('CompilerPrerequisiteOrder'),change);
  }
});
test('conditional or tolerated generated-program checks are refused',()=>{
  for(const field of ['if','continue-on-error']) {
    const mutated=structuredClone(workflow);
    mutated.jobs['js-rust'].steps.find(step=>step.run===command)[field]=field==='if'?'false':true;
    assert(issues(mutated).includes('OptionalSemanticCheck'),field);
  }
  assert(issues(workflow,true).includes('CompilerGateInGenericStage'));
});
test('carried JavaScript and Rust-only edits run the check while failed JavaScript remains blocking',()=>{
  assert.equal(workflow.jobs['js-rust'].if,eligibility);
  assert(!workflow.jobs['js-rust'].if.includes('outputs.js-changed'));
  const evaluate = (event, status, restore = false, cancelled = false) => {
    const source = eligibility.slice(3, -2).replaceAll('needs.js.result', 'result');
    return Function('github','vars','result','cancelled','return ('+source+');')(
      {event_name:event,ref:event==='push'?'refs/heads/main':'refs/pull/1500/merge'},
      {FORMAL_AI_FULL_PR_CI:restore?'run':'skip'},status,()=>cancelled);
  };
  for(const status of ['success','skipped','failure','cancelled']) {
    assert.equal(evaluate('push',status),['success','skipped'].includes(status));
    assert.equal(evaluate('pull_request',status),false);
    assert.equal(evaluate('pull_request',status,true),false);
    assert.equal(evaluate('push',status,false,true),false);
  }
  const mutated=structuredClone(workflow);mutated.jobs['js-rust'].if='always()';
  assert(issues(mutated).includes('CompilerJobEligibility'));
});
test('production release and dependency verification still invoke every generic registered stage',()=>{
  for(const filename of ['release.yml','dependencies-latest.yml']) {
    const source=fs.readFileSync(root+'/.github/workflows/'+filename,'utf8');
    for(const stage of ['rust','wasm','web']) assert(source.includes('scripts/run-ci-gates.rs --stage '+stage));
  }
});
