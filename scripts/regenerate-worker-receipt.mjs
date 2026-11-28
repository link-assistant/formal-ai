// Explicit hosted maintenance: compile the selected browser worker and preserve actual receipts.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {copyFileSync,mkdirSync,writeFileSync,readFileSync} from 'node:fs';
import {relative,resolve,sep} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {buildSelectedWasm,verifySelectedWasm,dependencyPaths} from './build-selected-wasm.mjs';
import {compilerIdentity,renderManifest,main as checkManifest} from './wasm-worker-inputs.mjs';

export function admission(environment,head) {
  assert.equal(environment.GITHUB_ACTIONS,'true','Hosted regeneration only');
  assert.equal(environment.GITHUB_EVENT_NAME,'workflow_dispatch','Explicit manual regeneration only');
  assert.equal(environment.FORMAL_AI_REGENERATION_TARGET,'worker-receipt','Exact target required');
  assert.match(head,/^[a-f0-9]{40}$/u);
  assert.equal(environment.GITHUB_SHA,head,'Checkout differs from dispatched source');
  assert.equal(environment.GITHUB_REPOSITORY,'link-assistant/formal-ai');
  for(const key of ['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT'])assert.match(environment[key]??'',/^[1-9][0-9]*$/u,'actual workflow identity required');
}

// The worker manifest describes compiler dependencies. Package and recipe
// metadata remain bound in the producer receipt without causing false drift.
export function compilerInputManifest(root,record,dependencyInfo) {
  assert.equal(createHash('sha256').update(dependencyInfo).digest('hex'),record.dependencyInfoSha256);
  const paths=[...new Set(dependencyPaths(dependencyInfo.toString('utf8'))
    .map(path=>relative(root,resolve(root,path)).split(sep).join('/')))].sort();
  const inputs=paths.map(path=>{const input=record.inputs.find(file=>file.path===path);
    assert(input,'compiler dependency missing from verified receipt: '+path);return {path,sha256:input.sha256};});
  return renderManifest({worker:record.output,compiler:compilerIdentity(record.compiler),inputs});
}

export function regenerate(root,environment=process.env) {
  const head=execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim();
  admission(environment,head);
  const result=buildSelectedWasm(root,{environment});
  verifySelectedWasm(root,resolve(root,'js'),result.receiptSha256,{bindings:{
    repository:environment.GITHUB_REPOSITORY,run:environment.GITHUB_RUN_ID,attempt:environment.GITHUB_RUN_ATTEMPT,
    event:'workflow_dispatch'}});
  const manifest=compilerInputManifest(root,result.record,readFileSync(resolve(root,'js/formal_ai_worker.d')));
  writeFileSync(resolve(root,'data/meta/wasm-worker-inputs.lino'),manifest);
  assert.equal(checkManifest(['--check','--repository',root]),0);
  const destination=resolve(root,'worker-regeneration');mkdirSync(destination,{recursive:false});
  for(const [source,name] of [['js/formal_ai_worker.wasm','formal_ai_worker.wasm'],
    ['js/formal_ai_worker.receipt.json','formal_ai_worker.receipt.json'],
    ['data/meta/wasm-worker-inputs.lino','wasm-worker-inputs.lino']])copyFileSync(resolve(root,source),resolve(destination,name));
  return result;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)
  regenerate(resolve(fileURLToPath(new URL('../',import.meta.url))));
