import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync,writeFileSync,mkdirSync,mkdtempSync,existsSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const YAML=createRequire(import.meta.url)('yaml');
const detectors=[['release.yml','detect-changes'],['coverage.yml','detect-changes'],['layered-ci.yml','changes']];
const paths=['/scripts/detect-code-changes.rs','/scripts/ci-pull-request-admission.mjs',
  '/scripts/lib/documentation-paths.mjs','/scripts/install-rust-script.sh','/changelog.d/'];
const jobs=detectors.map(([file,name])=>({file,job:YAML.parse(readFileSync(join(root,'.github/workflows',file),'utf8')).jobs[name]}));
function check(job){
  assert.equal(job['timeout-minutes'],5);
  assert.deepEqual(job.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'30'});
  const checkouts=job.steps.filter(step=>step.uses==='actions/checkout@v7');
  assert.equal(checkouts.length,1);
  assert.deepEqual(checkouts[0].with,{'persist-credentials':false,'fetch-depth':0,
    'sparse-checkout':paths.join('\n')+'\n','sparse-checkout-cone-mode':false});
  assert(job.steps.some(step=>step.run==='bash scripts/install-rust-script.sh'));
  assert(job.steps.some(step=>step.run?.includes('rust-script scripts/detect-code-changes.rs')));
  assert(job.steps.some(step=>step.run?.includes('node scripts/ci-pull-request-admission.mjs detect')));
}
test('every change detector retains complete history, native classification and bounded sparse checkout',()=>{
  for(const {file,job} of jobs){assert.doesNotThrow(()=>check(job),file);}
});
test('missing inputs, widened checkout, altered authority and relaxed bounds are rejected',()=>{
  const mutations=[
    job=>job['timeout-minutes']=10,
    job=>job.env.GIT_HTTP_LOW_SPEED_LIMIT='0',
    job=>job.env.GIT_HTTP_LOW_SPEED_TIME='300',
    job=>delete job.env,
    job=>job.steps[0].with['fetch-depth']=1,
    job=>job.steps[0].with['persist-credentials']=true,
    job=>job.steps[0].with.ref='foreign',
    job=>job.steps[0].with['sparse-checkout-cone-mode']=true,
    job=>job.steps[0].with['sparse-checkout']='**\n',
    ...paths.map(path=>job=>job.steps[0].with['sparse-checkout']=paths.filter(item=>item!==path).join('\n')+'\n')
  ];
  for(const {job} of jobs)for(const mutate of mutations){const value=structuredClone(job);mutate(value);assert.throws(()=>check(value));}
});
test('real sparse Git checkout preserves PR parents, full push differences, all tracked paths and waiting fragments',t=>{
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-detector-history-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const full=join(directory,'full'),sparse=join(directory,'sparse'),configuration=join(directory,'gitconfig');
  writeFileSync(configuration,'');mkdirSync(full);
  const env={...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:configuration,GIT_TERMINAL_PROMPT:'0',GIT_ALLOW_PROTOCOL:'file'};
  const git=(cwd,args,input)=>execFileSync('git',args,{cwd,env,input,encoding:'utf8',timeout:10000,maxBuffer:1024*1024}).trim();
  const put=(file,text)=>{const dest=join(full,file);mkdirSync(dirname(dest),{recursive:true});writeFileSync(dest,text);};
  git(full,['init','--initial-branch=main']);git(full,['config','user.name','Detector fixture']);git(full,['config','user.email','detector@example.invalid']);
  git(full,['config','uploadpack.allowFilter','true']);git(full,['config','uploadpack.allowAnySHA1InWant','true']);
  for(const file of ['scripts/detect-code-changes.rs','scripts/install-rust-script.sh'])put(file,readFileSync(join(root,file)));
  put('changelog.d/README.md','Fragment instructions\n');put('changelog.d/waiting.md','Waiting release\n');
  put('js/main.mjs','export const value=0;\n');put('rust/lib.rs','pub fn value() {}\n');put('assets/unrelated.data','not a detector input\n');
  git(full,['add','.']);git(full,['commit','-m','initial inputs']);const before=git(full,['rev-parse','HEAD']);
  git(full,['switch','-c','feature']);put('js/main.mjs','export const value=1;\n');git(full,['mv','rust/lib.rs','rust/runtime.rs']);git(full,['add','.']);git(full,['commit','-m','code first']);
  put('docs/notes.md','Documentation last\n');put('docs/case-studies/evidence.txt','Ignored by the classifier\n');git(full,['add','.']);git(full,['commit','-m','documentation last']);const feature=git(full,['rev-parse','HEAD']);
  assert(!git(full,['diff','--name-only',feature+'^',feature]).includes('js/main.mjs'));
  git(full,['switch','main']);git(full,['merge','--no-ff','--no-edit','feature']);const merge=git(full,['rev-parse','HEAD']);
  git(directory,['clone','--no-local','--filter=blob:none','--no-checkout',pathToFileURL(full).href,sparse]);
  git(sparse,['sparse-checkout','set','--no-cone','--stdin'],jobs[0].job.steps[0].with['sparse-checkout']);git(sparse,['checkout',merge]);
  assert.equal(git(sparse,['config','--get','remote.origin.promisor']),'true');assert.equal(git(sparse,['rev-parse','--is-shallow-repository']),'false');
  assert.equal(git(sparse,['rev-parse','HEAD^']),before);assert.equal(git(sparse,['rev-parse','HEAD^2']),feature);
  const difference=(cwd,base,head)=>git(cwd,['diff','--name-only',base,head]);
  for(const [base,head] of [['HEAD^','HEAD^2'],[before,feature],[before,merge]]){
    const actual=difference(sparse,base,head);assert.equal(actual,difference(full,base,head));assert(actual.includes('js/main.mjs'));assert(actual.includes('rust/runtime.rs'));assert(actual.includes('docs/notes.md'));
  }
  assert.equal(git(sparse,['ls-tree','--name-only','-r','HEAD']),git(full,['ls-tree','--name-only','-r','HEAD']));
  for(const file of ['scripts/detect-code-changes.rs','scripts/install-rust-script.sh'])assert.deepEqual(readFileSync(join(sparse,file)),readFileSync(join(root,file)));
  assert.deepEqual(readdirSync(join(sparse,'changelog.d')).sort(),readdirSync(join(full,'changelog.d')).sort());
  assert.equal(readFileSync(join(sparse,'changelog.d/waiting.md'),'utf8'),'Waiting release\n');
  for(const file of ['js/main.mjs','rust/runtime.rs','assets/unrelated.data','docs/notes.md'])assert.equal(existsSync(join(sparse,file)),false,file);
});
