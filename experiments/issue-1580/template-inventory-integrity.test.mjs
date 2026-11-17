import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import YAML from 'yaml';

const workflow=`name: Complete execution context
on:
  workflow_call:
    inputs:
      target: {type: string, required: true}
    outputs:
      artifact: {value: "\${{ jobs.build.outputs.artifact }}"}
env: {ROOT: retained}
defaults: {run: {shell: bash}}
jobs:
  build:
    runs-on: ubuntu-24.04
    environment: production
    container: {image: 'node:24', options: '--memory=128m'}
    services: {db: {image: 'postgres:17'}}
    strategy: {fail-fast: false, max-parallel: 1, matrix: {target: [linux]}}
    continue-on-error: false
    outputs: {artifact: "\${{ steps.build.outputs.artifact }}"}
    steps:
      - id: build
        working-directory: source
        env: {STEP: retained}
        run: echo hello
      - uses: actions/upload-artifact@v4
        with: {name: output, path: source/output}
  reuse:
    needs: build
    uses: ./local-workflow.yml
    secrets: inherit
    with: {target: linux}
`;

function fixture(run, commonJsYaml = false){
  const directory=mkdtempSync(join(tmpdir(),'issue-1580-inventory-'));
  try{
    const input=join(directory,'input'),output=join(directory,'output');
    const source=join(input,'fixture-complete','.github/workflows');
    mkdirSync(source,{recursive:true});
    const bytes=Buffer.from(workflow);
    const gitBlob=createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
    const sourcePath=join(source,'complete.yml');
    writeFileSync(sourcePath,bytes);
    writeFileSync(join(input,'manifest.json'),JSON.stringify([{repo:'fixture/fixture-template',sha:'a'.repeat(40)}]));
    writeFileSync(join(input,'fixture-complete-inventory.json'),JSON.stringify([{path:'.github/workflows/complete.yml',bytes:bytes.length,gitBlob}]));
    let generator='experiments/issue-1580/compare-template-trees.mjs';
    if (commonJsYaml) {
      generator=join(directory,'generator.mjs');
      writeFileSync(generator,readFileSync(new URL('./compare-template-trees.mjs',import.meta.url)));
      const packageDirectory=join(directory,'node_modules/yaml');
      mkdirSync(packageDirectory,{recursive:true});
      writeFileSync(join(packageDirectory,'package.json'),JSON.stringify({main:'index.cjs'}));
      // Dynamic CommonJS exports reproduce CI without a dependency install.
      const yamlPath=createRequire(import.meta.url).resolve('yaml');
      writeFileSync(join(packageDirectory,'index.cjs'),`Object.assign(module.exports, require(${JSON.stringify(yamlPath)}));\n`);
    }
    const invoke=()=>spawnSync(process.execPath,['--max-old-space-size=64',generator,input,output],{encoding:'utf8',timeout:5000,maxBuffer:65536});
    run({invoke,sourcePath,output,gitBlob});
  }finally{rmSync(directory,{recursive:true,force:true});}
}

test('workflow review inventory preserves the entire parsed execution context', {timeout:10000},()=>{
  fixture(({invoke,output,gitBlob})=>{
    const result=invoke();
    assert.equal(result.status,0,result.stderr);
    const row=JSON.parse(readFileSync(join(output,'fixture-workflows.jsonl'),'utf8'));
    assert.equal(row.gitBlob,gitBlob);
    assert.deepEqual(row.document,YAML.parse(workflow));
    assert.equal(row.document.jobs.build.steps[1].with.path,'source/output');
    assert.equal(row.document.jobs.reuse.secrets,'inherit');
    assert.equal(row.document.jobs.build.strategy['max-parallel'],1);
  });
});

test('workflow review refuses content that differs from the pinned Git blob', {timeout:10000},()=>{
  fixture(({invoke,sourcePath})=>{
    writeFileSync(sourcePath,workflow.replace('path: source/output','path: different/output'));
    const result=invoke();
    assert.equal(result.status,1,result.stderr);
    assert.match(result.stderr,/source drift: .github\/workflows\/complete.yml/u);
  });
});

test('workflow inventory runs when yaml exposes only dynamic CommonJS exports', {timeout:10000},()=>{
  fixture(({invoke,output})=>{
    const result=invoke();
    assert.equal(result.status,0,result.stderr);
    const row=JSON.parse(readFileSync(join(output,'fixture-workflows.jsonl'),'utf8'));
    assert.deepEqual(row.document,YAML.parse(workflow));
  },true);
});
