// Read-only syntactic inventory for Rust template practice 19.
// Candidate strings are not proofs of shell execution or condition dominance.
import {readdirSync, readFileSync} from 'node:fs';
import YAML from 'yaml';

const tools = [
  {name:'rust-script', uses:/\brust-script\s|scripts\/test-scripts\.sh/u, install:/cargo install rust-script\b|scripts\/install-rust-script\.sh/u},
  {name:'cargo-audit', uses:/\bcargo audit\b|\bcargo-audit\s|scripts\/check-rust-dependencies\.sh/u, install:/cargo install cargo-audit\b/u},
  {name:'cargo-llvm-cov', uses:/\bcargo llvm-cov\b|\bcargo-llvm-cov\s/u, install:/cargo install cargo-llvm-cov\b/u},
  {name:'cargo-cyclonedx', uses:/\bcargo cyclonedx\b|\bcargo-cyclonedx\s/u, install:/cargo install cargo-cyclonedx\b/u},
  {name:'sccache', uses:/\bsccache\s|RUSTC_WRAPPER[=:]\s*sccache/u, install:/cargo install sccache\b/u},
];
const observations=[], findings=[], errors=[];
const files=readdirSync('.github/workflows').filter(n=>/\.ya?ml$/u.test(n)).sort();
for(const name of files) {
  const file='.github/workflows/'+name;
  const document=YAML.parseDocument(readFileSync(file,'utf8'));
  if(document.errors.length) {errors.push({file,errors:document.errors.map(x=>x.message)});continue;}
  const workflow=document.toJS({maxAliasCount:100});
  for(const [jobName,job]of Object.entries(workflow.jobs??{})) {
    for(const tool of tools) {
      const installs=[];
      for(const [index,step]of (job.steps??[]).entries()) {
        const run=String(step.run??'').split('\n').filter(l=>!l.trimStart().startsWith('#')).join('\n');
        const action=String(step.uses??'');
        const installed=(action.startsWith('taiki-e/install-action@')&&
          (action.split('@').at(-1)===tool.name||String(step.with?.tool??'').split(',').some(x=>x.trim().split('@')[0]===tool.name)))||
          (tool.name==='sccache'&&(action==='./.github/actions/setup-sccache'||action.startsWith('mozilla-actions/sccache-action@')))||
          tool.install.test(run);
        if(installed)installs.push({index,if:step.if??null,continueOnError:step['continue-on-error']??false,action,run});
        if(!installed&&tool.uses.test(run)) {
          const previous=installs.filter(x=>x.index<index);
          const row={file,job:jobName,tool:tool.name,index,if:step.if??null,run,installCandidates:previous,availabilityProved:false};
          observations.push(row);
          if(!previous.length)findings.push(row);
        }
      }
    }
  }
}
console.log(JSON.stringify({files:files.length,observations,findings,errors,scope:'syntactic candidates only; no shell, composite-action or condition-dominance proof',fullIssueComplete:false,independentAuthorshipCredit:0},null,2));
process.exitCode=errors.length?1:0;
