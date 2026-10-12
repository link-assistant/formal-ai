import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {buildStagedReleaseProjection} from './generate-staged-release.mjs';
import {compileMaintainedStagedAuthority,compileDeployedStagedAuthority,maintainedStagedInventory} from './maintained-staged-authority.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const YAML=createRequire(path.join(root,'package.json'))('yaml');
export function readCheckedReleaseOperationView() {
 assert.equal(arguments.length,0,'caller-selected root/source/authority refused');
 const regular=file=>{const stat=fs.lstatSync(path.join(root,file));assert.ok(stat.isFile()&&!stat.isSymbolicLink(),'regular current source required');};
 for(const file of ['.github/workflows/release.yml',evidence+'original-release-workflow.yml',evidence+'stage-source-coverage.json'])regular(file);
 const retained=fs.readFileSync(path.join(root,evidence,'original-release-workflow.yml'),'utf8');
 const canonical=fs.readFileSync(path.join(root,'.github/workflows/release.yml'),'utf8');
 const packetBytes=fs.readFileSync(path.join(root,evidence,'stage-source-coverage.json'),'utf8');
 const packet=JSON.parse(packetBytes);
 const projection=buildStagedReleaseProjection(retained,packet);
 assert.equal(projection.operations,52);assert.equal(projection.binding.length,104);
 for(const file of projection.outputs.keys())regular(file);
 for(const [file,bytes] of projection.outputs)assert.equal(fs.readFileSync(path.join(root,file),'utf8'),bytes,'current physical projection differs');
 const deployed=projection.outputs.get(evidence+'candidate-release-caller.yml');
 const mode=canonical===retained?'inline':canonical===deployed?'deployed':null;
 assert.ok(mode,'unknown physical caller cannot provide logical operations');
 assert.ok(!process.env.NODE_OPTIONS,'source compiler observations refuse external Node loader options');
 const result=spawnSync(process.execPath,[fileURLToPath(import.meta.url),'--compiler-proof'],
  {cwd:root,encoding:'utf8',timeout:10000,maxBuffer:1024*1024});
 assert.equal(result.status,0,result.stderr);assert.equal(result.signal,null);
 const line=result.stdout.trimEnd().split('\n').at(-1);
 assert.ok(line.startsWith('CHECKED_RELEASE_COMPILER '),'fresh-process compiler receipt required');
 const inventory=JSON.parse(line.slice('CHECKED_RELEASE_COMPILER '.length));
 assert.equal(inventory.operations,52);assert.equal(inventory.bindings,104);
 const physicalCaller=YAML.parse(canonical),physicalStages=YAML.parse(projection.outputs.get('.github/workflows/release-staged.yml'));
 if(mode==='deployed')for(const id of ['auto-release','manual-release']) {
  assert.deepEqual(physicalCaller.jobs[id].permissions,{contents:'write',packages:'write',actions:'read'});
 }
 for(const [id,job] of Object.entries(physicalStages.jobs)){
  assert.ok(job['timeout-minutes']<=30);assert.equal(job.concurrency,undefined);
  const stage=id.replace(/^(auto|manual)_/u,'');
  const required=stage==='prepare-source'?{contents:'write',actions:'read'}:
   stage==='publish-verify-images'?{contents:'read',packages:'write'}:stage==='create-release'?{contents:'write'}:undefined;
  assert.deepEqual(job.permissions,required,'full effective stage rights must remain visible');
 }
 assert.equal(fs.readFileSync(path.join(root,'.github/workflows/release.yml'),'utf8'),canonical,'caller changed during expansion');
 assert.equal(fs.readFileSync(path.join(root,evidence,'original-release-workflow.yml'),'utf8'),retained);
 assert.equal(fs.readFileSync(path.join(root,evidence,'stage-source-coverage.json'),'utf8'),packetBytes);
 return {mode,originalSource:retained,physicalCaller,physicalStages,inventory,
  canonicalSha256:hash(canonical),retainedSha256:hash(retained),productionAuthority:false};
}

if(process.argv[1] && import.meta.url===pathToFileURL(fs.realpathSync(process.argv[1])).href) {
 assert.deepEqual(process.argv.slice(2),['--compiler-proof'],'exact compiler observation command required');
 assert.ok(!process.env.NODE_OPTIONS,'external Node loader options refused');
 const retained=fs.readFileSync(path.join(root,evidence,'original-release-workflow.yml'),'utf8');
 const canonical=fs.readFileSync(path.join(root,'.github/workflows/release.yml'),'utf8');
 const receipt=canonical===retained?compileMaintainedStagedAuthority():compileDeployedStagedAuthority();
 console.log('CHECKED_RELEASE_COMPILER '+JSON.stringify(maintainedStagedInventory(receipt)));
}
