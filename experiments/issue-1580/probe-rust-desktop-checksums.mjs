// Execute the exact pinned Rust desktop checksum step with finite local assets.
// The candidate changes only the file selection in a disposable shell command.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {accessSync,constants,existsSync,mkdirSync,mkdtempSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';

const inventory=new URL('../../docs/case-studies/issue-1580/templates/rust-workflows.jsonl',import.meta.url);
const workflow=readFileSync(inventory,'utf8').trim().split('\n').map(line=>JSON.parse(line))
  .find(row=>row.path==='.github/workflows/desktop-release.yml');
const step=workflow.document.jobs.build.steps.find(row=>row.name==='Create checksum fragment');
const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');
assert.equal(workflow.gitBlob,'ecab57bcb89b29cf3d62f85137f461393630c170');
assert.equal(sha256(step.run),'5ebe6536be65e410293d758e82fbb833276babe4177876f34a97351e71c40662');
function executable(name){
  for(const directory of process.env.PATH.split(':')){
    const file=join(directory,name);
    try{accessSync(file,constants.X_OK);return file;}catch{}
  }
  assert.fail(`Required fixture executable is unavailable: ${name}`);
}
export function observeRustDesktopChecksums({candidate=false}={}){
  const observations=[];
  for(const branch of ['sha256sum','shasum'])for(const staleFragment of [false,true]){
    const temporary=mkdtempSync(join(tmpdir(),'issue-1580-rust-desktop-checksums-'));
    try{
      const bin=join(temporary,'bin');mkdirSync(bin);
      for(const name of ['find','sort','xargs','sed','shasum',...(branch==='sha256sum'?['sha256sum']:[])])symlinkSync(executable(name),join(bin,name));
      const directory=join(temporary,'desktop-dist');mkdirSync(directory);
      const assets={'app.zip':Buffer.from([0,1,2,3,255]),'second asset.exe':Buffer.from('finite second asset\n')};
      for(const [name,bytes]of Object.entries(assets))writeFileSync(join(directory,name),bytes);
      if(staleFragment)writeFileSync(join(directory,'SHA256SUMS-previous.partial'),'previous checksum fragment\n');
      let command=step.run;
      if(candidate){
        assert.equal(command.split('find . -type f ').length,3);
        command=command.replaceAll('find . -type f ',"find . -type f ! -name 'SHA256SUMS-*.partial' ");
      }
      const environment={...process.env,PATH:bin,MATRIX_LABEL:'fixture',BASH_ENV:'/dev/null',ENV:'/dev/null'};
      for(const key of Object.keys(environment))if(key.startsWith('BASH_FUNC_'))delete environment[key];
      const result=spawnSync(executable('bash'),['--noprofile','--norc','-e','-o','pipefail','-c',command],
        {cwd:temporary,env:environment,encoding:'utf8',maxBuffer:131072});
      assert.equal(result.error,undefined,String(result.error));
      const manifestName='SHA256SUMS-fixture.partial';
      const manifestPath=join(directory,manifestName);
      const manifest=existsSync(manifestPath)?readFileSync(manifestPath,'utf8'):'';
      const entries=manifest.trim().split('\n').filter(Boolean).map(line=>{
        const match=line.match(/^([a-f0-9]{64})  (.+)$/u);assert(match,`Malformed fixture digest: ${line}`);
        const file=join(directory,match[2]);
        return {name:match[2],digest:match[1],matchesCurrentBytes:existsSync(file)&&sha256(readFileSync(file))===match[1]};
      });
      observations.push({branch,staleFragment,exitCode:result.status,stderr:result.stderr,manifestName,manifest,entries,
        expectedAssetDigests:Object.fromEntries(Object.entries(assets).map(([name,bytes])=>[name,sha256(bytes)]))});
    }finally{rmSync(temporary,{recursive:true,force:true});}
  }
  return {repository:'link-foundation/rust-ai-driven-development-pipeline-template',
    sourceCommit:'309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb',workflowGitBlob:workflow.gitBlob,
    originalCommandSha256:sha256(step.run),candidateApplied:candidate,observations,
    rustBuildExecuted:false,hostedPublicationExecuted:false,independentAuthorshipCredit:0};
}
// Exercise this repository's real artifact selector before adopting a template change.
export function observeFormalDesktopChecksums(){
  const source=readFileSync(new URL('../../.github/workflows/desktop-release.yml',import.meta.url),'utf8');
  const selected=source.match(/^      - name: Collect artifacts and checksums\n[\s\S]*?^        run: \|\n((?:          .*\n|\n)+)/mu);
  assert(selected,'maintained desktop checksum command is present');
  const command=selected[1].replace(/^          /gmu,'').replaceAll('${{ matrix.label }}','fixture');
  const observations=[];
  for(const staleFragment of [false,true]){
    const temporary=mkdtempSync(join(tmpdir(),'issue-1580-formal-desktop-checksums-'));
    try{
      const directory=join(temporary,'release');mkdirSync(directory);
      const assets={'formal-ai-desktop-fixture.zip':Buffer.from([0,1,2,3,255]),
        'formal-ai-desktop-fixture.exe':Buffer.from('finite executable fixture\n'),
        'latest.yml':Buffer.from('version: 0.0.0-fixture\n')};
      for(const [name,bytes]of Object.entries(assets))writeFileSync(join(directory,name),bytes);
      if(staleFragment)writeFileSync(join(directory,'SHA256SUMS-previous.partial'),'previous checksum fragment\n');
      const environment={...process.env,BASH_ENV:'/dev/null',ENV:'/dev/null'};
      for(const key of Object.keys(environment))if(key.startsWith('BASH_FUNC_'))delete environment[key];
      const result=spawnSync(executable('bash'),['--noprofile','--norc','-e','-o','pipefail','-c',command],
        {cwd:temporary,env:environment,encoding:'utf8',maxBuffer:131072});
      assert.equal(result.error,undefined,String(result.error));
      const manifestName='SHA256SUMS-fixture.partial';
      const manifest=readFileSync(join(directory,manifestName),'utf8');
      const entries=manifest.trim().split('\n').filter(Boolean).map(line=>{
        const match=line.match(/^([a-f0-9]{64})  (.+)$/u);assert(match,`Malformed fixture digest: ${line}`);
        return {name:match[2],digest:match[1],matchesCurrentBytes:sha256(readFileSync(join(directory,match[2])))===match[1]};
      });
      observations.push({staleFragment,exitCode:result.status,stderr:result.stderr,manifestName,manifest,entries,
        expectedAssetDigests:Object.fromEntries(Object.entries(assets).map(([name,bytes])=>[name,sha256(bytes)]))});
    }finally{rmSync(temporary,{recursive:true,force:true});}
  }
  return {repository:'link-assistant/formal-ai',workflowSha256:sha256(source),commandSha256:sha256(command),
    observations,rustBuildExecuted:false,hostedPublicationExecuted:false,independentAuthorshipCredit:0};
}
if(process.argv[1]&&pathToFileURL(process.argv[1]).href===import.meta.url){
  assert(process.argv[2],'Usage: node experiments/issue-1580/probe-rust-desktop-checksums.mjs OUTPUT_JSON [--candidate]');
  assert(process.argv[3]===undefined||process.argv[3]==='--candidate','unknown option');
  const report=observeRustDesktopChecksums({candidate:process.argv[3]==='--candidate'});
  writeFileSync(process.argv[2],JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({observations:report.observations.length,candidate:report.candidateApplied,output:process.argv[2]}));
}
